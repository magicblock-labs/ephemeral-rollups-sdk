import {
  PublicKey,
  SystemProgram,
  TransactionInstruction,
} from "@solana/web3.js";

import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  DELEGATION_PROGRAM_ID,
  EPHEMERAL_SPL_TOKEN_PROGRAM_ID,
  HYDRA_PROGRAM_ID,
  TOKEN_PROGRAM_ID,
} from "../../constants.js";
import { encryptEd25519Recipient } from "./crypto.js";
import {
  deriveRentPda,
  deriveShuttleAta,
  deriveShuttleEphemeralAta,
  deriveVault,
} from "./ephemeralAta.js";
import { deriveTransferQueue } from "./transferQueue.js";

// ---------------------------------------------------------------------------
// Local constants (mirror the on-chain side)
// ---------------------------------------------------------------------------

const SCHEDULE_PRIVATE_TRANSFER_DISCRIMINATOR = 30;

const STASH_PDA_SEED = Buffer.from("stash");
const HYDRA_CRANK_SEED_PREFIX = Buffer.from("crank");
const BUFFER_SEED = Buffer.from("buffer");
const DELEGATION_RECORD_SEED = Buffer.from("delegation");
const DELEGATION_METADATA_SEED = Buffer.from("delegation-metadata");

// ---------------------------------------------------------------------------
// PDA derivation helpers
// ---------------------------------------------------------------------------

/**
 * Derive the swap-custody "stash" PDA for a `(user, mint)` pair.
 *
 * Seeds: `[b"stash", user, mint]` under `EPHEMERAL_SPL_TOKEN_PROGRAM_ID`.
 *
 * The stash PDA serves as both the owner of the destination ATA that
 * receives swap output and, at trigger time, the payer + owner signer
 * that the scheduled instruction self-CPIs into ix 25 with.
 */
export function deriveStashPda(
  user: PublicKey,
  mint: PublicKey,
): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [STASH_PDA_SEED, user.toBuffer(), mint.toBuffer()],
    EPHEMERAL_SPL_TOKEN_PROGRAM_ID,
  );
}

/**
 * Derive the stash ATA — the SPL associated token account where the swap
 * output lands. Owner is the stash PDA.
 */
export function deriveStashAta(
  user: PublicKey,
  mint: PublicKey,
  tokenProgram: PublicKey = TOKEN_PROGRAM_ID,
): [PublicKey, number] {
  const [stashPda] = deriveStashPda(user, mint);
  return deriveAtaWithBump(stashPda, mint, tokenProgram);
}

/**
 * Derive the Hydra crank PDA for a schedule.
 *
 * The on-chain Hydra seed overwrites the first 4 bytes of `stashPda` with
 * `shuttle_id_le` so each `(user, mint, shuttleId)` triple gets its own
 * crank. Callers that only know the user + mint should derive `stashPda`
 * via `deriveStashPda` first.
 */
export function deriveHydraCrankPda(
  stashPda: PublicKey,
  shuttleId: number,
): [PublicKey, number] {
  if (
    !Number.isInteger(shuttleId) ||
    shuttleId < 0 ||
    shuttleId > 0xffff_ffff
  ) {
    throw new Error("shuttleId must fit in u32");
  }
  return PublicKey.findProgramAddressSync(
    [HYDRA_CRANK_SEED_PREFIX, hydraSeed(stashPda, shuttleId)],
    HYDRA_PROGRAM_ID,
  );
}

/**
 * Build the 32-byte Hydra crank seed for `(stashPda, shuttleId)`.
 *
 * Must match the on-chain `derive_hydra_seed` exactly: overwrite the
 * first 4 bytes of the stash PDA with `shuttle_id_le`. No hashing — we
 * don't need cryptographic uniformity, just a distinct 32-byte output
 * per tuple.
 */
function hydraSeed(stashPda: PublicKey, shuttleId: number): Buffer {
  const seed = Buffer.from(stashPda.toBuffer()); // 32-byte copy
  seed.writeUInt32LE(shuttleId, 0);
  return seed;
}

// ---------------------------------------------------------------------------
// Instruction builder
// ---------------------------------------------------------------------------

