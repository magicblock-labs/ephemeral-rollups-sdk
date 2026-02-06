use crate::intent_bundle::no_vec::{CapacityError, NoVec};
use pinocchio::cpi::{invoke_with_bounds, MAX_STATIC_CPI_ACCOUNTS};
use pinocchio::error::ProgramError;
use pinocchio::instruction::{InstructionAccount, InstructionView};
use pinocchio::{AccountView, ProgramResult};
use solana_address::Address;

mod args;
mod no_vec;
pub mod types;

pub use args::{ActionArgs, ShortAccountMeta};
use types::MagicIntentBundle;
pub use types::*;

const MAX_ACTIONS_NUM: usize = 10u8 as usize;

/// Bincode 1.x u32 LE discriminant for `MagicBlockInstruction::ScheduleIntentBundle` (variant index 11).
const SCHEDULE_INTENT_BUNDLE_DISCRIMINANT: [u8; 4] = 11u32.to_le_bytes();

/// Builds a single `MagicBlockInstruction::ScheduleIntentBundle` instruction by aggregating
/// multiple independent intents (base actions, commits, commit+undelegate), normalizing them,
/// and producing a deduplicated account list plus the corresponding CPI `Instruction`.
pub struct MagicIntentBundleBuilder<'args> {
    payer: AccountView,
    magic_context: AccountView,
    magic_program: AccountView,
    intent_bundle: MagicIntentBundle<'args>,
}

impl<'args> MagicIntentBundleBuilder<'args> {
    pub fn new(payer: AccountView, magic_context: AccountView, magic_program: AccountView) -> Self {
        Self {
            payer,
            magic_context,
            magic_program,
            intent_bundle: MagicIntentBundle::default(),
        }
    }

    /// Starts building a Commit intent. Returns a [`CommitIntentBuilder`] that owns this parent.
    ///
    /// The returned builder lets you chain `.add_post_commit_actions()`, transition to other
    /// intents via `.commit_and_undelegate()`, or finalize via `.build_and_invoke()`.
    pub fn commit<'a>(self, accounts: &'a [AccountView]) -> CommitIntentBuilder<'a, 'args> {
        CommitIntentBuilder {
            parent: self,
            accounts,
            actions: NoVec::<CallHandler, MAX_ACTIONS_NUM>::new(),
        }
    }

    /// Starts building a CommitAndUndelegate intent. Returns a [`CommitAndUndelegateIntentBuilder`]
    /// that owns this parent.
    ///
    /// The returned builder lets you chain `.add_post_commit_actions()`,
    /// `.add_post_undelegate_actions()`, transition to other intents via `.commit()`,
    /// or finalize via `.build_and_invoke()`.
    pub fn commit_and_undelegate<'a>(
        self,
        accounts: &'a [AccountView],
    ) -> CommitAndUndelegateIntentBuilder<'a, 'args> {
        CommitAndUndelegateIntentBuilder {
            parent: self,
            accounts,
            post_commit_actions: NoVec::default(),
            post_undelegate_actions: NoVec::default(),
        }
    }

    /// Adds an intent to the bundle.
    ///
    /// If an intent of the same category already exists in the bundle:
    /// - base actions are appended
    /// - commit intents are merged (unique accounts/actions appended)
    /// - commit+undelegate intents are merged (unique accounts/actions appended)
    ///
    /// See `MagicIntentBundle::add_intent` for merge semantics.
    pub fn add_intent(mut self, intent: MagicIntent<'args>) -> Result<Self, ProgramError> {
        self.intent_bundle.add_intent(intent)?;
        Ok(self)
    }

    /// Adds (or merges) a `Commit` intent into the bundle.
    pub fn add_commit(mut self, commit: CommitIntent<'args>) -> Result<Self, ProgramError> {
        self.intent_bundle.add_intent(MagicIntent::Commit(commit))?;
        Ok(self)
    }

    /// Adds (or merges) a `CommitAndUndelegate` intent into the bundle.
    pub fn add_commit_and_undelegate(
        mut self,
        value: CommitAndUndelegateIntent<'args>,
    ) -> Result<Self, ProgramError> {
        self.intent_bundle
            .add_intent(MagicIntent::CommitAndUndelegate(value))?;
        Ok(self)
    }

    /// Adds standalone base-layer actions to be executed without any commit/undelegate semantics.
    pub fn add_standalone_actions<'newargs>(
        self,
        actions: &[CallHandler<'newargs>],
    ) -> Result<MagicIntentBundleBuilder<'newargs>, ProgramError>
    where
        'args: 'newargs,
    {
        let mut this = MagicIntentBundleBuilder::<'newargs> {
            payer: self.payer,
            magic_program: self.magic_program,
            magic_context: self.magic_context,
            intent_bundle: self.intent_bundle,
        };

        let mut standalone_actions = NoVec::<CallHandler<'newargs>, MAX_ACTIONS_NUM>::new();
        standalone_actions.try_append_slice(actions)?;
        this.intent_bundle
            .add_intent(MagicIntent::StandaloneActions(standalone_actions))?;
        Ok(this)
    }

    /// Normalizes the bundle, serializes it with bincode into `data_buf`, builds the
    /// CPI instruction, and invokes the magic program.
    ///
    /// `data_buf` must be large enough to hold the serialized `MagicIntentBundleArgs`.
    pub fn build_and_invoke(mut self, data_buf: &mut [u8]) -> ProgramResult {
        const OFFSET: usize = SCHEDULE_INTENT_BUNDLE_DISCRIMINANT.len();

        // Normalize: dedup within intents, resolve cross-intent overlaps
        self.intent_bundle.normalize()?;

        // Collect all unique accounts (payer + context first, then from intents)
        let mut all_accounts = NoVec::<AccountView, MAX_STATIC_CPI_ACCOUNTS>::new();
        all_accounts.try_append([self.payer, self.magic_context])?;
        self.intent_bundle
            .collect_unique_accounts(&mut all_accounts)?;

        // 3. Build the natural indices map: indices_map[i] = address of account at position i
        let mut indices_map = NoVec::<Address, MAX_STATIC_CPI_ACCOUNTS>::new();
        for account in all_accounts.iter() {
            indices_map.try_push(account.address().clone())?;
        }

        // 4. Convert intents to serializable args
        let args = self.intent_bundle.into_args(indices_map.as_slice())?;

        // 5. Write instruction discriminant + serialize args (bincode 1.x wire compat)
        data_buf[..OFFSET].copy_from_slice(&SCHEDULE_INTENT_BUNDLE_DISCRIMINANT);
        let args_len =
            bincode::encode_into_slice(&args, &mut data_buf[OFFSET..], bincode::config::legacy())
                .map_err(|_| ProgramError::InvalidInstructionData)?;
        let data_len = OFFSET + args_len;

        // 6. Build instruction account metas
        let mut instruction_accounts = NoVec::<InstructionAccount, MAX_STATIC_CPI_ACCOUNTS>::new();
        for account in all_accounts.iter() {
            instruction_accounts.try_push(InstructionAccount::from(account))?;
        }

        // 7. Build instruction view
        let ix = InstructionView {
            program_id: self.magic_program.address(),
            data: &data_buf[..data_len],
            accounts: instruction_accounts.as_slice(),
        };

        // 8. Build account refs for invoke
        let mut account_refs = NoVec::<&AccountView, MAX_STATIC_CPI_ACCOUNTS>::new();
        for account in all_accounts.iter() {
            account_refs.try_push(account)?;
        }

        invoke_with_bounds::<MAX_STATIC_CPI_ACCOUNTS>(&ix, account_refs.as_slice())
    }
}

