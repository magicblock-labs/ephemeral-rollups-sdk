import {
  AccountRole,
  Address,
  Instruction,
  getAddressEncoder,
  getProgramDerivedAddress,
} from "@solana/kit";
import { SYSTEM_PROGRAM_ADDRESS } from "@solana-program/system";

import {
  DELEGATION_PROGRAM_ID,
  EPHEMERAL_SPL_TOKEN_PROGRAM_ID,
  HYDRA_PROGRAM_ID,
} from "../../constants";
import { encryptEd25519Recipient } from "./crypto";
import {
  deriveRentPda,
  deriveShuttleAta,
  deriveShuttleEphemeralAta,
  deriveVault,
} from "./ephemeralAta";
import { deriveTransferQueue } from "./transferQueue";

const SCHEDULE_PRIVATE_TRANSFER_DISCRIMINATOR = 30;

const STASH_PDA_SEED = new TextEncoder().encode("stash");
const HYDRA_CRANK_SEED_PREFIX = new TextEncoder().encode("crank");
const BUFFER_SEED = new TextEncoder().encode("buffer");
const DELEGATION_RECORD_SEED = new TextEncoder().encode("delegation");
const DELEGATION_METADATA_SEED = new TextEncoder().encode(
  "delegation-metadata",
);

const TOKEN_PROGRAM_ADDRESS =
  "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA" as Address;
const ASSOCIATED_TOKEN_PROGRAM_ADDRESS =
  "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL" as Address;

export async function deriveStashPda(
  user: Address,
  mint: Address,
): Promise<[Address, number]> {
  const addressEncoder = getAddressEncoder();
  const [stashPda, bump] = await getProgramDerivedAddress({
    programAddress: EPHEMERAL_SPL_TOKEN_PROGRAM_ID,
    seeds: [
      STASH_PDA_SEED,
      addressEncoder.encode(user),
      addressEncoder.encode(mint),
    ],
  });
  return [stashPda, bump];
}

export async function deriveStashAta(
  user: Address,
  mint: Address,
  tokenProgram: Address = TOKEN_PROGRAM_ADDRESS,
): Promise<[Address, number]> {
  const [stashPda] = await deriveStashPda(user, mint);
  return deriveAtaWithBump(stashPda, mint, tokenProgram);
}

export async function deriveHydraCrankPda(
  stashPda: Address,
  shuttleId: number,
): Promise<[Address, number]> {
  if (
    !Number.isInteger(shuttleId) ||
    shuttleId < 0 ||
    shuttleId > 0xffff_ffff
  ) {
    throw new Error("shuttleId must fit in u32");
  }

  const [hydraCrankPda, bump] = await getProgramDerivedAddress({
    programAddress: HYDRA_PROGRAM_ID,
    seeds: [HYDRA_CRANK_SEED_PREFIX, hydraSeed(stashPda, shuttleId)],
  });
  return [hydraCrankPda, bump];
}

function hydraSeed(stashPda: Address, shuttleId: number): Buffer {
  const addressEncoder = getAddressEncoder();
  const seed = Buffer.from(addressEncoder.encode(stashPda));
  seed.writeUInt32LE(shuttleId, 0);
  return seed;
}

