use pinocchio::pubkey::Pubkey;

/// Represents all types of seeds used for PDAs
pub enum Seed<'a> {
    Delegation(&'a Pubkey),
    DelegationMetadata(&'a Pubkey),
    Buffer(&'a Pubkey),
    CommitState(&'a Pubkey),
    CommitRecord(&'a Pubkey),
    UndelegateBuffer(&'a Pubkey),
    ValidatorFeesVault(&'a Pubkey),
    EphemeralBalance { payer: &'a Pubkey, index: u8 },
    ProgramConfig(&'a Pubkey),
    FeesVault,
}

impl<'a> Seed<'a> {
    pub fn as_seed_slice(&self) -> Vec<&[u8]> {
        match self {
            Seed::Delegation(pubkey) => vec![b"delegation", pubkey.as_ref()],
            Seed::DelegationMetadata(pubkey) => vec![b"delegation-metadata", pubkey.as_ref()],
            Seed::Buffer(pubkey) => vec![b"buffer", pubkey.as_ref()],
            Seed::CommitState(pubkey) => vec![b"state-diff", pubkey.as_ref()],
            Seed::CommitRecord(pubkey) => vec![b"commit-state-record", pubkey.as_ref()],
            Seed::UndelegateBuffer(pubkey) => vec![b"undelegate-buffer", pubkey.as_ref()],
            Seed::ValidatorFeesVault(pubkey) => vec![b"v-fees-vault", pubkey.as_ref()],
            Seed::ProgramConfig(program_id) => vec![b"p-conf", program_id.as_ref()],
            Seed::FeesVault => vec![b"fees-vault"],
            Seed::EphemeralBalance { payer, index } => {
                let index_ref = std::slice::from_ref(index);
                vec![b"balance", payer.as_ref(), index_ref]
            }
        }
    }
}
