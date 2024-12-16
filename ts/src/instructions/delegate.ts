import * as beet from "@metaplex-foundation/beet";
import * as web3 from "@solana/web3.js";
import { DELEGATION_PROGRAM_ID } from "../constants";
import {
  bufferPdaFromDelegatedAccountAndOwnerProgramID,
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
  "DelegateInstructionArgs"
);
export const delegateInstructionDiscriminator = [0, 0, 0, 0, 0, 0, 0, 0];

// Define the DelegateAccountArgs structure
interface DelegateAccountArgs {
  commit_frequency_ms: number;
  seeds: Uint8Array[][];
  validator?: web3.PublicKey;
}
// Function to create a delegate instruction
export function createDelegateInstruction(
  accounts: {
    payer: web3.PublicKey;
    delegateAccount: web3.PublicKey;
    ownerProgram: web3.PublicKey;
    buffer?: web3.PublicKey;
    delegationRecord?: web3.PublicKey;
    delegationMetadata?: web3.PublicKey;
    systemProgram?: web3.PublicKey;
  },
  args?: DelegateAccountArgs,
  programId = DELEGATION_PROGRAM_ID
) {
  const delegationRecordPda = delegationRecordPdaFromDelegatedAccount(
    accounts.delegateAccount
  );
  const delegationMetadataPda = delegationMetadataPdaFromDelegatedAccount(
    accounts.delegateAccount
  );
  const bufferPda = bufferPdaFromDelegatedAccountAndOwnerProgramID(
    accounts.delegateAccount,
    accounts.ownerProgram
  );

  args = args ?? {
    commit_frequency_ms: 4294967295, // 2 ** 4 - 1,
    seeds: [],
    validator: undefined,
  };

  const keys: web3.AccountMeta[] = [
    { pubkey: accounts.payer, isWritable: false, isSigner: true },
    { pubkey: accounts.delegateAccount, isWritable: true, isSigner: true },
    { pubkey: accounts.ownerProgram, isWritable: false, isSigner: false },
    {
      pubkey: accounts.buffer ?? bufferPda,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: accounts.delegationRecord ?? delegationRecordPda,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: accounts.delegationMetadata ?? delegationMetadataPda,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: accounts.systemProgram ?? web3.SystemProgram.programId,
      isWritable: false,
      isSigner: false,
    },
  ];

  const [data] = delegateStruct.serialize({
    instructionDiscriminator: delegateInstructionDiscriminator,
    commit_frequency_ms: args.commit_frequency_ms,
    seeds: args.seeds.map((seed) => seed.map(Number)),
    validator: args.validator ? args.validator.toBytes() : undefined,
  });

  return new web3.TransactionInstruction({
    programId,
    keys,
    data,
  });
}