/// Builder of Commit Intent.
///
/// Created via [`MagicIntentBundleBuilder::commit()`]. Owns the parent builder
/// and returns it (or a sibling sub-builder) on every transition/terminal call.
pub struct CommitIntentBuilder<'a, 'args> {
    parent: MagicIntentBundleBuilder<'args>,
    accounts: &'a [AccountView],
    actions: NoVec<CallHandler<'args>, MAX_ACTIONS_NUM>,
}

impl<'a, 'args> CommitIntentBuilder<'a, 'args> {
    /// Adds post-commit actions. Chainable.
    pub fn add_post_commit_actions<'new_args>(
        self,
        actions: &[CallHandler<'new_args>],
    ) -> Result<CommitIntentBuilder<'a, 'new_args>, ProgramError>
    where
        'args: 'new_args,
    {
        let mut this = CommitIntentBuilder::<'a, 'new_args> {
            parent: self.parent,
            accounts: self.accounts,
            actions: self.actions,
        };
        this.actions.try_append_slice(actions)?;
        Ok(this)
    }

    /// Transition: finalizes this commit intent and starts a commit-and-undelegate intent.
    pub fn commit_and_undelegate<'cau>(
        self,
        accounts: &'cau [AccountView],
    ) -> Result<CommitAndUndelegateIntentBuilder<'cau, 'args>, ProgramError> {
        Ok(self.done()?.commit_and_undelegate(accounts))
    }

    /// Transition: finalizes this commit intent and adds standalone base-layer actions.
    pub fn add_standalone_actions<'newargs>(
        self,
        actions: &[CallHandler<'newargs>],
    ) -> Result<MagicIntentBundleBuilder<'newargs>, ProgramError>
    where
        'args: 'newargs,
    {
        self.done()?.add_standalone_actions(actions)
    }

    /// Terminal: finalizes this commit intent, builds the instruction and invokes it.
    pub fn build_and_invoke(self, data_buf: &mut [u8]) -> ProgramResult {
        self.done()?.build_and_invoke(data_buf)
    }

    /// Finalizes this commit intent and folds it into the parent bundle.
    fn done(self) -> Result<MagicIntentBundleBuilder<'args>, ProgramError> {
        let Self {
            mut parent,
            accounts: committed_accounts,
            actions,
        } = self;

        let mut accounts = NoVec::<AccountView, MAX_STATIC_CPI_ACCOUNTS>::new();
        accounts.try_append_slice(committed_accounts)?;
        let commit = CommitIntent { accounts, actions };

        parent
            .intent_bundle
            .add_intent(MagicIntent::Commit(commit))?;
        Ok(parent)
    }
}

