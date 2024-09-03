use solana_program::account_info::AccountInfo;
use solana_program::entrypoint::ProgramResult;
use solana_program::instruction::{AccountMeta, Instruction};
use solana_program::program::invoke;

/// CPI to trigger a commit for one or more accounts in the ER
#[inline(always)]
pub fn commit_accounts<'a, 'info>(
    payer: &'a AccountInfo<'info>,
    account_infos: Vec<&'a AccountInfo<'info>>,
    magic_program: &'a AccountInfo<'info>,
) -> ProgramResult {
    let mut accounts = vec![payer];
    accounts.extend(account_infos.iter());
    let ix = create_schedule_commit_ix(magic_program, &accounts, false);
    invoke(&ix, &accounts.into_iter().cloned().collect::<Vec<_>>())
}

/// CPI to trigger a commit and undelegate one or more accounts in the ER
#[inline(always)]
pub fn commit_and_undelegate_accounts<'a, 'info>(
    payer: &'a AccountInfo<'info>,
    account_infos: Vec<&'a AccountInfo<'info>>,
    magic_program: &'a AccountInfo<'info>,
) -> ProgramResult {
    let mut accounts = vec![payer];
    accounts.extend(account_infos.iter());
    let ix = create_schedule_commit_ix(magic_program, &accounts, true);
    invoke(&ix, &accounts.into_iter().cloned().collect::<Vec<_>>())
}

pub fn create_schedule_commit_ix<'a, 'info>(
    magic_program: &'a AccountInfo<'info>,
    account_infos: &[&'a AccountInfo<'info>],
    allow_undelegation: bool,
) -> Instruction {
    let instruction_data = if allow_undelegation {
        vec![2, 0, 0, 0]
    } else {
        vec![1, 0, 0, 0]
    };
    let account_metas = account_infos
        .iter()
        .map(|x| AccountMeta {
            pubkey: *x.key,
            is_signer: x.is_signer,
            is_writable: x.is_writable,
        })
        .collect::<Vec<AccountMeta>>();
    Instruction::new_with_bytes(*magic_program.key, &instruction_data, account_metas)
}