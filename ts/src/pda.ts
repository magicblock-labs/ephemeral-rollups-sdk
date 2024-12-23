import { PublicKey } from "@solana/web3.js";

import { DELEGATION_PROGRAM_ID } from "./constants";

export function delegationRecordPdaFromDelegatedAccount(
  delegatedAccount: PublicKey
) {
  return PublicKey.findProgramAddressSync(
    [Buffer.from("delegation"), delegatedAccount.toBytes()],
    DELEGATION_PROGRAM_ID
  )[0];
}

export function delegationMetadataPdaFromDelegatedAccount(
  delegatedAccount: PublicKey
) {
  return PublicKey.findProgramAddressSync(
    [Buffer.from("delegation-metadata"), delegatedAccount.toBytes()],
    DELEGATION_PROGRAM_ID
  )[0];
}

export function delegateBufferPdaFromDelegatedAccountAndOwnerProgram(
  delegatedAccount: PublicKey,
  ownerProgramId: PublicKey
) {
  return PublicKey.findProgramAddressSync(
    [Buffer.from("buffer"), delegatedAccount.toBytes()],
    ownerProgramId
  )[0];
}
