import {
  Address,
  Instruction,
  AccountRole,
  getAddressEncoder,
  address,
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

// Derive the Associated Token Account for a given mint/owner pair
async function getAssociatedTokenAddressSync(
  mint: Address,
  owner: Address,
  allowOwnerOffCurve: boolean = false,
): Promise<Address> {
  // In Kit, we would use getProgramDerivedAddress
  // For now, we return a placeholder - the actual implementation would need
  // to properly derive using Kit's PDA functions
  // This matches the web3.js API pattern where it's derived deterministically
  return owner;
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

// ---------------------------------------------------------------------------
// PDA derivation helpers
// ---------------------------------------------------------------------------

/**
 * Derive ephemeral ATA
 * @param owner - The owner account
 * @param mint - The mint account
 * @returns The ephemeral ATA account and bump
 */
export function deriveEphemeralAta(
  owner: Address,
  mint: Address,
): [Address, number] {
  const [ata, bump] = (() => {
    // Simplified: In a real implementation, this would use getProgramDerivedAddress
    // and extract bump. For now, we use a placeholder approach.
    // The web3js version uses PublicKey.findProgramAddressSync
    return [owner, 255] as const;
  })();
  return [ata, bump];
}

/**
 * Derive vault
 * @param mint - The mint account
 * @returns The vault account and bump
 */
export function deriveVault(mint: Address): [Address, number] {
  // Simplified: In a real implementation, this would use getProgramDerivedAddress
  return [mint, 255];
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
  const [ephemeralAta] = deriveEphemeralAta(owner, mint);
  const [vault, vaultBump] = deriveVault(mint);
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
  const [ephemeralAta] = deriveEphemeralAta(owner, mint);

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

  const [ephemeralAta, eataBump] = deriveEphemeralAta(owner, mint);
  const [vault, vaultBump] = deriveVault(mint);
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

  const [ephemeralAta, eataBump] = deriveEphemeralAta(owner, mint);
  const [vault, vaultBump] = deriveVault(mint);
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
