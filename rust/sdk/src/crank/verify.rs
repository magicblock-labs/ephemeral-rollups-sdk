//! Verify that the current instruction is being executed by a Hydra crank.
//!
//! Hydra executes a crank as a top-level `Trigger` instruction followed, in the
//! same transaction, by the crank's scheduled instructions. `Trigger`
//! byte-compares those follow-up instructions against the template stored in
//! the crank account, so a scheduled program can authenticate its caller by
//! reading the instructions sysvar: the `Trigger` sits `position + 1`
//! instructions before the current one, and its first account is the crank.
//!
//! This replaces checking the legacy crank signer PDA
//! (`crank_signer_pda(authority)`, seed `b"crank-executor"`), which only exists
//! for magic-program cranks and cannot be used with Hydra (Hydra rejects signer
//! metas in scheduled instructions).

use solana_program::{program_error::ProgramError, sysvar::instructions};

use crate::compat::{self, AsModern, Compat};

/// Returns the crank that triggered the current instruction, or `None` if the
/// instruction is not executed by a Hydra crank at `position`.
///
/// `position` is the index of the current instruction within the crank's
/// scheduled instructions (`0` = scheduled right after `Trigger`).
///
/// The result is only meaningful for a crank whose schedule has more than
/// `position` instructions: compare it against a crank you created yourself
/// (see [`require_hydra_crank`]). Any crank can be created permissionlessly,
/// so `Some(_)` alone does **not** mean the call is trusted.
///
/// Fails if `instructions_sysvar` is not the instructions sysvar or if the
/// instruction is not top-level (invoked through CPI).
pub fn hydra_crank_caller(
    instructions_sysvar: &compat::AccountInfo,
    position: u16,
) -> Result<Option<compat::Pubkey>, compat::ProgramError> {
    hydra_crank_caller_modern(instructions_sysvar.as_modern(), position)
        .map(|crank| crank.map(|c| solana_address::Address::new_from_array(c).compat()))
        .map_err(Compat::compat)
}

/// Fails unless the current instruction is the scheduled instruction at
/// `position` of `expected_crank`, executed by Hydra (base layer or ephemeral
/// rollup). Use `position = 0` for single-instruction cranks.
///
/// The instructions sysvar must be passed to the scheduled instruction as a
/// read-only account. Do **not** list the crank PDA in your own instruction's
/// accounts: `Trigger` marks it writable, which breaks Hydra's byte-match.
///
/// Derive `expected_crank` from a crank your program controls, e.g. with
/// [`find_base_crank_pda`](super::hydra::find_base_crank_pda) or
/// [`find_ephemeral_crank_pda`](super::hydra::find_ephemeral_crank_pda).
pub fn require_hydra_crank(
    instructions_sysvar: &compat::AccountInfo,
    expected_crank: &compat::Pubkey,
    position: u16,
) -> compat::ProgramResult {
    require_hydra_crank_modern(
        instructions_sysvar.as_modern(),
        expected_crank.as_modern().as_array(),
        position,
    )
    .compat()
}

fn hydra_crank_caller_modern(
    instructions_sysvar: &solana_program::account_info::AccountInfo,
    position: u16,
) -> Result<Option<[u8; 32]>, ProgramError> {
    if instructions_sysvar.key != &instructions::ID {
        return Err(ProgramError::UnsupportedSysvar);
    }
    ensure_top_level()?;
    let data = instructions_sysvar.try_borrow_data()?;
    hydra_trigger_crank(&data, position)
}

fn require_hydra_crank_modern(
    instructions_sysvar: &solana_program::account_info::AccountInfo,
    expected_crank: &[u8; 32],
    position: u16,
) -> Result<(), ProgramError> {
    match hydra_crank_caller_modern(instructions_sysvar, position)? {
        Some(crank) if &crank == expected_crank => Ok(()),
        _ => Err(ProgramError::IncorrectAuthority),
    }
}

#[cfg(target_os = "solana")]
fn ensure_top_level() -> Result<(), ProgramError> {
    use solana_program::instruction::{get_stack_height, TRANSACTION_LEVEL_STACK_HEIGHT};
    if get_stack_height() != TRANSACTION_LEVEL_STACK_HEIGHT {
        return Err(ProgramError::IncorrectAuthority);
    }
    Ok(())
}

#[cfg(not(target_os = "solana"))]
fn ensure_top_level() -> Result<(), ProgramError> {
    Ok(())
}

