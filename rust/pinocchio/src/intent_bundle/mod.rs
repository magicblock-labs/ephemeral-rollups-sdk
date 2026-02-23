use crate::intent_bundle::no_vec::{CapacityError, NoVec};
use pinocchio::cpi::{invoke_with_bounds, MAX_STATIC_CPI_ACCOUNTS};
use pinocchio::error::ProgramError;
use pinocchio::instruction::{InstructionAccount, InstructionView};
use pinocchio::{AccountView, ProgramResult};
use solana_address::Address;

mod args;
mod commit;
mod commit_and_undelegate;
mod no_vec;
pub mod types;

use crate::intent_bundle::args::MagicIntentBundleArgs;
use crate::intent_bundle::commit::CommitIntentBuilder;
use crate::intent_bundle::commit_and_undelegate::CommitAndUndelegateIntentBuilder;
pub use args::{ActionArgs, ShortAccountMeta};
use types::MagicIntentBundle;
pub use types::*;

const MAX_ACTIONS_NUM: usize = 10;
const _: () = assert!(MAX_ACTIONS_NUM <= u8::MAX as usize);

/// Bincode 1.x u32 LE discriminant for `MagicBlockInstruction::ScheduleIntentBundle` (variant index 11).
const SCHEDULE_INTENT_BUNDLE_DISCRIMINANT: [u8; 4] = 11u32.to_le_bytes();

/// Builds a single `MagicBlockInstruction::ScheduleIntentBundle` instruction by aggregating
/// multiple independent intents (base actions, commits, commit+undelegate), normalizing them,
/// and producing a deduplicated account list plus the corresponding CPI `Instruction`.
pub struct MagicIntentBundleBuilder<'acc, 'args> {
    payer: AccountView,
    magic_context: AccountView,
    magic_program: AccountView,
    intent_bundle: MagicIntentBundle<'acc, 'args>,
}

impl MagicIntentBundleBuilder<'static, 'static> {
    pub fn new(payer: AccountView, magic_context: AccountView, magic_program: AccountView) -> Self {
        Self {
            payer,
            magic_context,
            magic_program,
            intent_bundle: MagicIntentBundle::default(),
        }
    }
}

