use crate::{
    consts::{ESPL_TOKEN_PROGRAM_ID, PERMISSION_PROGRAM_ID},
    solana_compat::solana::{AccountMeta, Instruction, Pubkey},
    spl::EphemeralSplDiscriminator,
};

/// Build a reset ephemeral ATA permission instruction.
///
/// For details on the flag byte, see the [MemberFlags](`crate::access_control::structs::Member`) struct.
pub fn reset_ephemeral_ata_permission(
    eata: Pubkey,
    permission: Pubkey,
    owner: Pubkey,
    bump: u8,
    flag_byte: u8,
) -> Instruction {
    Instruction {
        program_id: ESPL_TOKEN_PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(eata, false),
            AccountMeta::new(permission, false),
            AccountMeta::new_readonly(owner, true),
            AccountMeta::new_readonly(PERMISSION_PROGRAM_ID, false),
        ],
        data: vec![
            EphemeralSplDiscriminator::ResetEphemeralAtaPermission as u8,
            bump,
            flag_byte,
        ],
    }
}
