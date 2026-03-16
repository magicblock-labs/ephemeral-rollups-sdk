import {
  PublicKey,
  TransactionInstruction,
  SystemProgram,
  AccountInfo,
} from "@solana/web3.js";

import {
  DELEGATION_PROGRAM_ID,
  MAGIC_CONTEXT_ID,
  MAGIC_PROGRAM_ID,
  PERMISSION_PROGRAM_ID,
  EPHEMERAL_SPL_TOKEN_PROGRAM_ID,
} from "../../constants.js";
import {
  delegateBufferPdaFromDelegatedAccountAndOwnerProgram,
  delegationRecordPdaFromDelegatedAccount,
  delegationMetadataPdaFromDelegatedAccount,
  permissionPdaFromAccount,
} from "../../pda.js";

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
// Accounts
// ---------------------------------------------------------------------------

/**
 * Ephemeral ATA
 */
export interface EphemeralAta {
  /// The owner of the eata
  owner: PublicKey;
  /// The mint associated with this account
  mint: PublicKey;
  /// The amount of tokens this account holds.
  amount: bigint;
}

/**
 * Decode ephemeral ATA
 * @param info - The account info
 * @returns The decoded ephemeral ATA
 */
export function decodeEphemeralAta(info: AccountInfo<Buffer>): EphemeralAta {
  if (info.data.length < 72) {
    throw new Error("Invalid EphemeralAta account data length");
  }
  const owner = new PublicKey(info.data.subarray(0, 32));
  const mint = new PublicKey(info.data.subarray(32, 64));
  const amount = BigInt(info.data.readBigUInt64LE(64));
  return {
    owner,
    mint,
    amount,
  };
}

/**
 * Encode ephemeral ATA to bytes
 * @param eata - The ephemeral ATA to encode
 * @returns The encoded bytes
 */
export function encodeEphemeralAta(eata: EphemeralAta): Buffer {
  const buffer = Buffer.alloc(72);
  buffer.set(eata.owner.toBytes(), 0);
  buffer.set(eata.mint.toBytes(), 32);
  buffer.writeBigUInt64LE(eata.amount, 64);
  return buffer;
}

/**
 * Global Vault
 */
export interface GlobalVault {
  /// The mint associated with this vault
  mint: PublicKey;
}

/**
 * Decode global vault
 * @param info - The account info
 * @returns The decoded global vault
 */
export function decodeGlobalVault(info: AccountInfo<Buffer>): GlobalVault {
  if (info.data.length < 32) {
    throw new Error("Invalid GlobalVault account data length");
  }
  const mint = new PublicKey(info.data.subarray(0, 32));
  return { mint };
}

/**
 * Encode global vault to bytes
 * @param vault - The global vault to encode
 * @returns The encoded bytes
 */
export function encodeGlobalVault(vault: GlobalVault): Buffer {
  const buffer = Buffer.alloc(32);
  buffer.set(vault.mint.toBytes(), 0);
  return buffer;
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
  owner: PublicKey,
  mint: PublicKey,
): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [owner.toBuffer(), mint.toBuffer()],
    EPHEMERAL_SPL_TOKEN_PROGRAM_ID,
  );
}

/**
 * Derive vault
 * @param mint - The mint account
 * @returns The vault account and bump
 */
export function deriveVault(mint: PublicKey): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [mint.toBuffer()],
    EPHEMERAL_SPL_TOKEN_PROGRAM_ID,
  );
}

/**
 * Derive vault ATA
 * @param mint - The mint account
 * @param vault - The vault account
 * @returns The vault ATA account
 */
export function deriveVaultAta(mint: PublicKey, vault: PublicKey): PublicKey {
  return getAssociatedTokenAddressSync(mint, vault, true);
}

/**
 * Derive shuttle metadata PDA
 * @param owner - The owner account
 * @param mint - The mint account
 * @param shuttleId - The shuttle id (u32)
 * @returns The shuttle metadata account and bump
 */