/**
 * Build a `schedule_private_transfer` instruction (discriminator 30).
 *
 * Appends to a swap transaction so the swap's `destinationTokenAccount`
 * (which the caller sets to the stash ATA) gets privately forwarded to
 * `destinationOwner` after the scheduled ix 25 fires via Hydra.
 *
 * Accounts passed (7):
 *   0 user (signer, writable — funds the stash PDA)
 *   1 stash_pda (writable)
 *   2 rent_pda (writable — sponsors Hydra crank rent + CRANKER_REWARD)
 *   3 hydra_crank_pda (writable — created by the Hydra CPI)
 *   4 hydra_program
 *   5 system_program
 *   6 token_program
 *
 * All 14 pubkeys that ix 25 will consume at trigger time are derived
 * on-chain from 10 client-supplied bumps (included in the instruction
 * data), so the outer tx stays compact.
 *
 * @param user The user who owns the stash PDA and signs the outer tx.
 * @param mint The output-mint of the swap.
 * @param shuttleId A u32 identifier for the shuttle (client-chosen).
 * @param destinationOwner The wallet that will ultimately receive the
 *   private transfer. Encrypted to `validator` before going on-chain.
 * @param minDelayMs Earliest the queued transfer may settle.
 * @param maxDelayMs Latest it may settle.
 * @param split Number of queue entries to split the transfer across.
 * @param validator The validator that owns the transfer-queue PDA.
 * @param tokenProgram Override for the SPL token program (defaults to
 *   classic Token).
 * @param clientRefId Optional u64 correlation id attached to each split.
 */
