import {
  Address,
  Instruction,
  AccountRole,
  getAddressEncoder,
  getProgramDerivedAddress,
  getAddressDecoder,
  getU64Encoder,
  getU64Decoder,
  getStructEncoder,
  getStructDecoder,
  combineCodec,
  Encoder,
  Decoder,
  Codec,
  AccountInfoWithBase64EncodedData,
  AccountInfoBase,
} from "@solana/kit";
import { SYSTEM_PROGRAM_ADDRESS } from "@solana-program/system";
import { blake2b } from "@noble/hashes/blake2b";
import { edwardsToMontgomeryPub } from "@noble/curves/ed25519";
import * as nacl from "tweetnacl";

import {
  DELEGATION_PROGRAM_ID,
  MAGIC_CONTEXT_ID,
  MAGIC_PROGRAM_ID,
  PERMISSION_PROGRAM_ID,
  EPHEMERAL_SPL_TOKEN_PROGRAM_ID,
} from "../../constants";
import {
  delegateBufferPdaFromDelegatedAccountAndOwnerProgram,
  delegationRecordPdaFromDelegatedAccount,
  delegationMetadataPdaFromDelegatedAccount,
  permissionPdaFromAccount,
} from "../../pda";
import {
  depositAndQueueTransferIx,
  deriveTransferQueue,
  initTransferQueueIx,
} from "./transferQueue";

// SPL Token program IDs
const U64_ENCODER = getU64Encoder();

const TOKEN_PROGRAM_ADDRESS =
  "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA" as const;
const ASSOCIATED_TOKEN_PROGRAM_ADDRESS =
  "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL" as const;
const QUEUED_TRANSFER_FLAG_CREATE_IDEMPOTENT_ATA = 1 << 0;

/**
 * Derive the Associated Token Account for a given mint/owner pair.
 * Mirrors the behavior of @solana/spl-token's getAssociatedTokenAddressSync.
 * @param mint - The mint account address
 * @param owner - The owner account address
 * @param allowOwnerOffCurve - Whether to allow off-curve owner (for PDAs)
 * @returns The Associated Token Account address
 */
async function getAssociatedTokenAddressSync(
  mint: Address,
  owner: Address,
  allowOwnerOffCurve: boolean = true,
): Promise<Address> {
  const addressEncoder = getAddressEncoder();
  const [ata] = await getProgramDerivedAddress({
    programAddress: ASSOCIATED_TOKEN_PROGRAM_ADDRESS as Address,
    seeds: [
      addressEncoder.encode(owner),
      addressEncoder.encode(TOKEN_PROGRAM_ADDRESS as Address),
      addressEncoder.encode(mint),
    ],
  });
  return ata;
}

function createTransferInstruction(
  source: Address,
  destination: Address,
  owner: Address,
  amount: bigint,
): Instruction {
  return {
    accounts: [
      { address: source, role: AccountRole.WRITABLE },
      { address: destination, role: AccountRole.WRITABLE },
      { address: owner, role: AccountRole.READONLY_SIGNER },
    ],
    data: encodeAmountInstructionData(3, amount),
    programAddress: TOKEN_PROGRAM_ADDRESS as Address,
  };
}

function encryptEd25519Recipient(
  plaintext: Uint8Array,
  recipient: Address,
): Buffer {
  const recipientBytes = getAddressEncoder().encode(recipient);
  const recipientX25519 = edwardsToMontgomeryPub(
    new Uint8Array(recipientBytes),
  );
  const ephemeral = nacl.box.keyPair();
  const nonce = blake2b(
    Buffer.concat([
      Buffer.from(ephemeral.publicKey),
      Buffer.from(recipientX25519),
    ]),
    { dkLen: nacl.box.nonceLength },
  );
  const ciphertext = nacl.box(
    plaintext,
    nonce,
    recipientX25519,
    ephemeral.secretKey,
  );

  return Buffer.concat([
    Buffer.from(ephemeral.publicKey),
    Buffer.from(ciphertext),
  ]);
}

function encodeLengthPrefixedBytes(bytes: Uint8Array): Buffer {
  if (bytes.length > 0xff) {
    throw new Error("encrypted private transfer payload exceeds u8 length");
  }

  return Buffer.concat([Buffer.from([bytes.length]), Buffer.from(bytes)]);
}

function packPrivateTransferSuffix(
  minDelayMs: bigint,
  maxDelayMs: bigint,
  split: number,
  flags: number = 0,
): Buffer {
  const suffix = Buffer.alloc(8 + 8 + 4 + 1);
  suffix.writeBigUInt64LE(minDelayMs, 0);
  suffix.writeBigUInt64LE(maxDelayMs, 8);
  suffix.writeUInt32LE(split, 16);
  suffix.writeUInt8(flags, 20);
  return suffix;
}

function u32leBuffer(value: number): Buffer {
  const out = Buffer.alloc(4);
  out.writeUInt32LE(value, 0);
  return out;
}

function u64leBuffer(value: bigint): Buffer {
  const out = Buffer.alloc(8);
  out.writeBigUInt64LE(value, 0);
  return out;
}

// ---------------------------------------------------------------------------
// Accounts
// ---------------------------------------------------------------------------

/**
 * Ephemeral ATA
 */
export interface EphemeralAta {
  /// The owner of the eata
  owner: Address;
  /// The mint associated with this account
  mint: Address;
  /// The amount of tokens this account holds.
  amount: bigint;
}

/**
 * Get encoder for Ephemeral ATA
 */
export function getEphemeralAtaEncoder(): Encoder<EphemeralAta> {
  return getStructEncoder([
    ["owner", getAddressEncoder()],
    ["mint", getAddressEncoder()],
    ["amount", getU64Encoder()],
  ]);
}

/**
 * Get decoder for Ephemeral ATA
 */
export function getEphemeralAtaDecoder(): Decoder<EphemeralAta> {
  return getStructDecoder([
    ["owner", getAddressDecoder()],
    ["mint", getAddressDecoder()],
    ["amount", getU64Decoder()],
  ]);
}

/**
 * Get codec for Ephemeral ATA
 */
export function getEphemeralAtaCodec(): Codec<EphemeralAta> {
  return combineCodec(getEphemeralAtaEncoder(), getEphemeralAtaDecoder());
}

/**
 * Decode ephemeral ATA from account data
 * @param account - The account info with base64 encoded data
 * @returns The decoded ephemeral ATA
 */
export function decodeEphemeralAta(
  account: AccountInfoBase & AccountInfoWithBase64EncodedData,
): EphemeralAta {
  const codec = getEphemeralAtaCodec();
  return codec.decode(Buffer.from(account.data[0], "base64"));
}

/**
 * Global Vault
 */
export interface GlobalVault {
  /// The mint associated with this vault
  mint: Address;
}

/**
 * Get encoder for Global Vault
 */
export function getGlobalVaultEncoder(): Encoder<GlobalVault> {
  return getStructEncoder([["mint", getAddressEncoder()]]);
}

/**
 * Get decoder for Global Vault
 */
export function getGlobalVaultDecoder(): Decoder<GlobalVault> {
  return getStructDecoder([["mint", getAddressDecoder()]]);
}

/**
 * Get codec for Global Vault
 */
export function getGlobalVaultCodec(): Codec<GlobalVault> {
  return combineCodec(getGlobalVaultEncoder(), getGlobalVaultDecoder());
}

/**
 * Decode global vault from account data
 * @param account - The account info with base64 encoded data
 * @returns The decoded global vault
 */
export function decodeGlobalVault(
  account: AccountInfoBase & AccountInfoWithBase64EncodedData,
): GlobalVault {
  const codec = getGlobalVaultCodec();
  return codec.decode(Buffer.from(account.data[0], "base64"));
}

// ---------------------------------------------------------------------------
// PDA derivation helpers
// ---------------------------------------------------------------------------

/**
 * Derive ephemeral ATA
 * @param owner - The owner account
 * @param mint - The mint account
 * @returns The ephemeral ATA account and bump
 */
export async function deriveEphemeralAta(
  owner: Address,
  mint: Address,
): Promise<[Address, number]> {
  const addressEncoder = getAddressEncoder();
  const [ata, bump] = await getProgramDerivedAddress({
    programAddress: EPHEMERAL_SPL_TOKEN_PROGRAM_ID,
    seeds: [addressEncoder.encode(owner), addressEncoder.encode(mint)],
  });
  return [ata, bump];
}

