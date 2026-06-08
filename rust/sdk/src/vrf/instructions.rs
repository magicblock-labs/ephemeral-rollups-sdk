use crate::compat::{self, Compat, Modern, Pubkey};
use crate::vrf::consts;
use crate::vrf::types::{RequestRandomness, SerializableAccountMeta};

/// Parameters for creating a request randomness instruction
#[derive(Default)]
pub struct RequestRandomnessParams {
    pub payer: Pubkey,
    pub oracle_queue: Pubkey,
    pub callback_program_id: Pubkey,
    pub callback_discriminator: Vec<u8>,
    pub accounts_metas: Option<Vec<SerializableAccountMeta>>,
    pub caller_seed: [u8; 32],
    pub callback_args: Option<Vec<u8>>,
}

pub fn create_request_randomness_ix(params: RequestRandomnessParams) -> compat::Instruction {
    let payer = params.payer.modern();
    let oracle_queue = params.oracle_queue.modern();
    let callback_program_id = params.callback_program_id.modern();
    let program_identity =
        compat::latest::Pubkey::find_program_address(&[consts::IDENTITY], &callback_program_id).0;

    compat::latest::Instruction {
        program_id: consts::VRF_PROGRAM_ID.modern(),
        accounts: vec![
            compat::latest::AccountMeta::new(payer, true),
            compat::latest::AccountMeta::new_readonly(program_identity, true),
            compat::latest::AccountMeta::new(oracle_queue, false),
            compat::latest::AccountMeta::new_readonly(compat::latest::system_program::ID, false),
            compat::latest::AccountMeta::new_readonly(compat::latest::slot_hashes::ID, false),
        ],
        data: RequestRandomness {
            caller_seed: params.caller_seed,
            callback_program_id: params.callback_program_id,
            callback_discriminator: params.callback_discriminator,
            callback_accounts_metas: params.accounts_metas.unwrap_or_default(),
            callback_args: params.callback_args.unwrap_or_default(),
        }
        .to_bytes(),
    }
    .compat()
}

pub fn create_request_regular_randomness_ix(
    params: RequestRandomnessParams,
) -> compat::Instruction {
    #[allow(deprecated)]
    let mut ix = create_request_randomness_ix(params);
    ix.data[0] = 8;
    ix
}
