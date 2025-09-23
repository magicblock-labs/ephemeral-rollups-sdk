use borsh::BorshSerialize;
use solana_program::account_info::AccountInfo;
use solana_program::entrypoint::ProgramResult;
use solana_program::instruction::{AccountMeta, Instruction};
use solana_program::program_error::ProgramError;
use solana_program::program_memory::{sol_memcpy, sol_memset};
use solana_program::pubkey::Pubkey;

// TODO: import from the delegation program crate once open-sourced
use crate::consts::BUFFER;
use crate::types::DelegateAccountArgs;
use crate::utils::{close_pda_with_system_transfer, create_pda, seeds_with_bump};

pub struct DelegateAccounts<'a, 'info> {
    pub payer: &'a AccountInfo<'info>,
    pub pda: &'a AccountInfo<'info>,
    pub owner_program: &'a AccountInfo<'info>,
    pub buffer: &'a AccountInfo<'info>,
    pub delegation_record: &'a AccountInfo<'info>,
    pub delegation_metadata: &'a AccountInfo<'info>,
    pub delegation_program: &'a AccountInfo<'info>,
    pub system_program: &'a AccountInfo<'info>,
}

pub struct DelegateConfig {
    pub commit_frequency_ms: u32,
    pub validator: Option<Pubkey>,
}

impl Default for DelegateConfig {
    fn default() -> Self {
        DelegateConfig {
            commit_frequency_ms: DelegateAccountArgs::default().commit_frequency_ms,
            validator: DelegateAccountArgs::default().validator,
        }
    }
}

#[allow(clippy::needless_lifetimes)]
pub fn delegate_account<'a, 'info>(
    accounts: DelegateAccounts<'a, 'info>,
    pda_seeds: &[&[u8]],
    config: DelegateConfig,
) -> ProgramResult {
    let buffer_seeds: &[&[u8]] = &[BUFFER, accounts.pda.key.as_ref()];

    let (_, delegate_account_bump) =
        Pubkey::find_program_address(pda_seeds, accounts.owner_program.key);

    let (_, buffer_pda_bump) =
        Pubkey::find_program_address(buffer_seeds, accounts.owner_program.key);

    // Pda signer seeds
    let delegate_account_bump_slice: &[u8] = &[delegate_account_bump];
    let pda_signer_seeds: &[&[&[u8]]] =
        &[&*seeds_with_bump(pda_seeds, delegate_account_bump_slice)];

    // Buffer signer seeds
    let buffer_bump_slice: &[u8] = &[buffer_pda_bump];
    let buffer_signer_seeds: &[&[&[u8]]] = &[&*seeds_with_bump(buffer_seeds, buffer_bump_slice)];

    let data_len = accounts.pda.data_len();

    // Create the Buffer PDA
    create_pda(
        accounts.buffer,
        accounts.owner_program.key,
        data_len,
        buffer_signer_seeds,
        accounts.system_program,
        accounts.payer,
        false,
    )?;

    // Copy PDA -> buffer (RO pda, RW buffer)
    {
        let pda_ro = accounts.pda.try_borrow_data()?;
        let mut buf = accounts.buffer.try_borrow_mut_data()?;
        sol_memcpy(&mut buf, &pda_ro, data_len);
    }

    // Zero PDA (single RW borrow)
    {
        let mut pda_mut = accounts.pda.try_borrow_mut_data()?;
        sol_memset(&mut pda_mut, 0, data_len);
    }

    accounts.pda.assign(accounts.delegation_program.key);

    let seeds_vec: Vec<Vec<u8>> = pda_seeds.iter().map(|&slice| slice.to_vec()).collect();

    let delegation_args = DelegateAccountArgs {
        commit_frequency_ms: config.commit_frequency_ms,
        seeds: seeds_vec,
        validator: config.validator,
    };

    cpi_delegate(
        accounts.payer,
        accounts.pda,
        accounts.owner_program,
        accounts.buffer,
        accounts.delegation_record,
        accounts.delegation_metadata,
        accounts.system_program,
        pda_signer_seeds,
        delegation_args,
    )?;

    close_pda_with_system_transfer(
        accounts.buffer,
        buffer_signer_seeds,
        accounts.payer,
        accounts.system_program,
    )?;
    Ok(())
}