/**
 * Derive vault
 * @param mint - The mint account
 * @returns The vault account and bump
 */
export async function deriveVault(mint: Address): Promise<[Address, number]> {
  const addressEncoder = getAddressEncoder();
  const [vault, bump] = await getProgramDerivedAddress({
    programAddress: EPHEMERAL_SPL_TOKEN_PROGRAM_ID,
    seeds: [addressEncoder.encode(mint)],
  });
  return [vault, bump];
}

/**
 * Derive global rent PDA
 * @returns The rent PDA account and bump
 */
export async function deriveRentPda(): Promise<[Address, number]> {
  const [rentPda, bump] = await getProgramDerivedAddress({
    programAddress: EPHEMERAL_SPL_TOKEN_PROGRAM_ID,
    seeds: [new Uint8Array([114, 101, 110, 116])],
  });
  return [rentPda, bump];
}

/**
 * Derive delegated lamports PDA
 * @param payer - The payer account
 * @param destination - The destination delegated account
 * @param salt - User-provided 32-byte salt
 * @returns The delegated lamports PDA and bump
 */
export async function deriveLamportsPda(
  payer: Address,
  destination: Address,
  salt: Uint8Array,
): Promise<[Address, number]> {
  if (salt.length !== 32) {
    throw new Error("salt must be exactly 32 bytes");
  }

  const addressEncoder = getAddressEncoder();
  const [lamportsPda, bump] = await getProgramDerivedAddress({
    programAddress: EPHEMERAL_SPL_TOKEN_PROGRAM_ID,
    seeds: [
      Buffer.from("lamports"),
      addressEncoder.encode(payer),
      addressEncoder.encode(destination),
      Buffer.from(salt),
    ],
  });
  return [lamportsPda, bump];
}

/**
 * Derive vault ATA
 * @param mint - The mint account
 * @param vault - The vault account
 * @returns The vault ATA account
 */
export async function deriveVaultAta(
  mint: Address,
  vault: Address,
): Promise<Address> {
  return getAssociatedTokenAddressSync(mint, vault, true);
}

/**
 * Derive shuttle metadata PDA
 * @param owner - The owner account
 * @param mint - The mint account
 * @param shuttleId - The shuttle id (u32)
 * @returns The shuttle metadata account and bump
 */
export async function deriveShuttleEphemeralAta(
  owner: Address,
  mint: Address,
  shuttleId: number,
): Promise<[Address, number]> {
  if (
    !Number.isInteger(shuttleId) ||
    shuttleId < 0 ||
    shuttleId > 0xffff_ffff
  ) {
    throw new Error("shuttleId must fit in u32");
  }

  const shuttleIdSeed = new Uint8Array(4);
  new DataView(shuttleIdSeed.buffer).setUint32(0, shuttleId, true);

  const addressEncoder = getAddressEncoder();
  const [shuttle, bump] = await getProgramDerivedAddress({
    programAddress: EPHEMERAL_SPL_TOKEN_PROGRAM_ID,
    seeds: [
      addressEncoder.encode(owner),
      addressEncoder.encode(mint),
      shuttleIdSeed,
    ],
  });
  return [shuttle, bump];
}

/**
 * Derive shuttle EATA PDA
 * @param shuttleEphemeralAta - The shuttle metadata account
 * @param mint - The mint account
 * @returns The shuttle EATA account and bump
 */
export async function deriveShuttleAta(
  shuttleEphemeralAta: Address,
  mint: Address,
): Promise<[Address, number]> {
  const addressEncoder = getAddressEncoder();
  const [shuttleAta, bump] = await getProgramDerivedAddress({
    programAddress: EPHEMERAL_SPL_TOKEN_PROGRAM_ID,
    seeds: [
      addressEncoder.encode(shuttleEphemeralAta),
      addressEncoder.encode(mint),
    ],
  });
  return [shuttleAta, bump];
}

/**
 * Derive shuttle wallet ATA
 * @param mint - The mint account
 * @param shuttleEphemeralAta - The shuttle metadata account
 * @returns The shuttle wallet ATA account
 */
export async function deriveShuttleWalletAta(
  mint: Address,
  shuttleEphemeralAta: Address,
): Promise<Address> {
  return getAssociatedTokenAddressSync(mint, shuttleEphemeralAta, true);
}

// ---------------------------------------------------------------------------
// Instruction builders
// ---------------------------------------------------------------------------

/**
 * Init ephemeral ATA
 * @param ephemeralAta - The ephemeral ATA account
 * @param owner - The owner account
 * @param mint - The mint account
 * @param payer - The payer account
 * @returns The init ephemeral ATA instruction
 */
export function initEphemeralAtaIx(
  ephemeralAta: Address,
  owner: Address,
  mint: Address,
  payer: Address,
): Instruction {
  return {
    accounts: [
      { address: ephemeralAta, role: AccountRole.WRITABLE },
      { address: payer, role: AccountRole.WRITABLE_SIGNER },
      { address: owner, role: AccountRole.READONLY },
      { address: mint, role: AccountRole.READONLY },
      { address: SYSTEM_PROGRAM_ADDRESS, role: AccountRole.READONLY },
    ],
    data: new Uint8Array([0]),
    programAddress: EPHEMERAL_SPL_TOKEN_PROGRAM_ID,
  };
}

/**
 * Init vault ATA
 * @param payer - The payer account
 * @param vaultAta - The vault ATA account
 * @param vault - The vault account
 * @param mint - The mint account
 * @returns The init vault ATA instruction
 */
export function initVaultAtaIx(
  payer: Address,
  vaultAta: Address,
  vault: Address,
  mint: Address,
): Instruction {
  // This is a simplified implementation that would normally use SPL token's
  // createAssociatedTokenAccountIdempotentInstruction
  // For Kit, we create it manually with the same structure
  return {
    accounts: [
      { address: payer, role: AccountRole.WRITABLE_SIGNER },
      { address: vaultAta, role: AccountRole.WRITABLE },
      { address: vault, role: AccountRole.READONLY },
      { address: mint, role: AccountRole.READONLY },
      { address: SYSTEM_PROGRAM_ADDRESS, role: AccountRole.READONLY },
      { address: TOKEN_PROGRAM_ADDRESS as Address, role: AccountRole.READONLY },
    ],
    data: new Uint8Array([1]),
    programAddress: ASSOCIATED_TOKEN_PROGRAM_ADDRESS as Address,
  };
}

/**
 * Init vault account
 * @param vault - The vault account
 * @param mint - The mint account
 * @param payer - The payer account
 * @param vaultEphemeralAta - The vault ephemeral ATA account
 * @param vaultAta - The vault ATA account
 * @returns The init vault account instruction
 */
export function initVaultIx(
  vault: Address,
  mint: Address,
  payer: Address,
  vaultEphemeralAta: Address,
  vaultAta: Address,
): Instruction {
  return {
    accounts: [
      { address: vault, role: AccountRole.WRITABLE },
      { address: payer, role: AccountRole.WRITABLE_SIGNER },
      { address: mint, role: AccountRole.READONLY },
      { address: vaultEphemeralAta, role: AccountRole.WRITABLE },
      { address: vaultAta, role: AccountRole.WRITABLE },
      { address: TOKEN_PROGRAM_ADDRESS as Address, role: AccountRole.READONLY },
      {
        address: ASSOCIATED_TOKEN_PROGRAM_ADDRESS as Address,
        role: AccountRole.READONLY,
      },
      { address: SYSTEM_PROGRAM_ADDRESS, role: AccountRole.READONLY },
    ],
    data: new Uint8Array([1]),
    programAddress: EPHEMERAL_SPL_TOKEN_PROGRAM_ID,
  };
}

/**
 * Init global rent PDA
 * @param payer - The payer account
 * @param rentPda - The rent PDA account
 * @returns The init rent PDA instruction
 */
export function initRentPdaIx(payer: Address, rentPda: Address): Instruction {
  return {
    accounts: [
      { address: payer, role: AccountRole.WRITABLE_SIGNER },
      { address: rentPda, role: AccountRole.WRITABLE },
      { address: SYSTEM_PROGRAM_ADDRESS, role: AccountRole.READONLY },
    ],
    data: new Uint8Array([23]),
    programAddress: EPHEMERAL_SPL_TOKEN_PROGRAM_ID,
  };
}

