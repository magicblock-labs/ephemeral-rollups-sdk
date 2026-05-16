use crate::modernize;
use crate::types::DelegateAccountArgs;
use crate::utils::{close_pda_with_system_transfer, create_pda, seeds_with_bump};
use dlp_api::args::{DelegateArgs, DelegateWithActionsArgs, PostDelegationActions};
use dlp_api::delegate_buffer_seeds_from_delegated_account;
use dlp_api::discriminator::DlpDiscriminator;
use solana_program::instruction::{AccountMeta, Instruction};
use solana_program::program_error::ProgramError;

use crate::compat::{
    self,
    borsh::{self, BorshSerialize},
    AsModern, Compat, Modern,
};

use solana_address::Address;
use solana_program::{program::invoke_signed, program_memory::sol_memset};

pub const DELEGATION_PROGRAM_ID: compat::Pubkey =
    compat::Pubkey::new_from_array(dlp_api::consts::DELEGATION_PROGRAM_ID.to_bytes());

pub struct DelegateAccounts<'a, 'info> {
    pub payer: &'a compat::AccountInfo<'info>,
    pub pda: &'a compat::AccountInfo<'info>,
    pub owner_program: &'a compat::AccountInfo<'info>,
    pub buffer: &'a compat::AccountInfo<'info>,
    pub delegation_record: &'a compat::AccountInfo<'info>,
    pub delegation_metadata: &'a compat::AccountInfo<'info>,
    pub delegation_program: &'a compat::AccountInfo<'info>,
    pub system_program: &'a compat::AccountInfo<'info>,
}

pub struct DelegateConfig {
    pub commit_frequency_ms: u32,
    pub validator: Option<compat::Pubkey>,
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
) -> compat::ProgramResult {
    let buffer_seeds: &[&[u8]] = delegate_buffer_seeds_from_delegated_account!(accounts.pda.key);

    let (_, delegate_account_bump) =
        Address::find_program_address(pda_seeds, accounts.owner_program.key.as_modern());

    let (_, buffer_pda_bump) =
        Address::find_program_address(buffer_seeds, accounts.owner_program.key.as_modern());

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
        buf.copy_from_slice(&pda_ro);
    }

    // Zero PDA (single RW borrow)
    {
        let mut pda_mut = accounts.pda.try_borrow_mut_data()?;
        #[allow(unused_unsafe)]
        unsafe {
            sol_memset(&mut pda_mut, 0, data_len)
        };
    }

    // Assign the PDA to the delegation program if not already assigned
    if accounts.pda.owner != accounts.system_program.key {
        accounts.pda.assign(accounts.system_program.key);
    }
    if accounts.pda.owner != accounts.delegation_program.key {
        invoke_signed(
            &solana_system_interface::instruction::assign(
                accounts.pda.key.as_modern(),
                accounts.delegation_program.key.as_modern(),
            ),
            &[accounts.pda.modern(), accounts.system_program.modern()],
            pda_signer_seeds,
        )
        .compat()?;
    }

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