export async function schedulePrivateTransferIx(
  user: Address,
  mint: Address,
  shuttleId: number,
  destinationOwner: Address,
  minDelayMs: bigint,
  maxDelayMs: bigint,
  split: number,
  validator: Address,
  clientRefId?: bigint,
  tokenProgram: Address = TOKEN_PROGRAM_ADDRESS,
): Promise<Instruction> {
  if (
    !Number.isInteger(shuttleId) ||
    shuttleId < 0 ||
    shuttleId > 0xffff_ffff
  ) {
    throw new Error("shuttleId must fit in u32");
  }
  if (!Number.isInteger(split) || split <= 0 || split > 0xffff_ffff) {
    throw new Error("split must fit in u32");
  }
  if (
    minDelayMs < 0n ||
    maxDelayMs < 0n ||
    (clientRefId !== undefined && clientRefId < 0n)
  ) {
    throw new Error("delays and clientRefId must be non-negative");
  }
  const U64_MAX = 0xffff_ffff_ffff_ffffn;
  if (
    minDelayMs > U64_MAX ||
    maxDelayMs > U64_MAX ||
    (clientRefId !== undefined && clientRefId > U64_MAX)
  ) {
    throw new Error("delays and clientRefId must fit in u64");
  }
  if (maxDelayMs < minDelayMs) {
    throw new Error("maxDelayMs must be greater than or equal to minDelayMs");
  }

  const addressEncoder = getAddressEncoder();
  const [stashPda, stashBump] = await deriveStashPda(user, mint);
  const [, stashAtaBump] = await deriveAtaWithBump(
    stashPda,
    mint,
    tokenProgram,
  );
  const [rentPda] = await deriveRentPda();
  const [shuttleEphemeralAta, shuttleBump] = await deriveShuttleEphemeralAta(
    stashPda,
    mint,
    shuttleId,
  );
  const [shuttleAta, shuttleEataBump] = await deriveShuttleAta(
    shuttleEphemeralAta,
    mint,
  );
  const [, shuttleWalletAtaBump] = await deriveAtaWithBump(
    shuttleEphemeralAta,
    mint,
    tokenProgram,
  );
  const [, bufferBump] = await getProgramDerivedAddress({
    programAddress: EPHEMERAL_SPL_TOKEN_PROGRAM_ID,
    seeds: [BUFFER_SEED, addressEncoder.encode(shuttleAta)],
  });
  const [, delegationRecordBump] = await getProgramDerivedAddress({
    programAddress: DELEGATION_PROGRAM_ID,
    seeds: [DELEGATION_RECORD_SEED, addressEncoder.encode(shuttleAta)],
  });
  const [, delegationMetadataBump] = await getProgramDerivedAddress({
    programAddress: DELEGATION_PROGRAM_ID,
    seeds: [DELEGATION_METADATA_SEED, addressEncoder.encode(shuttleAta)],
  });
  const [vault, globalVaultBump] = await deriveVault(mint);
  const [, vaultTokenBump] = await deriveAtaWithBump(vault, mint, tokenProgram);
  const [, queueBump] = await deriveTransferQueue(mint, validator);
  const [hydraCrankPda] = await deriveHydraCrankPda(stashPda, shuttleId);

  const encryptedDestination = encryptEd25519Recipient(
    new Uint8Array(addressEncoder.encode(destinationOwner)),
    validator,
  );
  const encryptedSuffix = encryptEd25519Recipient(
    packPrivateTransferSuffix(minDelayMs, maxDelayMs, split, clientRefId),
    validator,
  );

  const data = Buffer.concat([
    Buffer.from([SCHEDULE_PRIVATE_TRANSFER_DISCRIMINATOR]),
    u32leBuffer(shuttleId),
    Buffer.from([stashBump]),
    Buffer.from(addressEncoder.encode(mint)),
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
    encodeLengthPrefixedBytes(new Uint8Array(addressEncoder.encode(validator))),
    encodeLengthPrefixedBytes(encryptedDestination),
    encodeLengthPrefixedBytes(encryptedSuffix),
  ]);

  return {
    accounts: [
      { address: user, role: AccountRole.WRITABLE_SIGNER },
      { address: stashPda, role: AccountRole.WRITABLE },
      { address: rentPda, role: AccountRole.WRITABLE },
      { address: hydraCrankPda, role: AccountRole.WRITABLE },
      { address: HYDRA_PROGRAM_ID, role: AccountRole.READONLY },
      { address: SYSTEM_PROGRAM_ADDRESS, role: AccountRole.READONLY },
      { address: tokenProgram, role: AccountRole.READONLY },
    ],
    data,
    programAddress: EPHEMERAL_SPL_TOKEN_PROGRAM_ID,
  };
}

async function deriveAtaWithBump(
  wallet: Address,
  mint: Address,
  tokenProgram: Address,
): Promise<[Address, number]> {
  const addressEncoder = getAddressEncoder();
  const [ata, bump] = await getProgramDerivedAddress({
    programAddress: ASSOCIATED_TOKEN_PROGRAM_ADDRESS,
    seeds: [
      addressEncoder.encode(wallet),
      addressEncoder.encode(tokenProgram),
      addressEncoder.encode(mint),
    ],
  });
  return [ata, bump];
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
