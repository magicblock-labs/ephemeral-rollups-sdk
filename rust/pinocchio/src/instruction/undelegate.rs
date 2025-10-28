use core::mem::MaybeUninit;
use pinocchio::pubkey::Pubkey;
use pinocchio::{
    account_info::AccountInfo,
    cpi::MAX_CPI_ACCOUNTS,
    instruction::{Seed, Signer},
    program_error::ProgramError,
    pubkey,
    pubkey::find_program_address,
    sysvars::{rent::Rent, Sysvar},
    ProgramResult,
};
use pinocchio_system::instructions::CreateAccount;

use crate::utils::get_seeds;

#[allow(clippy::cloned_ref_to_slice_refs)]
pub fn undelegate(
    delegated_account: &AccountInfo,
    owner_program: &Pubkey,
    buffer: &AccountInfo,
    payer: &AccountInfo,
    mut callback_args: &[u8],
) -> ProgramResult {
    if !buffer.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Parse PDA seeds from instruction data: Borsh-serialized Vec<Vec<u8>>.
    // Format: u32 vec_len, then for each: u32 elem_len, then elem_len bytes.
    let read_u32 = |bytes: &mut &[u8]| -> Result<u32, ProgramError> {
        if bytes.len() < 4 {
            return Err(ProgramError::InvalidInstructionData);
        }
        let val = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        *bytes = &bytes[4..];
        Ok(val)
    };

    let seeds_len = read_u32(&mut callback_args)? as usize;
    if seeds_len > 16 {
        // Limit to 16 seeds (15 + bump typically). Adjust as necessary.
        return Err(ProgramError::InvalidInstructionData);
    }

    // Collect slices into a fixed-size stack array, avoiding heap allocations.
    let mut seed_refs: [&[u8]; 16] = [&[]; 16];
    let mut i = 0usize;
    while i < seeds_len {
        let elem_len = read_u32(&mut callback_args)? as usize;
        if callback_args.len() < elem_len {
            return Err(ProgramError::InvalidInstructionData);
        }
        let (head, rest) = callback_args.split_at(elem_len);
        seed_refs[i] = head;
        callback_args = rest;
        i += 1;
    }

    // Any trailing bytes are invalid
    if !callback_args.is_empty() {
        return Err(ProgramError::InvalidInstructionData);
    }
    let pda_seeds: &[&[u8]] = &seed_refs[..seeds_len];

    // Find delegate
    let (_, delegate_account_bump) = find_program_address(pda_seeds, owner_program);

    // Get Delegated Pda Signer Seeds
    let binding = &[delegate_account_bump];
    let delegate_bump = Seed::from(binding);
    let packed_pda_seeds = get_seeds(pda_seeds)?;

    let num_seeds = packed_pda_seeds.len() + 1;
    if num_seeds > MAX_CPI_ACCOUNTS {
        return Err(ProgramError::InvalidArgument);
    }

    const UNINIT_SEED: MaybeUninit<Seed> = MaybeUninit::<Seed>::uninit();
    let mut combined_seeds = [UNINIT_SEED; MAX_CPI_ACCOUNTS];

    unsafe {
        for i in 0..num_seeds - 1 {
            let seed = packed_pda_seeds.get_unchecked(i);
            combined_seeds
                .get_unchecked_mut(i)
                .write(Seed::from(seed.as_ref()));
        }

        combined_seeds
            .get_unchecked_mut(num_seeds - 1)
            .write(delegate_bump);
    }

    let all_delegate_seeds =
        unsafe { core::slice::from_raw_parts(combined_seeds.as_ptr() as *const Seed, num_seeds) };

    let pda_signer_seeds = Signer::from(all_delegate_seeds);

    // Create the original PDA Account Delegated
    pubkey::log(delegated_account.key());

    CreateAccount {
        from: payer,
        to: delegated_account,
        lamports: Rent::get()?.minimum_balance(buffer.data_len()),
        space: buffer.data_len() as u64,
        owner: owner_program,
    }
    .invoke_signed(&[pda_signer_seeds.clone()])?;

    let mut data = delegated_account.try_borrow_mut_data()?;
    let buffer_data = buffer.try_borrow_data()?;
    (*data).copy_from_slice(&buffer_data);

    Ok(())
}
