use std::mem::MaybeUninit;
use std::ops::Deref;

use pinocchio::{
    account_info::AccountInfo,
    cpi::{invoke_signed, MAX_CPI_ACCOUNTS},
    instruction::{AccountMeta, Instruction, Seed, Signer},
    program_error::ProgramError,
};

use crate::{consts::DELEGATION_PROGRAM_ID, types::DelegateAccountArgs};

pub struct Seeds<'a>(&'a [Seed<'a>]);

impl<'a> Seeds<'a> {
    /// Returns the inner slice of seeds
    pub fn as_slice(&self) -> &'a [Seed<'a>] {
        self.0
    }
}

impl<'a> Deref for Seeds<'a> {
    type Target = [Seed<'a>];

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl<'a> TryFrom<&'a [&[u8]]> for Seeds<'a> {
    type Error = ProgramError;

    fn try_from(seeds_array: &'a [&[u8]]) -> Result<Self, Self::Error> {
        let seeds_len = seeds_array.len();
        if seeds_len >= MAX_CPI_ACCOUNTS {
            return Err(ProgramError::InvalidArgument);
        }

        let mut seeds: [MaybeUninit<Seed<'a>>; MAX_CPI_ACCOUNTS] =
            [const { MaybeUninit::uninit() }; MAX_CPI_ACCOUNTS];

        for i in 0..seeds_len {
            // SAFETY: The number of seeds has been validated to be less than
            // `MAX_CPI_ACCOUNTS`.
            unsafe {
                let seed = seeds_array.get_unchecked(i).as_ref();
                seeds.get_unchecked_mut(i).write(Seed::from(seed));
            }
        }

        // SAFETY: The seeds have been validated.
        Ok(Seeds(unsafe {
            std::slice::from_raw_parts(seeds.as_ptr() as *const Seed<'a>, seeds_len)
        }))
    }
}

pub fn close_pda_acc(
    payer: &AccountInfo,
    pda_acc: &AccountInfo,
    system_program: &AccountInfo,
) -> Result<(), ProgramError> {
    unsafe {
        *payer.borrow_mut_lamports_unchecked() += *pda_acc.borrow_lamports_unchecked();
        *pda_acc.borrow_mut_lamports_unchecked() = 0;
    }

    pda_acc
        .realloc(0, false)
        .map_err(|_| ProgramError::AccountDataTooSmall)?;
    unsafe {
        pda_acc.assign(system_program.key());
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub fn cpi_delegate(
    payer: &AccountInfo,
    pda_acc: &AccountInfo,
    owner_program: &AccountInfo,
    buffer_acc: &AccountInfo,
    delegation_record: &AccountInfo,
    delegation_metadata: &AccountInfo,
    system_program: &AccountInfo,
    delegate_args: DelegateAccountArgs,
    signer_seeds: Signer<'_, '_>,
) -> Result<(), ProgramError> {
    let account_metas = vec![
        AccountMeta::new(payer.key(), true, true),
        AccountMeta::new(pda_acc.key(), true, false),
        AccountMeta::readonly(owner_program.key()),
        AccountMeta::new(buffer_acc.key(), false, false),
        AccountMeta::new(delegation_record.key(), true, false),
        AccountMeta::readonly(delegation_metadata.key()),
        AccountMeta::readonly(system_program.key()),
    ];

    let data = [0u8; 8];
    let delegate_args = delegate_args
        .try_to_serialize()
        .map_err(|_| ProgramError::InvalidArgument)?;
    let data = [&data, &delegate_args[..]].concat();

    let instruction = Instruction {
        program_id: &DELEGATION_PROGRAM_ID,
        accounts: &account_metas,
        data: &data,
    };

    let acc_infos = [
        payer,
        pda_acc,
        owner_program,
        buffer_acc,
        delegation_record,
        delegation_metadata,
        system_program,
    ];

    invoke_signed(&instruction, &acc_infos, &[signer_seeds])?;
    Ok(())
}

pub fn create_schedule_commit_ix<'a>(
    payer: &'a AccountInfo,
    account_infos: &'a [AccountInfo],
    magic_context: &'a AccountInfo,
    allow_undelegation: bool,
) -> Result<([u8; 4], &'a [AccountMeta<'a>]), ProgramError> {
    let num_accounts = 2 + account_infos.len(); // 2 for payer and magic_context

    if num_accounts > MAX_CPI_ACCOUNTS {
        return Err(ProgramError::InvalidArgument);
    }

    let instruction_data = if allow_undelegation {
        [2, 0, 0, 0]
    } else {
        [1, 0, 0, 0]
    };
    const UNINIT_META: MaybeUninit<AccountMeta> = MaybeUninit::<AccountMeta>::uninit();
    let mut account_metas = [UNINIT_META; MAX_CPI_ACCOUNTS];

    unsafe {
        // SAFETY: num_accounts <= MAX_CPI_ACCOUNTS
        account_metas
            .get_unchecked_mut(0)
            .write(AccountMeta::new(payer.key(), true, true));
        account_metas.get_unchecked_mut(1).write(AccountMeta::new(
            magic_context.key(),
            true,
            false,
        ));

        for i in 0..account_infos.len() {
            let account = account_infos.get_unchecked(i);
            account_metas
                .get_unchecked_mut(2 + i)
                .write(AccountMeta::new(account.key(), true, true));
        }
    }

    Ok((instruction_data, unsafe {
        core::slice::from_raw_parts(account_metas.as_ptr() as *const AccountMeta, num_accounts)
    }))
}

pub fn concate_accounts_with_remaining_accounts<'a, T>(
    account_infos: &[&T],
    remaining_accounts: &[T],
) -> Result<&'a [&'a T], ProgramError> {
    let accounts_len = account_infos.len() + remaining_accounts.len();

    if accounts_len == 0 {
        return Ok(&[]);
    }

    if accounts_len > MAX_CPI_ACCOUNTS {
        return Err(ProgramError::InvalidArgument);
    }

    let mut accounts: [MaybeUninit<&T>; MAX_CPI_ACCOUNTS] =
        [const { MaybeUninit::uninit() }; MAX_CPI_ACCOUNTS];
    let mut accounts_offset = 0;

    for i in 0..account_infos.len() {
        // SAFETY: The number of accounts has been validated to be less than
        // `MAX_CPI_ACCOUNTS`.
        unsafe {
            accounts
                .get_unchecked_mut(accounts_offset)
                .write(account_infos.get_unchecked(i));
        }

        accounts_offset += 1;
    }

    for i in 0..remaining_accounts.len() {
        // SAFETY: The number of accounts has been validated to be less than
        // `MAX_CPI_ACCOUNTS`.
        unsafe {
            accounts
                .get_unchecked_mut(accounts_offset)
                .write(remaining_accounts.get_unchecked(i));
        }

        accounts_offset += 1;
    }
    // SAFETY: The accounts have been validated.
    Ok(unsafe { std::slice::from_raw_parts(accounts.as_ptr() as *const &T, accounts_offset) })
}

#[cfg(test)]
mod tests {
    use super::*;
    use pinocchio::pubkey::Pubkey;

