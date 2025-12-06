import {
  PublicKey,
  TransactionInstruction,
  AccountMeta,
} from "@solana/web3.js";
import { PERMISSION_PROGRAM_ID, MAGIC_PROGRAM_ID, MAGIC_CONTEXT_ID } from "../../constants";

/**
 * CommitAndUndelegatePermission instruction arguments
 */
// eslint-disable-next-line @typescript-eslint/no-empty-interface
export interface CommitAndUndelegatePermissionInstructionArgs {}

/**
 * Instruction: CommitAndUndelegatePermission
 * Discriminator: 6
 */
export function createCommitAndUndelegatePermissionInstruction(
  accounts: {
    delegatedAccount: PublicKey;
    permission: PublicKey;
    permissionProgram?: PublicKey;
  },
  args?: CommitAndUndelegatePermissionInstructionArgs,
): TransactionInstruction {

  const keys: AccountMeta[] = [
    { pubkey: accounts.delegatedAccount, isWritable: true, isSigner: true },
    { pubkey: accounts.permission, isWritable: true, isSigner: false },
    { pubkey: MAGIC_PROGRAM_ID, isWritable: false, isSigner: false },
    { pubkey: MAGIC_CONTEXT_ID, isWritable: true, isSigner: false },
  ];

  const instructionData = serializeCommitAndUndelegatePermissionInstructionData(args);

  return new TransactionInstruction({
    programId: accounts.permissionProgram ?? PERMISSION_PROGRAM_ID,
    keys,
    data: instructionData,
  });
}

export function serializeCommitAndUndelegatePermissionInstructionData(
  args?: CommitAndUndelegatePermissionInstructionArgs,
): Buffer {
  const discriminator = 6;
  const buffer = Buffer.alloc(1);
  let offset = 0;

  // Write discriminator (u8)
  buffer[offset++] = discriminator;

  return buffer.subarray(0, offset);
}
