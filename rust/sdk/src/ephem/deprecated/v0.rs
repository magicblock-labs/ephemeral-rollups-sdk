use crate::solana_compat::solana::{invoke, AccountInfo, AccountMeta, Instruction, ProgramResult};
use magicblock_magic_program_api::instruction::MagicBlockInstruction;

/// CPI to trigger a commit for one or more accounts in the ER
#[inline(always)]
pub fn commit_accounts<'a, 'info>(
    payer: &'a AccountInfo<'info>,
    account_infos: Vec<&'a AccountInfo<'info>>,
    magic_context: &'a AccountInfo<'info>,
    magic_program: &'a AccountInfo<'info>,
) -> ProgramResult {
    let ix = create_schedule_commit_ix(payer, &account_infos, magic_context, magic_program, false);
    let mut all_accounts = vec![payer.clone(), magic_context.clone()];
    all_accounts.extend(account_infos.into_iter().cloned());
    invoke(&ix, &all_accounts)
}

/// CPI to trigger a commit and undelegate one or more accounts in the ER
#[inline(always)]
pub fn commit_and_undelegate_accounts<'a, 'info>(
    payer: &'a AccountInfo<'info>,
    account_infos: Vec<&'a AccountInfo<'info>>,
    magic_context: &'a AccountInfo<'info>,
    magic_program: &'a AccountInfo<'info>,
) -> ProgramResult {
    let ix = create_schedule_commit_ix(payer, &account_infos, magic_context, magic_program, true);
    let mut all_accounts = vec![payer.clone(), magic_context.clone()];
    all_accounts.extend(account_infos.into_iter().cloned());
    invoke(&ix, &all_accounts)
}

pub fn create_schedule_commit_ix<'a, 'info>(
    payer: &'a AccountInfo<'info>,
    account_infos: &[&'a AccountInfo<'info>],
    magic_context: &'a AccountInfo<'info>,
    magic_program: &'a AccountInfo<'info>,
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
    account_metas.extend(account_infos.iter().map(|x| AccountMeta {
        pubkey: *x.key,
        is_signer: x.is_signer,
        is_writable: x.is_writable,
    }));
    Instruction::new_with_bincode(*magic_program.key, &instruction, account_metas)
}