impl<'acc, 'args> MagicIntentBundleBuilder<'acc, 'args> {
    /// Starts building a Commit intent. Returns a [`CommitIntentBuilder`] that owns this parent.
    ///
    /// The returned builder lets you chain `.add_post_commit_actions()`, transition to other
    /// intents via `.commit_and_undelegate()`, or finalize via `.build_and_invoke()`.
    pub fn commit<'new_acc>(
        self,
        accounts: &'new_acc [AccountView],
    ) -> CommitIntentBuilder<'new_acc, 'args, &'static [CallHandler<'static>]>
    where
        'acc: 'new_acc,
    {
        CommitIntentBuilder::new(self, accounts)
    }

    /// Starts building a CommitAndUndelegate intent. Returns a [`CommitAndUndelegateIntentBuilder`]
    /// that owns this parent.
    ///
    /// The returned builder lets you chain `.add_post_commit_actions()`,
    /// `.add_post_undelegate_actions()`, transition to other intents via `.commit()`,
    /// or finalize via `.build_and_invoke()`.
    pub fn commit_and_undelegate<'new_acc>(
        self,
        accounts: &'new_acc [AccountView],
    ) -> CommitAndUndelegateIntentBuilder<
        'new_acc,
        'args,
        &'static [CallHandler<'static>],
        &'static [CallHandler<'static>],
    >
    where
        'acc: 'new_acc,
    {
        CommitAndUndelegateIntentBuilder::new(self, accounts)
    }

    /// Adds standalone base-layer actions to be executed without any commit/undelegate semantics.
    pub fn set_standalone_actions<'new_args>(
        self,
        actions: &'new_args [CallHandler<'new_args>],
    ) -> MagicIntentBundleBuilder<'acc, 'new_args>
    where
        'args: 'new_args,
    {
        let MagicIntentBundle {
            standalone_actions: _,
            commit_intent,
            commit_and_undelegate_intent,
        } = self.intent_bundle;

        MagicIntentBundleBuilder {
            payer: self.payer,
            magic_program: self.magic_program,
            magic_context: self.magic_context,
            intent_bundle: MagicIntentBundle {
                standalone_actions: actions,
                commit_intent,
                commit_and_undelegate_intent,
            },
        }
    }

    pub fn into_args(
        all_accounts: &[AccountView],
        intent_bundle: MagicIntentBundle<'_, 'args>,
    ) -> Result<MagicIntentBundleArgs<'args>, ProgramError> {
        let mut indices_map = NoVec::<Address, MAX_STATIC_CPI_ACCOUNTS>::new();
        for account in all_accounts {
            indices_map.try_push(account.address().clone())?;
        }

        // 4. Convert intents to serializable args
        intent_bundle.into_args(indices_map.as_slice())
    }

    pub fn encode_into_slice(
        all_accounts: &[AccountView],
        intent_bundle: MagicIntentBundle<'_, 'args>,
        data_buf: &mut [u8],
    ) -> Result<usize, ProgramError> {
        const OFFSET: usize = SCHEDULE_INTENT_BUNDLE_DISCRIMINANT.len();

        let mut indices_map = NoVec::<Address, MAX_STATIC_CPI_ACCOUNTS>::new();
        for account in all_accounts {
            indices_map.try_push(account.address().clone())?;
        }

        // 4. Convert intents to serializable args
        let args = intent_bundle.into_args(indices_map.as_slice())?;

        data_buf[..OFFSET].copy_from_slice(&SCHEDULE_INTENT_BUNDLE_DISCRIMINANT);
        let args_len =
            bincode::encode_into_slice(&args, &mut data_buf[OFFSET..], bincode::config::legacy())
                .map_err(|_| ProgramError::InvalidInstructionData)?;

        Ok(OFFSET + args_len)
    }

    /// Normalizes the bundle, serializes it with bincode into `data_buf`, builds the
    /// CPI instruction, and invokes the magic program.
    ///
    /// `data_buf` must be large enough to hold the serialized `MagicIntentBundleArgs`.
    pub fn build_and_invoke(self, data_buf: &mut [u8]) -> ProgramResult {
        const OFFSET: usize = SCHEDULE_INTENT_BUNDLE_DISCRIMINANT.len();

        // Guard: buffer must be large enough for at least the discriminant plus
        // one byte of payload; otherwise the slice indexing below would panic.
        if data_buf.len() <= OFFSET {
            return Err(ProgramError::InvalidInstructionData);
        }

        // Validate: ensure intents have at least one committed account, no overlap
        self.intent_bundle.validate()?;

        // Collect all unique accounts (payer + context first, then from intents)
        let mut all_accounts = NoVec::<AccountView, MAX_STATIC_CPI_ACCOUNTS>::new();
        all_accounts.try_append([self.payer, self.magic_context])?;
        self.intent_bundle
            .collect_unique_accounts(&mut all_accounts)?;

        // 3. Build the natural indices map: indices_map[i] = address of account at position i
        let data_len =
            Self::encode_into_slice(all_accounts.as_slice(), self.intent_bundle, data_buf)?;
        // let args = Self::into_args(all_accounts.as_slice(), self.intent_bundle)?;
        //
        // // 5. Write instruction discriminant + serialize args (bincode 1.x wire compat)
        // data_buf[..OFFSET].copy_from_slice(&SCHEDULE_INTENT_BUNDLE_DISCRIMINANT);
        // let args_len =
        //     bincode::encode_into_slice(&args, &mut data_buf[OFFSET..], bincode::config::legacy())
        //         .map_err(|_| ProgramError::InvalidInstructionData)?;
        // let data_len = OFFSET + args_len;

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

/// Custom error code returned when a `NoVec` capacity limit is exceeded.
pub const CAPACITY_EXCEEDED_ERROR: u32 = 0xEB_00_00_01;

impl<T> From<CapacityError<T>> for ProgramError {
    fn from(_: CapacityError<T>) -> Self {
        ProgramError::Custom(CAPACITY_EXCEEDED_ERROR)
    }
}

// ---------------------------------------------------------------------------
// Test-only: serialize builder output without CPI
// ---------------------------------------------------------------------------

#[cfg(test)]
impl MagicIntentBundleBuilder<'_, '_> {
    /// Reproduces the logic of `build_and_invoke` but serializes the
    /// `MagicIntentBundleArgs` into the provided buffer instead of invoking CPI.
    /// Returns the number of bytes written.
    fn build_serialized(self, buf: &mut [u8]) -> usize {
        self.intent_bundle.validate().unwrap();

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
impl<'acc, 'args>
    CommitAndUndelegateIntentBuilder<
        'acc,
        'args,
        &'args [CallHandler<'args>],
        &'args [CallHandler<'args>],
    >
{
    fn build_serialized(self, buf: &mut [u8]) -> usize {
        self.fold().build_serialized(buf)
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

    // SDK builder
    use ephemeral_rollups_sdk::ephem::{
        CallHandler as SdkCallHandler, MagicIntentBundleBuilder as SdkBuilder,
    };
    use magicblock_magic_program_api::args::ActionArgs as SdkActionArgs;
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

    #[test]
    fn test() {
        assert_eq!(
            size_of::<NoVec::<&Address, MAX_STATIC_CPI_ACCOUNTS>>(),
            8 * 64 + 4 + 4
        );
        assert_eq!(
            size_of::<NoVec::<Address, MAX_STATIC_CPI_ACCOUNTS>>(),
            32 * 64 + 4 + 4
        );
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
        let handler = CallHandler {
            destination_program: Address::new_from_array(dest_addr),
            escrow_authority: escrow_view,
            args: ActionArgs::new(&action_data),
            compute_units: 200_000,
            accounts: &[],
        };
        let commit_accs = [p_acc1.as_account_view()];
        let mut buf = [0u8; CPI_DATA_BUF_SIZE];

        let pino_len = MagicIntentBundleBuilder::new(
            p_payer.as_account_view(),
            p_ctx.as_account_view(),
            p_prog.as_account_view(),
        )
        .commit(&commit_accs)
        .add_post_commit_actions(&[handler])
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

        let post_commit = CallHandler {
            destination_program: Address::new_from_array(dest1_addr),
            escrow_authority: p_escrow1.as_account_view(),
            args: ActionArgs::new(&commit_data),
            compute_units: 100_000,
            accounts: &[],
        };
        let post_undelegate = CallHandler {
            destination_program: Address::new_from_array(dest2_addr),
            escrow_authority: p_escrow2.as_account_view(),
            args: ActionArgs::new(&undelegate_data),
            compute_units: 50_000,
            accounts: &[],
        };
        let cau_accs = [p_acc1.as_account_view()];
        let mut buf = [0u8; CPI_DATA_BUF_SIZE];
        let pino_len = MagicIntentBundleBuilder::new(
            p_payer.as_account_view(),
            p_ctx.as_account_view(),
            p_prog.as_account_view(),
        )
        .commit_and_undelegate(&cau_accs)
        .add_post_commit_actions(&[post_commit])
        .add_post_undelegate_actions(&[post_undelegate])
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

        let commit_handler = CallHandler {
            destination_program: Address::new_from_array(dest1_addr),
            escrow_authority: p_escrow1.as_account_view(),
            args: ActionArgs::new(&commit_data),
            compute_units: 100_000,
            accounts: &[],
        };
        let undelegate_handler = CallHandler {
            destination_program: Address::new_from_array(dest2_addr),
            escrow_authority: p_escrow2.as_account_view(),
            args: ActionArgs::new(&undelegate_data),
            compute_units: 50_000,
            accounts: &[],
        };
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
        .commit_and_undelegate(&cau_accs)
        .add_post_undelegate_actions(&[undelegate_handler])
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

    /// Demonstrates that the builder API supports conditional building:
    /// actions can be optionally attached based on runtime state.
    #[test]
    fn test_conditional_building() {
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

        let mut p_payer = MockRuntimeAccount::new(payer_addr);
        let mut p_ctx = MockRuntimeAccount::new(ctx_addr);
        let mut p_commit = MockRuntimeAccount::new(commit_acc_addr);
        let mut p_cau = MockRuntimeAccount::new(cau_acc_addr);
        let mut p_escrow1 = MockRuntimeAccount::new(escrow1_addr);
        let mut p_escrow2 = MockRuntimeAccount::new(escrow2_addr);
        let mut p_prog = MockRuntimeAccount::new(prog_addr);

        let post_commit_handler = CallHandler {
            destination_program: Address::new_from_array(dest1_addr),
            escrow_authority: p_escrow1.as_account_view(),
            args: ActionArgs::new(&commit_data),
            compute_units: 100_000,
            accounts: &[],
        };
        let post_undelegate_handler = CallHandler {
            destination_program: Address::new_from_array(dest2_addr),
            escrow_authority: p_escrow2.as_account_view(),
            args: ActionArgs::new(&undelegate_data),
            compute_units: 50_000,
            accounts: &[],
        };

        let commit_accs = [p_commit.as_account_view()];
        let cau_accs = [p_cau.as_account_view()];
        let post_commit_actions = &[post_commit_handler];
        let post_undelegate_actions = &[post_undelegate_handler];

        // Runtime conditions that control which optional actions get attached
        let should_add_post_commit = true;
        let should_add_post_undelegate = false;

        let mut buf = [0u8; CPI_DATA_BUF_SIZE];

        // Build commit intent - conditionally attach post-commit actions
        let builder = MagicIntentBundleBuilder::new(
            p_payer.as_account_view(),
            p_ctx.as_account_view(),
            p_prog.as_account_view(),
        )
        .commit(&commit_accs);

        let builder = if should_add_post_commit {
            builder.add_post_commit_actions(post_commit_actions)
        } else {
            builder.fold()
        };

        // Build commit-and-undelegate intent - conditionally attach post-undelegate actions
        let builder = builder.commit_and_undelegate(&cau_accs);

        let builder = if should_add_post_undelegate {
            builder.add_post_undelegate_actions(post_undelegate_actions)
        } else {
            builder
        };

        let _len = builder.build_serialized(&mut buf);
        std::hint::black_box(&buf);
    }
}