export function deriveShuttleEphemeralAta(
  owner: PublicKey,
  mint: PublicKey,
  shuttleId: number,
): [PublicKey, number] {
  if (
    !Number.isInteger(shuttleId) ||
    shuttleId < 0 ||
    shuttleId > 0xffff_ffff
  ) {
    throw new Error("shuttleId must fit in u32");
  }

  const shuttleIdSeed = Buffer.alloc(4);
  shuttleIdSeed.writeUInt32LE(shuttleId, 0);

  return PublicKey.findProgramAddressSync(
    [owner.toBuffer(), mint.toBuffer(), shuttleIdSeed],
    EPHEMERAL_SPL_TOKEN_PROGRAM_ID,
  );
}

/**
 * Derive shuttle EATA PDA
 * @param shuttleEphemeralAta - The shuttle metadata account
 * @param mint - The mint account
 * @returns The shuttle EATA account and bump
 */
export function deriveShuttleAta(
  shuttleEphemeralAta: PublicKey,
  mint: PublicKey,
): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [shuttleEphemeralAta.toBuffer(), mint.toBuffer()],
    EPHEMERAL_SPL_TOKEN_PROGRAM_ID,
  );
}

/**
 * Derive shuttle wallet ATA
 * @param mint - The mint account
 * @param shuttleEphemeralAta - The shuttle metadata account
 * @returns The shuttle wallet ATA account
 */
