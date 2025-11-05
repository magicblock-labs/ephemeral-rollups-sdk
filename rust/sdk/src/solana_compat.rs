#[cfg(not(feature = "modular-sdk"))]
pub mod solana {
    pub use solana_program::account_info::AccountInfo;
    pub use solana_program::entrypoint::ProgramResult;
    pub use solana_program::instruction::{AccountMeta, Instruction};
    pub use solana_program::program::invoke;
    pub use solana_program::program::invoke_signed;
    pub use solana_program::program_error::ProgramError;
    pub use solana_program::program_memory::sol_memset;
    pub use solana_program::pubkey::Pubkey;
    pub use solana_program::sysvar::rent::Rent;
    pub use solana_program::sysvar::Sysvar;
    pub use solana_system_interface::instruction as system_instruction;
    pub use solana_system_interface::program as system_program;

    #[inline(always)]
    pub fn resize(target_account: &AccountInfo, new_len: usize) -> ProgramResult {
        #[cfg(not(feature = "disable-realloc"))]
        {
            #[allow(deprecated)]
            target_account.realloc(new_len, false)
        }

        #[cfg(feature = "disable-realloc")]
        {
            target_account.resize(new_len)
        }
    }
}

#[cfg(feature = "modular-sdk")]
pub mod solana {
    pub use solana_account_info::AccountInfo;
    pub use solana_instruction::{AccountMeta, Instruction};
    pub use solana_program_error::ProgramError;
    pub use solana_program_memory::sol_memset;
    pub use solana_cpi::invoke;
    pub use solana_cpi::invoke_signed;
    pub use solana_pubkey::Pubkey;
    pub use solana_system_interface::program as system_program;
    pub use solana_system_interface::instruction as system_instruction;
    pub use solana_sysvar::rent::Rent;
    pub use solana_sysvar::Sysvar;
    pub type ProgramResult = Result<(), ProgramError>;

    #[inline(always)]
    pub fn resize(target_account: &AccountInfo, new_len: usize) -> ProgramResult {
        target_account.resize(new_len)
    }
}
