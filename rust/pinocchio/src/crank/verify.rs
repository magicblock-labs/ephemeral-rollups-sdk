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

use pinocchio::{
    error::ProgramError, sysvars::instructions::INSTRUCTIONS_ID, AccountView, Address,
};

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
    instructions_sysvar: &AccountView,
    position: u16,
) -> Result<Option<Address>, ProgramError> {
    if instructions_sysvar.address() != &INSTRUCTIONS_ID {
        return Err(ProgramError::UnsupportedSysvar);
    }
    ensure_top_level()?;
    let data = instructions_sysvar.try_borrow()?;
    Ok(hydra_trigger_crank(&data, position)?.map(Address::new_from_array))
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
    instructions_sysvar: &AccountView,
    expected_crank: &Address,
    position: u16,
) -> Result<(), ProgramError> {
    match hydra_crank_caller(instructions_sysvar, position)? {
        Some(crank) if &crank == expected_crank => Ok(()),
        _ => Err(ProgramError::IncorrectAuthority),
    }
}

#[cfg(any(target_os = "solana", target_arch = "bpf"))]
fn ensure_top_level() -> Result<(), ProgramError> {
    const TRANSACTION_LEVEL_STACK_HEIGHT: u64 = 1;
    // SAFETY: syscall with no arguments.
    if unsafe { pinocchio::syscalls::sol_get_stack_height() } != TRANSACTION_LEVEL_STACK_HEIGHT {
        return Err(ProgramError::IncorrectAuthority);
    }
    Ok(())
}

#[cfg(not(any(target_os = "solana", target_arch = "bpf")))]
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
    extern crate std;
    use super::*;
    use solana_program::{
        instruction::{AccountMeta, Instruction},
        pubkey::Pubkey,
        sysvar::instructions::{
            construct_instructions_data, BorrowedAccountMeta, BorrowedInstruction, ID,
        },
    };
    use std::{vec, vec::Vec};

    const CRANK: [u8; 32] = [7; 32];
    const OTHER_CRANK: [u8; 32] = [8; 32];

    fn trigger(program: [u8; 32], crank: [u8; 32]) -> Instruction {
        Instruction::new_with_bytes(
            Pubkey::new_from_array(program),
            &[hydra_api::ix::TRIGGER],
            vec![
                AccountMeta::new(Pubkey::new_from_array(crank), false),
                AccountMeta::new(Pubkey::new_from_array([1; 32]), true),
                AccountMeta::new_readonly(ID, false),
            ],
        )
    }

    fn scheduled() -> Instruction {
        Instruction::new_with_bytes(
            Pubkey::new_from_array([9; 32]),
            &[42],
            vec![AccountMeta::new_readonly(ID, false)],
        )
    }

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

    fn crank_at(ixs: &[Instruction], current: u16, position: u16) -> Option<[u8; 32]> {
        hydra_trigger_crank(&sysvar_data(ixs, current), position).unwrap()
    }

    #[test]
    fn instructions_sysvar_id_matches() {
        assert_eq!(INSTRUCTIONS_ID.as_array(), &ID.to_bytes());
    }

    #[test]
    fn finds_base_and_ephemeral_trigger() {
        for program in [hydra_api::base::ID, hydra_api::ephemeral::ID] {
            let ixs = [trigger(program.to_bytes(), CRANK), scheduled()];
            assert_eq!(crank_at(&ixs, 1, 0), Some(CRANK));
        }
    }

    #[test]
    fn finds_trigger_for_later_position() {
        let ixs = [
            trigger(hydra_api::base::ID.to_bytes(), OTHER_CRANK),
            scheduled(),
            scheduled(),
            scheduled(),
        ];
        assert_eq!(crank_at(&ixs, 3, 2), Some(OTHER_CRANK));
        assert_eq!(crank_at(&ixs, 3, 0), None);
    }

    #[test]
    fn ignores_appended_ix_and_missing_trigger() {
        let ixs = [
            trigger(hydra_api::base::ID.to_bytes(), CRANK),
            scheduled(),
            scheduled(),
        ];
        assert_eq!(crank_at(&ixs, 2, 0), None);
        assert_eq!(crank_at(&[scheduled()], 0, 0), None);
    }

    #[test]
    fn ignores_non_hydra_and_non_trigger() {
        assert_eq!(
            crank_at(&[trigger([5; 32], CRANK), scheduled()], 1, 0),
            None
        );
        let mut cancel = trigger(hydra_api::base::ID.to_bytes(), CRANK);
        cancel.data = vec![hydra_api::ix::CANCEL];
        assert_eq!(crank_at(&[cancel, scheduled()], 1, 0), None);
    }

    #[test]
    fn rejects_malformed_sysvar() {
        assert_eq!(
            hydra_trigger_crank(&[5, 0, 1], 0),
            Err(ProgramError::InvalidInstructionData)
        );
    }
}
