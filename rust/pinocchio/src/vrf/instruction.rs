use core::mem::MaybeUninit;

use pinocchio::{
    cpi::{invoke_signed, Signer},
    instruction::{InstructionAccount, InstructionView},
    AccountView, ProgramResult,
};

use crate::vrf::consts::{
    REQUEST_HIGH_PRIORITY_SCOPED_RANDOMNESS_DISCRIMINATOR, REQUEST_SCOPED_RANDOMNESS_DISCRIMINATOR,
};
use crate::vrf::types::RequestRandomness;

/// Number of accounts in the VRF `RequestRandomness` instruction.
const REQUEST_RANDOMNESS_ACCOUNTS: usize = 5;

/// CPI helper for the VRF `RequestRandomness` instruction.
///
/// Account order (matches the canonical `ephemeral_vrf_sdk`):
/// 1. `payer` — writable signer, funds the request.
/// 2. `program_identity` — readonly signer; the PDA `["identity"]` of the
///    program that owns the callback (i.e. `request.callback_program_id`).
///    When invoked via CPI from that program, it is signed with the seeds
///    `[IDENTITY, &[bump]]` (see [`invoke_signed`](Self::invoke_signed)).
/// 3. `oracle_queue` — writable; the randomness queue (e.g.
///    [`DEFAULT_QUEUE`](crate::vrf::consts::DEFAULT_QUEUE)).
/// 4. `system_program` — readonly.
/// 5. `slot_hashes` — readonly; the `SlotHashes` sysvar.
pub struct RequestRandomnessCpi<'a> {
    pub payer: &'a AccountView,
    pub program_identity: &'a AccountView,
    pub oracle_queue: &'a AccountView,
    pub system_program: &'a AccountView,
    pub slot_hashes: &'a AccountView,
    pub vrf_program: &'a AccountView,
    pub request: RequestRandomness<'a>,
}

impl<'a> RequestRandomnessCpi<'a> {
    /// Exact size of the buffer required by [`invoke`](Self::invoke) /
    /// [`invoke_signed`](Self::invoke_signed).
    #[inline(always)]
    pub fn serialized_size(&self) -> usize {
        self.request.serialized_size()
    }

    #[inline(always)]
    fn discriminator(&self) -> u64 {
        if self.request.high_priority {
            REQUEST_HIGH_PRIORITY_SCOPED_RANDOMNESS_DISCRIMINATOR
        } else {
            REQUEST_SCOPED_RANDOMNESS_DISCRIMINATOR
        }
    }

    /// Build the [`InstructionView`] for this request, writing the account metas
    /// into the provided buffer. `data` must be the already-serialized
    /// instruction data (see [`RequestRandomness::serialize_into`]).
    fn instruction<'b>(
        &'b self,
        data: &'b mut [u8],
        account_metas: &'b mut [MaybeUninit<InstructionAccount<'b>>; REQUEST_RANDOMNESS_ACCOUNTS],
    ) -> InstructionView<'b, 'b, 'b, 'b> {
        let data_len = self
            .request
            .serialize_into(data, self.discriminator())
            .expect("Failed to serialize request randomness");

        unsafe {
            account_metas
                .get_unchecked_mut(0)
                .write(InstructionAccount::writable_signer(self.payer.address()));
            account_metas
                .get_unchecked_mut(1)
                .write(InstructionAccount::readonly_signer(
                    self.program_identity.address(),
                ));
            account_metas
                .get_unchecked_mut(2)
                .write(InstructionAccount::writable(self.oracle_queue.address()));
            account_metas
                .get_unchecked_mut(3)
                .write(InstructionAccount::readonly(self.system_program.address()));
            account_metas
                .get_unchecked_mut(4)
                .write(InstructionAccount::readonly(self.slot_hashes.address()));
        }

        InstructionView {
            program_id: self.vrf_program.address(),
            accounts: unsafe {
                core::slice::from_raw_parts(
                    account_metas.as_ptr() as *const InstructionAccount,
                    REQUEST_RANDOMNESS_ACCOUNTS,
                )
            },
            data: &data[..data_len],
        }
    }

    /// Issue the CPI. Expecting the caller to provide the program identity as a signer.
    ///
    /// `data_buf` must be at least [`serialized_size`](Self::serialized_size)
    /// bytes long; the serialized instruction data is written into it.
    pub fn invoke_signed(&self, data_buf: &mut [u8], signers: &[Signer<'_, '_>]) -> ProgramResult {
        const UNINIT: MaybeUninit<InstructionAccount> = MaybeUninit::<InstructionAccount>::uninit();
        let mut account_metas = [UNINIT; REQUEST_RANDOMNESS_ACCOUNTS];
        let instruction = self.instruction(data_buf, &mut account_metas);

        let account_infos: [&AccountView; REQUEST_RANDOMNESS_ACCOUNTS] = [
            self.payer,
            self.program_identity,
            self.oracle_queue,
            self.system_program,
            self.slot_hashes,
        ];

        invoke_signed(&instruction, &account_infos, signers)
    }
}

