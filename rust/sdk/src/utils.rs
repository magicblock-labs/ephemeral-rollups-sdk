use solana_program::{
    program::{invoke, invoke_signed},
    rent::Rent,
    sysvar::Sysvar,
};

use crate::{
    compat::{account_info::AccountInfo, Compatize, Modernize, ProgramResult, Pubkey},
    modernize,
};

/// Creates a new pda
#[inline(always)]
pub fn create_pda<'a, 'info>(
    target_account: &'a AccountInfo<'info>, // < 3.0,  < 1.0
    owner: &Pubkey,                         // < 3.0, < 1.0
    space: usize,
    pda_seeds: &[&[&[u8]]],
    system_program: &'a AccountInfo<'info>,
    payer: &'a AccountInfo<'info>,
    rent_exempt: bool,
) -> ProgramResult {
    modernize!(target_account, owner, system_program, payer);

    let rent = Rent::get().map_err(|err| err.compat())?;
    if target_account.lamports().eq(&0) {
        let lamports = if rent_exempt {
            rent.minimum_balance(space)
        } else {
            0
        };
        // If balance is zero, create account
        invoke_signed(
            &solana_system_interface::instruction::create_account(
                payer.key,
                target_account.key,
                lamports,
                space as u64,
                owner,
            ),
            &[
                payer.clone(),
                target_account.clone(),
                system_program.clone(),
            ],
            pda_seeds,
        )
        .compat()?;
    } else {
        // Otherwise, if balance is nonzero:
        // 1) transfer sufficient lamports for rent exemption
        if rent_exempt {
            let rent_exempt_balance = rent
                .minimum_balance(space)
                .saturating_sub(target_account.lamports());
            if rent_exempt_balance.gt(&0) {
                invoke(
                    &solana_system_interface::instruction::transfer(
                        payer.key,
                        target_account.key,
                        rent_exempt_balance,
                    ),
                    &[
                        payer.as_ref().clone(),
                        target_account.as_ref().clone(),
                        system_program.as_ref().clone(),
                    ],
                )
                .compat()?;
            }
        }

        // 2) allocate space for the account
        invoke_signed(
            &solana_system_interface::instruction::allocate(target_account.key, space as u64),
            &[
                target_account.as_ref().clone(),
                system_program.as_ref().clone(),
            ],
            pda_seeds,
        )
        .compat()?;

        // 3) assign our program as the owner
        invoke_signed(
            &solana_system_interface::instruction::assign(target_account.key, owner),
            &[
                target_account.as_ref().clone(),
                system_program.as_ref().clone(),
            ],
            pda_seeds,
        )
        .compat()?;
    }

    Ok(())
}

/// Close PDA
#[inline(always)]
pub fn close_pda<'a, 'info>(
    target_account: &'a AccountInfo<'info>,
    destination: &'a AccountInfo<'info>,
) -> ProgramResult {
    modernize!(target_account, destination);

    // Transfer tokens from the account to the destination.
    let dest_starting_lamports = destination.lamports();
    **destination.lamports.borrow_mut() = dest_starting_lamports
        .checked_add(target_account.lamports())
        .unwrap();
    **target_account.lamports.borrow_mut() = 0;

    target_account.assign(&solana_system_interface::program::id());

    target_account.resize(0).compat()
}

/// Close PDA with transfer
#[inline(always)]
pub fn close_pda_with_system_transfer<'a, 'info>(
    target_account: &'a AccountInfo<'info>,
    seeds: &[&[&[u8]]],
    destination: &'a AccountInfo<'info>,
    system_program: &'a AccountInfo<'info>,
) -> ProgramResult {
    modernize!(target_account, destination, system_program);

    target_account.resize(0).compat()?;

    target_account.assign(&solana_system_interface::program::id());
    if target_account.lamports() > 0 {
        let transfer_instruction = solana_system_interface::instruction::transfer(
            target_account.key,
            destination.key,
            target_account.lamports(),
        );
        solana_program::program::invoke_signed(
            &transfer_instruction,
            &[
                target_account.clone(),
                destination.clone(),
                system_program.clone(),
            ],
            seeds,
        )
        .compat()?;
    }

    Ok(())
}

/// Seeds with bump
#[inline(always)]
pub fn seeds_with_bump<'a>(seeds: &'a [&'a [u8]], bump: &'a [u8]) -> Vec<&'a [u8]> {
    let mut v = Vec::from(seeds);
    v.push(bump);
    v
}
