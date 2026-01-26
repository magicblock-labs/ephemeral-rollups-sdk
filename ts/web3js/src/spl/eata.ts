import {
  PublicKey,
  TransactionInstruction,
  SystemProgram,
} from "@solana/web3.js";

import {
  DEFAULT_PRIVATE_VALIDATOR,
  DEFAULT_VALIDATOR,
  DELEGATION_PROGRAM_ID,
  MAGIC_CONTEXT_ID,
  MAGIC_PROGRAM_ID,
  PERMISSION_PROGRAM_ID,
} from "../constants.js";
import {
  delegateBufferPdaFromDelegatedAccountAndOwnerProgram,
  delegationRecordPdaFromDelegatedAccount,
  delegationMetadataPdaFromDelegatedAccount,
  permissionPdaFromAccount,
} from "../pda.js";
// Minimal SPL Token helpers (vendored) to avoid importing @solana/spl-token.
// This prevents bundlers from pulling transitive deps like spl-token-group and
// also avoids package.exports issues when targeting browsers.

// SPL Token program IDs
const TOKEN_PROGRAM_ID = new PublicKey(
  "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
);
const ASSOCIATED_TOKEN_PROGRAM_ID = new PublicKey(
  "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL",
);

// Derive the Associated Token Account for a given mint/owner pair. Mirrors the
// behavior of @solana/spl-token's getAssociatedTokenAddressSync.
function getAssociatedTokenAddressSync(
  mint: PublicKey,
  owner: PublicKey,
  allowOwnerOffCurve: boolean = false,
  programId: PublicKey = TOKEN_PROGRAM_ID,
  associatedTokenProgramId: PublicKey = ASSOCIATED_TOKEN_PROGRAM_ID,
): PublicKey {
  // If the owner is not on curve and off-curve owners are not allowed, throw.
  // Note: Pass allowOwnerOffCurve=true when deriving ATAs for PDA owners (e.g., vaults).
  // For regular wallet owners, the default false is used.
  if (!allowOwnerOffCurve && !PublicKey.isOnCurve(owner)) {
    throw new Error("Owner public key is off-curve");
  }

  const [ata] = PublicKey.findProgramAddressSync(
    [owner.toBuffer(), programId.toBuffer(), mint.toBuffer()],
    associatedTokenProgramId,
  );
  return ata;
}

// Build an idempotent ATA create instruction. Mirrors
// @solana/spl-token's createAssociatedTokenAccountIdempotentInstruction.
function createAssociatedTokenAccountIdempotentInstruction(
  payer: PublicKey,
  associatedToken: PublicKey,
  owner: PublicKey,
  mint: PublicKey,
  programId: PublicKey = TOKEN_PROGRAM_ID,
  associatedTokenProgramId: PublicKey = ASSOCIATED_TOKEN_PROGRAM_ID,
): TransactionInstruction {
  // Instruction index 1 = CreateIdempotent
  const data = Buffer.from([1]);
  return new TransactionInstruction({
    programId: associatedTokenProgramId,
    keys: [
      { pubkey: payer, isSigner: true, isWritable: true },
      { pubkey: associatedToken, isSigner: false, isWritable: true },
      { pubkey: owner, isSigner: false, isWritable: false },
      { pubkey: mint, isSigner: false, isWritable: false },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
      { pubkey: programId, isSigner: false, isWritable: false },
    ],
    data,
  });
}

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

export const EATA_PROGRAM_ID = new PublicKey(
  "SPLxh1LVZzEkX99H6rqYizhytLWPZVV296zyYDPagv2",
);

// ---------------------------------------------------------------------------
// PDA derivation helpers
// ---------------------------------------------------------------------------

export function deriveEphemeralAta(
  owner: PublicKey,
  mint: PublicKey,
): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [owner.toBuffer(), mint.toBuffer()],
    EATA_PROGRAM_ID,
  );
}

