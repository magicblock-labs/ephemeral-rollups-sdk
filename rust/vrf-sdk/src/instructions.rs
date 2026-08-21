use crate::compat::{self, Compat, Modern, Pubkey};
use crate::consts;
use crate::types::{RequestRandomness, SerializableAccountMeta};

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

/// Build the base request-randomness instruction (discriminator defaults to high-priority `3`;
/// callers overwrite `data[0]` to select the variant).
fn build_request_ix(params: RequestRandomnessParams) -> compat::Instruction {
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

/// Requests randomness using the scoped (per-callback-program) VRF identity, regular priority.
///
/// This is the default request builder. The fulfillment signs the callback with the scoped
/// identity PDA ([`crate::consts::scoped_vrf_identity`]) rather than the global one, so the
/// callback must validate that PDA (see the `#[vrf_callback]` macro). For the legacy
/// global-identity behavior, use [`create_request_legacy_randomness_ix`].
pub fn create_request_randomness_ix(params: RequestRandomnessParams) -> compat::Instruction {
    let mut ix = build_request_ix(params);
    ix.data[0] = 10;
    ix
}

/// Legacy global-identity randomness request (high priority).
#[deprecated(
    note = "Legacy global-identity request (high priority). Use create_request_randomness_ix (scoped, regular) or create_request_high_priority_scoped_randomness_ix."
)]
pub fn create_request_legacy_randomness_ix(params: RequestRandomnessParams) -> compat::Instruction {
    build_request_ix(params)
}

/// Legacy global-identity randomness request (regular priority).
#[deprecated(
    note = "Legacy global-identity request (regular priority). Use create_request_randomness_ix (scoped, regular)."
)]
pub fn create_request_regular_randomness_ix(
    params: RequestRandomnessParams,
) -> compat::Instruction {
    let mut ix = build_request_ix(params);
    ix.data[0] = 8;
    ix
}

/// Scoped (per-callback identity) randomness request, high priority.
pub fn create_request_high_priority_scoped_randomness_ix(
    params: RequestRandomnessParams,
) -> compat::Instruction {
    let mut ix = build_request_ix(params);
    ix.data[0] = 11;
    ix
}
