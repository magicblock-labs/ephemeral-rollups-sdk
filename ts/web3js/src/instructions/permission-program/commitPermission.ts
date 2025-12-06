import {
  PublicKey,
  TransactionInstruction,
  AccountMeta,
} from "@solana/web3.js";
import { PERMISSION_PROGRAM_ID, MAGIC_PROGRAM_ID, MAGIC_CONTEXT_ID } from "../../constants";

/**
 * CommitPermission instruction arguments
 */
// eslint-disable-next-line @typescript-eslint/no-empty-interface
export interface CommitPermissionInstructionArgs {}

/**
 * Instruction: CommitPermission
 * Discriminator: 5
 */
export function createCommitPermissionInstruction(
  accounts: {
    delegatedAccount: PublicKey;
    permission: PublicKey;
    permissionProgram?: PublicKey;
  },
  args?: CommitPermissionInstructionArgs,
): TransactionInstruction {

  const keys: AccountMeta[] = [
    { pubkey: accounts.delegatedAccount, isWritable: true, isSigner: true },
    { pubkey: accounts.permission, isWritable: true, isSigner: false },
    { pubkey: MAGIC_PROGRAM_ID, isWritable: false, isSigner: false },
    { pubkey: MAGIC_CONTEXT_ID, isWritable: true, isSigner: false },
  ];

  const instructionData = serializeCommitPermissionInstructionData(args);

  return new TransactionInstruction({
    programId: accounts.permissionProgram ?? PERMISSION_PROGRAM_ID,
    keys,
    data: instructionData,
  });
}

export function serializeCommitPermissionInstructionData(
  args?: CommitPermissionInstructionArgs,
): Buffer {
  const discriminator = 5;
  const buffer = Buffer.alloc(1);
  let offset = 0;

  // Write discriminator (u8)
  buffer[offset++] = discriminator;

  return buffer.subarray(0, offset);
}
