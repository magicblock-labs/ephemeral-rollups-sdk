import {
  PublicKey,
  TransactionInstruction,
  SystemProgram,
  AccountMeta,
} from "@solana/web3.js";
import { DELEGATION_PROGRAM_ID } from "../../constants";
import {
  delegateBufferPdaFromDelegatedAccountAndOwnerProgram,
  delegationRecordPdaFromDelegatedAccount,
  delegationMetadataPdaFromDelegatedAccount,
} from "../../pda";

/**
 * Delegate instruction arguments
 */
export interface DelegateInstructionArgs {
  commitFrequencyMs?: number;
  seeds?: Uint8Array[];
  validator?: PublicKey | null;
}

/**
 * Instruction: Delegate
 * Discriminator: [0,0,0,0,0,0,0,0]
 */
export function createDelegateInstruction(
  accounts: {
    payer: PublicKey;
    delegatedAccount: PublicKey;
    ownerProgram: PublicKey;
    validator?: PublicKey;
  },
  args?: DelegateInstructionArgs,
): TransactionInstruction {
  const delegateBuffer = delegateBufferPdaFromDelegatedAccountAndOwnerProgram(
    accounts.delegatedAccount,
    accounts.ownerProgram,
  );
  const delegationRecord = delegationRecordPdaFromDelegatedAccount(
    accounts.delegatedAccount,
  );
  const delegationMetadata = delegationMetadataPdaFromDelegatedAccount(
    accounts.delegatedAccount,
  );

  const keys: AccountMeta[] = [
    { pubkey: accounts.payer, isWritable: true, isSigner: true },
    { pubkey: accounts.delegatedAccount, isWritable: true, isSigner: true },
    { pubkey: accounts.ownerProgram, isWritable: false, isSigner: false },
    { pubkey: delegateBuffer, isWritable: true, isSigner: false },
    { pubkey: delegationRecord, isWritable: true, isSigner: false },
    { pubkey: delegationMetadata, isWritable: true, isSigner: false },
    { pubkey: SystemProgram.programId, isWritable: false, isSigner: false },
  ];

  const instructionData = serializeDelegateInstructionData({
    validator: accounts.validator,
    ...args,
  });

  return new TransactionInstruction({
    programId: DELEGATION_PROGRAM_ID,
    keys,
    data: instructionData,
  });
}

export function serializeDelegateInstructionData(
  args?: DelegateInstructionArgs,
): Buffer {
  const delegateInstructionDiscriminator = [0, 0, 0, 0, 0, 0, 0, 0];
  const commitFrequencyMs = args?.commitFrequencyMs ?? 0xffffffff;
  const seeds = args?.seeds ?? [];
  const validator = args?.validator;
  const buffer = Buffer.alloc(1024);
  let offset = 0;

  // Write discriminator
  for (let i = 0; i < 8; i++) {
    buffer[offset++] = delegateInstructionDiscriminator[i];
  }

  // Write commit_frequency_ms (u32)
  buffer.writeUInt32LE(commitFrequencyMs, offset);
  offset += 4;

  // Write seeds (vec<vec<u8>>)
  buffer.writeUInt32LE(seeds.length, offset);
  offset += 4;

  for (const seed of seeds) {
    buffer.writeUInt32LE(seed.length, offset);
    offset += 4;
    buffer.set(seed, offset);
    offset += seed.length;
  }

  // Write validator (Option<Pubkey>)
  if (validator) {
    buffer[offset++] = 1; // Some discriminant
    buffer.set(validator.toBuffer(), offset);
    offset += 32;
  } else {
    buffer[offset++] = 0; // None discriminant
  }

  return buffer.subarray(0, offset);
}
