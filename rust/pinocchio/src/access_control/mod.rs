/// Pinocchio-native access control module
///
/// Provides instruction builders and utilities for Pinocchio programs
/// to interact with the magicblock_permission_api program.
///
/// This module follows the same patterns as the delegation program's
/// Pinocchio integration, providing:
/// - Lightweight instruction builders
/// - PDA helpers for permission accounts
/// - Stack-allocated buffers (no Vec)
/// - Custom serialization (no Borsh)
pub mod instructions;
pub mod pda;
pub mod seeds;
pub mod structs;
pub mod utils;

pub use instructions::*;
pub use pda::*;
pub use seeds::*;
pub use structs::*;
pub use utils::*;
