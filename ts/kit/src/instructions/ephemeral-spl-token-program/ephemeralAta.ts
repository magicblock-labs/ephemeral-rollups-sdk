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

// SPL Token program IDs
const U64_ENCODER = getU64Encoder();

const TOKEN_PROGRAM_ADDRESS =
  "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA" as const;
const ASSOCIATED_TOKEN_PROGRAM_ADDRESS =
  "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL" as const;

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
  allowOwnerOffCurve: boolean = false,
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
 * @param bump - The bump
 * @returns The init ephemeral ATA instruction
 */
export function initEphemeralAtaIx(
  ephemeralAta: Address,
  owner: Address,
  mint: Address,
  payer: Address,
  bump: number,
): Instruction {
  return {
    accounts: [
      { address: ephemeralAta, role: AccountRole.WRITABLE },
      { address: payer, role: AccountRole.WRITABLE_SIGNER },
      { address: owner, role: AccountRole.READONLY },
      { address: mint, role: AccountRole.READONLY },
      { address: SYSTEM_PROGRAM_ADDRESS, role: AccountRole.READONLY },
    ],
    data: new Uint8Array([0, bump]),
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
 * @param bump - The bump
 * @param vaultAta - The vault ATA account
 * @returns The init vault account instruction
 */
export async function initVaultIx(
  vault: Address,
  mint: Address,
  payer: Address,
  bump: number,
  vaultAta: Address,
): Promise<Instruction> {
  const [vaultEphemeralAta] = await deriveEphemeralAta(vault, mint);

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
    data: new Uint8Array([1, bump]),
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
  bump: number,
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
    data = new Uint8Array(34);
    data[0] = 4;
    data[1] = bump;
    const validatorBytes = encoder.encode(validator);
    data.set(validatorBytes, 2);
  } else {
    data = new Uint8Array(2);
    data[0] = 4;
    data[1] = bump;
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
  bump: number,
): Instruction {
  if (
    !Number.isInteger(shuttleId) ||
    shuttleId < 0 ||
    shuttleId > 0xffff_ffff
  ) {
    throw new Error("shuttleId must fit in u32");
  }

  const data = new Uint8Array(6);
  data[0] = 11;
  new DataView(data.buffer).setUint32(1, shuttleId, true);
  data[5] = bump;

  return {
    accounts: [
      { address: payer, role: AccountRole.WRITABLE_SIGNER },
      { address: shuttleEphemeralAta, role: AccountRole.WRITABLE },
      { address: shuttleAta, role: AccountRole.READONLY },
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
  bump: number,
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
    data = new Uint8Array(34);
    data[0] = 13;
    data[1] = bump;
    data.set(addressEncoder.encode(validator), 2);
  } else {
    data = new Uint8Array(2);
    data[0] = 13;
    data[1] = bump;
  }

  return {
    accounts: [
      { address: payer, role: AccountRole.WRITABLE_SIGNER },
      { address: shuttleEphemeralAta, role: AccountRole.READONLY },
      { address: shuttleAta, role: AccountRole.READONLY },
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
 * @param shuttleEphemeralAta - The shuttle metadata account
 * @param shuttleAta - The shuttle EATA account
 * @param shuttleWalletAta - The shuttle wallet ATA account
 * @returns The undelegate shuttle instruction
 */
export function undelegateAndCloseShuttleEphemeralAtaIx(
  payer: Address,
  shuttleEphemeralAta: Address,
  shuttleAta: Address,
  shuttleWalletAta: Address,
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
      { address: shuttleEphemeralAta, role: AccountRole.READONLY },
      { address: shuttleAta, role: AccountRole.READONLY },
      { address: shuttleWalletAta, role: AccountRole.WRITABLE },
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
  const [vault, vaultBump] = await deriveVault(mint);
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
    data: encodeAmountInstructionData(3, amount, vaultBump),
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
  bump: number,
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
    data: new Uint8Array([6, bump, flags]),
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
  bump: number,
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
    data: new Uint8Array([9, bump, flags]),
    programAddress: EPHEMERAL_SPL_TOKEN_PROGRAM_ID,
  };
}

/**
 * Delegate EATA permission
 * @param payer - The payer account
 * @param ephemeralAta - The ephemeral ATA account
 * @param bump - The bump
 * @param validator - The validator account
 * @returns The delegate EATA permission instruction
 */
export async function delegateEataPermissionIx(
  payer: Address,
  ephemeralAta: Address,
  bump: number,
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
    data: new Uint8Array([7, bump]),
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
  escrowIndex?: number;
  idempotent?: boolean;
  private?: boolean;
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

  const [ephemeralAta, eataBump] = await deriveEphemeralAta(owner, mint);
  const [vault, vaultBump] = await deriveVault(mint);
  const [vaultEphemeralAta, vaultEataBump] = await deriveEphemeralAta(
    vault,
    mint,
  );
  const vaultAta = await deriveVaultAta(mint, vault);
  const ownerAta = await getAssociatedTokenAddressSync(mint, owner);

  if (initIfMissing) {
    instructions.push(
      initEphemeralAtaIx(ephemeralAta, owner, mint, payer, eataBump),
    );
  }

  if (initVaultIfMissing) {
    instructions.push(
      await initVaultIx(vault, mint, payer, vaultBump, vaultAta),
      initVaultAtaIx(payer, vaultAta, vault, mint),
      await delegateIx(payer, vaultEphemeralAta, vaultEataBump, validator),
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
    instructions.push(
      await createEataPermissionIx(ephemeralAta, payer, eataBump),
    );
  }

  instructions.push(await delegateIx(payer, ephemeralAta, eataBump, validator));

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
  const initVaultIfMissing = opts?.initVaultIfMissing ?? false;
  const initAtasIfMissing = opts?.initAtasIfMissing ?? false;
  const isPrivate = opts?.private ?? false;

  const randomShuttleId = (): number => {
    const cryptoObj = (globalThis as any)?.crypto;
    if (cryptoObj?.getRandomValues !== undefined) {
      const buf = new Uint32Array(1);
      cryptoObj.getRandomValues(buf);
      return buf[0];
    }
    return Math.floor(Math.random() * 0x1_0000_0000);
  };

  const shuttleId = opts?.shuttleId ?? randomShuttleId();

  const instructions: Instruction[] = [];

  const [ephemeralAta, eataBump] = await deriveEphemeralAta(owner, mint);
  const [vault, vaultBump] = await deriveVault(mint);
  const [vaultEphemeralAta, vaultEataBump] = await deriveEphemeralAta(
    vault,
    mint,
  );
  const vaultAta = await deriveVaultAta(mint, vault);
  const ownerAta = await getAssociatedTokenAddressSync(mint, owner);

  const [shuttleEphemeralAta, shuttleBump] = await deriveShuttleEphemeralAta(
    owner,
    mint,
    shuttleId,
  );
  const [shuttleAta, shuttleAtaBump] = await deriveShuttleAta(
    shuttleEphemeralAta,
    mint,
  );
  const shuttleWalletAta = await deriveShuttleWalletAta(
    mint,
    shuttleEphemeralAta,
  );

  if (initVaultIfMissing) {
    instructions.push(
      await initVaultIx(vault, mint, payer, vaultBump, vaultAta),
      initVaultAtaIx(payer, vaultAta, vault, mint),
      await delegateIx(payer, vaultEphemeralAta, vaultEataBump, validator),
    );
  }

  if (initAtasIfMissing) {
    instructions.push(
      initVaultAtaIx(payer, ownerAta, owner, mint),
      initVaultAtaIx(payer, shuttleWalletAta, shuttleEphemeralAta, mint),
    );
  }

  instructions.push(
    initEphemeralAtaIx(ephemeralAta, owner, mint, payer, eataBump),
  );

  if (isPrivate) {
    instructions.push(
      await createEataPermissionIx(ephemeralAta, payer, eataBump),
    );
  }

  instructions.push(
    await delegateIx(payer, ephemeralAta, eataBump, validator),
    initShuttleEphemeralAtaIx(
      payer,
      shuttleEphemeralAta,
      shuttleAta,
      shuttleWalletAta,
      owner,
      mint,
      shuttleId,
      shuttleBump,
    ),
  );

  if (amount > 0n) {
    instructions.push(
      transferToVaultIx(
        shuttleAta,
        vault,
        mint,
        ownerAta,
        vaultAta,
        owner,
        amount,
      ),
    );
  }

  instructions.push(
    await delegateShuttleEphemeralAtaIx(
      payer,
      shuttleEphemeralAta,
      shuttleAta,
      shuttleAtaBump,
      validator,
    ),
  );

  return instructions;
}

/**
 * High-level method to delegate SPL tokens.
 *
 * By default this uses the idempotent shuttle flow. Set `idempotent: false`
 * to use the legacy direct delegation flow. Set `private: true` to add
 * `createEataPermissionIx`.
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
