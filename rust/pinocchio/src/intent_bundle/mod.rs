use crate::intent_bundle::no_vec::{CapacityError, NoVec};
use pinocchio::cpi::{invoke_signed_with_bounds, Signer, MAX_STATIC_CPI_ACCOUNTS};
use pinocchio::error::ProgramError;
use pinocchio::instruction::{InstructionAccount, InstructionView};
use pinocchio::{AccountView, ProgramResult};
use solana_address::Address;

mod args;
mod commit;
mod commit_and_undelegate;
mod no_vec;
mod serialize;
pub mod types;

use crate::intent_bundle::commit::CommitIntentBuilder;
use crate::intent_bundle::commit_and_undelegate::CommitAndUndelegateIntentBuilder;
use crate::intent_bundle::serialize::{MagicIntentBundleSerialize, DISCRIMINANT_SIZE};
pub use args::{ActionArgs, ShortAccountMeta};
use types::MagicIntentBundle;
pub use types::{
    ActionCallback, CallHandler, CommitAndUndelegateIntent, CommitIntent, MagicIntent,
};

const MAX_ACTIONS_NUM: usize = 10;
const _: () = assert!(MAX_ACTIONS_NUM <= u8::MAX as usize);
/// Custom error code returned when a `NoVec` capacity limit is exceeded.
pub const CAPACITY_EXCEEDED_ERROR: u32 = 0xEB_00_00_01;

/// Builds a single `MagicBlockInstruction::ScheduleIntentBundle` instruction by aggregating
/// multiple independent intents (base actions, commits, commit+undelegate), normalizing them,
/// and producing a deduplicated account list plus the corresponding CPI `Instruction`.
pub struct MagicIntentBundleBuilder<'acc, 'args> {
    payer: AccountView,
    magic_context: AccountView,
    magic_program: AccountView,
    magic_fee_vault: Option<AccountView>,
    intent_bundle: MagicIntentBundle<'acc, 'args>,
}

impl MagicIntentBundleBuilder<'static, 'static> {
    pub fn new(payer: AccountView, magic_context: AccountView, magic_program: AccountView) -> Self {
        Self {
            payer,
            magic_context,
            magic_program,
            magic_fee_vault: None,
            intent_bundle: MagicIntentBundle::default(),
        }
    }
}