/**
 * Transfer tokens to vault
 * @param ephemeralAta - The ephemeral ATA account
 * @param vault - The vault account
 * @param mint - The mint account
 * @param sourceAta - The source ATA account
 * @param vaultAta - The vault ATA account
 * @param owner - The owner account
 * @param amount - The amount of tokens to transfer
 * @returns The transfer tokens to vault instruction
 */
export function transferToVaultIx(
  ephemeralAta: Address,
  vault: Address,
  mint: Address,
  sourceAta: Address,
  vaultAta: Address,
  owner: Address,
  amount: bigint,
): Instruction {
  return {
    accounts: [
      { address: ephemeralAta, role: AccountRole.WRITABLE },
      { address: vault, role: AccountRole.READONLY },
      { address: mint, role: AccountRole.READONLY },
      { address: sourceAta, role: AccountRole.WRITABLE },
      { address: vaultAta, role: AccountRole.WRITABLE },
      { address: owner, role: AccountRole.READONLY_SIGNER },
      { address: TOKEN_PROGRAM_ADDRESS as Address, role: AccountRole.READONLY },
    ],
    data: encodeAmountInstructionData(2, amount),
    programAddress: EPHEMERAL_SPL_TOKEN_PROGRAM_ID,
  };
}

/**
 * Delegate instruction
 * @param payer - The payer account
 * @param ephemeralAta - The ephemeral ATA account
 * @param bump - The bump
 * @param validator - The validator account
 * @returns The delegate instruction
 */
export async function delegateIx(
  payer: Address,
  ephemeralAta: Address,
  validator?: Address,
): Promise<Instruction> {
  const delegateBuffer =
    await delegateBufferPdaFromDelegatedAccountAndOwnerProgram(
      ephemeralAta,
      EPHEMERAL_SPL_TOKEN_PROGRAM_ID,
    );
  const delegationRecord =
    await delegationRecordPdaFromDelegatedAccount(ephemeralAta);
  const delegationMetadata =
    await delegationMetadataPdaFromDelegatedAccount(ephemeralAta);

  const encoder = getAddressEncoder();
  let data: Uint8Array;
  if (validator) {
    data = new Uint8Array(33);
    data[0] = 4;
    const validatorBytes = encoder.encode(validator);
    data.set(validatorBytes, 1);
  } else {
    data = new Uint8Array(1);
    data[0] = 4;
  }

  return {
    accounts: [
      { address: payer, role: AccountRole.WRITABLE_SIGNER },
      { address: ephemeralAta, role: AccountRole.WRITABLE },
      { address: EPHEMERAL_SPL_TOKEN_PROGRAM_ID, role: AccountRole.READONLY },
      { address: delegateBuffer, role: AccountRole.WRITABLE },
      { address: delegationRecord, role: AccountRole.WRITABLE },
      { address: delegationMetadata, role: AccountRole.WRITABLE },
      { address: DELEGATION_PROGRAM_ID, role: AccountRole.READONLY },
      { address: SYSTEM_PROGRAM_ADDRESS, role: AccountRole.READONLY },
    ],
    data,
    programAddress: EPHEMERAL_SPL_TOKEN_PROGRAM_ID,
  };
}

/**
 * Initialize shuttle ephemeral ATA + wallet ATA
 * @param payer - The payer account
 * @param shuttleEphemeralAta - The shuttle metadata account
 * @param shuttleAta - The shuttle EATA account
 * @param shuttleWalletAta - The shuttle wallet ATA account
 * @param owner - The owner account
 * @param mint - The mint account
 * @param shuttleId - The shuttle id (u32)
 * @param bump - The shuttle metadata bump
 * @returns The initialize shuttle instruction
 */
export function initShuttleEphemeralAtaIx(
  payer: Address,
  shuttleEphemeralAta: Address,
  shuttleAta: Address,
  shuttleWalletAta: Address,
  owner: Address,
  mint: Address,
  shuttleId: number,
): Instruction {
  if (
    !Number.isInteger(shuttleId) ||
    shuttleId < 0 ||
    shuttleId > 0xffff_ffff
  ) {
    throw new Error("shuttleId must fit in u32");
  }

  const data = new Uint8Array(5);
  data[0] = 11;
  new DataView(data.buffer).setUint32(1, shuttleId, true);

  return {
    accounts: [
      { address: payer, role: AccountRole.WRITABLE_SIGNER },
      { address: shuttleEphemeralAta, role: AccountRole.WRITABLE },
      { address: shuttleAta, role: AccountRole.WRITABLE },
      { address: shuttleWalletAta, role: AccountRole.WRITABLE },
      { address: owner, role: AccountRole.READONLY },
      { address: mint, role: AccountRole.READONLY },
      { address: TOKEN_PROGRAM_ADDRESS as Address, role: AccountRole.READONLY },
      {
        address: ASSOCIATED_TOKEN_PROGRAM_ADDRESS as Address,
        role: AccountRole.READONLY,
      },
      { address: SYSTEM_PROGRAM_ADDRESS, role: AccountRole.READONLY },
    ],
    data,
    programAddress: EPHEMERAL_SPL_TOKEN_PROGRAM_ID,
  };
}

/**
 * Delegate shuttle ephemeral ATA
 * @param payer - The payer account
 * @param shuttleEphemeralAta - The shuttle metadata account
 * @param shuttleAta - The shuttle EATA account
 * @param bump - The shuttle EATA bump
 * @param validator - Optional validator address
 * @returns The delegate shuttle instruction
 */
export async function delegateShuttleEphemeralAtaIx(
  payer: Address,
  shuttleEphemeralAta: Address,
  shuttleAta: Address,
  validator?: Address,
): Promise<Instruction> {
  const delegateBuffer =
    await delegateBufferPdaFromDelegatedAccountAndOwnerProgram(
      shuttleAta,
      EPHEMERAL_SPL_TOKEN_PROGRAM_ID,
    );
  const delegationRecord =
    await delegationRecordPdaFromDelegatedAccount(shuttleAta);
  const delegationMetadata =
    await delegationMetadataPdaFromDelegatedAccount(shuttleAta);

  const addressEncoder = getAddressEncoder();
  let data: Uint8Array;
  if (validator) {
    data = new Uint8Array(33);
    data[0] = 13;
    data.set(addressEncoder.encode(validator), 1);
  } else {
    data = new Uint8Array(1);
    data[0] = 13;
  }

  return {
    accounts: [
      { address: payer, role: AccountRole.WRITABLE_SIGNER },
      { address: shuttleEphemeralAta, role: AccountRole.READONLY },
      { address: shuttleAta, role: AccountRole.WRITABLE },
      { address: EPHEMERAL_SPL_TOKEN_PROGRAM_ID, role: AccountRole.READONLY },
      { address: delegateBuffer, role: AccountRole.WRITABLE },
      { address: delegationRecord, role: AccountRole.WRITABLE },
      { address: delegationMetadata, role: AccountRole.WRITABLE },
      { address: DELEGATION_PROGRAM_ID, role: AccountRole.READONLY },
      { address: SYSTEM_PROGRAM_ADDRESS, role: AccountRole.READONLY },
    ],
    data,
    programAddress: EPHEMERAL_SPL_TOKEN_PROGRAM_ID,
  };
}

/**
 * Initialize shuttle metadata/EATA/wallet ATA, deposit into the shuttle EATA,
 * then delegate it with implicit merge and cleanup.
 * @param payer - The payer account
 * @param shuttleEphemeralAta - The shuttle metadata account
 * @param shuttleAta - The shuttle EATA account
 * @param owner - The shuttle owner signer and deposit authority
 * @param sourceAta - The owner source token account
 * @param destinationAta - The destination token account for the merge
 * @param shuttleWalletAta - The shuttle wallet ATA account
 * @param mint - The mint account
 * @param shuttleId - The shuttle id (u32)
 * @param bump - The shuttle metadata bump
 * @param amount - The amount to deposit before delegation
 * @param validator - Optional validator address
 * @returns The setup+delegate shuttle-with-merge instruction
 */
