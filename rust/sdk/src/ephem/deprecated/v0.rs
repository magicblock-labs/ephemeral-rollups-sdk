use crate::compat::{self, AsModern, Compat, Modern};
use magicblock_magic_program_api::instruction::MagicBlockInstruction;
use solana_program::instruction::{AccountMeta, Instruction};
use solana_program::program::invoke;

/// CPI to trigger a commit of one or more accounts in the ER.
/// Pass `magic_fee_vault` when the payer is a delegated ephemeral balance account
/// so that commit fees can be collected by the magic program. The vault must be
/// the writable magic fee vault PDA for the current validator. Pass `None` when
/// no fee collection is required (e.g. the payer is not delegated).
#[inline(always)]
pub fn commit_accounts<'a, 'info>(
    payer: &'a compat::AccountInfo<'info>,
    account_infos: Vec<&'a compat::AccountInfo<'info>>,
    magic_context: &'a compat::AccountInfo<'info>,
    magic_program: &'a compat::AccountInfo<'info>,
    magic_fee_vault: Option<&'a compat::AccountInfo<'info>>,
) -> compat::ProgramResult {
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
    invoke(&ix.modern(), &all_accounts.modern()).compat()
}

/// CPI to trigger a commit and undelegate one or more accounts in the ER.
/// Pass `magic_fee_vault` when the payer is a delegated ephemeral balance account.
#[inline(always)]
pub fn commit_and_undelegate_accounts<'a, 'info>(
    payer: &'a compat::AccountInfo<'info>,
    account_infos: Vec<&'a compat::AccountInfo<'info>>,
    magic_context: &'a compat::AccountInfo<'info>,
    magic_program: &'a compat::AccountInfo<'info>,
    magic_fee_vault: Option<&'a compat::AccountInfo<'info>>,
) -> compat::ProgramResult {
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
    invoke(&ix.modern(), &all_accounts.modern()).compat()
}

pub fn create_schedule_commit_ix<'a, 'info>(
    payer: &'a compat::AccountInfo<'info>,
    account_infos: &[&'a compat::AccountInfo<'info>],
    magic_context: &'a compat::AccountInfo<'info>,
    magic_program: &'a compat::AccountInfo<'info>,
    magic_fee_vault: Option<&'a compat::AccountInfo<'info>>,
    allow_undelegation: bool,
) -> compat::Instruction {
    let instruction = if allow_undelegation {
        MagicBlockInstruction::ScheduleCommitAndUndelegate
    } else {
        MagicBlockInstruction::ScheduleCommit
    };
    let mut account_metas = vec![
        AccountMeta {
            pubkey: *payer.key.as_modern(),
            is_signer: true,
            is_writable: true,
        },
        AccountMeta {
            pubkey: *magic_context.key.as_modern(),
            is_signer: false,
            is_writable: true,
        },
    ];
    if let Some(vault) = magic_fee_vault {
        account_metas.push(AccountMeta {
            pubkey: *vault.key.as_modern(),
            is_signer: false,
            is_writable: true,
        });
    }
    account_metas.extend(account_infos.iter().map(|x| AccountMeta {
        pubkey: *x.key.as_modern(),
        is_signer: x.is_signer,
        is_writable: x.is_writable,
    }));
    Instruction::new_with_bincode(*magic_program.key.as_modern(), &instruction, account_metas)
        .compat()
}

/// CPI to trigger a commit-finalize for one or more accounts in the ER
#[inline(always)]
pub fn commit_finalize_accounts<'a, 'info>(
    payer: &'a compat::AccountInfo<'info>,
    account_infos: Vec<&'a compat::AccountInfo<'info>>,
    magic_context: &'a compat::AccountInfo<'info>,
    magic_program: &'a compat::AccountInfo<'info>,
) -> compat::ProgramResult {
    let ix = create_finalize_schedule_commit_ix(
        payer,
        &account_infos,
        magic_context,
        magic_program,
        false,
    );
    let mut all_accounts = vec![payer.clone(), magic_context.clone()];
    all_accounts.extend(account_infos.into_iter().cloned());
    invoke(&ix.modern(), &all_accounts.modern()).compat()
}
/// CPI to trigger a commit-finalize and undelegate one or more accounts in the ER
#[inline(always)]
pub fn commit_finalize_and_undelegate_accounts<'a, 'info>(
    payer: &'a compat::AccountInfo<'info>,
    account_infos: Vec<&'a compat::AccountInfo<'info>>,
    magic_context: &'a compat::AccountInfo<'info>,
    magic_program: &'a compat::AccountInfo<'info>,
) -> compat::ProgramResult {
    let ix = create_finalize_schedule_commit_ix(
        payer,
        &account_infos,
        magic_context,
        magic_program,
        true,
    );
    let mut all_accounts = vec![payer.clone(), magic_context.clone()];
    all_accounts.extend(account_infos.into_iter().cloned());
    invoke(&ix.modern(), &all_accounts.modern()).compat()
}
pub fn create_finalize_schedule_commit_ix<'a, 'info>(
    payer: &'a compat::AccountInfo<'info>,
    account_infos: &[&'a compat::AccountInfo<'info>],
    magic_context: &'a compat::AccountInfo<'info>,
    magic_program: &'a compat::AccountInfo<'info>,
    request_undelegation: bool,
) -> compat::Instruction {
    let instruction = MagicBlockInstruction::ScheduleCommitFinalize {
        request_undelegation,
    };
    let mut account_metas = vec![
        AccountMeta {
            pubkey: *payer.key.as_modern(),
            is_signer: true,
            is_writable: true,
        },
        AccountMeta {
            pubkey: *magic_context.key.as_modern(),
            is_signer: false,
            is_writable: true,
        },
    ];
    account_metas.extend(account_infos.iter().map(|x| AccountMeta {
        pubkey: *x.key.as_modern(),
        is_signer: x.is_signer,
        is_writable: x.is_writable,
    }));
    Instruction::new_with_bincode(*magic_program.key.as_modern(), &instruction, account_metas)
        .compat()
}
