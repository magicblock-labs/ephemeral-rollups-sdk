use crate::intent_bundle::no_vec::NoVec;
use crate::intent_bundle::MAX_ACTIONS_NUM;
use pinocchio::cpi::MAX_STATIC_CPI_ACCOUNTS;
use serde::Serialize;
use solana_address::Address;

// ---------------------------------------------------------
// Args types for serialization
// ---------------------------------------------------------

/// Action arguments containing escrow index and instruction data.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, bincode::Encode)]
pub struct ActionArgs<'a> {
    pub escrow_index: u8,
    pub data: &'a [u8],
}

impl<'a> ActionArgs<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self {
            escrow_index: 255,
            data,
        }
    }

    pub fn escrow_index(&self) -> u8 {
        self.escrow_index
    }

    pub fn data(&self) -> &'a [u8] {
        self.data
    }

    pub fn with_escrow_index(mut self, index: u8) -> Self {
        self.escrow_index = index;
        self
    }
}

/// Base action arguments for serialization.
#[derive(Clone, Debug, Serialize, bincode::Encode)]
pub struct BaseActionArgs<'args> {
    pub args: ActionArgs<'args>,
    pub compute_units: u32,
    pub escrow_authority: u8,
    #[bincode(with_serde)]
    pub destination_program: Address,
    pub accounts: NoVec<ShortAccountMeta, MAX_STATIC_CPI_ACCOUNTS>,
}

/// A compact account meta used for base-layer actions.
///
/// Unlike `solana_instruction::AccountMeta`, this type **does not** carry an
/// `is_signer` flag. Users cannot request signatures: the only signer available
/// is the validator.
#[derive(Debug, Default, Clone, Serialize, bincode::Encode)]
pub struct ShortAccountMeta {
    #[bincode(with_serde)]
    pub pubkey: Address,
    pub is_writable: bool,
}

/// Commit type arguments for serialization.
#[derive(Serialize, bincode::Encode)]
#[allow(clippy::large_enum_variant)]
pub enum CommitTypeArgs<'args> {
    // we generate it
    Standalone(NoVec<u8, MAX_STATIC_CPI_ACCOUNTS>), // slice or NoVec
    WithBaseActions {
        committed_accounts: NoVec<u8, MAX_STATIC_CPI_ACCOUNTS>,
        base_actions: NoVec<BaseActionArgs<'args>, MAX_ACTIONS_NUM>,
    },
}

/// Undelegate type arguments for serialization.
#[derive(Serialize, bincode::Encode)]
#[allow(clippy::large_enum_variant)]
pub enum UndelegateTypeArgs<'args> {
    Standalone,
    WithBaseActions {
        base_actions: NoVec<BaseActionArgs<'args>, MAX_ACTIONS_NUM>,
    },
}

/// Commit and undelegate arguments for serialization.
#[derive(Serialize, bincode::Encode)]
pub struct CommitAndUndelegateArgs<'args> {
    pub commit_type: CommitTypeArgs<'args>,
    pub undelegate_type: UndelegateTypeArgs<'args>,
}

/// Magic intent bundle arguments for serialization.
#[derive(Serialize, bincode::Encode)]
pub struct MagicIntentBundleArgs<'args> {
    pub commit: Option<CommitTypeArgs<'args>>,
    pub commit_and_undelegate: Option<CommitAndUndelegateArgs<'args>>,
    pub standalone_actions: NoVec<BaseActionArgs<'args>, MAX_ACTIONS_NUM>,
}

#[cfg(test)]
mod tests {
    extern crate alloc;
    use alloc::vec;
    use alloc::vec::Vec;

    use super::*;
    use magicblock_magic_program_api::args as sdk;
    use magicblock_magic_program_api::Pubkey;

    /// Helper to create a deterministic pubkey/address from a seed byte
    fn make_pubkey(seed: u8) -> Pubkey {
        Pubkey::new_from_array([seed; 32])
    }

    fn make_address(seed: u8) -> Address {
        Address::new_from_array([seed; 32])
    }

    /// Test ActionArgs serialization compatibility
    #[test]
    fn test_action_args_serialization() {
        let data: &[u8] = &[1, 2, 3, 4, 5];

        // SDK type (bincode 1.x)
        let sdk_args = sdk::ActionArgs {
            escrow_index: 42,
            data: data.to_vec(),
        };
        let sdk_bytes = bincode1::serialize(&sdk_args).unwrap();

        // Pinocchio type (bincode 2.x legacy)
        let pino_args = ActionArgs {
            escrow_index: 42,
            data,
        };
        let mut pino_buf = [0u8; 256];
        let pino_len =
            bincode::encode_into_slice(&pino_args, &mut pino_buf, bincode::config::legacy())
                .unwrap();

        assert_eq!(&pino_buf[..pino_len], &sdk_bytes[..], "ActionArgs mismatch");
    }

