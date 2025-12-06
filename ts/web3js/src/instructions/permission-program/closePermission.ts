import {
  PublicKey,
  TransactionInstruction,
  AccountMeta,
} from "@solana/web3.js";
import { PERMISSION_PROGRAM_ID } from "../../constants";

/**
 * ClosePermission instruction arguments
 */
// eslint-disable-next-line @typescript-eslint/no-empty-interface
export interface ClosePermissionInstructionArgs {}

/**
 * Instruction: ClosePermission
 * Discriminator: 3
 */
export function createClosePermissionInstruction(
  accounts: {
    permission: PublicKey;
    delegatedAccount: PublicKey;
    permissionProgram?: PublicKey;
  },
  args?: ClosePermissionInstructionArgs,
): TransactionInstruction {
  
  const keys: AccountMeta[] = [
    { pubkey: accounts.permission, isWritable: true, isSigner: false },
    { pubkey: accounts.delegatedAccount, isWritable: true, isSigner: true },
  ];

  const instructionData = serializeClosePermissionInstructionData(args);

  return new TransactionInstruction({
    programId: accounts.permissionProgram ?? PERMISSION_PROGRAM_ID,
    keys,
    data: instructionData,
  });
}

export function serializeClosePermissionInstructionData(
  args?: ClosePermissionInstructionArgs,
): Buffer {
  const discriminator = 3;
  const buffer = Buffer.alloc(1);
  let offset = 0;

  // Write discriminator (u8)
  buffer[offset++] = discriminator;

  return buffer.subarray(0, offset);
}
