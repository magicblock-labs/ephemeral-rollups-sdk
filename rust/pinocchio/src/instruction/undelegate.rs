use core::mem::MaybeUninit;
use pinocchio::pubkey::Pubkey;
use pinocchio::{
    account_info::AccountInfo,
    cpi::MAX_CPI_ACCOUNTS,
    instruction::{Seed, Signer},
    program_error::ProgramError,
    pubkey::find_program_address,
    sysvars::{rent::Rent, Sysvar},
    ProgramResult,
};
use pinocchio_system::instructions::CreateAccount;

#[inline(always)]
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

    // fast u32 reader (inlined to avoid closure)
    #[inline(always)]
    fn read_u32(bytes: &mut &[u8]) -> Result<u32, ProgramError> {
        if bytes.len() < 4 {
            return Err(ProgramError::InvalidInstructionData);
        }
        let val = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        *bytes = &bytes[4..];
        Ok(val)
    }

    // parse seeds vector
    let seeds_len = read_u32(&mut callback_args)? as usize;
    if seeds_len == 0 || seeds_len > 16 {
        return Err(ProgramError::InvalidInstructionData);
    }

    let mut seed_refs: [&[u8]; 16] = [&[]; 16];
    for seed_ref in seed_refs.iter_mut().take(seeds_len) {
        let elem_len = read_u32(&mut callback_args)? as usize;
        if callback_args.len() < elem_len {
            return Err(ProgramError::InvalidInstructionData);
        }
        *seed_ref = &callback_args[..elem_len];
        callback_args = &callback_args[elem_len..];
    }

    if !callback_args.is_empty() {
        return Err(ProgramError::InvalidInstructionData);
    }

    let pda_seeds = &seed_refs[..seeds_len];
    let (_, bump) = find_program_address(pda_seeds, owner_program);

    // collect seeds into static array (avoid dynamic alloc)
    const MAX: usize = MAX_CPI_ACCOUNTS;
    const UNINIT: MaybeUninit<Seed> = MaybeUninit::<Seed>::uninit();
    let mut combined: [MaybeUninit<Seed>; MAX] = [UNINIT; MAX];

    let mut count = 0usize;
    for seed in pda_seeds.iter() {
        combined[count].write(Seed::from(*seed));
        count += 1;
    }
    let bump_slice = [bump];
    combined[count].write(Seed::from(&bump_slice));
    count += 1;

    // interpret written slice safely
    let seeds_ptr = combined.as_ptr() as *const Seed;
    let seeds = unsafe { core::slice::from_raw_parts(seeds_ptr, count) };
    let signer = Signer::from(seeds);

    // create delegated account and copy buffer data
    let space = buffer.data_len() as u64;
    let lamports = Rent::get()?.minimum_balance(space as usize);

    CreateAccount {
        from: payer,
        to: delegated_account,
        lamports,
        space,
        owner: owner_program,
    }
    .invoke_signed(&[signer])?;

    let mut data = delegated_account.try_borrow_mut_data()?;
    let buffer_data = buffer.try_borrow_data()?;
    (*data).copy_from_slice(&buffer_data);

    Ok(())
}