/// Builder of CommitAndUndelegate Intent.
///
/// Created via [`MagicIntentBundleBuilder::commit_and_undelegate()`] or
/// [`CommitIntentBuilder::commit_and_undelegate()`]. Owns the parent builder
/// and returns it (or a sibling sub-builder) on every transition/terminal call.
pub struct CommitAndUndelegateIntentBuilder<'a, 'args> {
    parent: MagicIntentBundleBuilder<'args>,
    accounts: &'a [AccountView],
    post_commit_actions: NoVec<CallHandler<'args>, MAX_ACTIONS_NUM>,
    post_undelegate_actions: NoVec<CallHandler<'args>, MAX_ACTIONS_NUM>,
}

impl<'a, 'args> CommitAndUndelegateIntentBuilder<'a, 'args> {
    /// Adds post-commit actions. Chainable.
    pub fn add_post_commit_actions<'new_args>(
        self,
        actions: &[CallHandler<'new_args>],
    ) -> Result<CommitAndUndelegateIntentBuilder<'a, 'new_args>, ProgramError>
    where
        'args: 'new_args,
    {
        let mut this = CommitAndUndelegateIntentBuilder::<'a, 'new_args> {
            parent: self.parent,
            accounts: self.accounts,
            post_commit_actions: self.post_commit_actions,
            post_undelegate_actions: self.post_undelegate_actions,
        };
        this.post_commit_actions.try_append_slice(actions)?;
        Ok(this)
    }

    /// Adds post-undelegate actions. Chainable.
    pub fn add_post_undelegate_actions<'new_args>(
        self,
        actions: &[CallHandler<'new_args>],
    ) -> Result<CommitAndUndelegateIntentBuilder<'a, 'new_args>, ProgramError>
    where
        'args: 'new_args,
    {
        let mut this = CommitAndUndelegateIntentBuilder::<'a, 'new_args> {
            parent: self.parent,
            accounts: self.accounts,
            post_commit_actions: self.post_commit_actions,
            post_undelegate_actions: self.post_undelegate_actions,
        };
        this.post_undelegate_actions.try_append_slice(actions)?;
        Ok(this)
    }

    /// Transition: finalizes this commit-and-undelegate intent and starts a new commit intent.
    pub fn commit<'b>(
        self,
        accounts: &'b [AccountView],
    ) -> Result<CommitIntentBuilder<'b, 'args>, ProgramError> {
        Ok(self.done()?.commit(accounts))
    }

    /// Transition: finalizes this commit-and-undelegate intent and adds standalone base-layer actions.
    pub fn add_standalone_actions<'newargs>(
        self,
        actions: &[CallHandler<'newargs>],
    ) -> Result<MagicIntentBundleBuilder<'newargs>, ProgramError>
    where
        'args: 'newargs,
    {
        self.done()?.add_standalone_actions(actions)
    }

    /// Terminal: finalizes this intent, builds the instruction and invokes it.
    pub fn build_and_invoke(self, data_buf: &mut [u8]) -> ProgramResult {
        self.done()?.build_and_invoke(data_buf)
    }

    /// Finalizes this commit-and-undelegate intent and folds it into the parent bundle.
    fn done(self) -> Result<MagicIntentBundleBuilder<'args>, ProgramError> {
        let Self {
            mut parent,
            accounts: committed_accounts,
            post_commit_actions,
            post_undelegate_actions,
        } = self;

        let mut accounts = NoVec::<_, MAX_STATIC_CPI_ACCOUNTS>::new();
        accounts.try_append_slice(committed_accounts)?;
        let cau = CommitAndUndelegateIntent {
            accounts,
            post_commit_actions,
            post_undelegate_actions,
        };
        parent
            .intent_bundle
            .add_intent(MagicIntent::CommitAndUndelegate(cau))?;
        Ok(parent)
    }
}

impl<T> From<CapacityError<T>> for ProgramError {
    fn from(_: CapacityError<T>) -> Self {
        ProgramError::InvalidArgument
    }
}

// ---------------------------------------------------------------------------
// Test-only: serialize builder output without CPI
// ---------------------------------------------------------------------------

#[cfg(test)]
impl MagicIntentBundleBuilder<'_> {
    /// Reproduces the logic of `build_and_invoke` but serializes the
    /// `MagicIntentBundleArgs` into the provided buffer instead of invoking CPI.
    /// Returns the number of bytes written.
    fn build_serialized(mut self, buf: &mut [u8]) -> usize {
        self.intent_bundle.normalize().unwrap();

        let mut all_accounts = NoVec::<AccountView, MAX_STATIC_CPI_ACCOUNTS>::new();
        all_accounts.append([self.payer, self.magic_context]);
        self.intent_bundle
            .collect_unique_accounts(&mut all_accounts)
            .unwrap();

        let mut indices_map = NoVec::<Address, MAX_STATIC_CPI_ACCOUNTS>::new();
        for account in all_accounts.iter() {
            indices_map.push(account.address().clone());
        }

        let args = self
            .intent_bundle
            .into_args(indices_map.as_slice())
            .unwrap();
        buf[..4].copy_from_slice(&SCHEDULE_INTENT_BUNDLE_DISCRIMINANT);
        let args_len =
            bincode::encode_into_slice(&args, &mut buf[4..], bincode::config::legacy()).unwrap();
        4 + args_len
    }
}