#[allow(clippy::needless_lifetimes)]
pub fn delegate_account_with_actions<'a, 'info>(
    accounts: DelegateAccounts<'a, 'info>,
    pda_seeds: &[&[u8]],
    config: DelegateConfig,
    actions: PostDelegationActions,
    action_signer_infos: &'a [&'a compat::AccountInfo<'info>],
) -> compat::ProgramResult {
    let buffer_seeds: &[&[u8]] = delegate_buffer_seeds_from_delegated_account!(accounts.pda.key);

    let (_, delegate_account_bump) =
        Address::find_program_address(pda_seeds, accounts.owner_program.key.as_modern());

    let (_, buffer_pda_bump) =
        Address::find_program_address(buffer_seeds, accounts.owner_program.key.as_modern());

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
        buf.copy_from_slice(&pda_ro);
    }

    // Zero PDA (single RW borrow)
    {
        let mut pda_mut = accounts.pda.try_borrow_mut_data()?;
        #[allow(unused_unsafe)]
        unsafe {
            sol_memset(&mut pda_mut, 0, data_len)
        };
    }

    // Assign the PDA to the delegation program if not already assigned
    if accounts.pda.owner != accounts.system_program.key {
        accounts.pda.assign(accounts.system_program.key);
    }
    if accounts.pda.owner != accounts.delegation_program.key {
        invoke_signed(
            &solana_system_interface::instruction::assign(
                accounts.pda.key.as_modern(),
                accounts.delegation_program.key.as_modern(),
            ),
            &[accounts.pda.modern(), accounts.system_program.modern()],
            pda_signer_seeds,
        )
        .compat()?;
    }

    let seeds_vec: Vec<Vec<u8>> = pda_seeds.iter().map(|&slice| slice.to_vec()).collect();

    let delegate_args = DelegateArgs {
        commit_frequency_ms: config.commit_frequency_ms,
        seeds: seeds_vec,
        validator: config.validator,
    };
    let args = DelegateWithActionsArgs {
        delegate: delegate_args,
        actions,
    };

    cpi_delegate_with_actions(
        accounts.payer,
        accounts.pda,
        accounts.owner_program,
        accounts.buffer,
        accounts.delegation_record,
        accounts.delegation_metadata,
        accounts.system_program,
        pda_signer_seeds,
        args,
        action_signer_infos,
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
    delegated_account: &'a compat::AccountInfo<'info>,
    owner_program: &compat::Pubkey,
    buffer: &'a compat::AccountInfo<'info>,
    payer: &'a compat::AccountInfo<'info>,
    system_program: &'a compat::AccountInfo<'info>,
    account_signer_seeds: Vec<Vec<u8>>,
) -> compat::ProgramResult {
    if !buffer.is_signer {
        return Err(ProgramError::MissingRequiredSignature.compat());
    }
    if buffer.owner != &DELEGATION_PROGRAM_ID {
        return Err(ProgramError::InvalidAccountOwner.compat());
    }

    let account_seeds: Vec<&[u8]> = account_signer_seeds.iter().map(|v| v.as_slice()).collect();

    let (_, account_bump) =
        Address::find_program_address(account_seeds.as_ref(), owner_program.as_modern());

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
    payer: &'a compat::AccountInfo<'info>,
    delegate_account: &'a compat::AccountInfo<'info>,
    owner_program: &'a compat::AccountInfo<'info>,
    buffer: &'a compat::AccountInfo<'info>,
    delegation_record: &'a compat::AccountInfo<'info>,
    delegation_metadata: &'a compat::AccountInfo<'info>,
    system_program: &'a compat::AccountInfo<'info>,
    signers_seeds: &[&[&[u8]]],
    args: DelegateAccountArgs,
) -> compat::ProgramResult {
    modernize!(
        payer,
        delegate_account,
        owner_program,
        buffer,
        delegation_record,
        delegation_metadata,
        system_program
    );

    let mut data: Vec<u8> = vec![0u8; 8];
    args.serialize(&mut data)
        .map_err(|_| compat::latest::ProgramError::BorshIoError.compat())?;

    let delegation_instruction = Instruction {
        program_id: crate::id().to_bytes().into(),
        accounts: vec![
            AccountMeta::new(payer.key.to_bytes().into(), true),
            AccountMeta::new(delegate_account.key.to_bytes().into(), true),
            AccountMeta::new_readonly(owner_program.key.to_bytes().into(), false),
            AccountMeta::new(buffer.key.to_bytes().into(), false),
            AccountMeta::new(delegation_record.key.to_bytes().into(), false),
            AccountMeta::new(delegation_metadata.key.to_bytes().into(), false),
            AccountMeta::new_readonly(system_program.key.to_bytes().into(), false),
        ],
        data,
    };

    invoke_signed(
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
    .compat()
}

/// CPI to the delegation program to delegate the account with actions
#[allow(clippy::too_many_arguments)]
pub fn cpi_delegate_with_actions<'a, 'info>(
    payer: &'a compat::AccountInfo<'info>,
    delegate_account: &'a compat::AccountInfo<'info>,
    owner_program: &'a compat::AccountInfo<'info>,
    buffer: &'a compat::AccountInfo<'info>,
    delegation_record: &'a compat::AccountInfo<'info>,
    delegation_metadata: &'a compat::AccountInfo<'info>,
    system_program: &'a compat::AccountInfo<'info>,
    signers_seeds: &[&[&[u8]]],
    args: DelegateWithActionsArgs,
    action_signer_infos: &'a [&'a compat::AccountInfo<'info>],
) -> compat::ProgramResult {
    modernize!(
        payer,
        delegate_account,
        owner_program,
        buffer,
        delegation_record,
        delegation_metadata,
        system_program
    );

    let mut data = DlpDiscriminator::DelegateWithActions.to_vec();
    let payload =
        borsh::to_vec(&args).map_err(|_| ProgramError::InvalidInstructionData.compat())?;
    data.extend_from_slice(&payload);

    let mut accounts = vec![
        AccountMeta::new(payer.key.to_bytes().into(), true),
        AccountMeta::new(delegate_account.key.to_bytes().into(), true),
        AccountMeta::new_readonly(owner_program.key.to_bytes().into(), false),
        AccountMeta::new(buffer.key.to_bytes().into(), false),
        AccountMeta::new(delegation_record.key.to_bytes().into(), false),
        AccountMeta::new(delegation_metadata.key.to_bytes().into(), false),
        AccountMeta::new_readonly(system_program.key.to_bytes().into(), false),
    ];

    let mut signer_infos = Vec::new();
    for signer in &args.actions.signers {
        let info = action_signer_infos
            .iter()
            .find(|ai| *ai.key.as_array() == *signer)
            .ok_or(ProgramError::NotEnoughAccountKeys.compat())?
            .as_modern();
        accounts.push(AccountMeta::new_readonly(*info.key, true));
        signer_infos.push((*info).clone());
    }

    let delegation_instruction = Instruction {
        program_id: crate::id().to_bytes().into(),
        accounts,
        data,
    };

    let mut invoke_accounts = vec![
        payer.clone(),
        delegate_account.clone(),
        owner_program.clone(),
        buffer.clone(),
        delegation_record.clone(),
        delegation_metadata.clone(),
        system_program.clone(),
    ];
    invoke_accounts.extend(signer_infos);

    invoke_signed(&delegation_instruction, &invoke_accounts, signers_seeds).compat()
}
