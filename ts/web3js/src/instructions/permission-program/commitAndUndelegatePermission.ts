import {
  PublicKey,
  TransactionInstruction,
  AccountMeta,
} from "@solana/web3.js";
import { PERMISSION_PROGRAM_ID, MAGIC_PROGRAM_ID, MAGIC_CONTEXT_ID } from "../../constants";
import { permissionPdaFromAccount } from "../../pda";

/**
 * Instruction: CommitAndUndelegatePermission
 * Discriminator: [5, 0, 0, 0, 0, 0, 0, 0]
 */
export function createCommitAndUndelegatePermissionInstruction(
  accounts: {
    authority: PublicKey;
    permissionedAccount: PublicKey;
  },
): TransactionInstruction {
  const permission = permissionPdaFromAccount(accounts.permissionedAccount);

  const keys: AccountMeta[] = [
    { pubkey: accounts.authority, isWritable: false, isSigner: true },
    { pubkey: accounts.permissionedAccount, isWritable: true, isSigner: true },
    { pubkey: permission, isWritable: true, isSigner: false },
    { pubkey: MAGIC_PROGRAM_ID, isWritable: false, isSigner: false },
    { pubkey: MAGIC_CONTEXT_ID, isWritable: true, isSigner: false },
  ];

  const instructionData = serializeCommitAndUndelegatePermissionInstructionData();

  return new TransactionInstruction({
    programId: PERMISSION_PROGRAM_ID,
    keys,
    data: instructionData,
  });
}

export function serializeCommitAndUndelegatePermissionInstructionData(): Buffer {
  const discriminator = [5, 0, 0, 0, 0, 0, 0, 0];
  const buffer = Buffer.alloc(8);

  // Write discriminator (u64)
  for (let i = 0; i < 8; i++) {
    buffer[i] = discriminator[i];
  }

  return buffer;
}