impl<'acc, 'args> MagicIntentBundleBuilder<'acc, 'args> {
    /// Sets an optional magic fee vault account to be passed as the account at index 2
    /// (right after payer and magic_context). Required when the payer is delegated.
    pub fn magic_fee_vault(mut self, vault: AccountView) -> Self {
        self.magic_fee_vault = Some(vault);
        self
    }

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
            magic_fee_vault: self.magic_fee_vault,
            intent_bundle: MagicIntentBundle {
                standalone_actions: actions,
                commit_intent,
                commit_and_undelegate_intent,
            },
        }
    }

    /// Collects all unique accounts in intent bundle.
    #[inline(never)]
    fn collect_unique_account(
        &self,
    ) -> Result<NoVec<AccountView, MAX_STATIC_CPI_ACCOUNTS>, ProgramError> {
        let mut all_accounts = NoVec::<AccountView, MAX_STATIC_CPI_ACCOUNTS>::new();
        all_accounts.try_append([self.payer.clone(), self.magic_context.clone()])?;
        if let Some(ref vault) = self.magic_fee_vault {
            all_accounts.try_push(vault.clone())?;
        }
        self.intent_bundle
            .collect_unique_accounts(&mut all_accounts)?;

        Ok(all_accounts)
    }

    /// Normalizes the bundle, serializes it with bincode into `data_buf`, builds the
    /// CPI instruction, and invokes the magic program.
    ///
    /// `data_buf` must be large enough to hold the serialized `MagicIntentBundleArgs`.
    #[inline(never)]
    pub fn build_and_invoke(self, data_buf: &mut [u8]) -> ProgramResult {
        self.build_and_invoke_impl(data_buf, &[])
    }

    /// Equivalent to [`Self::build_and_invoke`], but signs the CPI with the
    /// provided PDA seeds.
    #[inline(never)]
    pub fn build_and_invoke_signed(
        self,
        data_buf: &mut [u8],
        signers_seeds: &[Signer<'_, '_>],
    ) -> ProgramResult {
        self.build_and_invoke_impl(data_buf, signers_seeds)
    }

    fn build_and_invoke_impl(
        self,
        data_buf: &mut [u8],
        signers_seeds: &[Signer<'_, '_>],
    ) -> ProgramResult {
        if data_buf.len() <= DISCRIMINANT_SIZE {
            return Err(ProgramError::InvalidInstructionData);
        }

        self.intent_bundle.validate()?;

        let all_accounts = self.collect_unique_account()?;
        self.invoke_all(&all_accounts, signers_seeds, data_buf)
    }

    #[inline(never)]
    fn invoke_all(
        self,
        all_accounts: &[AccountView],
        signers_seeds: &[Signer<'_, '_>],
        data_buf: &mut [u8],
    ) -> ProgramResult {
        let indices_map = create_indices_map(all_accounts)?;
        let serializable_intent = MagicIntentBundleSerialize::new(&indices_map, self.intent_bundle);
        let len = serializable_intent.encode_intent_into_slice(data_buf)?;
        Self::invoke_cpi_signed(
            all_accounts,
            self.magic_program.address(),
            &data_buf[..len],
            signers_seeds,
        )?;

        // Callback CPIs only need payer + magic_context (+ optional vault), not all bundle accounts.
        let mut callback_accounts = NoVec::<AccountView, 3>::new();
        callback_accounts.append([self.payer, self.magic_context]);
        if let Some(vault) = self.magic_fee_vault {
            callback_accounts.push(vault);
        }

        for (i, callback) in serializable_intent.action_callback_iter() {
            let len = callback.args(i as u8)?.encode_into_slice(data_buf)?;
            Self::invoke_cpi_signed(
                callback_accounts.as_slice(),
                self.magic_program.address(),
                &data_buf[..len],
                signers_seeds,
            )?;
        }
        Ok(())
    }

    /// Builds `instruction_accounts` + `ix`, then delegates to
    /// [`Self::do_invoke_signed`].
    #[inline(never)]
    fn invoke_cpi_signed(
        all_accounts: &[AccountView],
        program_id: &Address,
        data: &[u8],
        signers_seeds: &[Signer<'_, '_>],
    ) -> ProgramResult {
        let instruction_accounts = Self::instruction_accounts(all_accounts)?;

        let mut account_refs = NoVec::<&AccountView, MAX_STATIC_CPI_ACCOUNTS>::new();
        for account in all_accounts.iter() {
            account_refs.try_push(account)?;
        }

        let ix = InstructionView {
            program_id,
            data,
            accounts: instruction_accounts.as_slice(),
        };

        Self::do_invoke_signed(&ix, account_refs.as_slice(), signers_seeds)
    }

    /// Builds the CPI account metas for the magic program.
    ///
    /// The first account is always the magic payer, so it must be marked as a
    /// signer in the CPI instruction even when the backing `AccountView` is a
    /// PDA that will sign via `invoke_signed`.
    #[inline(never)]
    fn instruction_accounts(
        all_accounts: &[AccountView],
    ) -> Result<NoVec<InstructionAccount<'_>, MAX_STATIC_CPI_ACCOUNTS>, ProgramError> {
        let mut instruction_accounts = NoVec::<InstructionAccount, MAX_STATIC_CPI_ACCOUNTS>::new();
        let Some((payer, remaining_accounts)) = all_accounts.split_first() else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        instruction_accounts.try_push(InstructionAccount::new(
            payer.address(),
            payer.is_writable(),
            true,
        ))?;

        for account in remaining_accounts.iter() {
            instruction_accounts.try_push(InstructionAccount::from(account))?;
        }

        Ok(instruction_accounts)
    }

    /// Thin `#[inline(never)]` wrapper around [`invoke_with_bounds`] so its internal
    /// locals (large fixed-size arrays) live in their own stack frame, separate from
    /// [`Self::invoke_cpi`].
    #[inline(never)]
    fn do_invoke_signed(
        ix: &InstructionView,
        account_refs: &[&AccountView],
        signers_seeds: &[Signer<'_, '_>],
    ) -> ProgramResult {
        invoke_signed_with_bounds::<MAX_STATIC_CPI_ACCOUNTS>(ix, account_refs, signers_seeds)
    }
}

impl<T> From<CapacityError<T>> for ProgramError {
    fn from(_: CapacityError<T>) -> Self {
        ProgramError::Custom(CAPACITY_EXCEEDED_ERROR)
    }
}

#[inline(never)]
fn create_indices_map(
    accounts: &[AccountView],
) -> Result<NoVec<&Address, MAX_STATIC_CPI_ACCOUNTS>, ProgramError> {
    let mut indices_map = NoVec::<&Address, MAX_STATIC_CPI_ACCOUNTS>::new();
    for account in accounts {
        indices_map.try_push(account.address())?;
    }
    Ok(indices_map)
}

// ---------------------------------------------------------------------------
// Test-only: serialize builder output without CPI
// ---------------------------------------------------------------------------

