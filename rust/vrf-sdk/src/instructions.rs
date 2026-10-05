use crate::compat::{self, Compat, Modern, Pubkey};
use crate::consts;
use crate::types::{RequestRandomness, SerializableAccountMeta};

/// Parameters for creating a request randomness instruction
#[derive(Clone, Default)]
pub struct RequestRandomnessParams {
    pub payer: Pubkey,
    pub oracle_queue: Pubkey,
    pub callback_program_id: Pubkey,
    pub callback_discriminator: Vec<u8>,
    pub accounts_metas: Option<Vec<SerializableAccountMeta>>,
    pub caller_seed: [u8; 32],
    pub callback_args: Option<Vec<u8>>,
}

fn find_program_identity(callback_program_id: &Pubkey) -> Pubkey {
    crate::compat::latest::Pubkey::find_program_address(
        &[consts::IDENTITY],
        &callback_program_id.modern(),
    )
    .0
    .compat()
}

/// Build the base request-randomness instruction (discriminator defaults to high-priority `3`;
/// callers overwrite `data[0]` to select the variant).
fn build_request_ix(
    params: RequestRandomnessParams,
    program_identity: Pubkey,
) -> compat::Instruction {
    let payer = params.payer.modern();
    let oracle_queue = params.oracle_queue.modern();
    let program_identity = program_identity.modern();

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
/// The fulfillment signs the callback with the scoped identity PDA rather than the global one,
/// so the callback must validate that PDA (see the `#[vrf_callback]` macro). For the legacy
/// global-identity behavior, use [`create_request_legacy_randomness_ix`].
#[deprecated(
    note = "Derives the program identity PDA at runtime via find_program_address. Use `create_request_randomness_ix_const` with `program_identity_pubkey(&crate::ID)` instead."
)]
pub fn create_request_randomness_ix(params: RequestRandomnessParams) -> compat::Instruction {
    let program_identity = find_program_identity(&params.callback_program_id);
    let mut ix = build_request_ix(params, program_identity);
    ix.data[0] = 10;
    ix
}

/// Same as [`create_request_randomness_ix`], but uses a compile-time program identity PDA.
///
/// Derive the identity from the consumer program's `crate::ID` with
/// [`consts::program_identity_pubkey`] so `find_program_address` is not executed
/// at runtime:
///
/// ```ignore
/// const PROGRAM_IDENTITY: Pubkey =
///     ephemeral_vrf_sdk::consts::program_identity_pubkey(&crate::ID);
/// let ix = create_request_randomness_ix_const(params, PROGRAM_IDENTITY);
/// ```
pub fn create_request_randomness_ix_const(
    params: RequestRandomnessParams,
    program_identity: Pubkey,
) -> compat::Instruction {
    let mut ix = build_request_ix(params, program_identity);
    ix.data[0] = 10;
    ix
}

/// Legacy global-identity randomness request (high priority).
#[deprecated(
    note = "Legacy global-identity request (high priority). Use create_request_randomness_ix_const with program_identity_pubkey(&crate::ID)."
)]
pub fn create_request_legacy_randomness_ix(params: RequestRandomnessParams) -> compat::Instruction {
    let program_identity = find_program_identity(&params.callback_program_id);
    build_request_ix(params, program_identity)
}

/// Legacy global-identity randomness request (regular priority).
#[deprecated(
    note = "Legacy global-identity request (regular priority). Use create_request_randomness_ix_const with program_identity_pubkey(&crate::ID)."
)]
pub fn create_request_regular_randomness_ix(
    params: RequestRandomnessParams,
) -> compat::Instruction {
    let program_identity = find_program_identity(&params.callback_program_id);
    let mut ix = build_request_ix(params, program_identity);
    ix.data[0] = 8;
    ix
}

/// Scoped (per-callback identity) randomness request, high priority.
#[deprecated(
    note = "Derives the program identity PDA at runtime via find_program_address. Use `create_request_high_priority_scoped_randomness_ix_const` with `program_identity_pubkey(&crate::ID)` instead."
)]
pub fn create_request_high_priority_scoped_randomness_ix(
    params: RequestRandomnessParams,
) -> compat::Instruction {
    let program_identity = find_program_identity(&params.callback_program_id);
    let mut ix = build_request_ix(params, program_identity);
    ix.data[0] = 11;
    ix
}

/// Same as [`create_request_high_priority_scoped_randomness_ix`], but uses a
/// compile-time program identity PDA from [`consts::program_identity_pubkey`].
pub fn create_request_high_priority_scoped_randomness_ix_const(
    params: RequestRandomnessParams,
    program_identity: Pubkey,
) -> compat::Instruction {
    let mut ix = build_request_ix(params, program_identity);
    ix.data[0] = 11;
    ix
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_params() -> RequestRandomnessParams {
        RequestRandomnessParams {
            payer: Pubkey::new_from_array([1u8; 32]),
            oracle_queue: Pubkey::new_from_array([2u8; 32]),
            callback_program_id: Pubkey::new_from_array([3u8; 32]),
            callback_discriminator: vec![9, 9, 9, 9],
            accounts_metas: None,
            caller_seed: [7u8; 32],
            callback_args: Some(vec![1, 2]),
        }
    }

    #[test]
    fn const_request_matches_derived_identity() {
        const CALLBACK: Pubkey = Pubkey::new_from_array([3u8; 32]);
        const IDENTITY: Pubkey = consts::program_identity_pubkey(&CALLBACK);
        let params = sample_params();
        #[allow(deprecated)]
        let derived = create_request_randomness_ix(params.clone());
        let const_ix = create_request_randomness_ix_const(params, IDENTITY);

        assert_eq!(derived, const_ix);
        assert_eq!(const_ix.accounts[1].pubkey, IDENTITY);
        assert_eq!(const_ix.data[0], 10);
    }

    #[test]
    fn high_priority_const_request_matches_derived_identity() {
        const CALLBACK: Pubkey = Pubkey::new_from_array([3u8; 32]);
        const IDENTITY: Pubkey = consts::program_identity_pubkey(&CALLBACK);
        let params = sample_params();
        #[allow(deprecated)]
        let derived = create_request_high_priority_scoped_randomness_ix(params.clone());
        let const_ix = create_request_high_priority_scoped_randomness_ix_const(params, IDENTITY);

        assert_eq!(derived, const_ix);
        assert_eq!(const_ix.data[0], 11);
    }
}