export function deriveShuttleWalletAta(
  mint: PublicKey,
  shuttleEphemeralAta: PublicKey,
): PublicKey {
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
  ephemeralAta: PublicKey,
  owner: PublicKey,
  mint: PublicKey,
  payer: PublicKey,
  bump: number,
): TransactionInstruction {
  return new TransactionInstruction({
    programId: EPHEMERAL_SPL_TOKEN_PROGRAM_ID,
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

/**
 * Init vault ATA
 * @param payer - The payer account
 * @param vaultAta - The vault ATA account
 * @param vault - The vault account
 * @param mint - The mint account
 * @returns The init vault ATA instruction
 */
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

/**
 * Init vault account
 * @param vault - The vault account
 * @param mint - The mint account
 * @param payer - The payer account
 * @param bump - The bump
 * @returns The init vault account instruction
 */
export function initVaultIx(
  vault: PublicKey,
  mint: PublicKey,
  payer: PublicKey,
  bump: number,
): TransactionInstruction {
  const [vaultEphemeralAta] = deriveEphemeralAta(vault, mint);
  const vaultAta = deriveVaultAta(mint, vault);
  return new TransactionInstruction({
    programId: EPHEMERAL_SPL_TOKEN_PROGRAM_ID,
    keys: [
      { pubkey: vault, isSigner: false, isWritable: true },
      { pubkey: payer, isSigner: true, isWritable: true },
      { pubkey: mint, isSigner: false, isWritable: false },
      { pubkey: vaultEphemeralAta, isSigner: false, isWritable: true },
      { pubkey: vaultAta, isSigner: false, isWritable: true },
      { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
      {
        pubkey: ASSOCIATED_TOKEN_PROGRAM_ID,
        isSigner: false,
        isWritable: false,
      },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
    ],
    data: Buffer.from([1, bump]),
  });
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
  ephemeralAta: PublicKey,
  vault: PublicKey,
  mint: PublicKey,
  sourceAta: PublicKey,
  vaultAta: PublicKey,
  owner: PublicKey,
  amount: bigint,
): TransactionInstruction {
  return new TransactionInstruction({
    programId: EPHEMERAL_SPL_TOKEN_PROGRAM_ID,
    keys: [
      { pubkey: ephemeralAta, isSigner: false, isWritable: true },
      { pubkey: vault, isSigner: false, isWritable: false },
      { pubkey: mint, isSigner: false, isWritable: false },
      { pubkey: sourceAta, isSigner: false, isWritable: true },
      { pubkey: vaultAta, isSigner: false, isWritable: true },
      { pubkey: owner, isSigner: true, isWritable: false },
      { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
    ],
    data: encodeAmountInstructionData(2, amount),
  });
}

/**
 * Deposit SPL tokens (deposit_spl_tokens / discriminator 2).
 * Alias of transferToVaultIx for explicit semantics in shuttle flows.
 */
export function depositSplTokensIx(
  ephemeralAta: PublicKey,
  vault: PublicKey,
  mint: PublicKey,
  sourceAta: PublicKey,
  vaultAta: PublicKey,
  owner: PublicKey,
  amount: bigint,
): TransactionInstruction {
  return transferToVaultIx(
    ephemeralAta,
    vault,
    mint,
    sourceAta,
    vaultAta,
    owner,
    amount,
  );
}

/**
 * Delegate instruction
 * @param payer - The payer account
 * @param ephemeralAta - The ephemeral ATA account
 * @param bump - The bump
 * @param validator - The validator account
 * @returns The delegate instruction
 */
export function delegateEphemeralAtaIx(
  payer: PublicKey,
  ephemeralAta: PublicKey,
  bump: number,
  validator?: PublicKey,
): TransactionInstruction {
  const data = validator
    ? Buffer.concat([Buffer.from([4, bump]), validator.toBuffer()])
    : Buffer.from([4, bump]);
  return new TransactionInstruction({
    programId: EPHEMERAL_SPL_TOKEN_PROGRAM_ID,
    keys: [
      { pubkey: payer, isSigner: true, isWritable: true },
      { pubkey: ephemeralAta, isSigner: false, isWritable: true },
      {
        pubkey: EPHEMERAL_SPL_TOKEN_PROGRAM_ID,
        isSigner: false,
        isWritable: false,
      },
      {
        pubkey: delegateBufferPdaFromDelegatedAccountAndOwnerProgram(
          ephemeralAta,
          EPHEMERAL_SPL_TOKEN_PROGRAM_ID,
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
  payer: PublicKey,
  shuttleEphemeralAta: PublicKey,
  shuttleAta: PublicKey,
  shuttleWalletAta: PublicKey,
  owner: PublicKey,
  mint: PublicKey,
  shuttleId: number,
  bump: number,
): TransactionInstruction {
  if (
    !Number.isInteger(shuttleId) ||
    shuttleId < 0 ||
    shuttleId > 0xffff_ffff
  ) {
    throw new Error("shuttleId must fit in u32");
  }

  const data = Buffer.alloc(6);
  data[0] = 11;
  data.writeUInt32LE(shuttleId, 1);
  data[5] = bump;

  return new TransactionInstruction({
    programId: EPHEMERAL_SPL_TOKEN_PROGRAM_ID,
    keys: [
      { pubkey: payer, isSigner: true, isWritable: true },
      { pubkey: shuttleEphemeralAta, isSigner: false, isWritable: true },
      { pubkey: shuttleAta, isSigner: false, isWritable: false },
      { pubkey: shuttleWalletAta, isSigner: false, isWritable: true },
      { pubkey: owner, isSigner: false, isWritable: false },
      { pubkey: mint, isSigner: false, isWritable: false },
      { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
      {
        pubkey: ASSOCIATED_TOKEN_PROGRAM_ID,
        isSigner: false,
        isWritable: false,
      },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
    ],
    data,
  });
}

/**
 * Delegate shuttle ephemeral ATA
 * @param payer - The payer account
 * @param shuttleEphemeralAta - The shuttle metadata account
 * @param shuttleAta - The shuttle EATA account
 * @param bump - The shuttle EATA bump
 * @param validator - Optional validator pubkey
 * @returns The delegate shuttle instruction
 */
export function delegateShuttleEphemeralAtaIx(
  payer: PublicKey,
  shuttleEphemeralAta: PublicKey,
  shuttleAta: PublicKey,
  bump: number,
  validator?: PublicKey,
): TransactionInstruction {
  const data = validator
    ? Buffer.concat([Buffer.from([13, bump]), validator.toBuffer()])
    : Buffer.from([13, bump]);

  return new TransactionInstruction({
    programId: EPHEMERAL_SPL_TOKEN_PROGRAM_ID,
    keys: [
      { pubkey: payer, isSigner: true, isWritable: true },
      { pubkey: shuttleEphemeralAta, isSigner: false, isWritable: false },
      { pubkey: shuttleAta, isSigner: false, isWritable: false },
      {
        pubkey: EPHEMERAL_SPL_TOKEN_PROGRAM_ID,
        isSigner: false,
        isWritable: false,
      },
      {
        pubkey: delegateBufferPdaFromDelegatedAccountAndOwnerProgram(
          shuttleAta,
          EPHEMERAL_SPL_TOKEN_PROGRAM_ID,
        ),
        isSigner: false,
        isWritable: true,
      },
      {
        pubkey: delegationRecordPdaFromDelegatedAccount(shuttleAta),
        isSigner: false,
        isWritable: true,
      },
      {
        pubkey: delegationMetadataPdaFromDelegatedAccount(shuttleAta),
        isSigner: false,
        isWritable: true,
      },
      { pubkey: DELEGATION_PROGRAM_ID, isSigner: false, isWritable: false },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
    ],
    data,
  });
}

/**
 * Merge shuttle wallet ATA balance into destination ATA.
 * @param owner - The shuttle owner signer
 * @param destinationAta - Destination token account
 * @param shuttleEphemeralAta - The shuttle metadata account
 * @param shuttleWalletAta - The shuttle wallet ATA account (source)
 * @param mint - The mint account
 * @returns The merge shuttle instruction
 */
export function mergeShuttleIntoAtaIx(
  owner: PublicKey,
  destinationAta: PublicKey,
  shuttleEphemeralAta: PublicKey,
  shuttleWalletAta: PublicKey,
  mint: PublicKey,
): TransactionInstruction {
  return new TransactionInstruction({
    programId: EPHEMERAL_SPL_TOKEN_PROGRAM_ID,
    keys: [
      { pubkey: owner, isSigner: true, isWritable: false },
      { pubkey: destinationAta, isSigner: false, isWritable: true },
      { pubkey: shuttleEphemeralAta, isSigner: false, isWritable: false },
      { pubkey: shuttleWalletAta, isSigner: false, isWritable: true },
      { pubkey: mint, isSigner: false, isWritable: false },
      { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
    ],
    data: Buffer.from([15]),
  });
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
  payer: PublicKey,
  shuttleEphemeralAta: PublicKey,
  shuttleAta: PublicKey,
  shuttleWalletAta: PublicKey,
  escrowIndex?: number,
): TransactionInstruction {
  const data =
    escrowIndex === undefined
      ? Buffer.from([14])
      : Buffer.from([14, escrowIndex]);

  return new TransactionInstruction({
    programId: EPHEMERAL_SPL_TOKEN_PROGRAM_ID,
    keys: [
      { pubkey: payer, isSigner: true, isWritable: true },
      { pubkey: shuttleEphemeralAta, isSigner: false, isWritable: false },
      { pubkey: shuttleAta, isSigner: false, isWritable: false },
      { pubkey: shuttleWalletAta, isSigner: false, isWritable: true },
      { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
      { pubkey: MAGIC_CONTEXT_ID, isSigner: false, isWritable: true },
      { pubkey: MAGIC_PROGRAM_ID, isSigner: false, isWritable: false },
    ],
    data,
  });
}

/**
 * Withdraw SPL tokens from vault to user destination
 * @param owner - The owner account
 * @param mint - The mint account
 * @param amount - The amount of tokens to withdraw
 * @returns The withdraw SPL tokens instruction
 */
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
    programId: EPHEMERAL_SPL_TOKEN_PROGRAM_ID,
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
    data: encodeAmountInstructionData(3, amount, vaultBump),
  });
}

/**
 * Undelegate instruction
 * @param owner - The owner account
 * @param mint - The mint account
 * @returns The undelegate instruction
 */
export function undelegateIx(
  owner: PublicKey,
  mint: PublicKey,
): TransactionInstruction {
  const userAta = getAssociatedTokenAddressSync(mint, owner);
  const [ephemeralAta] = deriveEphemeralAta(owner, mint);

  return new TransactionInstruction({
    programId: EPHEMERAL_SPL_TOKEN_PROGRAM_ID,
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

/**
 * Create EATA permission
 * @param ephemeralAta - The ephemeral ATA account
 * @param payer - The payer account
 * @param bump - The bump
 * @param flags - The flags
 * @returns The create EATA permission instruction
 */
export function createEataPermissionIx(
  ephemeralAta: PublicKey,
  payer: PublicKey,
  bump: number,
  flags: number = 0,
): TransactionInstruction {
  const permission = permissionPdaFromAccount(ephemeralAta);

  return new TransactionInstruction({
    programId: EPHEMERAL_SPL_TOKEN_PROGRAM_ID,
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

/**
 * Reset EATA permission
 * @param ephemeralAta - The ephemeral ATA account
 * @param payer - The payer account
 * @param bump - The bump
 * @param flags - The flags
 * @returns The reset EATA permission instruction
 */
export function resetEataPermissionIx(
  ephemeralAta: PublicKey,
  payer: PublicKey,
  bump: number,
  flags: number = 0,
): TransactionInstruction {
  const permission = permissionPdaFromAccount(ephemeralAta);

  return new TransactionInstruction({
    programId: EPHEMERAL_SPL_TOKEN_PROGRAM_ID,
    keys: [
      { pubkey: ephemeralAta, isSigner: false, isWritable: false },
      { pubkey: permission, isSigner: false, isWritable: true },
      { pubkey: payer, isSigner: true, isWritable: false },
      { pubkey: PERMISSION_PROGRAM_ID, isSigner: false, isWritable: false },
    ],
    data: Buffer.from([9, bump, flags]),
  });
}

/**
 * Delegate EATA permission
 * @param payer - The payer account
 * @param ephemeralAta - The ephemeral ATA account
 * @param bump - The bump
 * @param validator - The validator account
 * @returns The delegate EATA permission instruction
 */
export function delegateEataPermissionIx(
  payer: PublicKey,
  ephemeralAta: PublicKey,
  bump: number,
  validator: PublicKey,
): TransactionInstruction {
  const permission = permissionPdaFromAccount(ephemeralAta);

  return new TransactionInstruction({
    programId: EPHEMERAL_SPL_TOKEN_PROGRAM_ID,
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

/**
 * Undelegate EATA permission
 * @param owner - The owner account
 * @param ephemeralAta - The ephemeral ATA account
 * @returns The undelegate EATA permission instruction
 */
export function undelegateEataPermissionIx(
  owner: PublicKey,
  ephemeralAta: PublicKey,
): TransactionInstruction {
  const permission = permissionPdaFromAccount(ephemeralAta);

  return new TransactionInstruction({
    programId: EPHEMERAL_SPL_TOKEN_PROGRAM_ID,
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

export interface DelegateSplOptions {
  payer?: PublicKey;
  validator?: PublicKey;
  initIfMissing?: boolean;
  initVaultIfMissing?: boolean;
  initAtasIfMissing?: boolean;
  shuttleId?: number;
  escrowIndex?: number;
  idempotent?: boolean;
  private?: boolean;
}

async function buildDelegateSplInstructions(
  owner: PublicKey,
  mint: PublicKey,
  amount: bigint,
  opts?: DelegateSplOptions,
): Promise<TransactionInstruction[]> {
  const payer = opts?.payer ?? owner;
  const validator = opts?.validator;
  const initIfMissing = opts?.initIfMissing ?? true;
  const initVaultIfMissing = opts?.initVaultIfMissing ?? initIfMissing;
  const isPrivate = opts?.private ?? false;

  const instructions: TransactionInstruction[] = [];

  const [ephemeralAta, eataBump] = deriveEphemeralAta(owner, mint);
  const [vault, vaultBump] = deriveVault(mint);
  const [vaultEphemeralAta, vaultEataBump] = deriveEphemeralAta(vault, mint);
  const vaultAta = deriveVaultAta(mint, vault);
  const ownerAta = getAssociatedTokenAddressSync(mint, owner);

  if (initIfMissing) {
    instructions.push(
      initEphemeralAtaIx(ephemeralAta, owner, mint, payer, eataBump),
    );
  }

  if (initVaultIfMissing) {
    instructions.push(
      initVaultIx(vault, mint, payer, vaultBump),
      initVaultAtaIx(payer, vaultAta, vault, mint),
      delegateEphemeralAtaIx(
        payer,
        vaultEphemeralAta,
        vaultEataBump,
        validator,
      ),
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
    instructions.push(createEataPermissionIx(ephemeralAta, payer, eataBump));
  }

  instructions.push(
    delegateEphemeralAtaIx(payer, ephemeralAta, eataBump, validator),
  );

  return instructions;
}

async function buildIdempotentDelegateSplInstructions(
  owner: PublicKey,
  mint: PublicKey,
  amount: bigint,
  opts?: DelegateSplOptions,
): Promise<TransactionInstruction[]> {
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

  const instructions: TransactionInstruction[] = [];

  const [ephemeralAta, eataBump] = deriveEphemeralAta(owner, mint);
  const [vault, vaultBump] = deriveVault(mint);
  const [vaultEphemeralAta, vaultEataBump] = deriveEphemeralAta(vault, mint);
  const vaultAta = deriveVaultAta(mint, vault);
  const ownerAta = getAssociatedTokenAddressSync(mint, owner);

  const [shuttleEphemeralAta, shuttleBump] = deriveShuttleEphemeralAta(
    owner,
    mint,
    shuttleId,
  );
  const [shuttleAta, shuttleAtaBump] = deriveShuttleAta(
    shuttleEphemeralAta,
    mint,
  );
  const shuttleWalletAta = deriveShuttleWalletAta(mint, shuttleEphemeralAta);

  if (initVaultIfMissing) {
    instructions.push(
      initVaultIx(vault, mint, payer, vaultBump),
      initVaultAtaIx(payer, vaultAta, vault, mint),
      delegateEphemeralAtaIx(
        payer,
        vaultEphemeralAta,
        vaultEataBump,
        validator,
      ),
    );
  }

  if (initAtasIfMissing) {
    instructions.push(
      createAssociatedTokenAccountIdempotentInstruction(
        payer,
        ownerAta,
        owner,
        mint,
      ),
      createAssociatedTokenAccountIdempotentInstruction(
        payer,
        shuttleWalletAta,
        shuttleEphemeralAta,
        mint,
      ),
    );
  }

  instructions.push(
    initEphemeralAtaIx(ephemeralAta, owner, mint, payer, eataBump),
  );

  if (isPrivate) {
    instructions.push(createEataPermissionIx(ephemeralAta, payer, eataBump));
  }

  instructions.push(
    delegateEphemeralAtaIx(payer, ephemeralAta, eataBump, validator),
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
      depositSplTokensIx(
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
    delegateShuttleEphemeralAtaIx(
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
  owner: PublicKey,
  mint: PublicKey,
  amount: bigint,
  opts?: DelegateSplOptions,
): Promise<TransactionInstruction[]> {
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
): Buffer {
  const data = Buffer.alloc(1 + 8 + suffix.length);
  data[0] = discriminator;
  data.writeBigUInt64LE(amount, 1);
  if (suffix.length > 0) {
    data.set(suffix, 9);
  }
  return data;
}