    #[repr(C)]
    #[derive(Clone, Copy, Default)]
    struct AccountMock {
        /// Borrow state of the account data.
        ///
        /// 0) We reuse the duplicate flag for this. We set it to 0b0000_0000.
        /// 1) We use the first four bits to track state of lamport borrow
        /// 2) We use the second four bits to track state of data borrow
        ///
        /// 4 bit state: [1 bit mutable borrow flag | u3 immmutable borrow flag]
        /// This gives us up to 7 immutable borrows. Note that does not mean 7
        /// duplicate account infos, but rather 7 calls to borrow lamports or
        /// borrow data across all duplicate account infos.
        borrow_state: u8,

        /// Indicates whether the transaction was signed by this account.
        is_signer: u8,

        /// Indicates whether the account is writable.
        is_writable: u8,

        /// Indicates whether this account represents a program.
        executable: u8,

        /// Account's original data length when it was serialized for the
        /// current program invocation.
        ///
        /// The value of this field is lazily initialized to the current data length
        /// and the [`SET_LEN_MASK`] flag on first access. When reading this field,
        /// the flag is cleared to retrieve the original data length by using the
        /// [`GET_LEN_MASK`] mask.
        ///
        /// Currently, this value is only used for `realloc` to determine if the
        /// account data length has changed from the original serialized length beyond
        /// the maximum permitted data increase.
        original_data_len: u32,

        /// Public key of the account.
        key: Pubkey,

        /// Program that owns this account. Modifiable by programs.
        owner: Pubkey,

        /// The lamports in the account. Modifiable by programs.
        lamports: u64,

        /// Length of the data. Modifiable by programs.
        data_len: u64,
    }

    fn create_account_info_mock(
        key: Pubkey,
        owner: Pubkey,
        is_signer: bool,
        is_writable: bool,
        lamports: u64,
        data_len: u64,
    ) -> AccountInfo {
        let is_signer: u8 = match is_signer {
            true => 1,
            false => 0,
        };
        let is_writable: u8 = match is_writable {
            true => 1,
            false => 0,
        };

        // Allocate the Account on the heap so it persists
        let account = Box::leak(Box::new(AccountMock {
            borrow_state: 0,
            is_signer,
            is_writable,
            executable: 0,
            original_data_len: 0,
            key,
            owner,
            lamports,
            data_len,
        }));
        // Create AccountInfo with pointer to the heap-allocated Account
        // AccountInfo is just a wrapper around *mut Account
        // We can't construct it directly since it's not public, so we use unsafe transmute
        unsafe { std::mem::transmute::<*mut AccountMock, AccountInfo>(account as *mut AccountMock) }
    }