export async function setupAndDelegateShuttleEphemeralAtaWithMergeIx(
  payer: Address,
  shuttleEphemeralAta: Address,
  shuttleAta: Address,
  owner: Address,
  sourceAta: Address,
  destinationAta: Address,
  shuttleWalletAta: Address,
  mint: Address,
  shuttleId: number,
  amount: bigint,
  validator?: Address,
): Promise<Instruction> {
  if (
    !Number.isInteger(shuttleId) ||
    shuttleId < 0 ||
    shuttleId > 0xffff_ffff
  ) {
    throw new Error("shuttleId must fit in u32");
  }

  const [rentPda] = await deriveRentPda();
  const [vault] = await deriveVault(mint);
  const vaultAta = await deriveVaultAta(mint, vault);
  const delegateBuffer =
    await delegateBufferPdaFromDelegatedAccountAndOwnerProgram(
      shuttleAta,
      EPHEMERAL_SPL_TOKEN_PROGRAM_ID,
    );
  const delegationRecord =
    await delegationRecordPdaFromDelegatedAccount(shuttleAta);
  const delegationMetadata =
    await delegationMetadataPdaFromDelegatedAccount(shuttleAta);

  const addressEncoder = getAddressEncoder();
  const data = new Uint8Array(validator ? 45 : 13);
  const dataView = new DataView(data.buffer, data.byteOffset, data.byteLength);
  data[0] = 24;
  dataView.setUint32(1, shuttleId, true);
  dataView.setBigUint64(5, amount, true);
  if (validator) {
    data.set(addressEncoder.encode(validator), 13);
  }

  return {
    accounts: [
      { address: payer, role: AccountRole.WRITABLE_SIGNER },
      { address: rentPda, role: AccountRole.WRITABLE },
      { address: shuttleEphemeralAta, role: AccountRole.WRITABLE },
      { address: shuttleAta, role: AccountRole.WRITABLE },
      { address: shuttleWalletAta, role: AccountRole.WRITABLE },
      { address: owner, role: AccountRole.READONLY_SIGNER },
      { address: EPHEMERAL_SPL_TOKEN_PROGRAM_ID, role: AccountRole.READONLY },
      { address: delegateBuffer, role: AccountRole.WRITABLE },
      { address: delegationRecord, role: AccountRole.WRITABLE },
      { address: delegationMetadata, role: AccountRole.WRITABLE },
      { address: DELEGATION_PROGRAM_ID, role: AccountRole.READONLY },
      {
        address: ASSOCIATED_TOKEN_PROGRAM_ADDRESS as Address,
        role: AccountRole.READONLY,
      },
      { address: SYSTEM_PROGRAM_ADDRESS, role: AccountRole.READONLY },
      { address: destinationAta, role: AccountRole.WRITABLE },
      { address: mint, role: AccountRole.READONLY },
      { address: TOKEN_PROGRAM_ADDRESS as Address, role: AccountRole.READONLY },
      { address: vault, role: AccountRole.READONLY },
      { address: sourceAta, role: AccountRole.WRITABLE },
      { address: vaultAta, role: AccountRole.WRITABLE },
    ],
    data,
    programAddress: EPHEMERAL_SPL_TOKEN_PROGRAM_ID,
  };
}

/**
 * Initialize shuttle metadata/EATA/wallet ATA, deposit into the shuttle EATA,
 * delegate it, then queue a private transfer as a third post-delegation action.
 * The destination owner is only carried inside the encrypted post-delegation action
 * and is not passed as a cleartext account to this outer instruction.
 * @param payer - The payer account
 * @param shuttleEphemeralAta - The shuttle metadata account
 * @param shuttleAta - The shuttle EATA account
 * @param owner - The shuttle owner signer and deposit authority
 * @param sourceAta - The owner source token account
 * @param destinationOwner - The destination owner for the delayed private transfer
 * @param shuttleWalletAta - The shuttle wallet ATA account
 * @param mint - The mint account
 * @param shuttleId - The shuttle id (u32)
 * @param bump - The shuttle metadata bump
 * @param amount - The amount to deposit before delegation
 * @param minDelayMs - Minimum transfer delay in milliseconds
 * @param maxDelayMs - Maximum transfer delay in milliseconds
 * @param split - Number of queued transfer splits
 * @param validator - Optional validator address
 * @returns The setup+delegate shuttle-with-merge-and-private-transfer instruction
 */
export async function depositAndDelegateShuttleEphemeralAtaWithMergeAndPrivateTransferIx(
  payer: Address,
  shuttleEphemeralAta: Address,
  shuttleAta: Address,
  owner: Address,
  sourceAta: Address,
  destinationOwner: Address,
  shuttleWalletAta: Address,
  mint: Address,
  shuttleId: number,
  amount: bigint,
  minDelayMs: bigint,
  maxDelayMs: bigint,
  split: number,
  validator?: Address,
): Promise<Instruction> {
  if (
    !Number.isInteger(shuttleId) ||
    shuttleId < 0 ||
    shuttleId > 0xffff_ffff
  ) {
    throw new Error("shuttleId must fit in u32");
  }
  if (amount < 0n) {
    throw new Error("amount must be non-negative");
  }
  if (minDelayMs < 0n || maxDelayMs < 0n) {
    throw new Error("delay values must be non-negative");
  }
  if (maxDelayMs < minDelayMs) {
    throw new Error("maxDelayMs must be greater than or equal to minDelayMs");
  }
  if (validator == null) {
    throw new Error("validator is required for encrypted private transfers");
  }
  if (!Number.isInteger(split) || split <= 0 || split > 0xffff_ffff) {
    throw new Error("split must fit in u32 and be positive");
  }

  const [rentPda] = await deriveRentPda();
  const [vault] = await deriveVault(mint);
  const vaultAta = await deriveVaultAta(mint, vault);
  const [queue] = await deriveTransferQueue(mint, validator);
  const delegateBuffer =
    await delegateBufferPdaFromDelegatedAccountAndOwnerProgram(
      shuttleAta,
      EPHEMERAL_SPL_TOKEN_PROGRAM_ID,
    );
  const delegationRecord =
    await delegationRecordPdaFromDelegatedAccount(shuttleAta);
  const delegationMetadata =
    await delegationMetadataPdaFromDelegatedAccount(shuttleAta);

  const addressEncoder = getAddressEncoder();
  const encryptedDestination = encryptEd25519Recipient(
    new Uint8Array(addressEncoder.encode(destinationOwner)),
    validator,
  );
  const encryptedSuffix = encryptEd25519Recipient(
    packPrivateTransferSuffix(
      minDelayMs,
      maxDelayMs,
      split,
      QUEUED_TRANSFER_FLAG_CREATE_IDEMPOTENT_ATA,
    ),
    validator,
  );
  const data = Buffer.concat([
    Buffer.from([25]),
    u32leBuffer(shuttleId),
    u64leBuffer(amount),
    encodeLengthPrefixedBytes(new Uint8Array(addressEncoder.encode(validator))),
    encodeLengthPrefixedBytes(encryptedDestination),
    encodeLengthPrefixedBytes(encryptedSuffix),
  ]);

  return {
    accounts: [
      { address: payer, role: AccountRole.WRITABLE_SIGNER },
      { address: rentPda, role: AccountRole.WRITABLE },
      { address: shuttleEphemeralAta, role: AccountRole.WRITABLE },
      { address: shuttleAta, role: AccountRole.WRITABLE },
      { address: shuttleWalletAta, role: AccountRole.WRITABLE },
      { address: owner, role: AccountRole.READONLY_SIGNER },
      { address: EPHEMERAL_SPL_TOKEN_PROGRAM_ID, role: AccountRole.READONLY },
      { address: delegateBuffer, role: AccountRole.WRITABLE },
      { address: delegationRecord, role: AccountRole.WRITABLE },
      { address: delegationMetadata, role: AccountRole.WRITABLE },
      { address: DELEGATION_PROGRAM_ID, role: AccountRole.READONLY },
      {
        address: ASSOCIATED_TOKEN_PROGRAM_ADDRESS as Address,
        role: AccountRole.READONLY,
      },
      { address: SYSTEM_PROGRAM_ADDRESS, role: AccountRole.READONLY },
      { address: mint, role: AccountRole.READONLY },
      { address: TOKEN_PROGRAM_ADDRESS as Address, role: AccountRole.READONLY },
      { address: vault, role: AccountRole.READONLY },
      { address: sourceAta, role: AccountRole.WRITABLE },
      { address: vaultAta, role: AccountRole.WRITABLE },
      { address: queue, role: AccountRole.WRITABLE },
    ],
    data,
    programAddress: EPHEMERAL_SPL_TOKEN_PROGRAM_ID,
  };
}