#[cfg(test)]
mod tests {
    extern crate alloc;

    use alloc::vec;
    use alloc::vec::Vec;

    use ephemeral_vrf_sdk::instructions::{
        create_request_scoped_randomness_ix, RequestRandomnessParams,
    };
    use pinocchio::account::RuntimeAccount;
    use pinocchio::Address;
    use solana_program::pubkey::Pubkey;

    use super::*;
    use crate::vrf::types::RequestRandomness;
    use crate::vrf::VRF_PROGRAM_ID;

    fn runtime_account(address: Address, is_signer: u8, is_writable: u8) -> RuntimeAccount {
        RuntimeAccount {
            borrow_state: 0,
            is_signer,
            is_writable,
            executable: 0,
            resize_delta: 0,
            address,
            owner: Address::new_from_array([0; 32]),
            lamports: 0,
            data_len: 0,
        }
    }

    #[test]
    fn instruction_view_matches_canonical() {
        let callback_program = [9u8; 32];
        let payer_key = [5u8; 32];
        let oracle_key = [6u8; 32];
        let caller_seed = [7u8; 32];
        let disc = [1u8, 2, 3, 4];
        let args = [9u8, 9];

        // Build the canonical instruction first; it is the source of truth for the
        // program-identity, system-program and slot-hashes account keys.
        let ix = create_request_scoped_randomness_ix(RequestRandomnessParams {
            payer: Pubkey::new_from_array(payer_key),
            oracle_queue: Pubkey::new_from_array(oracle_key),
            callback_program_id: Pubkey::new_from_array(callback_program),
            callback_discriminator: disc.to_vec(),
            accounts_metas: Some(Vec::new()),
            caller_seed,
            callback_args: Some(args.to_vec()),
        });

        let identity_key: [u8; 32] = ix.accounts[1].pubkey.to_bytes();
        let system_key: [u8; 32] = ix.accounts[3].pubkey.to_bytes();
        let slot_hashes_key: [u8; 32] = ix.accounts[4].pubkey.to_bytes();

        let mut payer = runtime_account(Address::new_from_array(payer_key), 1, 1);
        let mut identity = runtime_account(Address::new_from_array(identity_key), 0, 0);
        let mut oracle = runtime_account(Address::new_from_array(oracle_key), 0, 1);
        let mut system = runtime_account(Address::new_from_array(system_key), 0, 0);
        let mut slot_hashes = runtime_account(Address::new_from_array(slot_hashes_key), 0, 0);
        let mut vrf = runtime_account(VRF_PROGRAM_ID, 0, 0);

        let payer_view = unsafe { AccountView::new_unchecked(&mut payer as *mut RuntimeAccount) };
        let identity_view =
            unsafe { AccountView::new_unchecked(&mut identity as *mut RuntimeAccount) };
        let oracle_view = unsafe { AccountView::new_unchecked(&mut oracle as *mut RuntimeAccount) };
        let system_view = unsafe { AccountView::new_unchecked(&mut system as *mut RuntimeAccount) };
        let slot_hashes_view =
            unsafe { AccountView::new_unchecked(&mut slot_hashes as *mut RuntimeAccount) };
        let vrf_view = unsafe { AccountView::new_unchecked(&mut vrf as *mut RuntimeAccount) };

        let request = RequestRandomness {
            high_priority: false,
            caller_seed,
            callback_program_id: &Address::new_from_array(callback_program),
            callback_discriminator: &disc,
            callback_accounts_metas: &[],
            callback_args: &args,
        };

        let cpi = RequestRandomnessCpi {
            payer: &payer_view,
            program_identity: &identity_view,
            oracle_queue: &oracle_view,
            system_program: &system_view,
            slot_hashes: &slot_hashes_view,
            vrf_program: &vrf_view,
            request,
        };

        let mut data = vec![0u8; cpi.serialized_size()];
        const UNINIT: MaybeUninit<InstructionAccount> = MaybeUninit::<InstructionAccount>::uninit();
        let mut metas = [UNINIT; REQUEST_RANDOMNESS_ACCOUNTS];
        let view = cpi.instruction(&mut data, &mut metas);

        // Program id and data must match the canonical instruction byte-for-byte.
        assert_eq!(view.program_id.as_ref(), ix.program_id.as_ref());
        assert_eq!(view.data, ix.data.as_slice());

        // Account order, keys, and signer/writable flags must match.
        assert_eq!(view.accounts.len(), ix.accounts.len());
        for (got, expected) in view.accounts.iter().zip(ix.accounts.iter()) {
            assert_eq!(got.address.as_ref(), expected.pubkey.as_ref());
            assert_eq!(got.is_signer, expected.is_signer);
            assert_eq!(got.is_writable, expected.is_writable);
        }
    }
}
