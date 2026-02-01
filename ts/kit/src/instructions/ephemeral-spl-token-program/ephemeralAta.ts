import {
  Address,
  Instruction,
  AccountRole,
  getAddressEncoder,
  address,
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

// Default validator for private delegation
const DEFAULT_PRIVATE_VALIDATOR = address(
  "FnE6VJT5QNZdedZPnCoLsARgBwoE6DeJNjBs2H1gySXA",
);

// SPL Token program IDs
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
 * @returns The init vault account instruction
 */
export function initVaultIx(
  vault: Address,
  mint: Address,
  payer: Address,
  bump: number,
): Instruction {
  return {
    accounts: [
      { address: vault, role: AccountRole.WRITABLE },
      { address: payer, role: AccountRole.WRITABLE_SIGNER },
      { address: mint, role: AccountRole.READONLY },
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
    data: new Uint8Array([2, ...u64le(amount)]),
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
    data: new Uint8Array([3, ...u64le(amount), vaultBump]),
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

/**
 * High-level method to delegate SPL tokens
 * @param owner - The owner account
 * @param mint - The mint account
 * @param amount - The amount of tokens to delegate
 * @param opts - The options
 * @returns The instructions to delegate SPL tokens
 */
export async function delegateSpl(
  owner: Address,
  mint: Address,
  amount: bigint,
  opts?: {
    payer?: Address;
    validator?: Address;
    initIfMissing?: boolean;
  },
): Promise<Instruction[]> {
  const payer = opts?.payer ?? owner;
  const validator = opts?.validator;
  const initIfMissing = opts?.initIfMissing ?? true;

  const instructions: Instruction[] = [];

  const [ephemeralAta, eataBump] = await deriveEphemeralAta(owner, mint);
  const [vault, vaultBump] = await deriveVault(mint);
  const vaultAta = await deriveVaultAta(mint, vault);

  const ownerAta = await getAssociatedTokenAddressSync(mint, owner);

  if (initIfMissing) {
    instructions.push(
      initEphemeralAtaIx(ephemeralAta, owner, mint, payer, eataBump),
      initVaultAtaIx(payer, vaultAta, vault, mint),
      initVaultIx(vault, mint, payer, vaultBump),
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

  instructions.push(await delegateIx(payer, ephemeralAta, eataBump, validator));

  return instructions;
}

/**
 * High-level method to delegate private SPL tokens
 * @param owner - The owner account
 * @param mint - The mint account
 * @param amount - The amount of tokens to delegate
 * @param opts - The options
 * @returns The instructions to delegate private SPL tokens
 */
export async function delegatePrivateSpl(
  owner: Address,
  mint: Address,
  amount: bigint,
  opts?: {
    payer?: Address;
    validator?: Address;
    initIfMissing?: boolean;
    permissionFlags?: number;
    delegatePermission?: boolean;
  },
): Promise<Instruction[]> {
  const payer = opts?.payer ?? owner;
  const validator = opts?.validator ?? DEFAULT_PRIVATE_VALIDATOR;
  const initIfMissing = opts?.initIfMissing ?? true;
  const permissionFlags = opts?.permissionFlags ?? 0;
  const delegatePermission = opts?.delegatePermission ?? false;

  const instructions: Instruction[] = [];

  const [ephemeralAta, eataBump] = await deriveEphemeralAta(owner, mint);
  const [vault, vaultBump] = await deriveVault(mint);
  const vaultAta = await deriveVaultAta(mint, vault);

  const ownerAta = await getAssociatedTokenAddressSync(mint, owner);

  if (initIfMissing) {
    instructions.push(
      initEphemeralAtaIx(ephemeralAta, owner, mint, payer, eataBump),
      initVaultAtaIx(payer, vaultAta, vault, mint),
      initVaultIx(vault, mint, payer, vaultBump),
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

  instructions.push(await delegateIx(payer, ephemeralAta, eataBump, validator));

  // Create the EATA permission
  instructions.push(
    await createEataPermissionIx(
      ephemeralAta,
      payer,
      eataBump,
      permissionFlags,
    ),
  );

  // Optionally delegate the permission
  if (delegatePermission) {
    instructions.push(
      await delegateEataPermissionIx(payer, ephemeralAta, eataBump, validator),
    );
  }

  return instructions;
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

function u64le(n: bigint): number[] {
  if (n < 0n || n > 0xffff_ffff_ffff_ffffn) {
    throw new Error("amount out of range for u64");
  }
  const bytes = new Array<number>(8).fill(0);
  let x = n;
  for (let i = 0; i < 8; i++) {
    bytes[i] = Number(x & 0xffn);
    x >>= 8n;
  }
  return bytes; // little-endian
}
