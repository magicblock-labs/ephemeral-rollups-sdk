import { PublicKey } from "@solana/web3.js";

import { DELEGATION_PROGRAM_ID } from "./constants.js";

export function delegationRecordPdaFromDelegatedAccount(
  delegatedAccount: PublicKey,
) {
  return PublicKey.findProgramAddressSync(
    [Buffer.from("delegation"), delegatedAccount.toBytes()],
    DELEGATION_PROGRAM_ID,
  )[0];
}

export function delegationMetadataPdaFromDelegatedAccount(
  delegatedAccount: PublicKey,
) {
  return PublicKey.findProgramAddressSync(
    [Buffer.from("delegation-metadata"), delegatedAccount.toBytes()],
    DELEGATION_PROGRAM_ID,
  )[0];
}

export function delegateBufferPdaFromDelegatedAccountAndOwnerProgram(
  delegatedAccount: PublicKey,
  ownerProgramId: PublicKey,
) {
  return PublicKey.findProgramAddressSync(
    [Buffer.from("buffer"), delegatedAccount.toBytes()],
    ownerProgramId,
  )[0];
}

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

export function commitStatePdaFromDelegatedAccount(
  delegatedAccount: PublicKey,
) {
  return PublicKey.findProgramAddressSync(
    [Buffer.from("state-diff"), delegatedAccount.toBytes()],
    DELEGATION_PROGRAM_ID,
  )[0];
}

export function commitRecordPdaFromDelegatedAccount(
  delegatedAccount: PublicKey,
) {
  return PublicKey.findProgramAddressSync(
    [Buffer.from("commit-state-record"), delegatedAccount.toBytes()],
    DELEGATION_PROGRAM_ID,
  )[0];
}

export function undelegateBufferPdaFromDelegatedAccount(
  delegatedAccount: PublicKey,
) {
  return PublicKey.findProgramAddressSync(
    [Buffer.from("undelegate-buffer"), delegatedAccount.toBytes()],
    DELEGATION_PROGRAM_ID,
  )[0];
}

export function feesVaultPda() {
  return PublicKey.findProgramAddressSync(
    [Buffer.from("fees-vault")],
    DELEGATION_PROGRAM_ID,
  )[0];
}

export function validatorFeesVaultPdaFromValidator(validator: PublicKey) {
  return PublicKey.findProgramAddressSync(
    [Buffer.from("v-fees-vault"), validator.toBytes()],
    DELEGATION_PROGRAM_ID,
  )[0];
}