/// Parses raw instructions-sysvar data and returns the crank of the Hydra
/// `Trigger` located `position + 1` instructions before the current one.
pub(crate) fn hydra_trigger_crank(
    data: &[u8],
    position: u16,
) -> Result<Option<[u8; 32]>, ProgramError> {
    let current = read_u16(data, data.len().checked_sub(2))?;
    let Some(trigger_index) = current.checked_sub(position).and_then(|i| i.checked_sub(1)) else {
        return Ok(None);
    };
    let num_instructions = read_u16(data, Some(0))?;
    if trigger_index >= num_instructions {
        return Err(ProgramError::InvalidInstructionData);
    }

    let mut offset = read_u16(data, Some(2 + 2 * trigger_index as usize))? as usize;
    let num_accounts = read_u16(data, Some(offset))? as usize;
    offset += 2;
    // First meta: [flags: u8][pubkey: 32]. The crank is account 0 of `Trigger`.
    let first_account = data.get(offset + 1..offset + 33);
    offset += num_accounts * hydra_api::SERIALIZED_META_SIZE;
    let program_id = data
        .get(offset..offset + 32)
        .ok_or(ProgramError::InvalidInstructionData)?;
    offset += 32;
    let data_len = read_u16(data, Some(offset))? as usize;
    offset += 2;
    let ix_data = data
        .get(offset..offset + data_len)
        .ok_or(ProgramError::InvalidInstructionData)?;

    let is_hydra = program_id == hydra_api::base::ID.as_ref()
        || program_id == hydra_api::ephemeral::ID.as_ref();
    if !is_hydra || ix_data.first() != Some(&hydra_api::ix::TRIGGER) || num_accounts == 0 {
        return Ok(None);
    }
    let crank = first_account.ok_or(ProgramError::InvalidInstructionData)?;
    let mut out = [0u8; 32];
    out.copy_from_slice(crank);
    Ok(Some(out))
}

fn read_u16(data: &[u8], at: Option<usize>) -> Result<u16, ProgramError> {
    at.and_then(|at| data.get(at..at + 2))
        .map(|b| u16::from_le_bytes([b[0], b[1]]))
        .ok_or(ProgramError::InvalidInstructionData)
}

#[cfg(test)]
mod tests {
    use super::*;
    use solana_program::{
        account_info::AccountInfo,
        instruction::{AccountMeta, Instruction},
        pubkey::Pubkey,
        sysvar::instructions::{
            construct_instructions_data, BorrowedAccountMeta, BorrowedInstruction,
        },
    };

    const CRANK: Pubkey = Pubkey::new_from_array([7; 32]);
    const OTHER_CRANK: Pubkey = Pubkey::new_from_array([8; 32]);
    const MY_PROGRAM: Pubkey = Pubkey::new_from_array([9; 32]);

    fn trigger(program: Pubkey, crank: Pubkey) -> Instruction {
        Instruction::new_with_bytes(
            program,
            &[hydra_api::ix::TRIGGER],
            vec![
                AccountMeta::new(crank, false),
                AccountMeta::new(Pubkey::new_from_array([1; 32]), true),
                AccountMeta::new_readonly(instructions::ID, false),
            ],
        )
    }

    fn scheduled() -> Instruction {
        Instruction::new_with_bytes(
            MY_PROGRAM,
            &[42],
            vec![AccountMeta::new_readonly(instructions::ID, false)],
        )
    }

    /// Build instructions-sysvar data for `ixs` with `current` as the executing index.
    fn sysvar_data(ixs: &[Instruction], current: u16) -> Vec<u8> {
        let borrowed: Vec<BorrowedInstruction> = ixs
            .iter()
            .map(|ix| BorrowedInstruction {
                program_id: &ix.program_id,
                accounts: ix
                    .accounts
                    .iter()
                    .map(|m| BorrowedAccountMeta {
                        pubkey: &m.pubkey,
                        is_signer: m.is_signer,
                        is_writable: m.is_writable,
                    })
                    .collect(),
                data: &ix.data,
            })
            .collect();
        let mut data = construct_instructions_data(&borrowed);
        let len = data.len();
        data[len - 2..].copy_from_slice(&current.to_le_bytes());
        data
    }

