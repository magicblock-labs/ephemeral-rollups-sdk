use crate::{
    consts::{ESPL_TOKEN_PROGRAM_ID, PERMISSION_PROGRAM_ID},
    solana_compat::solana::{system_program, AccountMeta, Instruction, Pubkey},
    spl::EphemeralSplDiscriminator,
};

/// Create a new ephemeral ATA permission.
///
/// For details on the flag byte, see the [MemberFlags](`crate::access_control::structs::Member`) struct.
pub fn create_ephemeral_ata_permission(
    eata: Pubkey,
    permission: Pubkey,
    payer: Pubkey,
    eata_bump: u8,
    flag_byte: u8,
) -> Instruction {
    Instruction {
        program_id: ESPL_TOKEN_PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(eata, false),
            AccountMeta::new(permission, false),
            AccountMeta::new(payer, true),
            AccountMeta::new_readonly(system_program::id(), false),
            AccountMeta::new_readonly(PERMISSION_PROGRAM_ID, false),
        ],
        data: vec![
            EphemeralSplDiscriminator::CreateEphemeralAtaPermission as u8,
            eata_bump,
            flag_byte,
        ],
    }
}
