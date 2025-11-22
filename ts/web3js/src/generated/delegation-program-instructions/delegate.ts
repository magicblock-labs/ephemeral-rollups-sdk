import { PublicKey, TransactionInstruction, AccountMeta } from "@solana/web3.js";
import { DELEGATION_PROGRAM_ID } from "../../constants";

/**
 * Delegate instruction arguments
 */
export type DelegateInstructionArgs = {
  commitFrequencyMs: number;
  seeds: Uint8Array[];
  validator: PublicKey | null;
};

/**
 * Instruction: Delegate
 * Discriminator: [0,0,0,0,0,0,0,0]
 */
export function createDelegateInstruction(
  accounts: {
    payer: PublicKey;
    delegatedAccount: PublicKey;
    ownerProgram: PublicKey;
    delegateBuffer: PublicKey;
    delegationRecord: PublicKey;
    delegationMetadata: PublicKey;
    systemProgram: PublicKey;
  },
  args: DelegateInstructionArgs,
  programId = DELEGATION_PROGRAM_ID
): TransactionInstruction {
  const keys: AccountMeta[] = [
    { pubkey: accounts.payer, isWritable: true, isSigner: true },
    { pubkey: accounts.delegatedAccount, isWritable: true, isSigner: true },
    { pubkey: accounts.ownerProgram, isWritable: false, isSigner: false },
    { pubkey: accounts.delegateBuffer, isWritable: true, isSigner: false },
    { pubkey: accounts.delegationRecord, isWritable: true, isSigner: false },
    {
      pubkey: accounts.delegationMetadata,
      isWritable: true,
      isSigner: false,
    },
    { pubkey: accounts.systemProgram, isWritable: false, isSigner: false },
  ];

  const data = serializeDelegateInstructionData(args);

  return new TransactionInstruction({
    programId,
    keys,
    data,
  });
}

export function serializeDelegateInstructionData(
  args: DelegateInstructionArgs
): Buffer {
  const discriminator = [0, 0, 0, 0, 0, 0, 0, 0];
  let offset = 0;
  const buffer = Buffer.alloc(1024);

  // Write discriminator
  for (let i = 0; i < 8; i++) {
    buffer[offset++] = discriminator[i];
  }

  // Write commit_frequency_ms (u32)
  buffer.writeUInt32LE(args.commitFrequencyMs, offset);
  offset += 4;

  // Write seeds (vec<vec<u8>>)
  buffer.writeUInt32LE(args.seeds.length, offset);
  offset += 4;

  for (const seed of args.seeds) {
    buffer.writeUInt32LE(seed.length, offset);
    offset += 4;
    Buffer.from(seed).copy(buffer, offset);
    offset += seed.length;
  }

  // Write validator (option<pubkey>)
  if (args.validator !== null) {
    buffer[offset] = 1; // Some discriminant
    offset += 1;
    args.validator.toBuffer().copy(buffer, offset);
    offset += 32;
  } else {
    buffer[offset] = 0; // None discriminant
    offset += 1;
  }

  return buffer.slice(0, offset);
}
