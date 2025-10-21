import { Address, getAddressEncoder, getProgramDerivedAddress } from "@solana/kit";
import { DELEGATION_PROGRAM_ID } from "./constants";

export async function delegationRecordPdaFromDelegatedAccount(
  delegatedAccount: Address,
) {
  const addressEncoder = getAddressEncoder()
  const [delegationRecordPda, bump] = await getProgramDerivedAddress(
    {
      programAddress: DELEGATION_PROGRAM_ID,
      seeds: [Buffer.from("delegation"), addressEncoder.encode(delegatedAccount)]
    }
  );
  return delegationRecordPda
}

export async function delegationMetadataPdaFromDelegatedAccount(
  delegatedAccount: Address,
) {
    const addressEncoder = getAddressEncoder()
    const [delegationMetadataPda, bump] = await getProgramDerivedAddress(
      {
        programAddress: DELEGATION_PROGRAM_ID,
        seeds: [Buffer.from("delegation-metadata"), addressEncoder.encode(delegatedAccount)]
      }
    );


  return delegationMetadataPda
}

export async function delegateBufferPdaFromDelegatedAccountAndOwnerProgram(
  delegatedAccount: Address,
  ownerProgramId: Address,
) {


  const addressEncoder = getAddressEncoder()
  const [delegateBufferPda, bump] = await getProgramDerivedAddress(
    {
      programAddress: ownerProgramId,
      seeds: [Buffer.from("buffer"), addressEncoder.encode(delegatedAccount)]
    }
  );
  return delegateBufferPda
}