/**
 * Initialize shuttle metadata/EATA/wallet ATA, delegate it, then route a
 * withdraw round-trip through the delegated shuttle.
 */
export async function withdrawThroughDelegatedShuttleWithMergeIx(
  payer: Address,
  shuttleEphemeralAta: Address,
  shuttleAta: Address,
  owner: Address,
  ownerAta: Address,
  shuttleWalletAta: Address,
  mint: Address,
  shuttleId: number,
  amount: bigint,
  validator?: Address,
): Promise<Instruction> {
  if (
    !Number.isInteger(shuttleId) ||
    shuttleId < 0 ||
    shuttleId > 0xffff_ffff
  ) {
    throw new Error("shuttleId must fit in u32");
  }
  if (amount < 0n) {
    throw new Error("amount must be non-negative");
  }

  const [rentPda] = await deriveRentPda();
  const delegateBuffer =
    await delegateBufferPdaFromDelegatedAccountAndOwnerProgram(
      shuttleAta,
      EPHEMERAL_SPL_TOKEN_PROGRAM_ID,
    );
  const delegationRecord =
    await delegationRecordPdaFromDelegatedAccount(shuttleAta);
  const delegationMetadata =
    await delegationMetadataPdaFromDelegatedAccount(shuttleAta);

  const addressEncoder = getAddressEncoder();
  const data = new Uint8Array(validator ? 45 : 13);
  const dataView = new DataView(data.buffer, data.byteOffset, data.byteLength);
  data[0] = 26;
  dataView.setUint32(1, shuttleId, true);
  dataView.setBigUint64(5, amount, true);
  if (validator) {
    data.set(addressEncoder.encode(validator), 13);
  }

  return {
    accounts: [
      { address: payer, role: AccountRole.WRITABLE_SIGNER },
      { address: rentPda, role: AccountRole.WRITABLE },
      { address: shuttleEphemeralAta, role: AccountRole.WRITABLE },
      { address: shuttleAta, role: AccountRole.WRITABLE },
      { address: shuttleWalletAta, role: AccountRole.WRITABLE },
      { address: owner, role: AccountRole.READONLY_SIGNER },
      { address: EPHEMERAL_SPL_TOKEN_PROGRAM_ID, role: AccountRole.READONLY },
      { address: delegateBuffer, role: AccountRole.WRITABLE },
      { address: delegationRecord, role: AccountRole.WRITABLE },
      { address: delegationMetadata, role: AccountRole.WRITABLE },
      { address: DELEGATION_PROGRAM_ID, role: AccountRole.READONLY },
      {
        address: ASSOCIATED_TOKEN_PROGRAM_ADDRESS as Address,
        role: AccountRole.READONLY,
      },
      { address: SYSTEM_PROGRAM_ADDRESS, role: AccountRole.READONLY },
      { address: ownerAta, role: AccountRole.WRITABLE },
      { address: mint, role: AccountRole.READONLY },
      { address: TOKEN_PROGRAM_ADDRESS as Address, role: AccountRole.READONLY },
    ],
    data,
    programAddress: EPHEMERAL_SPL_TOKEN_PROGRAM_ID,
  };
}

/**
 * Create and delegate a sponsored lamports PDA, then schedule post-delegation
 * transfer and cleanup to move lamports to a base-layer destination.
 * @param payer - The payer and action signer
 * @param destination - The delegated destination base-layer account
 * @param amount - The lamports amount to transfer
 * @param salt - User-provided 32-byte salt
 * @returns The sponsored delegated lamports transfer instruction
 */
export async function lamportsDelegatedTransferIx(
  payer: Address,
  destination: Address,
  amount: bigint,
  salt: Uint8Array,
): Promise<Instruction> {
  if (amount < 0n) {
    throw new Error("amount must be non-negative");
  }
  if (salt.length !== 32) {
    throw new Error("salt must be exactly 32 bytes");
  }

  const [rentPda] = await deriveRentPda();
  const [lamportsPda] = await deriveLamportsPda(payer, destination, salt);
  const destinationDelegationRecord =
    await delegationRecordPdaFromDelegatedAccount(destination);

  const data = Buffer.alloc(41);
  data[0] = 20;
  data.writeBigUInt64LE(amount, 1);
  Buffer.from(salt).copy(data, 9);

  return {
    accounts: [
      { address: payer, role: AccountRole.WRITABLE_SIGNER },
      { address: rentPda, role: AccountRole.WRITABLE },
      { address: lamportsPda, role: AccountRole.WRITABLE },
      {
        address: EPHEMERAL_SPL_TOKEN_PROGRAM_ID,
        role: AccountRole.READONLY,
      },
      {
        address: await delegateBufferPdaFromDelegatedAccountAndOwnerProgram(
          lamportsPda,
          EPHEMERAL_SPL_TOKEN_PROGRAM_ID,
        ),
        role: AccountRole.WRITABLE,
      },
      {
        address: await delegationRecordPdaFromDelegatedAccount(lamportsPda),
        role: AccountRole.WRITABLE,
      },
      {
        address: await delegationMetadataPdaFromDelegatedAccount(lamportsPda),
        role: AccountRole.WRITABLE,
      },
      { address: DELEGATION_PROGRAM_ID, role: AccountRole.READONLY },
      { address: SYSTEM_PROGRAM_ADDRESS, role: AccountRole.READONLY },
      { address: destination, role: AccountRole.WRITABLE },
      {
        address: destinationDelegationRecord,
        role: AccountRole.READONLY,
      },
    ],
    data,
    programAddress: EPHEMERAL_SPL_TOKEN_PROGRAM_ID,
  };
}

/**
 * Merge shuttle wallet ATA into owner ATA
 * @param owner - The owner account
 * @param ownerAta - The owner ATA destination
 * @param shuttleEphemeralAta - The shuttle metadata account
 * @param shuttleWalletAta - The shuttle wallet ATA source
 * @param mint - The mint account
 * @returns The merge shuttle instruction
 */
export function mergeShuttleIntoAtaIx(
  owner: Address,
  ownerAta: Address,
  shuttleEphemeralAta: Address,
  shuttleWalletAta: Address,
  mint: Address,
): Instruction {
  return {
    accounts: [
      { address: owner, role: AccountRole.READONLY_SIGNER },
      { address: ownerAta, role: AccountRole.WRITABLE },
      { address: shuttleEphemeralAta, role: AccountRole.READONLY },
      { address: shuttleWalletAta, role: AccountRole.WRITABLE },
      { address: mint, role: AccountRole.READONLY },
      { address: TOKEN_PROGRAM_ADDRESS as Address, role: AccountRole.READONLY },
    ],
    data: new Uint8Array([15]),
    programAddress: EPHEMERAL_SPL_TOKEN_PROGRAM_ID,
  };
}

/**
 * Undelegate shuttle wallet ATA and close it when empty.
 * @param payer - The payer account
 * @param rentReimbursement - The rent reimbursement account
 * @param shuttleEphemeralAta - The shuttle metadata account
 * @param shuttleAta - The shuttle EATA account
 * @param shuttleWalletAta - The shuttle wallet ATA account
 * @param destinationAta - The destination token account used by the close handler
 * @returns The undelegate shuttle instruction
 */
export function undelegateAndCloseShuttleEphemeralAtaIx(
  payer: Address,
  rentReimbursement: Address,
  shuttleEphemeralAta: Address,
  shuttleAta: Address,
  shuttleWalletAta: Address,
  destinationAta: Address,
  escrowIndex?: number,
): Instruction {
  if (
    escrowIndex !== undefined &&
    (!Number.isInteger(escrowIndex) || escrowIndex < 0 || escrowIndex > 0xff)
  ) {
    throw new Error("escrowIndex must fit in u8");
  }

  const data =
    escrowIndex === undefined
      ? new Uint8Array([14])
      : new Uint8Array([14, escrowIndex]);

  return {
    accounts: [
      { address: payer, role: AccountRole.WRITABLE_SIGNER },
      { address: rentReimbursement, role: AccountRole.WRITABLE },
      { address: shuttleEphemeralAta, role: AccountRole.READONLY },
      { address: shuttleAta, role: AccountRole.READONLY },
      { address: shuttleWalletAta, role: AccountRole.WRITABLE },
      { address: destinationAta, role: AccountRole.WRITABLE },
      { address: TOKEN_PROGRAM_ADDRESS as Address, role: AccountRole.READONLY },
      { address: MAGIC_CONTEXT_ID, role: AccountRole.WRITABLE },
      { address: MAGIC_PROGRAM_ID, role: AccountRole.READONLY },
    ],
    data,
    programAddress: EPHEMERAL_SPL_TOKEN_PROGRAM_ID,
  };
}