export function deriveVault(mint: PublicKey): [PublicKey, number] {
  return PublicKey.findProgramAddressSync([mint.toBuffer()], EATA_PROGRAM_ID);
}

export function deriveVaultAta(mint: PublicKey, vault: PublicKey): PublicKey {
  return getAssociatedTokenAddressSync(mint, vault, true);
}

// ---------------------------------------------------------------------------
// Instruction builders
// ---------------------------------------------------------------------------

// Init ephemeral ATA
export function initEphemeralAtaIx(
  ephemeralAta: PublicKey,
  owner: PublicKey,
  mint: PublicKey,
  payer: PublicKey,
  bump: number,
): TransactionInstruction {
  return new TransactionInstruction({
    programId: EATA_PROGRAM_ID,
    keys: [
      { pubkey: ephemeralAta, isSigner: false, isWritable: true },
      { pubkey: payer, isSigner: true, isWritable: true },
      { pubkey: owner, isSigner: false, isWritable: false },
      { pubkey: mint, isSigner: false, isWritable: false },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
    ],
    data: Buffer.from([0, bump]),
  });
}

// Init vault ATA
export function initVaultAtaIx(
  payer: PublicKey,
  vaultAta: PublicKey,
  vault: PublicKey,
  mint: PublicKey,
): TransactionInstruction {
  return createAssociatedTokenAccountIdempotentInstruction(
    payer,
    vaultAta,
    vault,
    mint,
  );
}

// Init vault account
export function initVaultIx(
  vault: PublicKey,
  mint: PublicKey,
  payer: PublicKey,
  bump: number,
): TransactionInstruction {
  return new TransactionInstruction({
    programId: EATA_PROGRAM_ID,
    keys: [
      { pubkey: vault, isSigner: false, isWritable: true },
      { pubkey: payer, isSigner: true, isWritable: true },
      { pubkey: mint, isSigner: false, isWritable: false },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
    ],
    data: Buffer.from([1, bump]),
  });
}

