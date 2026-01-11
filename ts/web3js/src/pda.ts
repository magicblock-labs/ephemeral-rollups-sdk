import { PublicKey } from "@solana/web3.js";

import { DELEGATION_PROGRAM_ID, PERMISSION_PROGRAM_ID } from "./constants.js";


// ============================================================================
// Delegation Program PDAs
// ============================================================================

/**
 * Derives the delegation record PDA for a given delegated account
 * @param delegatedAccount - The delegated account address
 * @returns The delegation record PDA
 */
export function delegationRecordPdaFromDelegatedAccount(
  delegatedAccount: PublicKey,
) {
  return PublicKey.findProgramAddressSync(
    [Buffer.from("delegation"), delegatedAccount.toBytes()],
    DELEGATION_PROGRAM_ID,
  )[0];
}

/**
 * Derives the delegation metadata PDA for a given delegated account
 * @param delegatedAccount - The delegated account address
 * @returns The delegation metadata PDA
 */
export function delegationMetadataPdaFromDelegatedAccount(
  delegatedAccount: PublicKey,
) {
  return PublicKey.findProgramAddressSync(
    [Buffer.from("delegation-metadata"), delegatedAccount.toBytes()],
    DELEGATION_PROGRAM_ID,
  )[0];
}

/**
 * Derives the delegate buffer PDA for a given delegated account and owner program
 * @param delegatedAccount - The delegated account address
 * @param ownerProgramId - The owner program ID
 * @returns The delegate buffer PDA
 */
export function delegateBufferPdaFromDelegatedAccountAndOwnerProgram(
  delegatedAccount: PublicKey,
  ownerProgramId: PublicKey,
) {
  return PublicKey.findProgramAddressSync(
    [Buffer.from("buffer"), delegatedAccount.toBytes()],
    ownerProgramId,
  )[0];
}

/**
 * Derives the escrow PDA from an escrow authority address
 * @param escrowAuthority - The escrow authority address
 * @param index - The index of the ephemeral balance account (0-255)
 * @returns The escrow PDA
 */
export function escrowPdaFromEscrowAuthority(
  escrowAuthority: PublicKey,
  index: number = 255,
) {
  if (index < 0 || index > 255) {
    throw new Error("Index must be between 0 and 255");
  }
  return PublicKey.findProgramAddressSync(
    [Buffer.from("balance"), escrowAuthority.toBytes(), Buffer.from([index])],
    DELEGATION_PROGRAM_ID,
  )[0];
}

/**
 * Derives the undelegate buffer PDA for a given delegated account
 * @param delegatedAccount - The delegated account address
 * @returns The undelegate buffer PDA
 */
export function undelegateBufferPdaFromDelegatedAccount(
  delegatedAccount: PublicKey,
) {
  return PublicKey.findProgramAddressSync(
    [Buffer.from("undelegate-buffer"), delegatedAccount.toBytes()],
    DELEGATION_PROGRAM_ID,
  )[0];
}

/**
 * Derives the fees vault PDA
 * @returns The fees vault PDA
 */
export function feesVaultPda() {
  return PublicKey.findProgramAddressSync(
    [Buffer.from("fees-vault")],
    DELEGATION_PROGRAM_ID,
  )[0];
}

/**
 * Derives the validator fees vault PDA for a given validator
 * @param validator - The validator address
 * @returns The validator fees vault PDA
 */
export function validatorFeesVaultPdaFromValidator(validator: PublicKey) {
  return PublicKey.findProgramAddressSync(
    [Buffer.from("v-fees-vault"), validator.toBytes()],
    DELEGATION_PROGRAM_ID,
  )[0];
}

/**
 * Derives the commit state PDA for a given delegated account
 * @param delegatedAccount - The delegated account address
 * @returns The commit state PDA
 */
export function commitStatePdaFromDelegatedAccount(
  delegatedAccount: PublicKey,
) {
  return PublicKey.findProgramAddressSync(
    [Buffer.from("state-diff"), delegatedAccount.toBytes()],
    DELEGATION_PROGRAM_ID,
  )[0];
}

/**
 * Derives the commit record PDA for a given delegated account
 * @param delegatedAccount - The delegated account address
 * @returns The commit record PDA
 */
export function commitRecordPdaFromDelegatedAccount(
  delegatedAccount: PublicKey,
) {
  return PublicKey.findProgramAddressSync(
    [Buffer.from("commit-state-record"), delegatedAccount.toBytes()],
    DELEGATION_PROGRAM_ID,
  )[0];
}

// ============================================================================
// Permission Program PDAs
// ============================================================================

export const PERMISSION_SEED = Buffer.from("permission:");

/**
 * Derives the permission PDA for a given account
 * @param account - The account address
 * @returns The permission PDA
 */
export function permissionPdaFromAccount(account: PublicKey) {
  return PublicKey.findProgramAddressSync(
    [PERMISSION_SEED, account.toBuffer()],
    PERMISSION_PROGRAM_ID,
  )[0];
}
