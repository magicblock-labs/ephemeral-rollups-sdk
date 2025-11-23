import {
  Address,
  getAddressEncoder,
  getProgramDerivedAddress,
} from "@solana/kit";
import { DELEGATION_PROGRAM_ID } from "./constants";

export async function delegationRecordPdaFromDelegatedAccount(
  delegatedAccount: Address,
) {
  const addressEncoder = getAddressEncoder();
  const [delegationRecordPda] = await getProgramDerivedAddress({
    programAddress: DELEGATION_PROGRAM_ID,
    seeds: [Buffer.from("delegation"), addressEncoder.encode(delegatedAccount)],
  });
  return delegationRecordPda;
}

export async function delegationMetadataPdaFromDelegatedAccount(
  delegatedAccount: Address,
) {
  const addressEncoder = getAddressEncoder();
  const [delegationMetadataPda] = await getProgramDerivedAddress({
    programAddress: DELEGATION_PROGRAM_ID,
    seeds: [
      Buffer.from("delegation-metadata"),
      addressEncoder.encode(delegatedAccount),
    ],
  });

  return delegationMetadataPda;
}

export async function delegateBufferPdaFromDelegatedAccountAndOwnerProgram(
  delegatedAccount: Address,
  ownerProgramId: Address,
) {
  const addressEncoder = getAddressEncoder();
  const [delegateBufferPda] = await getProgramDerivedAddress({
    programAddress: ownerProgramId,
    seeds: [Buffer.from("buffer"), addressEncoder.encode(delegatedAccount)],
  });
  return delegateBufferPda;
}

/**
 * Derives the escrow PDA from a payer address
 * @param payer The payer address
 * @param index The index of the ephemeral balance account
 * @param programId The delegation program ID
 * @returns The derived ephemeral balance PDA
 */
export async function escrowPdaFromEscrowAuthority(
  escrowAuthority: Address,
  index: number = 255,
) {
  if (index < 0 || index > 255) {
    throw new Error("Index must be between 0 and 255");
  }
  const addressEncoder = getAddressEncoder();
  const [escrowPda] = await getProgramDerivedAddress({
    programAddress: escrowAuthority,
    seeds: [
      Buffer.from("balance"),
      addressEncoder.encode(escrowAuthority),
      Buffer.from([index]),
    ],
  });
  return escrowPda;
}