#[cfg(test)]
impl CommitIntentBuilder<'_, '_> {
    fn build_serialized(self, buf: &mut [u8]) -> usize {
        self.done().unwrap().build_serialized(buf)
    }
}

#[cfg(test)]
impl CommitAndUndelegateIntentBuilder<'_, '_> {
    fn build_serialized(self, buf: &mut [u8]) -> usize {
        self.done().unwrap().build_serialized(buf)
    }
}

// ---------------------------------------------------------------------------
// Tests: builder compatibility between pinocchio and SDK
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    extern crate std;

    use std::cell::RefCell;
    use std::rc::Rc;
    use std::vec;
    use std::vec::Vec;

    /// Solana CPI instruction data limit (1280 bytes).
    const CPI_DATA_BUF_SIZE: usize = 1280;

    use super::*;
    use crate::intent_bundle::args::ShortAccountMeta;

    // SDK builder
    use ephemeral_rollups_sdk::ephem::{
        CallHandler as SdkCallHandler, MagicIntentBundleBuilder as SdkBuilder,
    };
    use magicblock_magic_program_api::args::{
        ActionArgs as SdkActionArgs, ShortAccountMeta as SdkShortAccountMeta,
    };
    use magicblock_magic_program_api::Pubkey;
    use solana_program::account_info::AccountInfo;

    // -----------------------------------------------------------------
    // Mock helpers
    // -----------------------------------------------------------------

    /// Memory layout matching `RuntimeAccount` from `solana-account-view`.
    /// Used to back a pinocchio `AccountView` in tests.
    #[repr(C)]
    struct MockRuntimeAccount {
        borrow_state: u8,
        is_signer: u8,
        is_writable: u8,
        executable: u8,
        resize_delta: i32,
        address: [u8; 32],
        owner: [u8; 32],
        lamports: u64,
        data_len: u64,
    }

    impl MockRuntimeAccount {
        fn new(address: [u8; 32]) -> Self {
            Self {
                borrow_state: 0xFF, // NOT_BORROWED
                is_signer: 0,
                is_writable: 1,
                executable: 0,
                resize_delta: 0,
                address,
                owner: [0; 32],
                lamports: 1_000_000,
                data_len: 0,
            }
        }

        fn as_account_view(&mut self) -> AccountView {
            // SAFETY: MockRuntimeAccount has the same #[repr(C)] layout as
            // RuntimeAccount from solana-account-view. AccountView is a
            // #[repr(C)] wrapper around *mut RuntimeAccount.
            unsafe { core::mem::transmute(self as *mut Self) }
        }
    }

    /// Helper to hold owned data for an SDK `AccountInfo`.
    struct SdkTestAccount {
        key: Pubkey,
        lamports: u64,
        data: Vec<u8>,
        owner: Pubkey,
    }

    impl SdkTestAccount {
        fn new(address: [u8; 32]) -> Self {
            Self {
                key: Pubkey::new_from_array(address),
                lamports: 1_000_000,
                data: vec![],
                owner: Pubkey::new_from_array([0; 32]),
            }
        }

        fn as_account_info(&mut self) -> AccountInfo<'_> {
            AccountInfo {
                key: &self.key,
                is_signer: false,
                is_writable: true,
                lamports: Rc::new(RefCell::new(&mut self.lamports)),
                data: Rc::new(RefCell::new(&mut self.data)),
                owner: &self.owner,
                executable: false,
                rent_epoch: 0,
            }
        }

        fn as_signer_info(&mut self) -> AccountInfo<'_> {
            AccountInfo {
                key: &self.key,
                is_signer: true,
                is_writable: false,
                lamports: Rc::new(RefCell::new(&mut self.lamports)),
                data: Rc::new(RefCell::new(&mut self.data)),
                owner: &self.owner,
                executable: false,
                rent_epoch: 0,
            }
        }
    }

    // -----------------------------------------------------------------
    // Builder compatibility tests
    // -----------------------------------------------------------------

    /// Commit standalone (no actions).
    ///
    /// Both builders: `builder.commit(&[acc1, acc2]).build()`
    #[test]
    fn test_compat_commit_standalone() {
        let payer_addr = [0x01; 32];
        let ctx_addr = [0x02; 32];
        let acc1_addr = [0x03; 32];
        let acc2_addr = [0x04; 32];
        let prog_addr = [0xFF; 32];

        // --- Pinocchio builder ---
        let mut p_payer = MockRuntimeAccount::new(payer_addr);
        let mut p_ctx = MockRuntimeAccount::new(ctx_addr);
        let mut p_acc1 = MockRuntimeAccount::new(acc1_addr);
        let mut p_acc2 = MockRuntimeAccount::new(acc2_addr);
        let mut p_prog = MockRuntimeAccount::new(prog_addr);

        let commit_accs = [p_acc1.as_account_view(), p_acc2.as_account_view()];
        let mut buf = [0u8; CPI_DATA_BUF_SIZE];
        let pino_len = MagicIntentBundleBuilder::new(
            p_payer.as_account_view(),
            p_ctx.as_account_view(),
            p_prog.as_account_view(),
        )
        .commit(&commit_accs)
        .build_serialized(&mut buf);

        // --- SDK builder ---
        let mut s_payer = SdkTestAccount::new(payer_addr);
        let mut s_ctx = SdkTestAccount::new(ctx_addr);
        let mut s_acc1 = SdkTestAccount::new(acc1_addr);
        let mut s_acc2 = SdkTestAccount::new(acc2_addr);
        let mut s_prog = SdkTestAccount::new(prog_addr);

        let (accounts, ix) = SdkBuilder::new(
            s_payer.as_account_info(),
            s_ctx.as_account_info(),
            s_prog.as_account_info(),
        )
        .commit(&[s_acc1.as_account_info(), s_acc2.as_account_info()])
        .build();
        drop(accounts);

        assert_eq!(&buf[..pino_len], &ix.data, "commit standalone mismatch");
    }

    /// Commit with a post-commit action (handler).
    #[test]
    fn test_compat_commit_with_handler() {
        let payer_addr = [0x01; 32];
        let ctx_addr = [0x02; 32];
        let acc1_addr = [0x03; 32];
        let escrow_addr = [0x04; 32];
        let dest_addr = [0xDD; 32];
        let prog_addr = [0xFF; 32];
        let action_data = [0xAA, 0xBB, 0xCC];

        // --- Pinocchio ---
        let mut p_payer = MockRuntimeAccount::new(payer_addr);
        let mut p_ctx = MockRuntimeAccount::new(ctx_addr);
        let mut p_acc1 = MockRuntimeAccount::new(acc1_addr);
        let mut p_escrow = MockRuntimeAccount::new(escrow_addr);
        let mut p_prog = MockRuntimeAccount::new(prog_addr);

        let escrow_view = p_escrow.as_account_view();
        let handler = CallHandler::new(
            Address::new_from_array(dest_addr),
            escrow_view,
            ActionArgs::new(&action_data),
            200_000,
        );
        let commit_accs = [p_acc1.as_account_view()];
        let mut buf = [0u8; CPI_DATA_BUF_SIZE];
        let pino_len = MagicIntentBundleBuilder::new(
            p_payer.as_account_view(),
            p_ctx.as_account_view(),
            p_prog.as_account_view(),
        )
        .commit(&commit_accs)
        .add_post_commit_actions(&[handler])
        .unwrap()
        .build_serialized(&mut buf);

        // --- SDK ---
        let mut s_payer = SdkTestAccount::new(payer_addr);
        let mut s_ctx = SdkTestAccount::new(ctx_addr);
        let mut s_acc1 = SdkTestAccount::new(acc1_addr);
        let mut s_escrow = SdkTestAccount::new(escrow_addr);
        let mut s_prog = SdkTestAccount::new(prog_addr);

        let sdk_handler = SdkCallHandler {
            args: SdkActionArgs::new(action_data.to_vec()),
            compute_units: 200_000,
            escrow_authority: s_escrow.as_signer_info(),
            destination_program: Pubkey::new_from_array(dest_addr),
            accounts: vec![],
        };
        let (accounts, ix) = SdkBuilder::new(
            s_payer.as_account_info(),
            s_ctx.as_account_info(),
            s_prog.as_account_info(),
        )
        .commit(&[s_acc1.as_account_info()])
        .add_post_commit_actions([sdk_handler])
        .build();
        drop(accounts);

        assert_eq!(&buf[..pino_len], &ix.data, "commit with handler mismatch");
    }

    /// CommitAndUndelegate standalone (no actions).
    #[test]
    fn test_compat_commit_and_undelegate_standalone() {
        let payer_addr = [0x01; 32];
        let ctx_addr = [0x02; 32];
        let acc1_addr = [0x03; 32];
        let prog_addr = [0xFF; 32];

        // --- Pinocchio ---
        let mut p_payer = MockRuntimeAccount::new(payer_addr);
        let mut p_ctx = MockRuntimeAccount::new(ctx_addr);
        let mut p_acc1 = MockRuntimeAccount::new(acc1_addr);
        let mut p_prog = MockRuntimeAccount::new(prog_addr);

        let cau_accs = [p_acc1.as_account_view()];
        let mut buf = [0u8; CPI_DATA_BUF_SIZE];
        let pino_len = MagicIntentBundleBuilder::new(
            p_payer.as_account_view(),
            p_ctx.as_account_view(),
            p_prog.as_account_view(),
        )
        .commit_and_undelegate(&cau_accs)
        .build_serialized(&mut buf);

        // --- SDK ---
        let mut s_payer = SdkTestAccount::new(payer_addr);
        let mut s_ctx = SdkTestAccount::new(ctx_addr);
        let mut s_acc1 = SdkTestAccount::new(acc1_addr);
        let mut s_prog = SdkTestAccount::new(prog_addr);

        let (accounts, ix) = SdkBuilder::new(
            s_payer.as_account_info(),
            s_ctx.as_account_info(),
            s_prog.as_account_info(),
        )
        .commit_and_undelegate(&[s_acc1.as_account_info()])
        .build();
        drop(accounts);

        assert_eq!(
            &buf[..pino_len],
            &ix.data,
            "commit_and_undelegate standalone mismatch"
        );
    }

    /// CommitAndUndelegate with post-commit and post-undelegate actions.
    #[test]
    fn test_compat_commit_and_undelegate_with_actions() {
        let payer_addr = [0x01; 32];
        let ctx_addr = [0x02; 32];
        let acc1_addr = [0x03; 32];
        let escrow1_addr = [0x04; 32];
        let escrow2_addr = [0x05; 32];
        let dest1_addr = [0xAA; 32];
        let dest2_addr = [0xBB; 32];
        let prog_addr = [0xFF; 32];
        let commit_data = [1u8, 2, 3];
        let undelegate_data = [4u8, 5, 6];

        // --- Pinocchio ---
        let mut p_payer = MockRuntimeAccount::new(payer_addr);
        let mut p_ctx = MockRuntimeAccount::new(ctx_addr);
        let mut p_acc1 = MockRuntimeAccount::new(acc1_addr);
        let mut p_escrow1 = MockRuntimeAccount::new(escrow1_addr);
        let mut p_escrow2 = MockRuntimeAccount::new(escrow2_addr);
        let mut p_prog = MockRuntimeAccount::new(prog_addr);

        let post_commit = CallHandler::new(
            Address::new_from_array(dest1_addr),
            p_escrow1.as_account_view(),
            ActionArgs::new(&commit_data),
            100_000,
        );
        let post_undelegate = CallHandler::new(
            Address::new_from_array(dest2_addr),
            p_escrow2.as_account_view(),
            ActionArgs::new(&undelegate_data),
            50_000,
        );
        let cau_accs = [p_acc1.as_account_view()];
        let mut buf = [0u8; CPI_DATA_BUF_SIZE];
        let pino_len = MagicIntentBundleBuilder::new(
            p_payer.as_account_view(),
            p_ctx.as_account_view(),
            p_prog.as_account_view(),
        )
        .commit_and_undelegate(&cau_accs)
        .add_post_commit_actions(&[post_commit])
        .unwrap()
        .add_post_undelegate_actions(&[post_undelegate])
        .unwrap()
        .build_serialized(&mut buf);

        // --- SDK ---
        let mut s_payer = SdkTestAccount::new(payer_addr);
        let mut s_ctx = SdkTestAccount::new(ctx_addr);
        let mut s_acc1 = SdkTestAccount::new(acc1_addr);
        let mut s_escrow1 = SdkTestAccount::new(escrow1_addr);
        let mut s_escrow2 = SdkTestAccount::new(escrow2_addr);
        let mut s_prog = SdkTestAccount::new(prog_addr);

        let sdk_post_commit = SdkCallHandler {
            args: SdkActionArgs::new(commit_data.to_vec()),
            compute_units: 100_000,
            escrow_authority: s_escrow1.as_signer_info(),
            destination_program: Pubkey::new_from_array(dest1_addr),
            accounts: vec![],
        };
        let sdk_post_undelegate = SdkCallHandler {
            args: SdkActionArgs::new(undelegate_data.to_vec()),
            compute_units: 50_000,
            escrow_authority: s_escrow2.as_signer_info(),
            destination_program: Pubkey::new_from_array(dest2_addr),
            accounts: vec![],
        };
        let (accounts, ix) = SdkBuilder::new(
            s_payer.as_account_info(),
            s_ctx.as_account_info(),
            s_prog.as_account_info(),
        )
        .commit_and_undelegate(&[s_acc1.as_account_info()])
        .add_post_commit_actions([sdk_post_commit])
        .add_post_undelegate_actions([sdk_post_undelegate])
        .build();
        drop(accounts);

        assert_eq!(
            &buf[..pino_len],
            &ix.data,
            "commit_and_undelegate with actions mismatch"
        );
    }

    /// Standalone actions only (no commit / undelegate).
    #[test]
    fn test_compat_standalone_actions() {
        let payer_addr = [0x01; 32];
        let ctx_addr = [0x02; 32];
        let escrow_addr = [0x04; 32];
        let dest_addr = [0xA1; 32];
        let extra_addr = [0xB1; 32];
        let prog_addr = [0xFF; 32];
        let data = [0x10u8, 0x20];

        // --- Pinocchio ---
        let mut p_payer = MockRuntimeAccount::new(payer_addr);
        let mut p_ctx = MockRuntimeAccount::new(ctx_addr);
        let mut p_escrow = MockRuntimeAccount::new(escrow_addr);
        let mut p_prog = MockRuntimeAccount::new(prog_addr);

        let mut handler = CallHandler::new(
            Address::new_from_array(dest_addr),
            p_escrow.as_account_view(),
            ActionArgs::new(&data),
            100_000,
        );
        handler
            .add_accounts_slice(&[ShortAccountMeta {
                pubkey: Address::new_from_array(extra_addr),
                is_writable: true,
            }])
            .unwrap();
        let mut buf = [0u8; CPI_DATA_BUF_SIZE];
        let pino_len = MagicIntentBundleBuilder::new(
            p_payer.as_account_view(),
            p_ctx.as_account_view(),
            p_prog.as_account_view(),
        )
        .add_standalone_actions(&[handler])
        .unwrap()
        .build_serialized(&mut buf);

        // --- SDK ---
        let mut s_payer = SdkTestAccount::new(payer_addr);
        let mut s_ctx = SdkTestAccount::new(ctx_addr);
        let mut s_escrow = SdkTestAccount::new(escrow_addr);
        let mut s_prog = SdkTestAccount::new(prog_addr);

        let sdk_handler = SdkCallHandler {
            args: SdkActionArgs::new(data.to_vec()),
            compute_units: 100_000,
            escrow_authority: s_escrow.as_signer_info(),
            destination_program: Pubkey::new_from_array(dest_addr),
            accounts: vec![SdkShortAccountMeta {
                pubkey: Pubkey::new_from_array(extra_addr),
                is_writable: true,
            }],
        };
        let (accounts, ix) = SdkBuilder::new(
            s_payer.as_account_info(),
            s_ctx.as_account_info(),
            s_prog.as_account_info(),
        )
        .add_standalone_actions([sdk_handler])
        .build();
        drop(accounts);

        assert_eq!(&buf[..pino_len], &ix.data, "standalone actions mismatch");
    }

    /// Chained: commit then commit_and_undelegate.
    #[test]
    fn test_compat_commit_then_commit_and_undelegate() {
        let payer_addr = [0x01; 32];
        let ctx_addr = [0x02; 32];
        let acc1_addr = [0x03; 32];
        let acc2_addr = [0x04; 32];
        let prog_addr = [0xFF; 32];

        // --- Pinocchio ---
        let mut p_payer = MockRuntimeAccount::new(payer_addr);
        let mut p_ctx = MockRuntimeAccount::new(ctx_addr);
        let mut p_acc1 = MockRuntimeAccount::new(acc1_addr);
        let mut p_acc2 = MockRuntimeAccount::new(acc2_addr);
        let mut p_prog = MockRuntimeAccount::new(prog_addr);

        let commit_accs = [p_acc1.as_account_view()];
        let cau_accs = [p_acc2.as_account_view()];
        let mut buf = [0u8; CPI_DATA_BUF_SIZE];
        let pino_len = MagicIntentBundleBuilder::new(
            p_payer.as_account_view(),
            p_ctx.as_account_view(),
            p_prog.as_account_view(),
        )
        .commit(&commit_accs)
        .commit_and_undelegate(&cau_accs)
        .unwrap()
        .build_serialized(&mut buf);

        // --- SDK ---
        let mut s_payer = SdkTestAccount::new(payer_addr);
        let mut s_ctx = SdkTestAccount::new(ctx_addr);
        let mut s_acc1 = SdkTestAccount::new(acc1_addr);
        let mut s_acc2 = SdkTestAccount::new(acc2_addr);
        let mut s_prog = SdkTestAccount::new(prog_addr);

        let (accounts, ix) = SdkBuilder::new(
            s_payer.as_account_info(),
            s_ctx.as_account_info(),
            s_prog.as_account_info(),
        )
        .commit(&[s_acc1.as_account_info()])
        .commit_and_undelegate(&[s_acc2.as_account_info()])
        .build();
        drop(accounts);

        assert_eq!(
            &buf[..pino_len],
            &ix.data,
            "commit then commit_and_undelegate mismatch"
        );
    }

    /// All intent types combined: commit + commit_and_undelegate + standalone actions.
    #[test]
    fn test_compat_all_intents_combined() {
        let payer_addr = [0x01; 32];
        let ctx_addr = [0x02; 32];
        let commit_acc_addr = [0x03; 32];
        let cau_acc_addr = [0x04; 32];
        let escrow_addr = [0x05; 32];
        let dest_addr = [0xE1; 32];
        let prog_addr = [0xFF; 32];
        let standalone_data = [0xE0u8];

        // --- Pinocchio ---
        let mut p_payer = MockRuntimeAccount::new(payer_addr);
        let mut p_ctx = MockRuntimeAccount::new(ctx_addr);
        let mut p_commit = MockRuntimeAccount::new(commit_acc_addr);
        let mut p_cau = MockRuntimeAccount::new(cau_acc_addr);
        let mut p_escrow = MockRuntimeAccount::new(escrow_addr);
        let mut p_prog = MockRuntimeAccount::new(prog_addr);

        let handler = CallHandler::new(
            Address::new_from_array(dest_addr),
            p_escrow.as_account_view(),
            ActionArgs::new(&standalone_data),
            150_000,
        );
        let commit_accs = [p_commit.as_account_view()];
        let cau_accs = [p_cau.as_account_view()];
        let mut buf = [0u8; CPI_DATA_BUF_SIZE];
        let pino_len = MagicIntentBundleBuilder::new(
            p_payer.as_account_view(),
            p_ctx.as_account_view(),
            p_prog.as_account_view(),
        )
        .commit(&commit_accs)
        .commit_and_undelegate(&cau_accs)
        .unwrap()
        .add_standalone_actions(&[handler])
        .unwrap()
        .build_serialized(&mut buf);

        // --- SDK ---
        let mut s_payer = SdkTestAccount::new(payer_addr);
        let mut s_ctx = SdkTestAccount::new(ctx_addr);
        let mut s_commit = SdkTestAccount::new(commit_acc_addr);
        let mut s_cau = SdkTestAccount::new(cau_acc_addr);
        let mut s_escrow = SdkTestAccount::new(escrow_addr);
        let mut s_prog = SdkTestAccount::new(prog_addr);

        let sdk_handler = SdkCallHandler {
            args: SdkActionArgs::new(standalone_data.to_vec()),
            compute_units: 150_000,
            escrow_authority: s_escrow.as_signer_info(),
            destination_program: Pubkey::new_from_array(dest_addr),
            accounts: vec![],
        };
        let (accounts, ix) = SdkBuilder::new(
            s_payer.as_account_info(),
            s_ctx.as_account_info(),
            s_prog.as_account_info(),
        )
        .commit(&[s_commit.as_account_info()])
        .commit_and_undelegate(&[s_cau.as_account_info()])
        .add_standalone_actions([sdk_handler])
        .build();
        drop(accounts);

        assert_eq!(&buf[..pino_len], &ix.data, "all intents combined mismatch");
    }

    /// Full chain with actions on all intents.
    #[test]
    fn test_compat_full_chain_with_actions() {
        let payer_addr = [0x01; 32];
        let ctx_addr = [0x02; 32];
        let commit_acc_addr = [0x03; 32];
        let cau_acc_addr = [0x04; 32];
        let escrow1_addr = [0x05; 32];
        let escrow2_addr = [0x06; 32];
        let dest1_addr = [0xC1; 32];
        let dest2_addr = [0xD1; 32];
        let prog_addr = [0xFF; 32];
        let commit_data = [0xC0u8];
        let undelegate_data = [0xD0u8];

        // --- Pinocchio ---
        let mut p_payer = MockRuntimeAccount::new(payer_addr);
        let mut p_ctx = MockRuntimeAccount::new(ctx_addr);
        let mut p_commit = MockRuntimeAccount::new(commit_acc_addr);
        let mut p_cau = MockRuntimeAccount::new(cau_acc_addr);
        let mut p_escrow1 = MockRuntimeAccount::new(escrow1_addr);
        let mut p_escrow2 = MockRuntimeAccount::new(escrow2_addr);
        let mut p_prog = MockRuntimeAccount::new(prog_addr);

        let commit_handler = CallHandler::new(
            Address::new_from_array(dest1_addr),
            p_escrow1.as_account_view(),
            ActionArgs::new(&commit_data),
            100_000,
        );
        let undelegate_handler = CallHandler::new(
            Address::new_from_array(dest2_addr),
            p_escrow2.as_account_view(),
            ActionArgs::new(&undelegate_data),
            50_000,
        );
        let commit_accs = [p_commit.as_account_view()];
        let cau_accs = [p_cau.as_account_view()];
        let mut buf = [0u8; CPI_DATA_BUF_SIZE];
        let pino_len = MagicIntentBundleBuilder::new(
            p_payer.as_account_view(),
            p_ctx.as_account_view(),
            p_prog.as_account_view(),
        )
        .commit(&commit_accs)
        .add_post_commit_actions(&[commit_handler])
        .unwrap()
        .commit_and_undelegate(&cau_accs)
        .unwrap()
        .add_post_undelegate_actions(&[undelegate_handler])
        .unwrap()
        .build_serialized(&mut buf);

        // --- SDK ---
        let mut s_payer = SdkTestAccount::new(payer_addr);
        let mut s_ctx = SdkTestAccount::new(ctx_addr);
        let mut s_commit = SdkTestAccount::new(commit_acc_addr);
        let mut s_cau = SdkTestAccount::new(cau_acc_addr);
        let mut s_escrow1 = SdkTestAccount::new(escrow1_addr);
        let mut s_escrow2 = SdkTestAccount::new(escrow2_addr);
        let mut s_prog = SdkTestAccount::new(prog_addr);

        let sdk_commit_handler = SdkCallHandler {
            args: SdkActionArgs::new(commit_data.to_vec()),
            compute_units: 100_000,
            escrow_authority: s_escrow1.as_signer_info(),
            destination_program: Pubkey::new_from_array(dest1_addr),
            accounts: vec![],
        };
        let sdk_undelegate_handler = SdkCallHandler {
            args: SdkActionArgs::new(undelegate_data.to_vec()),
            compute_units: 50_000,
            escrow_authority: s_escrow2.as_signer_info(),
            destination_program: Pubkey::new_from_array(dest2_addr),
            accounts: vec![],
        };
        let (accounts, ix) = SdkBuilder::new(
            s_payer.as_account_info(),
            s_ctx.as_account_info(),
            s_prog.as_account_info(),
        )
        .commit(&[s_commit.as_account_info()])
        .add_post_commit_actions([sdk_commit_handler])
        .commit_and_undelegate(&[s_cau.as_account_info()])
        .add_post_undelegate_actions([sdk_undelegate_handler])
        .build();
        drop(accounts);

        assert_eq!(
            &buf[..pino_len],
            &ix.data,
            "full chain with actions mismatch"
        );
    }
}