/**
 * Withdraw SPL tokens from vault to user destination
 * @param owner - The owner account
 * @param mint - The mint account
 * @param amount - The amount of tokens to withdraw
 * @returns The withdraw SPL tokens instruction
 */
export async function withdrawSplIx(
  owner: Address,
  mint: Address,
  amount: bigint,
): Promise<Instruction> {
  const [ephemeralAta] = await deriveEphemeralAta(owner, mint);
  const [vault] = await deriveVault(mint);
  const vaultAta = await deriveVaultAta(mint, vault);
  const userDestAta = await getAssociatedTokenAddressSync(mint, owner);

  return {
    accounts: [
      { address: owner, role: AccountRole.READONLY_SIGNER },
      { address: ephemeralAta, role: AccountRole.WRITABLE },
      { address: vault, role: AccountRole.READONLY },
      { address: mint, role: AccountRole.READONLY },
      { address: vaultAta, role: AccountRole.WRITABLE },
      { address: userDestAta, role: AccountRole.WRITABLE },
      { address: TOKEN_PROGRAM_ADDRESS as Address, role: AccountRole.READONLY },
    ],
    data: encodeAmountInstructionData(3, amount),
    programAddress: EPHEMERAL_SPL_TOKEN_PROGRAM_ID,
  };
}

/**
 * Undelegate instruction
 * @param owner - The owner account
 * @param mint - The mint account
 * @returns The undelegate instruction
 */
export async function undelegateIx(
  owner: Address,
  mint: Address,
): Promise<Instruction> {
  const userAta = await getAssociatedTokenAddressSync(mint, owner);
  const [ephemeralAta] = await deriveEphemeralAta(owner, mint);

  return {
    accounts: [
      {
        address: owner,
        role: AccountRole.READONLY_SIGNER,
      },
      {
        address: userAta,
        role: AccountRole.WRITABLE,
      },
      {
        address: ephemeralAta,
        role: AccountRole.READONLY,
      },
      {
        address: MAGIC_CONTEXT_ID,
        role: AccountRole.WRITABLE,
      },
      {
        address: MAGIC_PROGRAM_ID,
        role: AccountRole.READONLY,
      },
    ],
    data: new Uint8Array([5]),
    programAddress: EPHEMERAL_SPL_TOKEN_PROGRAM_ID,
  };
}

/**
 * Create EATA permission
 * @param ephemeralAta - The ephemeral ATA account
 * @param payer - The payer account
 * @param bump - The bump
 * @param flags - The flags
 * @returns The create EATA permission instruction
 */
export async function createEataPermissionIx(
  ephemeralAta: Address,
  payer: Address,
  flags: number = 0,
): Promise<Instruction> {
  const permission = await permissionPdaFromAccount(ephemeralAta);

  return {
    accounts: [
      { address: ephemeralAta, role: AccountRole.WRITABLE },
      { address: permission, role: AccountRole.WRITABLE },
      { address: payer, role: AccountRole.WRITABLE_SIGNER },
      { address: SYSTEM_PROGRAM_ADDRESS, role: AccountRole.READONLY },
      { address: PERMISSION_PROGRAM_ID, role: AccountRole.READONLY },
    ],
    data: new Uint8Array([6, flags]),
    programAddress: EPHEMERAL_SPL_TOKEN_PROGRAM_ID,
  };
}

/**
 * Reset EATA permission
 * @param ephemeralAta - The ephemeral ATA account
 * @param payer - The payer account
 * @param bump - The bump
 * @param flags - The flags
 * @returns The reset EATA permission instruction
 */
export async function resetEataPermissionIx(
  ephemeralAta: Address,
  payer: Address,
  flags: number = 0,
): Promise<Instruction> {
  const permission = await permissionPdaFromAccount(ephemeralAta);

  return {
    accounts: [
      { address: ephemeralAta, role: AccountRole.READONLY },
      { address: permission, role: AccountRole.WRITABLE },
      { address: payer, role: AccountRole.READONLY_SIGNER },
      { address: PERMISSION_PROGRAM_ID, role: AccountRole.READONLY },
    ],
    data: new Uint8Array([9, flags]),
    programAddress: EPHEMERAL_SPL_TOKEN_PROGRAM_ID,
  };
}

/**
 * Delegate EATA permission
 * @param payer - The payer account
 * @param ephemeralAta - The ephemeral ATA account
 * @param validator - The validator account
 * @returns The delegate EATA permission instruction
 */
export async function delegateEataPermissionIx(
  payer: Address,
  ephemeralAta: Address,
  validator: Address,
): Promise<Instruction> {
  const permission = await permissionPdaFromAccount(ephemeralAta);

  return {
    accounts: [
      { address: payer, role: AccountRole.WRITABLE_SIGNER },
      { address: ephemeralAta, role: AccountRole.WRITABLE },
      { address: PERMISSION_PROGRAM_ID, role: AccountRole.READONLY },
      { address: permission, role: AccountRole.WRITABLE },
      { address: SYSTEM_PROGRAM_ADDRESS, role: AccountRole.READONLY },
      {
        address: await delegateBufferPdaFromDelegatedAccountAndOwnerProgram(
          permission,
          PERMISSION_PROGRAM_ID,
        ),
        role: AccountRole.WRITABLE,
      },
      {
        address: await delegationRecordPdaFromDelegatedAccount(permission),
        role: AccountRole.WRITABLE,
      },
      {
        address: await delegationMetadataPdaFromDelegatedAccount(permission),
        role: AccountRole.WRITABLE,
      },
      { address: DELEGATION_PROGRAM_ID, role: AccountRole.READONLY },
      { address: validator, role: AccountRole.READONLY },
    ],
    data: new Uint8Array([7]),
    programAddress: EPHEMERAL_SPL_TOKEN_PROGRAM_ID,
  };
}

/**
 * Undelegate EATA permission
 * @param owner - The owner account
 * @param ephemeralAta - The ephemeral ATA account
 * @returns The undelegate EATA permission instruction
 */
export async function undelegateEataPermissionIx(
  owner: Address,
  ephemeralAta: Address,
): Promise<Instruction> {
  const permission = await permissionPdaFromAccount(ephemeralAta);

  return {
    accounts: [
      { address: owner, role: AccountRole.READONLY_SIGNER },
      { address: ephemeralAta, role: AccountRole.WRITABLE },
      { address: permission, role: AccountRole.WRITABLE },
      { address: PERMISSION_PROGRAM_ID, role: AccountRole.READONLY },
      { address: MAGIC_PROGRAM_ID, role: AccountRole.READONLY },
      { address: MAGIC_CONTEXT_ID, role: AccountRole.WRITABLE },
    ],
    data: new Uint8Array([8]),
    programAddress: EPHEMERAL_SPL_TOKEN_PROGRAM_ID,
  };
}

// ---------------------------------------------------------------------------
// High-level SDK methods
// ---------------------------------------------------------------------------

export interface DelegateSplOptions {
  payer?: Address;
  validator?: Address;
  initIfMissing?: boolean;
  initVaultIfMissing?: boolean;
  initAtasIfMissing?: boolean;
  shuttleId?: number;
  idempotent?: boolean;
  private?: boolean;
}

export interface DelegateSplWithPrivateTransferOptions
  extends Omit<DelegateSplOptions, "private"> {
  minDelayMs?: bigint;
  maxDelayMs?: bigint;
  split?: number;
  initTransferQueueIfMissing?: boolean;
}

export interface WithdrawSplOptions
  extends Omit<DelegateSplOptions, "private" | "initVaultIfMissing"> {}

export type TransferBalance = "base" | "ephemeral";

export type TransferVisibility = "public" | "private";

