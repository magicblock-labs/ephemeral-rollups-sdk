import {
  PublicKey,
  TransactionInstruction,
  SystemProgram,
  AccountMeta,
} from "@solana/web3.js";
import { PERMISSION_PROGRAM_ID } from "../../constants";
import { permissionPdaFromAccount } from "../../pda";
import type { Member } from "../../access-control/types";

/**
 * Create permission instruction arguments
 */
export interface CreatePermissionInstructionArgs {
  members?: Member[];
}

/**
 * Instruction: CreatePermission
 * Discriminator: [0, 0, 0, 0, 0, 0, 0, 0]
 */
export function createCreatePermissionInstruction(
  accounts: {
    permissionedAccount: PublicKey;
    payer: PublicKey;
  },
  args?: CreatePermissionInstructionArgs,
): TransactionInstruction {
  const permission = permissionPdaFromAccount(accounts.permissionedAccount);

  const keys: AccountMeta[] = [
    { pubkey: accounts.permissionedAccount, isWritable: false, isSigner: true },
    { pubkey: permission, isWritable: true, isSigner: false },
    { pubkey: accounts.payer, isWritable: true, isSigner: true },
    { pubkey: SystemProgram.programId, isWritable: false, isSigner: false },
  ];

  const instructionData = serializeCreatePermissionInstructionData(args);

  return new TransactionInstruction({
    programId: PERMISSION_PROGRAM_ID,
    keys,
    data: instructionData,
  });
}

export function serializeCreatePermissionInstructionData(
  args?: CreatePermissionInstructionArgs,
): Buffer {
  const MAX_BUFFER_SIZE = 2048;
  const discriminator = [0, 0, 0, 0, 0, 0, 0, 0];
  const members = args?.members ?? [];
  const buffer = Buffer.alloc(MAX_BUFFER_SIZE);
  let offset = 0;

  // Write discriminator (u64)
  for (let i = 0; i < 8; i++) {
    buffer[offset++] = discriminator[i];
  }

  // Write members count (u32)
  if (offset + 4 > MAX_BUFFER_SIZE) {
    throw new Error(
      `Serialized data exceeds buffer size (${MAX_BUFFER_SIZE} bytes)`,
    );
  }
  buffer.writeUInt32LE(members.length, offset);
  offset += 4;

  // Write members
  for (const member of members) {
    if (offset + 33 > MAX_BUFFER_SIZE) {
      throw new Error(
        `Serialized data exceeds buffer size (${MAX_BUFFER_SIZE} bytes)`,
      );
    }
    buffer.set(member.pubkey.toBuffer(), offset);
    offset += 32;

    // Write flags (u8)
    buffer[offset++] = member.flags;
  }

  return buffer.subarray(0, offset);
}