export function schedulePrivateTransferIx(
  user: PublicKey,
  mint: PublicKey,
  shuttleId: number,
  destinationOwner: PublicKey,
  minDelayMs: bigint,
  maxDelayMs: bigint,
  split: number,
  validator: PublicKey,
  tokenProgram: PublicKey = TOKEN_PROGRAM_ID,
  clientRefId?: bigint,
): TransactionInstruction {
  if (
    !Number.isInteger(shuttleId) ||
    shuttleId < 0 ||
    shuttleId > 0xffff_ffff
  ) {
    throw new Error("shuttleId must fit in u32");
  }
  if (!Number.isInteger(split) || split <= 0 || split > 0xffff_ffff) {
    throw new Error("split must be a positive u32");
  }
  if (
    minDelayMs < 0n ||
    maxDelayMs < 0n ||
    (clientRefId !== undefined && clientRefId < 0n)
  ) {
    throw new Error("delays and clientRefId must be non-negative");
  }
  if (maxDelayMs < minDelayMs) {
    throw new Error("maxDelayMs must be greater than or equal to minDelayMs");
  }
  const U64_MAX = 0xffff_ffff_ffff_ffffn;
  if (
    minDelayMs > U64_MAX ||
    maxDelayMs > U64_MAX ||
    (clientRefId !== undefined && clientRefId > U64_MAX)
  ) {
    throw new Error("delays and clientRefId must fit in u64");
  }

  // -------- derive every pubkey + bump ix 25 will need --------
  const [stashPda, stashBump] = deriveStashPda(user, mint);
  const [, stashAtaBump] = deriveAtaWithBump(stashPda, mint, tokenProgram);
  const [rentPda] = deriveRentPda();
  const [shuttleEphemeralAta, shuttleBump] = deriveShuttleEphemeralAta(
    stashPda,
    mint,
    shuttleId,
  );
  const [shuttleAta, shuttleEataBump] = deriveShuttleAta(
    shuttleEphemeralAta,
    mint,
  );
  const [, shuttleWalletAtaBump] = deriveAtaWithBump(
    shuttleEphemeralAta,
    mint,
    tokenProgram,
  );
  const [, bufferBump] = PublicKey.findProgramAddressSync(
    [BUFFER_SEED, shuttleAta.toBuffer()],
    EPHEMERAL_SPL_TOKEN_PROGRAM_ID,
  );
  const [, delegationRecordBump] = PublicKey.findProgramAddressSync(
    [DELEGATION_RECORD_SEED, shuttleAta.toBuffer()],
    DELEGATION_PROGRAM_ID,
  );
  const [, delegationMetadataBump] = PublicKey.findProgramAddressSync(
    [DELEGATION_METADATA_SEED, shuttleAta.toBuffer()],
    DELEGATION_PROGRAM_ID,
  );
  const [vault, globalVaultBump] = deriveVault(mint);
  const [, vaultTokenBump] = deriveAtaWithBump(vault, mint, tokenProgram);
  const [, queueBump] = deriveTransferQueue(mint, validator);
  const [hydraCrankPda] = deriveHydraCrankPda(stashPda, shuttleId);

  // -------- build the encrypted payload (same shape as ix 25) --------
  const encryptedDestination = encryptEd25519Recipient(
    destinationOwner.toBytes(),
    validator,
  );
  const encryptedSuffix = encryptEd25519Recipient(
    packPrivateTransferSuffix(minDelayMs, maxDelayMs, split, clientRefId),
    validator,
  );

  // -------- wire data --------
  // Wire: [disc][shuttle_id][stash_bump][mint(32)][10 bumps][3 vardata blobs]
  // Offsets below mirror the on-chain processor's FIXED_PREFIX_LEN = 47
  // (shuttle_id..queue_bump) followed by validator/enc_dest/enc_suffix.
  const data = Buffer.concat([
    Buffer.from([SCHEDULE_PRIVATE_TRANSFER_DISCRIMINATOR]),
    u32leBuffer(shuttleId),
    Buffer.from([stashBump]),
    mint.toBuffer(),
    Buffer.from([shuttleBump]),
    Buffer.from([shuttleEataBump]),
    Buffer.from([shuttleWalletAtaBump]),
    Buffer.from([bufferBump]),
    Buffer.from([delegationRecordBump]),
    Buffer.from([delegationMetadataBump]),
    Buffer.from([globalVaultBump]),
    Buffer.from([vaultTokenBump]),
    Buffer.from([stashAtaBump]),
    Buffer.from([queueBump]),
    encodeLengthPrefixedBytes(validator.toBytes()),
    encodeLengthPrefixedBytes(encryptedDestination),
    encodeLengthPrefixedBytes(encryptedSuffix),
  ]);

  return new TransactionInstruction({
    programId: EPHEMERAL_SPL_TOKEN_PROGRAM_ID,
    keys: [
      { pubkey: user, isSigner: true, isWritable: true },
      { pubkey: stashPda, isSigner: false, isWritable: true },
      { pubkey: rentPda, isSigner: false, isWritable: true },
      { pubkey: hydraCrankPda, isSigner: false, isWritable: true },
      { pubkey: HYDRA_PROGRAM_ID, isSigner: false, isWritable: false },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
      { pubkey: tokenProgram, isSigner: false, isWritable: false },
    ],
    data,
  });
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

function deriveAtaWithBump(
  wallet: PublicKey,
  mint: PublicKey,
  tokenProgram: PublicKey,
): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [wallet.toBuffer(), tokenProgram.toBuffer(), mint.toBuffer()],
    ASSOCIATED_TOKEN_PROGRAM_ID,
  );
}

function encodeLengthPrefixedBytes(bytes: Uint8Array): Buffer {
  if (bytes.length > 0xff) {
    throw new Error("payload exceeds u8 length");
  }
  return Buffer.concat([Buffer.from([bytes.length]), Buffer.from(bytes)]);
}

function packPrivateTransferSuffix(
  minDelayMs: bigint,
  maxDelayMs: bigint,
  split: number,
  clientRefId?: bigint,
): Buffer {
  const suffix = Buffer.alloc(
    clientRefId === undefined ? 8 + 8 + 4 : 8 + 8 + 4 + 8,
  );
  suffix.writeBigUInt64LE(minDelayMs, 0);
  suffix.writeBigUInt64LE(maxDelayMs, 8);
  suffix.writeUInt32LE(split, 16);
  if (clientRefId !== undefined) {
    suffix.writeBigUInt64LE(clientRefId, 20);
  }
  return suffix;
}

function u32leBuffer(value: number): Buffer {
  const out = Buffer.alloc(4);
  out.writeUInt32LE(value, 0);
  return out;
}