#[cfg(test)]
impl MagicIntentBundleBuilder<'_, '_> {
    /// Mirrors `build_and_invoke` exactly (streaming `encode_into_slice` /
    /// `MagicIntentBundleSerialize`) but writes into `buf` instead of invoking CPI.
    /// Returns `(bytes_written, cpi_account_keys)` so tests can verify both the
    /// instruction data and the CPI account list against the SDK reference.
    fn build_serialized(self, buf: &mut [u8]) -> (usize, NoVec<Address, MAX_STATIC_CPI_ACCOUNTS>) {
        self.intent_bundle.validate().unwrap();
        let all_accounts = self.collect_unique_account().unwrap();
        let mut account_keys = NoVec::<Address, MAX_STATIC_CPI_ACCOUNTS>::new();
        for account in all_accounts.iter() {
            account_keys.push(account.address().clone());
        }
        let indices_map = create_indices_map(all_accounts.as_slice()).unwrap();
        let serializable = MagicIntentBundleSerialize::new(&indices_map, self.intent_bundle);
        let len = serializable.encode_intent_into_slice(buf).unwrap();
        (len, account_keys)
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
    fn build_serialized(self, buf: &mut [u8]) -> (usize, NoVec<Address, MAX_STATIC_CPI_ACCOUNTS>) {
        self.fold().build_serialized(buf)
    }
}

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
        ActionCallback as SdkActionCallback, CallHandler as SdkCallHandler,
        FoldableCauIntentBuilder, FoldableIntentBuilder, IntentInstructions,
        MagicIntentBundleBuilder as SdkBuilder,
    };
    use magicblock_magic_program_api::args::ActionArgs as SdkActionArgs;
    use magicblock_magic_program_api::Pubkey;
    use solana_program::account_info::AccountInfo;

    use crate::intent_bundle::serialize::MagicIntentBundleSerialize;

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
        fn new_unique() -> Self {
            Self::new(Pubkey::new_unique().to_bytes())
        }

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

    /// Commit with a post-commit action (handler).
    #[test]
    fn test_compat_commit_with_handler() {
        let dest_addr = [0xDD; 32];
        let action_data = [0xAA, 0xBB, 0xCC];

        // --- Pinocchio ---
        let mut p_payer = MockRuntimeAccount::new_unique();
        let mut p_ctx = MockRuntimeAccount::new_unique();
        let mut p_acc1 = MockRuntimeAccount::new_unique();
        let mut p_escrow = MockRuntimeAccount::new_unique();
        let mut p_prog = MockRuntimeAccount::new_unique();

        let escrow_view = p_escrow.as_account_view();
        let handler = CallHandler {
            destination_program: Address::new_from_array(dest_addr),
            escrow_authority: escrow_view,
            args: ActionArgs::new(&action_data),
            compute_units: 200_000,
            accounts: &[],
            callback: None,
        };
        let commit_accs = [p_acc1.as_account_view()];
        let mut buf = [0u8; CPI_DATA_BUF_SIZE];

        let (pino_len, pino_accounts) = MagicIntentBundleBuilder::new(
            p_payer.as_account_view(),
            p_ctx.as_account_view(),
            p_prog.as_account_view(),
        )
        .commit(&commit_accs)
        .add_post_commit_actions(&[handler])
        .build_serialized(&mut buf);

        // --- SDK ---
        let mut s_payer = SdkTestAccount::new(p_payer.address);
        let mut s_ctx = SdkTestAccount::new(p_ctx.address);
        let mut s_acc1 = SdkTestAccount::new(p_acc1.address);
        let mut s_escrow = SdkTestAccount::new(p_escrow.address);
        let mut s_prog = SdkTestAccount::new(p_prog.address);

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
        .build()
        .schedule_intent_ix;

        assert_eq!(&buf[..pino_len], &ix.data, "commit with handler mismatch");
        let sdk_addrs: Vec<Address> = accounts
            .iter()
            .map(|a| Address::new_from_array(a.key.to_bytes()))
            .collect();
        assert_eq!(
            pino_accounts.as_slice(),
            sdk_addrs.as_slice(),
            "commit with handler: account list mismatch"
        );
    }

    /// CommitAndUndelegate with post-commit and post-undelegate actions.
    #[test]
    fn test_compat_commit_and_undelegate_with_actions() {
        let dest1_addr = [0xAA; 32];
        let dest2_addr = [0xBB; 32];
        let commit_data = [1u8, 2, 3];
        let undelegate_data = [4u8, 5, 6];

        // --- Pinocchio ---
        let mut p_payer = MockRuntimeAccount::new_unique();
        let mut p_ctx = MockRuntimeAccount::new_unique();
        let mut p_acc1 = MockRuntimeAccount::new_unique();
        let mut p_escrow1 = MockRuntimeAccount::new_unique();
        let mut p_escrow2 = MockRuntimeAccount::new_unique();
        let mut p_prog = MockRuntimeAccount::new_unique();

        let post_commit = CallHandler {
            destination_program: Address::new_from_array(dest1_addr),
            escrow_authority: p_escrow1.as_account_view(),
            args: ActionArgs::new(&commit_data),
            compute_units: 100_000,
            accounts: &[],
            callback: None,
        };
        let post_undelegate = CallHandler {
            destination_program: Address::new_from_array(dest2_addr),
            escrow_authority: p_escrow2.as_account_view(),
            args: ActionArgs::new(&undelegate_data),
            compute_units: 50_000,
            accounts: &[],
            callback: None,
        };
        let cau_accs = [p_acc1.as_account_view()];
        let mut buf = [0u8; CPI_DATA_BUF_SIZE];
        let (pino_len, pino_accounts) = MagicIntentBundleBuilder::new(
            p_payer.as_account_view(),
            p_ctx.as_account_view(),
            p_prog.as_account_view(),
        )
        .commit_and_undelegate(&cau_accs)
        .add_post_commit_actions(&[post_commit])
        .add_post_undelegate_actions(&[post_undelegate])
        .build_serialized(&mut buf);

        // --- SDK ---
        let mut s_payer = SdkTestAccount::new(p_payer.address);
        let mut s_ctx = SdkTestAccount::new(p_ctx.address);
        let mut s_acc1 = SdkTestAccount::new(p_acc1.address);
        let mut s_escrow1 = SdkTestAccount::new(p_escrow1.address);
        let mut s_escrow2 = SdkTestAccount::new(p_escrow2.address);
        let mut s_prog = SdkTestAccount::new(p_prog.address);

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
        .build()
        .schedule_intent_ix;

        assert_eq!(
            &buf[..pino_len],
            &ix.data,
            "commit_and_undelegate with actions mismatch"
        );
        let sdk_addrs: Vec<Address> = accounts
            .iter()
            .map(|a| Address::new_from_array(a.key.to_bytes()))
            .collect();
        assert_eq!(
            pino_accounts.as_slice(),
            sdk_addrs.as_slice(),
            "commit_and_undelegate: account list mismatch"
        );
    }

    /// Full chain with actions on all intents.
    #[test]
    fn test_compat_full_chain_with_actions() {
        let dest1_addr = [0xC1; 32];
        let dest2_addr = [0xD1; 32];
        let commit_data = [0xC0u8];
        let undelegate_data = [0xD0u8];

        // --- Pinocchio ---
        let mut p_payer = MockRuntimeAccount::new_unique();
        let mut p_ctx = MockRuntimeAccount::new_unique();
        let mut p_commit = MockRuntimeAccount::new_unique();
        let mut p_cau = MockRuntimeAccount::new_unique();
        let mut p_escrow1 = MockRuntimeAccount::new_unique();
        let mut p_escrow2 = MockRuntimeAccount::new_unique();
        let mut p_prog = MockRuntimeAccount::new_unique();

        let commit_handler = CallHandler {
            destination_program: Address::new_from_array(dest1_addr),
            escrow_authority: p_escrow1.as_account_view(),
            args: ActionArgs::new(&commit_data),
            compute_units: 100_000,
            accounts: &[],
            callback: None,
        };
        let undelegate_handler = CallHandler {
            destination_program: Address::new_from_array(dest2_addr),
            escrow_authority: p_escrow2.as_account_view(),
            args: ActionArgs::new(&undelegate_data),
            compute_units: 50_000,
            accounts: &[],
            callback: None,
        };
        let commit_accs = [p_commit.as_account_view()];
        let cau_accs = [p_cau.as_account_view()];
        let mut buf = [0u8; CPI_DATA_BUF_SIZE];
        let (pino_len, pino_accounts) = MagicIntentBundleBuilder::new(
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
        let mut s_payer = SdkTestAccount::new(p_payer.address);
        let mut s_ctx = SdkTestAccount::new(p_ctx.address);
        let mut s_commit = SdkTestAccount::new(p_commit.address);
        let mut s_cau = SdkTestAccount::new(p_cau.address);
        let mut s_escrow1 = SdkTestAccount::new(p_escrow1.address);
        let mut s_escrow2 = SdkTestAccount::new(p_escrow2.address);
        let mut s_prog = SdkTestAccount::new(p_prog.address);

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
        .build()
        .schedule_intent_ix;

        assert_eq!(
            &buf[..pino_len],
            &ix.data,
            "full chain with actions mismatch"
        );
        let sdk_addrs: Vec<Address> = accounts
            .iter()
            .map(|a| Address::new_from_array(a.key.to_bytes()))
            .collect();
        assert_eq!(
            pino_accounts.as_slice(),
            sdk_addrs.as_slice(),
            "full chain: account list mismatch"
        );
    }

    /// Demonstrates that the builder API supports conditional building:
    /// actions can be optionally attached based on runtime state.
    #[test]
    fn test_conditional_building() {
        let dest1_addr = [0xC1; 32];
        let dest2_addr = [0xD1; 32];
        let commit_data = [0xC0u8];
        let undelegate_data = [0xD0u8];

        let mut p_payer = MockRuntimeAccount::new_unique();
        let mut p_ctx = MockRuntimeAccount::new_unique();
        let mut p_commit = MockRuntimeAccount::new_unique();
        let mut p_cau = MockRuntimeAccount::new_unique();
        let mut p_escrow1 = MockRuntimeAccount::new_unique();
        let mut p_escrow2 = MockRuntimeAccount::new_unique();
        let mut p_prog = MockRuntimeAccount::new_unique();

        let post_commit_handler = CallHandler {
            destination_program: Address::new_from_array(dest1_addr),
            escrow_authority: p_escrow1.as_account_view(),
            args: ActionArgs::new(&commit_data),
            compute_units: 100_000,
            accounts: &[],
            callback: None,
        };
        let post_undelegate_handler = CallHandler {
            destination_program: Address::new_from_array(dest2_addr),
            escrow_authority: p_escrow2.as_account_view(),
            args: ActionArgs::new(&undelegate_data),
            compute_units: 50_000,
            accounts: &[],
            callback: None,
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

        let (_len, _accounts) = builder.build_serialized(&mut buf);
        std::hint::black_box(&buf);
    }

    /// Verifies pinocchio-vs-SDK parity when magic_fee_vault is set:
    /// the vault must appear at index 2 in both the serialized payload and the CPI account list.
    #[test]
    fn test_compat_commit_with_magic_fee_vault() {
        // --- Pinocchio ---
        let mut p_payer = MockRuntimeAccount::new_unique();
        let mut p_ctx = MockRuntimeAccount::new_unique();
        let mut p_vault = MockRuntimeAccount::new_unique();
        let mut p_acc1 = MockRuntimeAccount::new_unique();
        let mut p_prog = MockRuntimeAccount::new_unique();

        let commit_accs = [p_acc1.as_account_view()];
        let mut buf = [0u8; CPI_DATA_BUF_SIZE];

        let (pino_len, pino_accounts) = MagicIntentBundleBuilder::new(
            p_payer.as_account_view(),
            p_ctx.as_account_view(),
            p_prog.as_account_view(),
        )
        .magic_fee_vault(p_vault.as_account_view())
        .commit(&commit_accs)
        .fold()
        .build_serialized(&mut buf);

        // Vault must be at index 2 in the CPI account list
        assert_eq!(
            pino_accounts.as_slice()[2],
            Address::new_from_array(p_vault.address),
            "vault should be at index 2 in pinocchio account list"
        );

        // --- SDK ---
        let mut s_payer = SdkTestAccount::new(p_payer.address);
        let mut s_ctx = SdkTestAccount::new(p_ctx.address);
        let mut s_vault = SdkTestAccount::new(p_vault.address);
        let mut s_acc1 = SdkTestAccount::new(p_acc1.address);
        let mut s_prog = SdkTestAccount::new(p_prog.address);

        let (accounts, ix) = SdkBuilder::new(
            s_payer.as_account_info(),
            s_ctx.as_account_info(),
            s_prog.as_account_info(),
        )
        .magic_fee_vault(s_vault.as_account_info())
        .commit(&[s_acc1.as_account_info()])
        .build()
        .schedule_intent_ix;

        assert_eq!(
            &buf[..pino_len],
            &ix.data,
            "commit with magic_fee_vault: serialized data mismatch"
        );

        let sdk_addrs: Vec<Address> = accounts
            .iter()
            .map(|a| Address::new_from_array(a.key.to_bytes()))
            .collect();
        assert_eq!(
            pino_accounts.as_slice(),
            sdk_addrs.as_slice(),
            "commit with magic_fee_vault: account list mismatch"
        );
        assert_eq!(
            sdk_addrs[2],
            Address::new_from_array(p_vault.address),
            "vault should be at index 2 in SDK account list"
        );
    }

    // -----------------------------------------------------------------
    // Callback serialization tests
    // -----------------------------------------------------------------

    /// Commit with one action carrying a callback: verify the `ScheduleIntentBundle`
    /// bytes and the `AddActionCallback` instruction bytes match the SDK output.
    #[test]
    fn test_compat_commit_with_callback() {
        let dest_addr = [0xDD; 32];
        let cb_dest_addr = [0xCB; 32];
        let action_data = [0xAA, 0xBB];
        let cb_disc = [0x01u8, 0x02];
        let cb_payload = [0x10u8, 0x20, 0x30];

        // --- Pinocchio ---
        let mut p_payer = MockRuntimeAccount::new_unique();
        let mut p_ctx = MockRuntimeAccount::new_unique();
        let mut p_acc = MockRuntimeAccount::new_unique();
        let mut p_escrow = MockRuntimeAccount::new_unique();
        let mut p_prog = MockRuntimeAccount::new_unique();

        let handler = CallHandler {
            destination_program: Address::new_from_array(dest_addr),
            escrow_authority: p_escrow.as_account_view(),
            args: ActionArgs::new(&action_data),
            compute_units: 200_000,
            accounts: &[],
            callback: Some(ActionCallback {
                destination_program: Address::new_from_array(cb_dest_addr),
                discriminator: &cb_disc,
                payload: &cb_payload,
                compute_units: 50_000,
                accounts: &[],
            }),
        };
        let commit_accs = [p_acc.as_account_view()];
        let handlers = [handler];
        let builder = MagicIntentBundleBuilder::new(
            p_payer.as_account_view(),
            p_ctx.as_account_view(),
            p_prog.as_account_view(),
        )
        .commit(&commit_accs)
        .add_post_commit_actions(&handlers);

        let all_accounts = builder.collect_unique_account().unwrap();
        let indices_map = create_indices_map(all_accounts.as_slice()).unwrap();
        let serializable = MagicIntentBundleSerialize::new(&indices_map, builder.intent_bundle);

        let mut intent_buf = [0u8; CPI_DATA_BUF_SIZE];
        let intent_len = serializable
            .encode_intent_into_slice(&mut intent_buf)
            .unwrap();

        let mut cb_buf = [0u8; CPI_DATA_BUF_SIZE];
        let mut cb_iter = serializable.action_callback_iter();
        let (cb_idx, cb) = cb_iter.next().expect("expected one callback");
        let cb_len = cb
            .args(cb_idx as u8)
            .unwrap()
            .encode_into_slice(&mut cb_buf)
            .unwrap();
        assert!(cb_iter.next().is_none(), "expected exactly one callback");

        // --- SDK ---
        let mut s_payer = SdkTestAccount::new(p_payer.address);
        let mut s_ctx = SdkTestAccount::new(p_ctx.address);
        let mut s_acc = SdkTestAccount::new(p_acc.address);
        let mut s_escrow = SdkTestAccount::new(p_escrow.address);
        let mut s_prog = SdkTestAccount::new(p_prog.address);

        let sdk_handler = SdkCallHandler {
            args: SdkActionArgs::new(action_data.to_vec()),
            compute_units: 200_000,
            escrow_authority: s_escrow.as_signer_info(),
            destination_program: Pubkey::new_from_array(dest_addr),
            accounts: vec![],
        };
        let sdk_cb = SdkActionCallback {
            destination_program: Pubkey::new_from_array(cb_dest_addr),
            discriminator: cb_disc.to_vec(),
            payload: cb_payload.to_vec(),
            compute_units: 50_000,
            accounts: vec![],
        };
        let IntentInstructions {
            schedule_intent_ix: (_, sdk_ix),
            add_callback_ixs,
        } = SdkBuilder::new(
            s_payer.as_account_info(),
            s_ctx.as_account_info(),
            s_prog.as_account_info(),
        )
        .commit(&[s_acc.as_account_info()])
        .add_post_commit_action(sdk_handler)
        .then(sdk_cb)
        .fold_builder()
        .build();

        assert_eq!(
            &intent_buf[..intent_len],
            sdk_ix.data.as_slice(),
            "intent mismatch"
        );
        assert_eq!(cb_idx, 0, "action index should be 0");
        assert_eq!(
            &cb_buf[..cb_len],
            add_callback_ixs[0].1.data.as_slice(),
            "callback ix mismatch"
        );
    }

    /// CAU with one post-commit callback (action 0) and one post-undelegate callback (action 1).
    #[test]
    fn test_compat_cau_with_callbacks() {
        let dest1_addr = [0xA1; 32];
        let dest2_addr = [0xA2; 32];
        let cb1_dest = [0xC1; 32];
        let cb2_dest = [0xC2; 32];
        let commit_data = [0x11u8];
        let undelegate_data = [0x22u8];
        let cb1_disc = [0xD1u8];
        let cb2_disc = [0xD2u8];

        // --- Pinocchio ---
        let mut p_payer = MockRuntimeAccount::new_unique();
        let mut p_ctx = MockRuntimeAccount::new_unique();
        let mut p_acc = MockRuntimeAccount::new_unique();
        let mut p_escrow1 = MockRuntimeAccount::new_unique();
        let mut p_escrow2 = MockRuntimeAccount::new_unique();
        let mut p_prog = MockRuntimeAccount::new_unique();

        let post_commit = CallHandler {
            destination_program: Address::new_from_array(dest1_addr),
            escrow_authority: p_escrow1.as_account_view(),
            args: ActionArgs::new(&commit_data),
            compute_units: 100_000,
            accounts: &[],
            callback: Some(ActionCallback {
                destination_program: Address::new_from_array(cb1_dest),
                discriminator: &cb1_disc,
                payload: &[],
                compute_units: 30_000,
                accounts: &[],
            }),
        };
        let post_undelegate = CallHandler {
            destination_program: Address::new_from_array(dest2_addr),
            escrow_authority: p_escrow2.as_account_view(),
            args: ActionArgs::new(&undelegate_data),
            compute_units: 50_000,
            accounts: &[],
            callback: Some(ActionCallback {
                destination_program: Address::new_from_array(cb2_dest),
                discriminator: &cb2_disc,
                payload: &[],
                compute_units: 20_000,
                accounts: &[],
            }),
        };
        let cau_accs = [p_acc.as_account_view()];
        let post_commit_handlers = [post_commit];
        let post_undelegate_handlers = [post_undelegate];
        let builder = MagicIntentBundleBuilder::new(
            p_payer.as_account_view(),
            p_ctx.as_account_view(),
            p_prog.as_account_view(),
        )
        .commit_and_undelegate(&cau_accs)
        .add_post_commit_actions(&post_commit_handlers)
        .add_post_undelegate_actions(&post_undelegate_handlers)
        .fold();

        let all_accounts = builder.collect_unique_account().unwrap();
        let indices_map = create_indices_map(all_accounts.as_slice()).unwrap();
        let serializable = MagicIntentBundleSerialize::new(&indices_map, builder.intent_bundle);

        let mut intent_buf = [0u8; CPI_DATA_BUF_SIZE];
        let intent_len = serializable
            .encode_intent_into_slice(&mut intent_buf)
            .unwrap();

        let mut cb_bufs = [[0u8; 512]; 2];
        let mut cb_results = [(0u8, 0usize); 2];
        for (n, (idx, cb)) in serializable.action_callback_iter().enumerate() {
            let len = cb
                .args(idx as u8)
                .unwrap()
                .encode_into_slice(&mut cb_bufs[n])
                .unwrap();
            cb_results[n] = (idx as u8, len);
        }

        // --- SDK ---
        let mut s_payer = SdkTestAccount::new(p_payer.address);
        let mut s_ctx = SdkTestAccount::new(p_ctx.address);
        let mut s_acc = SdkTestAccount::new(p_acc.address);
        let mut s_escrow1 = SdkTestAccount::new(p_escrow1.address);
        let mut s_escrow2 = SdkTestAccount::new(p_escrow2.address);
        let mut s_prog = SdkTestAccount::new(p_prog.address);

        let IntentInstructions {
            schedule_intent_ix: (_, sdk_ix),
            add_callback_ixs,
        } = SdkBuilder::new(
            s_payer.as_account_info(),
            s_ctx.as_account_info(),
            s_prog.as_account_info(),
        )
        .commit_and_undelegate(&[s_acc.as_account_info()])
        .add_post_commit_action(SdkCallHandler {
            args: SdkActionArgs::new(commit_data.to_vec()),
            compute_units: 100_000,
            escrow_authority: s_escrow1.as_signer_info(),
            destination_program: Pubkey::new_from_array(dest1_addr),
            accounts: vec![],
        })
        .then(SdkActionCallback {
            destination_program: Pubkey::new_from_array(cb1_dest),
            discriminator: cb1_disc.to_vec(),
            payload: vec![],
            compute_units: 30_000,
            accounts: vec![],
        })
        .add_post_undelegate_action(SdkCallHandler {
            args: SdkActionArgs::new(undelegate_data.to_vec()),
            compute_units: 50_000,
            escrow_authority: s_escrow2.as_signer_info(),
            destination_program: Pubkey::new_from_array(dest2_addr),
            accounts: vec![],
        })
        .then(SdkActionCallback {
            destination_program: Pubkey::new_from_array(cb2_dest),
            discriminator: cb2_disc.to_vec(),
            payload: vec![],
            compute_units: 20_000,
            accounts: vec![],
        })
        .fold_builder()
        .build();

        assert_eq!(
            &intent_buf[..intent_len],
            sdk_ix.data.as_slice(),
            "cau intent mismatch"
        );
        assert_eq!(add_callback_ixs.len(), 2, "expected 2 callbacks");
        assert_eq!(
            cb_results[0].0, 0,
            "first callback action index should be 0"
        );
        assert_eq!(
            cb_results[1].0, 1,
            "second callback action index should be 1"
        );
        assert_eq!(
            &cb_bufs[0][..cb_results[0].1],
            add_callback_ixs[0].1.data.as_slice(),
            "post-commit callback mismatch"
        );
        assert_eq!(
            &cb_bufs[1][..cb_results[1].1],
            add_callback_ixs[1].1.data.as_slice(),
            "post-undelegate callback mismatch"
        );
    }

    /// Commit with 2 actions: first has no callback, second has a callback.
    /// Verifies that the emitted action_index is 1 (not 0).
    #[test]
    fn test_callback_action_index_ordering() {
        let dest_addr = [0xDD; 32];
        let cb_dest_addr = [0xCB; 32];
        let data1 = [0x11u8];
        let data2 = [0x22u8];
        let cb_disc = [0xFFu8];

        let mut p_payer = MockRuntimeAccount::new_unique();
        let mut p_ctx = MockRuntimeAccount::new_unique();
        let mut p_acc = MockRuntimeAccount::new_unique();
        let mut p_escrow = MockRuntimeAccount::new_unique();
        let mut p_prog = MockRuntimeAccount::new_unique();

        let handler_no_cb = CallHandler {
            destination_program: Address::new_from_array(dest_addr),
            escrow_authority: p_escrow.as_account_view(),
            args: ActionArgs::new(&data1),
            compute_units: 100_000,
            accounts: &[],
            callback: None, // no callback on action 0
        };
        let handler_with_cb = CallHandler {
            destination_program: Address::new_from_array(dest_addr),
            escrow_authority: p_escrow.as_account_view(),
            args: ActionArgs::new(&data2),
            compute_units: 100_000,
            accounts: &[],
            callback: Some(ActionCallback {
                destination_program: Address::new_from_array(cb_dest_addr),
                discriminator: &cb_disc,
                payload: &[],
                compute_units: 25_000,
                accounts: &[],
            }),
        };
        let commit_accs = [p_acc.as_account_view()];
        let handlers = [handler_no_cb, handler_with_cb];
        let builder = MagicIntentBundleBuilder::new(
            p_payer.as_account_view(),
            p_ctx.as_account_view(),
            p_prog.as_account_view(),
        )
        .commit(&commit_accs)
        .add_post_commit_actions(&handlers);

        let all_accounts = builder.collect_unique_account().unwrap();
        let indices_map = create_indices_map(all_accounts.as_slice()).unwrap();
        let serializable = MagicIntentBundleSerialize::new(&indices_map, builder.intent_bundle);

        let mut cb_iter = serializable.action_callback_iter();
        let (cb_idx, cb) = cb_iter.next().expect("expected one callback");
        assert!(cb_iter.next().is_none(), "expected exactly one callback");
        assert_eq!(
            cb_idx, 1,
            "callback should have action index 1 (first action has no callback)"
        );

        // Verify the callback encodes consistently (non-zero length)
        let mut cb_buf = [0u8; 256];
        let cb_len = cb
            .args(cb_idx as u8)
            .unwrap()
            .encode_into_slice(&mut cb_buf)
            .unwrap();
        assert!(cb_len > 0);
    }

    #[test]
    fn test_instruction_accounts_force_payer_signer() {
        let mut payer = MockRuntimeAccount::new_unique();
        let mut context = MockRuntimeAccount::new_unique();
        let mut program = MockRuntimeAccount::new_unique();

        let all_accounts = [
            payer.as_account_view(),
            context.as_account_view(),
            program.as_account_view(),
        ];

        let instruction_accounts =
            MagicIntentBundleBuilder::instruction_accounts(&all_accounts).unwrap();

        assert_eq!(instruction_accounts.as_slice().len(), 3);
        assert!(instruction_accounts.as_slice()[0].is_signer);
        assert!(instruction_accounts.as_slice()[0].is_writable);
        assert!(!instruction_accounts.as_slice()[1].is_signer);
        assert!(!instruction_accounts.as_slice()[2].is_signer);
    }
}
