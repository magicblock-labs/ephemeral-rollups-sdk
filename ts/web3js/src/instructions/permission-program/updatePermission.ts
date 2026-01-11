import {
  PublicKey,
  TransactionInstruction,
  AccountMeta,
} from "@solana/web3.js";
import { PERMISSION_PROGRAM_ID } from "../../constants";
import { permissionPdaFromAccount } from "../../pda";

/**
 * Permission member with authorization info
 */
export interface Member {
  pubkey: PublicKey;
  authority: boolean;
}

/**
 * Update permission instruction arguments
 */
export interface UpdatePermissionInstructionArgs {
  members?: Member[];
}

/**
 * Instruction: UpdatePermission
 * Discriminator: [1, 0, 0, 0, 0, 0, 0, 0]
 */
export function createUpdatePermissionInstruction(
  accounts: {
    authority: PublicKey;
    permissionedAccount: PublicKey;
  },
  args?: UpdatePermissionInstructionArgs,
): TransactionInstruction {
  const permission = permissionPdaFromAccount(accounts.permissionedAccount);

  const keys: AccountMeta[] = [
    { pubkey: accounts.authority, isWritable: false, isSigner: true },
    { pubkey: accounts.permissionedAccount, isWritable: false, isSigner: true },
    { pubkey: permission, isWritable: true, isSigner: false },
  ];

  const instructionData = serializeUpdatePermissionInstructionData(args);

  return new TransactionInstruction({
    programId: PERMISSION_PROGRAM_ID,
    keys,
    data: instructionData,
  });
}

export function serializeUpdatePermissionInstructionData(
  args?: UpdatePermissionInstructionArgs,
): Buffer {
  const discriminator = [1, 0, 0, 0, 0, 0, 0, 0];
  const members = args?.members ?? [];
  const buffer = Buffer.alloc(2048);
  let offset = 0;

  // Write discriminator (u64)
  for (let i = 0; i < 8; i++) {
    buffer[offset++] = discriminator[i];
  }

  // Write members count (u32)
  buffer.writeUInt32LE(members.length, offset);
  offset += 4;

  // Write members
  for (const member of members) {
    buffer.set(member.pubkey.toBuffer(), offset);
    offset += 32;

    // Write authority flag (bool as u8)
    buffer[offset++] = member.authority ? 1 : 0;
  }

  return buffer.subarray(0, offset);
}
