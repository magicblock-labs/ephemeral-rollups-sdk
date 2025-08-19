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
    pub fn fill_seed_slice<'b>(
        &'a self,
        out: &'b mut [&'a [u8]; 3],
        index_buf: &'b mut [u8; 1],
    ) -> &'b [&'a [u8]]
    where
        'b: 'a,
    {
        match self {
            Seed::Delegation(pubkey) => {
                out[0] = b"delegation";
                out[1] = pubkey.as_ref();
                &out[..2]
            }
            Seed::DelegationMetadata(pubkey) => {
                out[0] = b"delegation-metadata";
                out[1] = pubkey.as_ref();
                &out[..2]
            }
            Seed::Buffer(pubkey) => {
                out[0] = b"buffer";
                out[1] = pubkey.as_ref();
                &out[..2]
            }
            Seed::CommitState(pubkey) => {
                out[0] = b"state-diff";
                out[1] = pubkey.as_ref();
                &out[..2]
            }
            Seed::CommitRecord(pubkey) => {
                out[0] = b"commit-state-record";
                out[1] = pubkey.as_ref();
                &out[..2]
            }
            Seed::UndelegateBuffer(pubkey) => {
                out[0] = b"undelegate-buffer";
                out[1] = pubkey.as_ref();
                &out[..2]
            }
            Seed::ValidatorFeesVault(pubkey) => {
                out[0] = b"v-fees-vault";
                out[1] = pubkey.as_ref();
                &out[..2]
            }
            Seed::ProgramConfig(program_id) => {
                out[0] = b"p-conf";
                out[1] = program_id.as_ref();
                &out[..2]
            }
            Seed::FeesVault => {
                out[0] = b"fees-vault";
                &out[..1]
            }
            Seed::EphemeralBalance { payer, index } => {
                out[0] = b"balance";
                out[1] = payer.as_ref();
                index_buf[0] = *index;
                out[2] = &index_buf[..];
                &out[..3]
            }
        }
    }
}