export interface TransferSplPrivateOptions {
  minDelayMs?: bigint;
  maxDelayMs?: bigint;
  split?: number;
}

export interface TransferSplOptions {
  visibility: TransferVisibility;
  fromBalance: TransferBalance;
  toBalance: TransferBalance;
  payer?: Address;
  validator?: Address;
  initIfMissing?: boolean;
  initAtasIfMissing?: boolean;
  initVaultIfMissing?: boolean;
  shuttleId?: number;
  privateTransfer?: TransferSplPrivateOptions;
}

function randomShuttleId(): number {
  const cryptoObj = (globalThis as any)?.crypto;
  if (cryptoObj?.getRandomValues !== undefined) {
    const buf = new Uint32Array(1);
    cryptoObj.getRandomValues(buf);
    return buf[0];
  }
  return Math.floor(Math.random() * 0x1_0000_0000);
}

async function buildDelegateSplInstructions(
  owner: Address,
  mint: Address,
  amount: bigint,
  opts?: DelegateSplOptions,
): Promise<Instruction[]> {
  const payer = opts?.payer ?? owner;
  const validator = opts?.validator;
  const initIfMissing = opts?.initIfMissing ?? true;
  const initVaultIfMissing = opts?.initVaultIfMissing ?? false;
  const isPrivate = opts?.private ?? false;

  const instructions: Instruction[] = [];

  const [ephemeralAta] = await deriveEphemeralAta(owner, mint);
  const [vault] = await deriveVault(mint);
  const [vaultEphemeralAta] = await deriveEphemeralAta(vault, mint);
  const vaultAta = await deriveVaultAta(mint, vault);
  const ownerAta = await getAssociatedTokenAddressSync(mint, owner);

  if (initIfMissing) {
    instructions.push(initEphemeralAtaIx(ephemeralAta, owner, mint, payer));
  }

  if (initVaultIfMissing) {
    instructions.push(
      initVaultIx(vault, mint, payer, vaultEphemeralAta, vaultAta),
      initVaultAtaIx(payer, vaultAta, vault, mint),
      await delegateIx(payer, vaultEphemeralAta, validator),
    );
  }

  instructions.push(
    transferToVaultIx(
      ephemeralAta,
      vault,
      mint,
      ownerAta,
      vaultAta,
      owner,
      amount,
    ),
  );

  if (isPrivate) {
    instructions.push(await createEataPermissionIx(ephemeralAta, payer));
  }

  instructions.push(await delegateIx(payer, ephemeralAta, validator));

  return instructions;
}

async function buildIdempotentDelegateSplInstructions(
  owner: Address,
  mint: Address,
  amount: bigint,
  opts?: DelegateSplOptions,
): Promise<Instruction[]> {
  const payer = opts?.payer ?? owner;
  const validator = opts?.validator;
  const initIfMissing = opts?.initIfMissing ?? true;
  const initVaultIfMissing = opts?.initVaultIfMissing ?? false;
  const initAtasIfMissing = opts?.initAtasIfMissing ?? false;
  const isPrivate = opts?.private ?? false;

  const shuttleId = opts?.shuttleId ?? randomShuttleId();

  const instructions: Instruction[] = [];

  const [ephemeralAta] = await deriveEphemeralAta(owner, mint);
  const [vault] = await deriveVault(mint);
  const [vaultEphemeralAta] = await deriveEphemeralAta(vault, mint);
  const vaultAta = await deriveVaultAta(mint, vault);
  const ownerAta = await getAssociatedTokenAddressSync(mint, owner);

  const [shuttleEphemeralAta] = await deriveShuttleEphemeralAta(
    owner,
    mint,
    shuttleId,
  );
  const [shuttleAta] = await deriveShuttleAta(shuttleEphemeralAta, mint);
  const shuttleWalletAta = await deriveShuttleWalletAta(
    mint,
    shuttleEphemeralAta,
  );

  if (initVaultIfMissing) {
    instructions.push(
      initVaultIx(vault, mint, payer, vaultEphemeralAta, vaultAta),
      initVaultAtaIx(payer, vaultAta, vault, mint),
      await delegateIx(payer, vaultEphemeralAta, validator),
    );
  }

  if (initAtasIfMissing) {
    instructions.push(initVaultAtaIx(payer, ownerAta, owner, mint));
  }

  if (initIfMissing) {
    instructions.push(initEphemeralAtaIx(ephemeralAta, owner, mint, payer));
  }

  if (isPrivate) {
    instructions.push(await createEataPermissionIx(ephemeralAta, payer));
  }

  instructions.push(await delegateIx(payer, ephemeralAta, validator));

  if (amount > 0n) {
    instructions.push(
      await setupAndDelegateShuttleEphemeralAtaWithMergeIx(
        payer,
        shuttleEphemeralAta,
        shuttleAta,
        owner,
        ownerAta,
        ownerAta,
        shuttleWalletAta,
        mint,
        shuttleId,
        amount,
        validator,
      ),
    );
  } else {
    instructions.push(
      initShuttleEphemeralAtaIx(
        payer,
        shuttleEphemeralAta,
        shuttleAta,
        shuttleWalletAta,
        owner,
        mint,
        shuttleId,
      ),
      await delegateShuttleEphemeralAtaIx(
        payer,
        shuttleEphemeralAta,
        shuttleAta,
        validator,
      ),
    );
  }

  return instructions;
}

/**
 * High-level method to delegate SPL tokens.
 *
 * By default this uses the setup+deposit+delegate idempotent shuttle flow.
 * Set `idempotent: false` to use the legacy direct delegation flow.
 * Set `private: true` to add `createEataPermissionIx`.
 */
export async function delegateSpl(
  owner: Address,
  mint: Address,
  amount: bigint,
  opts?: DelegateSplOptions,
): Promise<Instruction[]> {
  if (opts?.idempotent === false) {
    return buildDelegateSplInstructions(owner, mint, amount, opts);
  }

  return buildIdempotentDelegateSplInstructions(owner, mint, amount, opts);
}

export async function delegateSplWithPrivateTransfer(
  owner: Address,
  mint: Address,
  amount: bigint,
  opts?: DelegateSplWithPrivateTransferOptions,
): Promise<Instruction[]> {
  const payer = opts?.payer ?? owner;
  const validator = opts?.validator;
  const initIfMissing = opts?.initIfMissing ?? true;
  const initVaultIfMissing = opts?.initVaultIfMissing ?? false;
  const initAtasIfMissing = opts?.initAtasIfMissing ?? false;
  const initTransferQueueIfMissing = opts?.initTransferQueueIfMissing ?? false;
  const shuttleId = opts?.shuttleId ?? randomShuttleId();
  const minDelayMs = opts?.minDelayMs ?? 0n;
  const maxDelayMs = opts?.maxDelayMs ?? minDelayMs;
  const split = opts?.split ?? 1;

  if (validator == null) {
    throw new Error("validator is required for encrypted private transfers");
  }

  const instructions: Instruction[] = [];

  const [ephemeralAta] = await deriveEphemeralAta(owner, mint);
  const [vault] = await deriveVault(mint);
  const [vaultEphemeralAta] = await deriveEphemeralAta(vault, mint);
  const vaultAta = await deriveVaultAta(mint, vault);
  const [queue] = await deriveTransferQueue(mint, validator);
  const ownerAta = await getAssociatedTokenAddressSync(mint, owner);
  const [shuttleEphemeralAta] = await deriveShuttleEphemeralAta(
    owner,
    mint,
    shuttleId,
  );
  const [shuttleAta] = await deriveShuttleAta(shuttleEphemeralAta, mint);
  const shuttleWalletAta = await deriveShuttleWalletAta(
    mint,
    shuttleEphemeralAta,
  );

  if (initVaultIfMissing) {
    instructions.push(
      initVaultIx(vault, mint, payer, vaultEphemeralAta, vaultAta),
      initVaultAtaIx(payer, vaultAta, vault, mint),
      await delegateIx(payer, vaultEphemeralAta, validator),
    );
  }

  if (initTransferQueueIfMissing) {
    instructions.push(await initTransferQueueIx(payer, queue, mint, validator));
  }

  if (initAtasIfMissing) {
    instructions.push(initVaultAtaIx(payer, ownerAta, owner, mint));
  }

  if (initIfMissing) {
    instructions.push(initEphemeralAtaIx(ephemeralAta, owner, mint, payer));
  }

  instructions.push(
    await delegateIx(payer, ephemeralAta, validator),
    await depositAndDelegateShuttleEphemeralAtaWithMergeAndPrivateTransferIx(
      payer,
      shuttleEphemeralAta,
      shuttleAta,
      owner,
      ownerAta,
      owner,
      shuttleWalletAta,
      mint,
      shuttleId,
      amount,
      minDelayMs,
      maxDelayMs,
      split,
      validator,
    ),
  );

  return instructions;
}

