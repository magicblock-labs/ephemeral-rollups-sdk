import * as beet from "@metaplex-foundation/beet";
import { AccountMeta, Address, AccountRole, address, Instruction } from "@solana/kit";
import { SYSTEM_PROGRAM_ADDRESS } from "@solana-program/system";
import { DELEGATION_PROGRAM_ID } from "../constants";
import {
  delegateBufferPdaFromDelegatedAccountAndOwnerProgram,
  delegationMetadataPdaFromDelegatedAccount,
  delegationRecordPdaFromDelegatedAccount,
} from "../pda";

export const delegateStruct = new beet.FixableBeetArgsStruct<{
  instructionDiscriminator: number[];
  commit_frequency_ms: beet.bignum;
  seeds: number[][];
  validator?: beet.COption<Uint8Array>;
}>(
  [
    ["instructionDiscriminator", beet.uniformFixedSizeArray(beet.u8, 8)],
    ["commit_frequency_ms", beet.u32],
    ["seeds", beet.array(beet.array(beet.u8))],
    ["validator", beet.coption(beet.uniformFixedSizeArray(beet.u8, 32))],
  ],
  "DelegateInstructionArgs",
);
export const delegateInstructionDiscriminator = [0, 0, 0, 0, 0, 0, 0, 0];

// Define the DelegateAccountArgs structure
interface DelegateAccountArgs {
  commit_frequency_ms: number;
  seeds: Uint8Array[][];
  validator?: Address;
}
// Function to create a delegate instruction
export async function createDelegateInstruction(
  accountsInput: {
    payer: Address;
    delegatedAccount: Address;
    ownerProgram: Address;
    delegationRecord?: Address;
    delegationMetadata?: Address;
    systemProgram?: Address;
    validator?: Address;
  },
  args?: DelegateAccountArgs,
  programId = DELEGATION_PROGRAM_ID,
) {
  const delegateBufferPda = await delegateBufferPdaFromDelegatedAccountAndOwnerProgram(
    accountsInput.delegatedAccount,
    accountsInput.ownerProgram,
  );

  const delegationRecordPda = await delegationRecordPdaFromDelegatedAccount(
    accountsInput.delegatedAccount,
  );
  const delegationMetadataPda = await delegationMetadataPdaFromDelegatedAccount(
    accountsInput.delegatedAccount,
  );

  args = args ?? {
    commit_frequency_ms: 4294967295, // 2 ** 4 - 1,
    seeds: [],
  };

  const accounts: AccountMeta[] = [
    { 
      address: accountsInput.payer, 
      role: AccountRole.READONLY_SIGNER
    },
    { 
      address: accountsInput.delegatedAccount, 
      role: AccountRole.WRITABLE_SIGNER
    },
    { 
      address: accountsInput.ownerProgram,
      role: AccountRole.READONLY
    },
    {
      address: delegateBufferPda,
      role: AccountRole.WRITABLE
    },
    {
      address: accountsInput.delegationRecord ?? delegationRecordPda,
      role: AccountRole.WRITABLE
    },
    {
      address: accountsInput.delegationMetadata ?? delegationMetadataPda,
      role: AccountRole.WRITABLE
    },
    {
      address: accountsInput.systemProgram ?? SYSTEM_PROGRAM_ADDRESS,
      role: AccountRole.READONLY
    },
    // Only add validator if it exists
    ...(accountsInput.validator ? [{
      address: accountsInput.validator,
      role: AccountRole.READONLY
    }] : [])
  ];

  const [data] = delegateStruct.serialize({
    instructionDiscriminator: delegateInstructionDiscriminator,
    commit_frequency_ms: args.commit_frequency_ms,
    seeds: args.seeds.map((seed) => seed.map(Number)),
  });

  const delegateInstruction : Instruction = {
    accounts,
    data,
    programAddress: programId
  }

  return delegateInstruction
}