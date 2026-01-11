import {
  PublicKey,
  TransactionInstruction,
  SystemProgram,
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
  const discriminator = [0, 0, 0, 0, 0, 0, 0, 0];
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
