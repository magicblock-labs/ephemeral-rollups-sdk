use pinocchio::{error::ProgramError, instruction::InstructionAccount, Address};

/// Size of the 8-byte instruction discriminator prefix that precedes the
/// Borsh-serialized [`RequestRandomness`] payload.
pub const DISCRIMINATOR_PREFIX_SIZE: usize = 8;

/// Borsh-serialized size of a single [`SerializableAccountMeta`]
/// (`32` pubkey + `1` is_signer + `1` is_writable).
pub const SERIALIZABLE_ACCOUNT_META_SIZE: usize = 34;

/// The Borsh-compatible payload of the VRF `RequestRandomness` instruction.
///
/// This mirrors the canonical `ephemeral_vrf_sdk::types::RequestRandomness`, but
/// borrows the variable-length fields so it can be serialized without allocating
/// (suitable for on-chain CPI on the Solana sBPF target).
///
/// The serialized payload is prefixed with an 8-byte discriminator (see
/// [`crate::vrf::consts::REQUEST_RANDOMNESS_DISCRIMINATOR`] and
/// [`crate::vrf::consts::REQUEST_REGULAR_RANDOMNESS_DISCRIMINATOR`]) and is then
/// followed by the Borsh encoding of the fields below in order.
pub struct RequestRandomness<'a> {
    /// Whether to use high priority for the request.
    pub high_priority: bool,
    /// Caller-provided seed mixed into the randomness derivation.
    pub caller_seed: [u8; 32],
    /// Program that owns the callback instruction invoked once randomness is fulfilled.
    pub callback_program_id: Address,
    /// Discriminator of the callback instruction.
    pub callback_discriminator: &'a [u8],
    /// Extra account metas forwarded to the callback instruction.
    pub callback_accounts_metas: &'a [InstructionAccount<'a>],
    /// Extra serialized args appended to the callback instruction data.
    pub callback_args: &'a [u8],
}

impl<'a> RequestRandomness<'a> {
    /// Exact number of bytes produced by [`serialize_into`](Self::serialize_into),
    /// for the given parameters.
    pub const fn serialized_size_for(
        callback_discriminator_len: usize,
        callback_accounts_metas_len: usize,
        callback_args_len: usize,
    ) -> usize {
        DISCRIMINATOR_PREFIX_SIZE
            + 32 // caller_seed
            + 32 // callback_program_id
            + 4 // callback_discriminator len prefix
            + callback_discriminator_len
            + 4 // callback_accounts_metas len prefix
            + callback_accounts_metas_len * SERIALIZABLE_ACCOUNT_META_SIZE
            + 4 // callback_args len prefix
            + callback_args_len
    }

    /// Exact number of bytes produced by [`serialize_into`](Self::serialize_into),
    /// including the 8-byte discriminator prefix.
    pub fn serialized_size(&self) -> usize {
        DISCRIMINATOR_PREFIX_SIZE
            + 32 // caller_seed
            + 32 // callback_program_id
            + 4 // callback_discriminator len prefix
            + self.callback_discriminator.len()
            + 4 // callback_accounts_metas len prefix
            + self.callback_accounts_metas.len() * SERIALIZABLE_ACCOUNT_META_SIZE
            + 4 // callback_args len prefix
            + self.callback_args.len()
    }

    /// Serialize the discriminator prefix and the Borsh payload into `data`.
    ///
    /// `discriminator` is written as the first byte of the 8-byte prefix; the
    /// remaining seven bytes are zero. Returns the number of bytes written.
    pub fn serialize_into(
        &self,
        data: &mut [u8],
        discriminator: u64,
    ) -> Result<usize, ProgramError> {
        let required = self.serialized_size();
        if data.len() < required {
            return Err(ProgramError::InvalidInstructionData);
        }

        let mut offset = 0;

        data[0..8].copy_from_slice(discriminator.to_le_bytes().as_ref());
        offset += DISCRIMINATOR_PREFIX_SIZE;

        write_bytes(data, &mut offset, &self.caller_seed)?;
        write_bytes(data, &mut offset, self.callback_program_id.as_ref())?;

        write_bytes(
            data,
            &mut offset,
            &(self.callback_discriminator.len() as u32).to_le_bytes(),
        )?;
        write_bytes(data, &mut offset, self.callback_discriminator)?;

        write_bytes(
            data,
            &mut offset,
            &(self.callback_accounts_metas.len() as u32).to_le_bytes(),
        )?;
        for meta in self.callback_accounts_metas {
            write_bytes(data, &mut offset, meta.address.as_ref())?;
            write_bytes(data, &mut offset, &[meta.is_signer as u8])?;
            write_bytes(data, &mut offset, &[meta.is_writable as u8])?;
        }

        write_bytes(
            data,
            &mut offset,
            &(self.callback_args.len() as u32).to_le_bytes(),
        )?;
        write_bytes(data, &mut offset, self.callback_args)?;

        Ok(offset)
    }
}