    #[test]
    fn test_seeds_try_from() {
        let seeds = vec![b"seed1".as_slice(), b"seed2".as_slice()];
        let result = Seeds::try_from(seeds.as_slice()).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].as_ref(), b"seed1");
        assert_eq!(result[1].as_ref(), b"seed2");
    }

    #[test]
    fn test_close_pda_acc() {
        let payer = create_account_info_mock(
            Pubkey::from([7u8; 32]),
            Pubkey::from([1u8; 32]),
            true,
            true,
            1000000,
            4567,
        );
        let pda_acc = create_account_info_mock(
            Pubkey::from([2u8; 32]),
            Pubkey::from([11u8; 32]),
            false,
            true,
            500000,
            1234,
        );
        let system_program = create_account_info_mock(
            Pubkey::from([3u8; 32]),
            Pubkey::from([2u8; 32]),
            false,
            false,
            1000000,
            4567,
        );

        assert!(close_pda_acc(&payer, &pda_acc, &system_program).is_ok());
        assert_eq!(payer.lamports(), 1500000);
        assert_eq!(pda_acc.lamports(), 0);
        assert_eq!(pda_acc.data_len(), 0);
    }

    #[test]
    fn test_create_schedule_commit_ix() {
        let payer = create_account_info_mock(
            Pubkey::from([7u8; 32]),
            Pubkey::from([1u8; 32]),
            true,
            true,
            1000000,
            4567,
        );
        let magic_context = create_account_info_mock(
            Pubkey::from([3u8; 32]),
            Pubkey::from([2u8; 32]),
            true,
            true,
            1000000,
            4567,
        );
        let account_1 = create_account_info_mock(
            Pubkey::from([1u8; 32]),
            Pubkey::from([11u8; 32]),
            true,
            true,
            1000000,
            4567,
        );
        let account_2 = create_account_info_mock(
            Pubkey::from([2u8; 32]),
            Pubkey::from([22u8; 32]),
            true,
            true,
            1000000,
            4567,
        );

        let account_infos = &[account_1.clone(), account_2.clone()];

        let (data, metas) =
            create_schedule_commit_ix(&payer, account_infos, &magic_context, true).unwrap();

        assert_eq!(data, [2, 0, 0, 0]);
        assert_eq!(metas.len(), 4);
        assert_eq!(metas[0].pubkey, payer.key());
        assert_eq!(metas[1].pubkey, magic_context.key());
        assert_eq!(metas[2].pubkey, account_1.key());
        assert_eq!(metas[3].pubkey, account_2.key());
    }

    #[test]
    fn test_concate_accounts_with_remaining_accounts_empty() {
        let accounts: &[&AccountInfo] = &[];
        let remaining_accounts: &[AccountInfo] = &[];
        let result =
            concate_accounts_with_remaining_accounts(accounts, remaining_accounts).unwrap();
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_concate_accounts_with_remaining_accounts_validates_minimum_accounts() {
        let payer = create_account_info_mock(
            Pubkey::from([7u8; 32]),
            Pubkey::from([1u8; 32]),
            true,
            true,
            1000000,
            4567,
        );
        let magic_context = create_account_info_mock(
            Pubkey::from([3u8; 32]),
            Pubkey::from([2u8; 32]),
            true,
            true,
            1000000,
            4567,
        );
        let magic_program = create_account_info_mock(
            Pubkey::from([4u8; 32]),
            Pubkey::from([3u8; 32]),
            true,
            true,
            1000000,
            4567,
        );
        let accounts: &[&AccountInfo] = &[
            &payer.clone(),
            &magic_context.clone(),
            &magic_program.clone(),
        ];

        let account_1 = create_account_info_mock(
            Pubkey::from([1u8; 32]),
            Pubkey::from([11u8; 32]),
            true,
            true,
            1000000,
            4567,
        );
        let remaining_accounts: &[AccountInfo] = &[account_1.clone()];

        let expected_accounts: &[&AccountInfo] = &[
            &payer,         // Placeholder for payer
            &magic_context, // Placeholder for magic_context
            &magic_program, // Placeholder for magic_program
            &account_1,     // Placeholder for account 1
        ];

        let result =
            concate_accounts_with_remaining_accounts(accounts, remaining_accounts).unwrap();
        assert_eq!(result.len(), expected_accounts.len());
        for (i, account) in result.iter().enumerate() {
            assert_eq!(account.key(), expected_accounts[i].key());
        }
    }

    #[test]
    fn test_concate_accounts_without_remaining_accounts() {
        let payer = create_account_info_mock(
            Pubkey::from([7u8; 32]),
            Pubkey::from([1u8; 32]),
            true,
            true,
            1000000,
            4567,
        );
        let magic_context = create_account_info_mock(
            Pubkey::from([3u8; 32]),
            Pubkey::from([2u8; 32]),
            true,
            true,
            1000000,
            4567,
        );
        let magic_program = create_account_info_mock(
            Pubkey::from([4u8; 32]),
            Pubkey::from([3u8; 32]),
            true,
            true,
            1000000,
            4567,
        );
        let accounts: &[&AccountInfo] = &[
            &payer.clone(),
            &magic_context.clone(),
            &magic_program.clone(),
        ];

        let remaining_accounts: &[AccountInfo] = &[];

        let expected: &[&AccountInfo] = &[&payer, &magic_context, &magic_program];

        let result =
            concate_accounts_with_remaining_accounts(accounts, remaining_accounts).unwrap();
        assert_eq!(result.len(), expected.len());
        for (i, account) in result.iter().enumerate() {
            assert_eq!(account.key(), expected[i].key());
        }
    }
}
