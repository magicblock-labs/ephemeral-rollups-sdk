use core::mem::MaybeUninit;
use pinocchio::{
    address::MAX_SEEDS,
    cpi::{Seed, Signer},
    error::ProgramError,
    sysvars::{rent::Rent, Sysvar},
    AccountView, Address, ProgramResult,
};
use pinocchio_system::instructions::CreateAccount;

#[inline(always)]
pub fn undelegate(
    delegated_account: &AccountView,
    owner_program: &Address,
    buffer: &AccountView,
    payer: &AccountView,
    mut callback_args: &[u8],
) -> ProgramResult {
    if !buffer.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }

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
    let (_, bump) = Address::find_program_address(pda_seeds, owner_program);

    // collect seeds into static array (avoid dynamic alloc)
    const UNINIT: MaybeUninit<Seed> = MaybeUninit::<Seed>::uninit();
    let mut combined: [MaybeUninit<Seed>; MAX_SEEDS] = [UNINIT; MAX_SEEDS];

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
    let lamports = Rent::get()?.try_minimum_balance(space as usize)?;

    CreateAccount {
        from: payer,
        to: delegated_account,
        lamports,
        space,
        owner: owner_program,
    }
    .invoke_signed(&[signer])?;

    let mut data = delegated_account.try_borrow_mut()?;
    let buffer_data = buffer.try_borrow()?;
    (*data).copy_from_slice(&buffer_data);

    Ok(())
}