#[inline(always)]
fn write_bytes(data: &mut [u8], offset: &mut usize, bytes: &[u8]) -> Result<(), ProgramError> {
    let end = offset
        .checked_add(bytes.len())
        .ok_or(ProgramError::InvalidInstructionData)?;
    if end > data.len() {
        return Err(ProgramError::InvalidInstructionData);
    }
    data[*offset..end].copy_from_slice(bytes);
    *offset = end;
    Ok(())
}

#[cfg(test)]
mod tests {
    extern crate alloc;

    use alloc::vec;
    use alloc::vec::Vec;

    use ephemeral_vrf_sdk::instructions::{
        create_request_randomness_ix, create_request_regular_randomness_ix, RequestRandomnessParams,
    };
    use ephemeral_vrf_sdk::types::SerializableAccountMeta as SdkMeta;
    use pinocchio::Address;
    use solana_program::pubkey::Pubkey;

    use super::*;
    use crate::vrf::consts::{
        REQUEST_RANDOMNESS_DISCRIMINATOR, REQUEST_REGULAR_RANDOMNESS_DISCRIMINATOR,
    };

    type MetaTuple = (Address, bool, bool);

    fn serialize(
        caller_seed: [u8; 32],
        callback_program: [u8; 32],
        disc: &[u8],
        metas: &[MetaTuple],
        args: &[u8],
        discriminator: u64,
    ) -> Vec<u8> {
        let p_metas: Vec<InstructionAccount> = metas
            .iter()
            .map(|(k, is_signer, is_writable)| {
                InstructionAccount::new(k, *is_writable, *is_signer)
            })
            .collect();
        let req = RequestRandomness {
            high_priority: discriminator == REQUEST_RANDOMNESS_DISCRIMINATOR,
            caller_seed,
            callback_program_id: Address::new_from_array(callback_program),
            callback_discriminator: disc,
            callback_accounts_metas: &p_metas,
            callback_args: args,
        };
        let mut buf = vec![0u8; req.serialized_size()];
        let len = req.serialize_into(&mut buf, discriminator).unwrap();
        // serialize_into must report exactly the precomputed size.
        assert_eq!(len, req.serialized_size());
        buf.truncate(len);
        buf
    }

    fn canonical(
        caller_seed: [u8; 32],
        callback_program: [u8; 32],
        disc: &[u8],
        metas: &[MetaTuple],
        args: &[u8],
    ) -> RequestRandomnessParams {
        let sdk_metas: Vec<SdkMeta> = metas
            .iter()
            .map(|(k, s, w)| SdkMeta {
                pubkey: Pubkey::new_from_array(k.to_bytes()),
                is_signer: *s,
                is_writable: *w,
            })
            .collect();
        RequestRandomnessParams {
            payer: Pubkey::new_from_array([5u8; 32]),
            oracle_queue: Pubkey::new_from_array([6u8; 32]),
            callback_program_id: Pubkey::new_from_array(callback_program),
            callback_discriminator: disc.to_vec(),
            accounts_metas: Some(sdk_metas),
            caller_seed,
            callback_args: Some(args.to_vec()),
        }
    }