    /// Test BaseActionArgs serialization compatibility
    #[test]
    fn test_base_action_args_serialization() {
        let data: &[u8] = &[10, 20, 30];

        // SDK type
        let sdk_args = sdk::BaseActionArgs {
            args: sdk::ActionArgs {
                escrow_index: 5,
                data: data.to_vec(),
            },
            compute_units: 200_000,
            escrow_authority: 2,
            destination_program: make_pubkey(0xDE),
            accounts: vec![
                sdk::ShortAccountMeta {
                    pubkey: make_pubkey(0x11),
                    is_writable: true,
                },
                sdk::ShortAccountMeta {
                    pubkey: make_pubkey(0x22),
                    is_writable: false,
                },
            ],
        };
        let sdk_bytes = bincode1::serialize(&sdk_args).unwrap();

        // Pinocchio type
        let mut pino_accounts = NoVec::<ShortAccountMeta, MAX_STATIC_CPI_ACCOUNTS>::new();
        pino_accounts.push(ShortAccountMeta {
            pubkey: make_address(0x11),
            is_writable: true,
        });
        pino_accounts.push(ShortAccountMeta {
            pubkey: make_address(0x22),
            is_writable: false,
        });

        let pino_args = BaseActionArgs {
            args: ActionArgs {
                escrow_index: 5,
                data,
            },
            compute_units: 200_000,
            escrow_authority: 2,
            destination_program: make_address(0xDE),
            accounts: pino_accounts,
        };
        let mut pino_buf = [0u8; 512];
        let pino_len =
            bincode::encode_into_slice(&pino_args, &mut pino_buf, bincode::config::legacy())
                .unwrap();

        assert_eq!(
            &pino_buf[..pino_len],
            &sdk_bytes[..],
            "BaseActionArgs mismatch"
        );
    }

    /// Test CommitTypeArgs::WithBaseActions serialization compatibility
    #[test]
    fn test_commit_type_with_base_actions_serialization() {
        let indices: Vec<u8> = vec![1, 3];
        let data: &[u8] = &[0xAA, 0xBB];

        // SDK type
        let sdk_commit = sdk::CommitTypeArgs::WithBaseActions {
            committed_accounts: indices.clone(),
            base_actions: vec![sdk::BaseActionArgs {
                args: sdk::ActionArgs {
                    escrow_index: 10,
                    data: data.to_vec(),
                },
                compute_units: 100_000,
                escrow_authority: 0,
                destination_program: make_pubkey(0xCC),
                accounts: vec![],
            }],
        };
        let sdk_bytes = bincode1::serialize(&sdk_commit).unwrap();

        // Pinocchio type
        let mut pino_indices = NoVec::<u8, MAX_STATIC_CPI_ACCOUNTS>::new();
        for i in &indices {
            pino_indices.push(*i);
        }
        let mut pino_base_actions = NoVec::<BaseActionArgs, MAX_ACTIONS_NUM>::new();
        pino_base_actions.push(BaseActionArgs {
            args: ActionArgs {
                escrow_index: 10,
                data,
            },
            compute_units: 100_000,
            escrow_authority: 0,
            destination_program: make_address(0xCC),
            accounts: NoVec::new(),
        });
        let pino_commit = CommitTypeArgs::WithBaseActions {
            committed_accounts: pino_indices,
            base_actions: pino_base_actions,
        };
        let mut pino_buf = [0u8; 512];
        let pino_len =
            bincode::encode_into_slice(&pino_commit, &mut pino_buf, bincode::config::legacy())
                .unwrap();

        assert_eq!(
            &pino_buf[..pino_len],
            &sdk_bytes[..],
            "CommitTypeArgs::WithBaseActions mismatch"
        );
    }

    /// Test UndelegateTypeArgs serialization compatibility
    #[test]
    fn test_undelegate_type_args_serialization() {
        // Standalone variant
        let sdk_standalone = sdk::UndelegateTypeArgs::Standalone;
        let sdk_standalone_bytes = bincode1::serialize(&sdk_standalone).unwrap();

        let pino_standalone = UndelegateTypeArgs::Standalone;
        let mut pino_buf = [0u8; 256];
        let pino_len =
            bincode::encode_into_slice(&pino_standalone, &mut pino_buf, bincode::config::legacy())
                .unwrap();

        assert_eq!(
            &pino_buf[..pino_len],
            &sdk_standalone_bytes[..],
            "UndelegateTypeArgs::Standalone mismatch"
        );

        // WithBaseActions variant
        let data: &[u8] = &[1, 2, 3];
        let sdk_with_actions = sdk::UndelegateTypeArgs::WithBaseActions {
            base_actions: vec![sdk::BaseActionArgs {
                args: sdk::ActionArgs {
                    escrow_index: 255,
                    data: data.to_vec(),
                },
                compute_units: 50_000,
                escrow_authority: 1,
                destination_program: make_pubkey(0xEE),
                accounts: vec![],
            }],
        };
        let sdk_with_actions_bytes = bincode1::serialize(&sdk_with_actions).unwrap();

        let mut pino_base_actions = NoVec::<BaseActionArgs, MAX_ACTIONS_NUM>::new();
        pino_base_actions.push(BaseActionArgs {
            args: ActionArgs {
                escrow_index: 255,
                data,
            },
            compute_units: 50_000,
            escrow_authority: 1,
            destination_program: make_address(0xEE),
            accounts: NoVec::new(),
        });
        let pino_with_actions = UndelegateTypeArgs::WithBaseActions {
            base_actions: pino_base_actions,
        };
        let mut pino_buf2 = [0u8; 512];
        let pino_len2 = bincode::encode_into_slice(
            &pino_with_actions,
            &mut pino_buf2,
            bincode::config::legacy(),
        )
        .unwrap();

        assert_eq!(
            &pino_buf2[..pino_len2],
            &sdk_with_actions_bytes[..],
            "UndelegateTypeArgs::WithBaseActions mismatch"
        );
    }

