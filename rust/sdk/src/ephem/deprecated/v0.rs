use crate::solana_compat::solana::{invoke, AccountInfo, AccountMeta, Instruction, ProgramResult};
use magicblock_magic_program_api::instruction::MagicBlockInstruction;

#[inline(always)]
pub fn commit_accounts<'a, 'info>(
    payer: &'a AccountInfo<'info>,
    account_infos: Vec<&'a AccountInfo<'info>>,
    magic_context: &'a AccountInfo<'info>,
    magic_program: &'a AccountInfo<'info>,
    magic_fee_vault: Option<&'a AccountInfo<'info>>,
) -> ProgramResult {
    let ix = create_schedule_commit_ix(
        payer,
        &account_infos,
        magic_context,
        magic_program,
        magic_fee_vault,
        false,
    );
    let mut all_accounts = vec![payer.clone(), magic_context.clone()];
    if let Some(vault) = magic_fee_vault {
        all_accounts.push(vault.clone());
    }
    all_accounts.extend(account_infos.into_iter().cloned());
    invoke(&ix, &all_accounts)
}

/// CPI to trigger a commit and undelegate one or more accounts in the ER.
/// Pass `magic_fee_vault` when the payer is a delegated ephemeral balance account.
#[inline(always)]
pub fn commit_and_undelegate_accounts<'a, 'info>(
    payer: &'a AccountInfo<'info>,
    account_infos: Vec<&'a AccountInfo<'info>>,
    magic_context: &'a AccountInfo<'info>,
    magic_program: &'a AccountInfo<'info>,
    magic_fee_vault: Option<&'a AccountInfo<'info>>,
) -> ProgramResult {
    let ix = create_schedule_commit_ix(
        payer,
        &account_infos,
        magic_context,
        magic_program,
        magic_fee_vault,
        true,
    );
    let mut all_accounts = vec![payer.clone(), magic_context.clone()];
    if let Some(vault) = magic_fee_vault {
        all_accounts.push(vault.clone());
    }
    all_accounts.extend(account_infos.into_iter().cloned());
    invoke(&ix, &all_accounts)
}

pub fn create_schedule_commit_ix<'a, 'info>(
    payer: &'a AccountInfo<'info>,
    account_infos: &[&'a AccountInfo<'info>],
    magic_context: &'a AccountInfo<'info>,
    magic_program: &'a AccountInfo<'info>,
    magic_fee_vault: Option<&'a AccountInfo<'info>>,
    allow_undelegation: bool,
) -> Instruction {
    let instruction = if allow_undelegation {
        MagicBlockInstruction::ScheduleCommitAndUndelegate
    } else {
        MagicBlockInstruction::ScheduleCommit
    };
    let mut account_metas = vec![
        AccountMeta {
            pubkey: *payer.key,
            is_signer: true,
            is_writable: true,
        },
        AccountMeta {
            pubkey: *magic_context.key,
            is_signer: false,
            is_writable: true,
        },
    ];
    if let Some(vault) = magic_fee_vault {
        account_metas.push(AccountMeta {
            pubkey: *vault.key,
            is_signer: false,
            is_writable: true,
        });
    }
    account_metas.extend(account_infos.iter().map(|x| AccountMeta {
        pubkey: *x.key,
        is_signer: x.is_signer,
        is_writable: x.is_writable,
    }));
    Instruction::new_with_bincode(*magic_program.key, &instruction, account_metas)
}

/// CPI to trigger a commit-finalize for one or more accounts in the ER
#[inline(always)]
pub fn commit_finalize_accounts<'a, 'info>(
    payer: &'a AccountInfo<'info>,
    account_infos: Vec<&'a AccountInfo<'info>>,
    magic_context: &'a AccountInfo<'info>,
    magic_program: &'a AccountInfo<'info>,
) -> ProgramResult {
    let ix = create_finalize_schedule_commit_ix(
        payer,
        &account_infos,
        magic_context,
        magic_program,
        false,
    );
    let mut all_accounts = vec![payer.clone(), magic_context.clone()];
    all_accounts.extend(account_infos.into_iter().cloned());
    invoke(&ix, &all_accounts)
}
/// CPI to trigger a commit-finalize and undelegate one or more accounts in the ER
#[inline(always)]
pub fn commit_finalize_and_undelegate_accounts<'a, 'info>(
    payer: &'a AccountInfo<'info>,
    account_infos: Vec<&'a AccountInfo<'info>>,
    magic_context: &'a AccountInfo<'info>,
    magic_program: &'a AccountInfo<'info>,
) -> ProgramResult {
    let ix = create_finalize_schedule_commit_ix(
        payer,
        &account_infos,
        magic_context,
        magic_program,
        true,
    );
    let mut all_accounts = vec![payer.clone(), magic_context.clone()];
    all_accounts.extend(account_infos.into_iter().cloned());
    invoke(&ix, &all_accounts)
}
pub fn create_finalize_schedule_commit_ix<'a, 'info>(
    payer: &'a AccountInfo<'info>,
    account_infos: &[&'a AccountInfo<'info>],
    magic_context: &'a AccountInfo<'info>,
    magic_program: &'a AccountInfo<'info>,
    request_undelegation: bool,
) -> Instruction {
    let instruction = MagicBlockInstruction::ScheduleCommitFinalize {
        request_undelegation,
    };
    let mut account_metas = vec![
        AccountMeta {
            pubkey: *payer.key,
            is_signer: true,
            is_writable: true,
        },
        AccountMeta {
            pubkey: *magic_context.key,
            is_signer: false,
            is_writable: true,
        },
    ];
    account_metas.extend(account_infos.iter().map(|x| AccountMeta {
        pubkey: *x.key,
        is_signer: x.is_signer,
        is_writable: x.is_writable,
    }));
    Instruction::new_with_bincode(*magic_program.key, &instruction, account_metas)
}