/// Undelegate an account
pub fn undelegate_account<'a, 'info>(
    delegated_account: &'a AccountInfo<'info>,
    owner_program: &Pubkey,
    buffer: &'a AccountInfo<'info>,
    payer: &'a AccountInfo<'info>,
    system_program: &'a AccountInfo<'info>,
    account_signer_seeds: Vec<Vec<u8>>,
) -> ProgramResult {
    if !buffer.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let account_seeds: Vec<&[u8]> = account_signer_seeds.iter().map(|v| v.as_slice()).collect();

    let (_, account_bump) = Pubkey::find_program_address(account_seeds.as_ref(), owner_program);

    // Account signer seeds
    let account_bump_slice: &[u8] = &[account_bump];
    let account_signer_seeds: &[&[&[u8]]] = &[&*seeds_with_bump(
        account_seeds.as_ref(),
        account_bump_slice,
    )];

    // Re-create the original PDA
    create_pda(
        delegated_account,
        owner_program,
        buffer.data_len(),
        account_signer_seeds,
        system_program,
        payer,
        true,
    )?;

    let mut data = delegated_account.try_borrow_mut_data()?;
    let buffer_data = buffer.try_borrow_data()?;
    (*data).copy_from_slice(&buffer_data);
    Ok(())
}

/// CPI to the delegation program to delegate the account
#[allow(clippy::too_many_arguments)]
pub fn cpi_delegate<'a, 'info>(
    payer: &'a AccountInfo<'info>,
    delegate_account: &'a AccountInfo<'info>,
    owner_program: &'a AccountInfo<'info>,
    buffer: &'a AccountInfo<'info>,
    delegation_record: &'a AccountInfo<'info>,
    delegation_metadata: &'a AccountInfo<'info>,
    system_program: &'a AccountInfo<'info>,
    signers_seeds: &[&[&[u8]]],
    args: DelegateAccountArgs,
) -> ProgramResult {
    let mut data: Vec<u8> = vec![0u8; 8];
    args.serialize(&mut data)?;

    let delegation_instruction = Instruction {
        program_id: crate::id(),
        accounts: vec![
            AccountMeta::new(*payer.key, true),
            AccountMeta::new(*delegate_account.key, true),
            AccountMeta::new_readonly(*owner_program.key, false),
            AccountMeta::new(*buffer.key, false),
            AccountMeta::new(*delegation_record.key, false),
            AccountMeta::new(*delegation_metadata.key, false),
            AccountMeta::new_readonly(*system_program.key, false),
        ],
        data,
    };

    solana_program::program::invoke_signed(
        &delegation_instruction,
        &[
            payer.clone(),
            delegate_account.clone(),
            owner_program.clone(),
            buffer.clone(),
            delegation_record.clone(),
            delegation_metadata.clone(),
            system_program.clone(),
        ],
        signers_seeds,
    )
}

#[cfg(feature = "light")]
pub fn cpi_delegate_compressed<'a, 'info, 'info2: 'info>(
    payer: &'a AccountInfo<'info>,
    delegated_account: &'a AccountInfo<'info>,
    owner_program: &'a AccountInfo<'info>,
    remaining_accounts: &'a [AccountInfo<'info2>],
    signer_seeds: &[&[u8]],
    args: crate::types::DelegateCompressedArgs,
) -> ProgramResult {
    use solana_program::msg;

    if owner_program.key == &crate::id() {
        return Err(ProgramError::IncorrectProgramId);
    }

    let mut data = 16_u64.to_le_bytes().to_vec();
    data.extend_from_slice(&borsh::to_vec(&args).unwrap());

    let remaining_accounts_metas = remaining_accounts
        .iter()
        .map(|a| AccountMeta {
            pubkey: *a.key,
            is_signer: a.is_signer,
            is_writable: a.is_writable,
        })
        .collect::<Vec<_>>();
    let ix = Instruction {
        program_id: crate::id(),
        accounts: [
            &[
                AccountMeta::new(*payer.key, true),
                AccountMeta::new_readonly(*delegated_account.key, true),
                AccountMeta::new_readonly(*owner_program.key, false),
            ],
            remaining_accounts_metas.as_slice(),
        ]
        .concat(),
        data,
    };
    msg!(
        "cpi_delegate_compressed: {}",
        ix.accounts
            .iter()
            .map(|a| format!("{:?}", a))
            .collect::<Vec<_>>()
            .join("\n")
    );

    solana_program::program::invoke_signed(
        &ix,
        &vec![
            &[
                payer.clone(),
                delegated_account.clone(),
                owner_program.clone(),
            ],
            remaining_accounts,
        ]
        .concat(),
        &[signer_seeds],
    )
}
