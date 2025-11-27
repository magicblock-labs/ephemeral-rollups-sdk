import {
  PublicKey,
  TransactionInstruction,
  AccountMeta,
} from "@solana/web3.js";
import { PERMISSION_PROGRAM_ID } from "../../constants";

/**
 * UpdatePermission instruction arguments
 */
export interface UpdatePermissionInstructionArgs {}

/**
 * Instruction: UpdatePermission
 * Discriminator: 2
 */
export function createUpdatePermissionInstruction(
  accounts: {
    permission: PublicKey;
    delegatedAccount: PublicKey;
    group: PublicKey;
  },
  args?: UpdatePermissionInstructionArgs,
): TransactionInstruction {
  const keys: AccountMeta[] = [
    { pubkey: accounts.permission, isWritable: true, isSigner: false },
    { pubkey: accounts.delegatedAccount, isWritable: false, isSigner: true },
    { pubkey: accounts.group, isWritable: false, isSigner: false },
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
  const discriminator = 2;
  const buffer = Buffer.alloc(1);
  let offset = 0;

  // Write discriminator (u8)
  buffer[offset++] = discriminator;

  return buffer.subarray(0, offset);
}