// Transfer tokens to vault
export function transferToVaultIx(
  ephemeralAta: PublicKey,
  vault: PublicKey,
  mint: PublicKey,
  sourceAta: PublicKey,
  vaultAta: PublicKey,
  owner: PublicKey,
  amount: bigint,
): TransactionInstruction {
  return new TransactionInstruction({
    programId: EATA_PROGRAM_ID,
    keys: [
      { pubkey: ephemeralAta, isSigner: false, isWritable: true },
      { pubkey: vault, isSigner: false, isWritable: false },
      { pubkey: mint, isSigner: false, isWritable: false },
      { pubkey: sourceAta, isSigner: false, isWritable: true },
      { pubkey: vaultAta, isSigner: false, isWritable: true },
      { pubkey: owner, isSigner: true, isWritable: false },
      { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
    ],
    data: Buffer.from([2, ...u64le(amount)]),
  });
}

// Delegate instruction
export function delegateIx(
  payer: PublicKey,
  ephemeralAta: PublicKey,
  bump: number,
  validator?: PublicKey,
): TransactionInstruction {
  const data = validator
    ? Buffer.concat([Buffer.from([4, bump]), validator.toBuffer()])
    : Buffer.from([4, bump]);
  return new TransactionInstruction({
    programId: EATA_PROGRAM_ID,
    keys: [
      { pubkey: payer, isSigner: true, isWritable: true },
      { pubkey: ephemeralAta, isSigner: false, isWritable: true },
      { pubkey: EATA_PROGRAM_ID, isSigner: false, isWritable: false },
      {
        pubkey: delegateBufferPdaFromDelegatedAccountAndOwnerProgram(
          ephemeralAta,
          EATA_PROGRAM_ID,
        ),
        isSigner: false,
        isWritable: true,
      },
      {
        pubkey: delegationRecordPdaFromDelegatedAccount(ephemeralAta),
        isSigner: false,
        isWritable: true,
      },
      {
        pubkey: delegationMetadataPdaFromDelegatedAccount(ephemeralAta),
        isSigner: false,
        isWritable: true,
      },
      { pubkey: DELEGATION_PROGRAM_ID, isSigner: false, isWritable: false },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
    ],
    data,
  });
}

// Withdraw SPL tokens from vault to user destination
// Now derives all required accounts from the provided owner + mint
export function withdrawSplIx(
  owner: PublicKey,
  mint: PublicKey,
  amount: bigint,
): TransactionInstruction {
  const [ephemeralAta] = deriveEphemeralAta(owner, mint);
  const [vault, vaultBump] = deriveVault(mint);
  const vaultAta = deriveVaultAta(mint, vault);
  const userDestAta = getAssociatedTokenAddressSync(mint, owner);

  return new TransactionInstruction({
    programId: EATA_PROGRAM_ID,
    keys: [
      { pubkey: owner, isSigner: true, isWritable: false },
      { pubkey: ephemeralAta, isSigner: false, isWritable: true },
      { pubkey: vault, isSigner: false, isWritable: false },
      { pubkey: mint, isSigner: false, isWritable: false },
      { pubkey: vaultAta, isSigner: false, isWritable: true },
      { pubkey: userDestAta, isSigner: false, isWritable: true },
      { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
    ],
    // [WITHDRAW_OPCODE, amount(le u64), vault_bump]
    data: Buffer.from([3, ...u64le(amount), vaultBump]),
  });
}

// Undelegate instruction
export function undelegateIx(
  owner: PublicKey,
  mint: PublicKey,
): TransactionInstruction {
  const userAta = getAssociatedTokenAddressSync(mint, owner);
  const [ephemeralAta] = deriveEphemeralAta(owner, mint);

  return new TransactionInstruction({
    programId: EATA_PROGRAM_ID,
    keys: [
      {
        pubkey: owner,
        isSigner: true,
        isWritable: false,
      },
      {
        pubkey: userAta,
        isSigner: false,
        isWritable: true,
      },
      {
        pubkey: ephemeralAta,
        isSigner: false,
        isWritable: false,
      },
      {
        pubkey: MAGIC_CONTEXT_ID,
        isSigner: false,
        isWritable: true,
      },
      {
        pubkey: MAGIC_PROGRAM_ID,
        isSigner: false,
        isWritable: false,
      },
    ],
    data: Buffer.from([5]),
  });
}

// Create EATA permission
export function createEataPermissionIx(
  ephemeralAta: PublicKey,
  payer: PublicKey,
  bump: number,
  flags: number = 0,
): TransactionInstruction {
  const permission = permissionPdaFromAccount(ephemeralAta);

  return new TransactionInstruction({
    programId: EATA_PROGRAM_ID,
    keys: [
      { pubkey: ephemeralAta, isSigner: false, isWritable: true },
      { pubkey: permission, isSigner: false, isWritable: true },
      { pubkey: payer, isSigner: true, isWritable: true },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
      { pubkey: PERMISSION_PROGRAM_ID, isSigner: false, isWritable: false },
    ],
    data: Buffer.from([6, bump, flags]),
  });
}

// Delegate EATA permission
export function delegateEataPermissionIx(
  payer: PublicKey,
  ephemeralAta: PublicKey,
  bump: number,
  validator: PublicKey,
): TransactionInstruction {
  const permission = permissionPdaFromAccount(ephemeralAta);

  return new TransactionInstruction({
    programId: EATA_PROGRAM_ID,
    keys: [
      { pubkey: payer, isSigner: true, isWritable: true },
      { pubkey: ephemeralAta, isSigner: false, isWritable: true },
      { pubkey: PERMISSION_PROGRAM_ID, isSigner: false, isWritable: false },
      { pubkey: permission, isSigner: false, isWritable: true },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
      {
        pubkey: delegateBufferPdaFromDelegatedAccountAndOwnerProgram(
          permission,
          PERMISSION_PROGRAM_ID,
        ),
        isSigner: false,
        isWritable: true,
      },
      {
        pubkey: delegationRecordPdaFromDelegatedAccount(permission),
        isSigner: false,
        isWritable: true,
      },
      {
        pubkey: delegationMetadataPdaFromDelegatedAccount(permission),
        isSigner: false,
        isWritable: true,
      },
      { pubkey: DELEGATION_PROGRAM_ID, isSigner: false, isWritable: false },
      { pubkey: validator, isSigner: false, isWritable: false },
    ],
    data: Buffer.from([7, bump]),
  });
}

// Undelegate EATA permission
export function undelegateEataPermissionIx(
  owner: PublicKey,
  ephemeralAta: PublicKey,
): TransactionInstruction {
  const permission = permissionPdaFromAccount(ephemeralAta);

  return new TransactionInstruction({
    programId: EATA_PROGRAM_ID,
    keys: [
      { pubkey: owner, isSigner: true, isWritable: false },
      { pubkey: ephemeralAta, isSigner: false, isWritable: true },
      { pubkey: permission, isSigner: false, isWritable: true },
      { pubkey: PERMISSION_PROGRAM_ID, isSigner: false, isWritable: false },
      { pubkey: MAGIC_PROGRAM_ID, isSigner: false, isWritable: false },
      { pubkey: MAGIC_CONTEXT_ID, isSigner: false, isWritable: true },
    ],
    data: Buffer.from([8]),
  });
}

// ---------------------------------------------------------------------------
// High-level SDK methods
// ---------------------------------------------------------------------------

export async function delegateSpl(
  owner: PublicKey,
  mint: PublicKey,
  amount: bigint,
  opts?: {
    payer?: PublicKey;
    validator?: PublicKey;
    initIfMissing?: boolean;
  },
): Promise<TransactionInstruction[]> {
  const payer = opts?.payer ?? owner;
  const validator = opts?.validator; // Use the default validator authority if not specified
  const initIfMissing = opts?.initIfMissing ?? true;

  const instructions: TransactionInstruction[] = [];

  const [ephemeralAta, eataBump] = deriveEphemeralAta(owner, mint);
  const [vault, vaultBump] = deriveVault(mint);
  const vaultAta = deriveVaultAta(mint, vault);

  const ownerAta = getAssociatedTokenAddressSync(mint, owner);

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

  instructions.push(delegateIx(payer, ephemeralAta, eataBump, validator));

  return instructions;
}

export async function delegatePrivateSpl(
  owner: PublicKey,
  mint: PublicKey,
  amount: bigint,
  opts?: {
    payer?: PublicKey;
    validator?: PublicKey;
    initIfMissing?: boolean;
    permissionFlags?: number;
    delegatePermission?: boolean;
  },
): Promise<TransactionInstruction[]> {
  const payer = opts?.payer ?? owner;
  const validator = opts?.validator ?? DEFAULT_PRIVATE_VALIDATOR;
  const initIfMissing = opts?.initIfMissing ?? true;
  const permissionFlags = opts?.permissionFlags ?? 0;
  const delegatePermission = opts?.delegatePermission ?? false;

  const instructions: TransactionInstruction[] = [];

  const [ephemeralAta, eataBump] = deriveEphemeralAta(owner, mint);
  const [vault, vaultBump] = deriveVault(mint);
  const vaultAta = deriveVaultAta(mint, vault);

  const ownerAta = getAssociatedTokenAddressSync(mint, owner);

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

  instructions.push(delegateIx(payer, ephemeralAta, eataBump, validator));

  // Create the EATA permission
  instructions.push(
    createEataPermissionIx(ephemeralAta, payer, eataBump, permissionFlags),
  );

  // Optionally delegate the permission
  if (delegatePermission) {
    instructions.push(
      delegateEataPermissionIx(payer, ephemeralAta, eataBump, validator),
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