    #[test]
    fn matches_canonical_request_randomness() {
        let caller_seed = [7u8; 32];
        let callback_program = [9u8; 32];
        let disc = [10u8, 20, 30, 40, 50, 60, 70, 80];
        let metas: [MetaTuple; 3] = [
            (Address::new_from_array([1u8; 32]), true, false),
            (Address::new_from_array([2u8; 32]), false, true),
            (Address::new_from_array([3u8; 32]), true, true),
        ];
        let args = [100u8, 101, 102];

        let ours = serialize(
            caller_seed,
            callback_program,
            &disc,
            &metas,
            &args,
            REQUEST_RANDOMNESS_DISCRIMINATOR,
        );
        let ix = create_request_randomness_ix(canonical(
            caller_seed,
            callback_program,
            &disc,
            &metas,
            &args,
        ));
        assert_eq!(ours, ix.data);
        // First byte is the ephemeral discriminator.
        assert_eq!(ours[0..8], REQUEST_RANDOMNESS_DISCRIMINATOR.to_le_bytes());
    }

    #[test]
    fn matches_canonical_regular_randomness() {
        let caller_seed = [1u8; 32];
        let callback_program = [2u8; 32];
        let disc = [9u8, 8, 7];
        let metas: [MetaTuple; 1] = [(Address::new_from_array([4u8; 32]), false, false)];
        let args = [1u8];

        let ours = serialize(
            caller_seed,
            callback_program,
            &disc,
            &metas,
            &args,
            REQUEST_REGULAR_RANDOMNESS_DISCRIMINATOR,
        );
        let ix = create_request_regular_randomness_ix(canonical(
            caller_seed,
            callback_program,
            &disc,
            &metas,
            &args,
        ));
        assert_eq!(ours, ix.data);
        assert_eq!(
            ours[0..8],
            REQUEST_REGULAR_RANDOMNESS_DISCRIMINATOR.to_le_bytes()
        );
    }

    #[test]
    fn matches_canonical_empty_metas_and_args() {
        let caller_seed = [0u8; 32];
        let callback_program = [0xABu8; 32];
        let disc = [42u8];

        let ours = serialize(
            caller_seed,
            callback_program,
            &disc,
            &[],
            &[],
            REQUEST_RANDOMNESS_DISCRIMINATOR,
        );
        let ix =
            create_request_randomness_ix(canonical(caller_seed, callback_program, &disc, &[], &[]));
        assert_eq!(ours, ix.data);
    }

    #[test]
    fn serialize_into_rejects_undersized_buffer() {
        let req = RequestRandomness {
            high_priority: true,
            caller_seed: [0u8; 32],
            callback_program_id: Address::new_from_array([0u8; 32]),
            callback_discriminator: &[1, 2, 3],
            callback_accounts_metas: &[],
            callback_args: &[],
        };
        let mut buf = vec![0u8; req.serialized_size() - 1];
        assert!(req.serialize_into(&mut buf, 3).is_err());
    }

    #[test]
    fn matches_canonical_roll_dice_callback_request() {
        let caller_seed = [42u8; 32];
        let callback_program = [0xCDu8; 32];
        let disc = [2u8, 0, 0, 0, 0, 0, 0, 0];
        let player = Address::new_from_array([0x11u8; 32]);
        let meta = InstructionAccount {
            address: &player,
            is_signer: false,
            is_writable: true,
        };
        let args = [7u8];

        let ours = {
            let req = RequestRandomness {
                high_priority: false,
                caller_seed,
                callback_program_id: Address::new_from_array(callback_program),
                callback_discriminator: &disc,
                callback_accounts_metas: &[meta],
                callback_args: &args,
            };
            let mut buf = vec![0u8; req.serialized_size()];
            req.serialize_into(&mut buf, REQUEST_REGULAR_RANDOMNESS_DISCRIMINATOR).unwrap();
            buf
        };

        let ix = create_request_regular_randomness_ix(canonical(
            caller_seed,
            callback_program,
            &disc,
            &[(player, false, true)],
            &args,
        ));
        assert_eq!(ours, ix.data);

        // Callback discriminator bytes must survive request serialization intact.
        let payload = &ours[8..];
        let disc_len = u32::from_le_bytes(payload[64..68].try_into().unwrap()) as usize;
        assert_eq!(disc_len, disc.len());
        assert_eq!(&payload[68..68 + disc_len], disc);
    }
}