export async function transferSpl(
  from: Address,
  to: Address,
  mint: Address,
  amount: bigint,
  opts: TransferSplOptions,
): Promise<Instruction[]> {
  const payer = opts.payer ?? from;
  const validator = opts.validator;
  const initIfMissing = opts.initIfMissing ?? false;
  const initAtasIfMissing = opts.initAtasIfMissing ?? false;
  const initVaultIfMissing = opts.initVaultIfMissing ?? false;
  const shuttleId = opts.shuttleId ?? randomShuttleId();
  const minDelayMs = opts.privateTransfer?.minDelayMs ?? 0n;
  const maxDelayMs = opts.privateTransfer?.maxDelayMs ?? minDelayMs;
  const split = opts.privateTransfer?.split ?? 1;

  const fromAta = await getAssociatedTokenAddressSync(mint, from);
  const toAta = await getAssociatedTokenAddressSync(mint, to);

  if (opts.fromBalance === "ephemeral") {
    switch (opts.visibility) {
      case "private":
        if (opts.toBalance === "base") {
          if (validator == null) {
            throw new Error(
              "validator is required for private ephemeral-to-base transfers",
            );
          }

          const [queue] = await deriveTransferQueue(mint, validator);
          const [vault] = await deriveVault(mint);
          const vaultAta = await deriveVaultAta(mint, vault);

          return [
            depositAndQueueTransferIx(
              queue,
              vault,
              mint,
              fromAta,
              vaultAta,
              toAta,
              from,
              amount,
              minDelayMs,
              maxDelayMs,
              split,
            ),
          ];
        }

        if (opts.toBalance === "ephemeral") {
          return [createTransferInstruction(fromAta, toAta, from, amount)];
        }

        break;

      case "public":
        if (opts.toBalance === "ephemeral") {
          return [createTransferInstruction(fromAta, toAta, from, amount)];
        }

        break;
    }
  }

  const instructions: Instruction[] = [];

  if (initVaultIfMissing) {
    const [vault] = await deriveVault(mint);
    const [vaultEphemeralAta] = await deriveEphemeralAta(vault, mint);
    const vaultAta = await deriveVaultAta(mint, vault);

    instructions.push(
      initVaultIx(vault, mint, payer, vaultEphemeralAta, vaultAta),
      initVaultAtaIx(payer, vaultAta, vault, mint),
      await delegateIx(payer, vaultEphemeralAta, validator),
    );
  }

  if (opts.fromBalance === "base" && initAtasIfMissing) {
    instructions.push(initVaultAtaIx(payer, fromAta, from, mint));
  }

  switch (opts.visibility) {
    case "private":
      if (opts.fromBalance === "base" && opts.toBalance === "base") {
        const [shuttleEphemeralAta] = await deriveShuttleEphemeralAta(
          from,
          mint,
          shuttleId,
        );
        const [shuttleAta] = await deriveShuttleAta(shuttleEphemeralAta, mint);
        const shuttleWalletAta = await deriveShuttleWalletAta(
          mint,
          shuttleEphemeralAta,
        );

        return [
          ...instructions,
          await depositAndDelegateShuttleEphemeralAtaWithMergeAndPrivateTransferIx(
            payer,
            shuttleEphemeralAta,
            shuttleAta,
            from,
            fromAta,
            to,
            shuttleWalletAta,
            mint,
            shuttleId,
            amount,
            minDelayMs,
            maxDelayMs,
            split,
            validator,
          ),
        ];
      }

      if (opts.fromBalance === "base" && opts.toBalance === "ephemeral") {
        if (initIfMissing) {
          const [toEphemeralAta] = await deriveEphemeralAta(to, mint);

          instructions.push(
            initVaultAtaIx(payer, toAta, to, mint),
            initEphemeralAtaIx(toEphemeralAta, to, mint, payer),
            await delegateIx(payer, toEphemeralAta, validator),
          );
        }

        const [shuttleEphemeralAta] = await deriveShuttleEphemeralAta(
          from,
          mint,
          shuttleId,
        );
        const [shuttleAta] = await deriveShuttleAta(shuttleEphemeralAta, mint);
        const shuttleWalletAta = await deriveShuttleWalletAta(
          mint,
          shuttleEphemeralAta,
        );

        return [
          ...instructions,
          await setupAndDelegateShuttleEphemeralAtaWithMergeIx(
            payer,
            shuttleEphemeralAta,
            shuttleAta,
            from,
            fromAta,
            toAta,
            shuttleWalletAta,
            mint,
            shuttleId,
            amount,
            validator,
          ),
        ];
      }

      break;

    case "public":
      if (opts.fromBalance === "base" && opts.toBalance === "base") {
        return [
          ...instructions,
          createTransferInstruction(fromAta, toAta, from, amount),
        ];
      }

      break;
  }

  throw new Error(
    `transferSpl route not implemented: visibility=${opts.visibility}, fromBalance=${opts.fromBalance}, toBalance=${opts.toBalance}`,
  );
}

async function buildIdempotentWithdrawSplInstructions(
  owner: Address,
  mint: Address,
  amount: bigint,
  opts?: WithdrawSplOptions,
): Promise<Instruction[]> {
  const payer = opts?.payer ?? owner;
  const validator = opts?.validator;
  const initIfMissing = opts?.initIfMissing ?? true;
  const initAtasIfMissing = opts?.initAtasIfMissing ?? false;
  const shuttleId = opts?.shuttleId ?? randomShuttleId();

  const instructions: Instruction[] = [];

  const [ephemeralAta] = await deriveEphemeralAta(owner, mint);
  const ownerAta = await getAssociatedTokenAddressSync(mint, owner);
  const [shuttleEphemeralAta] = await deriveShuttleEphemeralAta(
    owner,
    mint,
    shuttleId,
  );
  const [shuttleAta] = await deriveShuttleAta(shuttleEphemeralAta, mint);
  const shuttleWalletAta = await deriveShuttleWalletAta(
    mint,
    shuttleEphemeralAta,
  );

  if (initAtasIfMissing) {
    instructions.push(initVaultAtaIx(payer, ownerAta, owner, mint));
  }

  if (initIfMissing) {
    instructions.push(initEphemeralAtaIx(ephemeralAta, owner, mint, payer));
  }

  instructions.push(
    await delegateIx(payer, ephemeralAta, validator),
    await withdrawThroughDelegatedShuttleWithMergeIx(
      payer,
      shuttleEphemeralAta,
      shuttleAta,
      owner,
      ownerAta,
      shuttleWalletAta,
      mint,
      shuttleId,
      amount,
      validator,
    ),
  );

  return instructions;
}

export async function withdrawSpl(
  owner: Address,
  mint: Address,
  amount: bigint,
  opts?: WithdrawSplOptions,
): Promise<Instruction[]> {
  if (opts?.idempotent === false) {
    const instructions: Instruction[] = [];
    if (opts?.initAtasIfMissing === true) {
      const payer = opts.payer ?? owner;
      const ownerAta = await getAssociatedTokenAddressSync(mint, owner);
      instructions.push(initVaultAtaIx(payer, ownerAta, owner, mint));
    }
    instructions.push(await withdrawSplIx(owner, mint, amount));
    return instructions;
  }

  return buildIdempotentWithdrawSplInstructions(owner, mint, amount, opts);
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

function encodeAmountInstructionData(
  discriminator: number,
  amount: bigint,
  ...suffix: number[]
): Uint8Array {
  const amountBytes = U64_ENCODER.encode(amount);
  const data = new Uint8Array(1 + amountBytes.length + suffix.length);
  data[0] = discriminator;
  data.set(amountBytes, 1);
  if (suffix.length > 0) {
    data.set(suffix, 1 + amountBytes.length);
  }
  return data;
}
