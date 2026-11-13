use solana_program::{
    program::{invoke, invoke_signed},
    rent::Rent,
    sysvar::Sysvar,
};

/// Creates a new pda
#[inline(always)]
pub fn create_pda<'a, 'info>(
    target_account: &'a solana_program::account_info::AccountInfo<'info>,
    owner: &solana_program::pubkey::Pubkey,
    space: usize,
    pda_seeds: &[&[&[u8]]],
    system_program: &'a solana_program::account_info::AccountInfo<'info>,
    payer: &'a solana_program::account_info::AccountInfo<'info>,
    rent_exempt: bool,
) -> solana_program::entrypoint_deprecated::ProgramResult {
    let rent = Rent::get()?;
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
        )?;
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
                )?;
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
        )?;

        // 3) assign our program as the owner
        invoke_signed(
            &solana_system_interface::instruction::assign(target_account.key, owner),
            &[
                target_account.as_ref().clone(),
                system_program.as_ref().clone(),
            ],
            pda_seeds,
        )?;
    }

    Ok(())
}

/// Close PDA
#[inline(always)]
pub fn close_pda<'a, 'info>(
    target_account: &'a solana_program::account_info::AccountInfo<'info>,
    destination: &'a solana_program::account_info::AccountInfo<'info>,
) -> solana_program::entrypoint_deprecated::ProgramResult {
    // Transfer tokens from the account to the destination.
    let dest_starting_lamports = destination.lamports();
    **destination.lamports.borrow_mut() = dest_starting_lamports
        .checked_add(target_account.lamports())
        .unwrap();
    **target_account.lamports.borrow_mut() = 0;

    target_account.assign(&solana_system_interface::program::id());

    target_account.resize(0)
}

/// Close PDA with transfer
#[inline(always)]
pub fn close_pda_with_system_transfer<'a, 'info>(
    target_account: &'a solana_program::account_info::AccountInfo<'info>,
    seeds: &[&[&[u8]]],
    destination: &'a solana_program::account_info::AccountInfo<'info>,
    system_program: &'a solana_program::account_info::AccountInfo<'info>,
) -> solana_program::entrypoint_deprecated::ProgramResult {
    target_account.resize(0)?;

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
        )?;
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