    /// Test MagicIntentBundleArgs serialization compatibility (full bundle)
    #[test]
    fn test_magic_intent_bundle_args_serialization() {
        let commit_indices: Vec<u8> = vec![2, 3];
        let cau_indices: Vec<u8> = vec![4, 5, 6];
        let action_data: &[u8] = &[0xFF, 0xFE];

        // SDK type
        let sdk_bundle = sdk::MagicIntentBundleArgs {
            commit: Some(sdk::CommitTypeArgs::Standalone(commit_indices.clone())),
            commit_and_undelegate: Some(sdk::CommitAndUndelegateArgs {
                commit_type: sdk::CommitTypeArgs::Standalone(cau_indices.clone()),
                undelegate_type: sdk::UndelegateTypeArgs::Standalone,
            }),
            standalone_actions: vec![sdk::BaseActionArgs {
                args: sdk::ActionArgs {
                    escrow_index: 0,
                    data: action_data.to_vec(),
                },
                compute_units: 300_000,
                escrow_authority: 7,
                destination_program: make_pubkey(0x99),
                accounts: vec![sdk::ShortAccountMeta {
                    pubkey: make_pubkey(0x88),
                    is_writable: true,
                }],
            }],
        };
        let sdk_bytes = bincode1::serialize(&sdk_bundle).unwrap();

        // Pinocchio type
        let mut pino_commit_indices = NoVec::<u8, MAX_STATIC_CPI_ACCOUNTS>::new();
        for i in &commit_indices {
            pino_commit_indices.push(*i);
        }

        let mut pino_cau_indices = NoVec::<u8, MAX_STATIC_CPI_ACCOUNTS>::new();
        for i in &cau_indices {
            pino_cau_indices.push(*i);
        }

        let mut pino_action_accounts = NoVec::<ShortAccountMeta, MAX_STATIC_CPI_ACCOUNTS>::new();
        pino_action_accounts.push(ShortAccountMeta {
            pubkey: make_address(0x88),
            is_writable: true,
        });

        let mut pino_standalone = NoVec::<BaseActionArgs, MAX_ACTIONS_NUM>::new();
        pino_standalone.push(BaseActionArgs {
            args: ActionArgs {
                escrow_index: 0,
                data: action_data,
            },
            compute_units: 300_000,
            escrow_authority: 7,
            destination_program: make_address(0x99),
            accounts: pino_action_accounts,
        });

        let pino_bundle = MagicIntentBundleArgs {
            commit: Some(CommitTypeArgs::Standalone(pino_commit_indices)),
            commit_and_undelegate: Some(CommitAndUndelegateArgs {
                commit_type: CommitTypeArgs::Standalone(pino_cau_indices),
                undelegate_type: UndelegateTypeArgs::Standalone,
            }),
            standalone_actions: pino_standalone,
        };
        let mut pino_buf = [0u8; 1024];
        let pino_len =
            bincode::encode_into_slice(&pino_bundle, &mut pino_buf, bincode::config::legacy())
                .unwrap();

        assert_eq!(
            &pino_buf[..pino_len],
            &sdk_bytes[..],
            "MagicIntentBundleArgs mismatch"
        );
    }

    /// Test MagicIntentBundleArgs with all None/empty fields
    #[test]
    fn test_magic_intent_bundle_args_empty_serialization() {
        // SDK type
        let sdk_bundle = sdk::MagicIntentBundleArgs {
            commit: None,
            commit_and_undelegate: None,
            standalone_actions: vec![],
        };
        let sdk_bytes = bincode1::serialize(&sdk_bundle).unwrap();

        // Pinocchio type
        let pino_bundle = MagicIntentBundleArgs {
            commit: None,
            commit_and_undelegate: None,
            standalone_actions: NoVec::new(),
        };
        let mut pino_buf = [0u8; 256];
        let pino_len =
            bincode::encode_into_slice(&pino_bundle, &mut pino_buf, bincode::config::legacy())
                .unwrap();

        assert_eq!(
            &pino_buf[..pino_len],
            &sdk_bytes[..],
            "Empty MagicIntentBundleArgs mismatch"
        );
    }
}
