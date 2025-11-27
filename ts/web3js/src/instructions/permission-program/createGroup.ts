import {
  PublicKey,
  TransactionInstruction,
  SystemProgram,
  AccountMeta,
} from "@solana/web3.js";
import { PERMISSION_PROGRAM_ID } from "../../constants";

/**
 * CreateGroup instruction arguments
 */
export interface CreateGroupInstructionArgs {
  id: PublicKey;
  members: PublicKey[];
}

/**
 * Instruction: CreateGroup
 * Discriminator: 0
 */
export function createCreateGroupInstruction(
  accounts: {
    group: PublicKey;
    payer: PublicKey;
  },
  args: CreateGroupInstructionArgs,
): TransactionInstruction {
  const keys: AccountMeta[] = [
    { pubkey: accounts.group, isWritable: true, isSigner: false },
    { pubkey: accounts.payer, isWritable: true, isSigner: true },
    { pubkey: SystemProgram.programId, isWritable: false, isSigner: false },
  ];

  const instructionData = serializeCreateGroupInstructionData(args);

  return new TransactionInstruction({
    programId: PERMISSION_PROGRAM_ID,
    keys,
    data: instructionData,
  });
}

export function serializeCreateGroupInstructionData(
  args: CreateGroupInstructionArgs,
): Buffer {
  const discriminator = 0;
  const buffer = Buffer.alloc(10000);
  let offset = 0;

  // Write discriminator (u8)
  buffer[offset++] = discriminator;

  // Write id (PublicKey)
  buffer.set(args.id.toBuffer(), offset);
  offset += 32;

  // Write members count (u32)
  buffer.writeUInt32LE(args.members.length, offset);
  offset += 4;

  // Write members (vec<PublicKey>)
  for (const member of args.members) {
    buffer.set(member.toBuffer(), offset);
    offset += 32;
  }

  return buffer.subarray(0, offset);
}