    fn check(
        key: &Pubkey,
        data: &mut [u8],
        expected: &Pubkey,
        position: u16,
    ) -> Result<(), ProgramError> {
        let mut lamports = 0;
        let owner = Pubkey::default();
        let ai = AccountInfo::new(key, false, false, &mut lamports, data, &owner, false);
        require_hydra_crank_modern(&ai, expected.as_array(), position)
    }

    fn ok(ixs: &[Instruction], current: u16, position: u16) -> Result<(), ProgramError> {
        check(
            &instructions::ID,
            &mut sysvar_data(ixs, current),
            &CRANK,
            position,
        )
    }

    #[test]
    fn accepts_base_and_ephemeral_trigger() {
        for program in [hydra_api::base::ID, hydra_api::ephemeral::ID] {
            assert_eq!(ok(&[trigger(program, CRANK), scheduled()], 1, 0), Ok(()));
        }
    }

    #[test]
    fn accepts_later_position_in_crank() {
        let ixs = [
            trigger(hydra_api::base::ID, CRANK),
            scheduled(),
            scheduled(),
            scheduled(),
        ];
        assert_eq!(ok(&ixs, 3, 2), Ok(()));
        // Same instruction, wrong position: points at a scheduled ix, not Trigger.
        assert_eq!(ok(&ixs, 3, 0), Err(ProgramError::IncorrectAuthority));
    }

    #[test]
    fn accepts_trigger_after_compute_budget_ix() {
        let budget =
            Instruction::new_with_bytes(Pubkey::new_from_array([3; 32]), &[2, 0, 0, 0, 0], vec![]);
        let ixs = [budget, trigger(hydra_api::base::ID, CRANK), scheduled()];
        assert_eq!(ok(&ixs, 2, 0), Ok(()));
    }

    #[test]
    fn rejects_other_crank() {
        let ixs = [trigger(hydra_api::base::ID, OTHER_CRANK), scheduled()];
        assert_eq!(ok(&ixs, 1, 0), Err(ProgramError::IncorrectAuthority));
    }

    #[test]
    fn rejects_without_trigger() {
        assert_eq!(
            ok(&[scheduled()], 0, 0),
            Err(ProgramError::IncorrectAuthority)
        );
        assert_eq!(
            ok(&[scheduled(), scheduled()], 1, 0),
            Err(ProgramError::IncorrectAuthority)
        );
    }

    #[test]
    fn rejects_non_hydra_program_and_non_trigger_ix() {
        let fake = trigger(Pubkey::new_from_array([5; 32]), CRANK);
        assert_eq!(
            ok(&[fake, scheduled()], 1, 0),
            Err(ProgramError::IncorrectAuthority)
        );

        let mut cancel = trigger(hydra_api::base::ID, CRANK);
        cancel.data = vec![hydra_api::ix::CANCEL];
        assert_eq!(
            ok(&[cancel, scheduled()], 1, 0),
            Err(ProgramError::IncorrectAuthority)
        );
    }

    #[test]
    fn rejects_ix_appended_after_the_crank() {
        // [Trigger(CRANK), scheduled, attacker-appended] — the appended ix must
        // not pass by claiming position 0, since Trigger is 2 ixs back.
        let ixs = [
            trigger(hydra_api::base::ID, CRANK),
            scheduled(),
            scheduled(),
        ];
        assert_eq!(ok(&ixs, 2, 0), Err(ProgramError::IncorrectAuthority));
    }

    #[test]
    fn rejects_wrong_sysvar_account() {
        let mut data = sysvar_data(&[trigger(hydra_api::base::ID, CRANK), scheduled()], 1);
        assert_eq!(
            check(&Pubkey::new_from_array([4; 32]), &mut data, &CRANK, 0),
            Err(ProgramError::UnsupportedSysvar)
        );
    }

    #[test]
    fn rejects_malformed_sysvar() {
        assert_eq!(
            check(&instructions::ID, &mut [5, 0, 1], &CRANK, 0),
            Err(ProgramError::InvalidInstructionData)
        );
    }

    #[test]
    fn caller_returns_crank() {
        let data = sysvar_data(
            &[trigger(hydra_api::ephemeral::ID, OTHER_CRANK), scheduled()],
            1,
        );
        assert_eq!(
            hydra_trigger_crank(&data, 0),
            Ok(Some(OTHER_CRANK.to_bytes()))
        );
        assert_eq!(hydra_trigger_crank(&data, 1), Ok(None));
    }
}
