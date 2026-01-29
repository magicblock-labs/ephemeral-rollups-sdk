import {
  PublicKey,
  TransactionInstruction,
  AccountMeta,
} from "@solana/web3.js";
import {
  PERMISSION_PROGRAM_ID,
  MAGIC_PROGRAM_ID,
  MAGIC_CONTEXT_ID,
} from "../../constants";
import { permissionPdaFromAccount } from "../../pda";

/**
 * Instruction: CommitPermission
 * Discriminator: [4, 0, 0, 0, 0, 0, 0, 0]
 *
 * Accounts:
 *   0. `[signer?]` authority - Either this or permissionedAccount must be a signer
 *   1. `[writable, signer?]` permissionedAccount - Either this or authority must be a signer
 *   2. `[writable]` permission
 *   3. `[]` magic_program
 *   4. `[writable]` magic_context
 *
 * Note: The processor validates that at least one of authority or permissionedAccount
 * is authorized (either as a direct signer or as a permission member).
 */
export function createCommitPermissionInstruction(accounts: {
  authority: [PublicKey, boolean];
  permissionedAccount: [PublicKey, boolean];
}): TransactionInstruction {
  const permission = permissionPdaFromAccount(accounts.permissionedAccount[0]);

  // Either authority or permissionedAccount must be a signer
  const keys: AccountMeta[] = [
    {
      pubkey: accounts.authority[0],
      isWritable: accounts.authority[1],
      isSigner: accounts.authority[1],
    },
    {
      pubkey: accounts.permissionedAccount[0],
      isWritable: true,
      isSigner: accounts.permissionedAccount[1],
    },
    { pubkey: permission, isWritable: true, isSigner: false },
    { pubkey: MAGIC_PROGRAM_ID, isWritable: false, isSigner: false },
    { pubkey: MAGIC_CONTEXT_ID, isWritable: true, isSigner: false },
  ];

  const instructionData = serializeCommitPermissionInstructionData();

  return new TransactionInstruction({
    programId: PERMISSION_PROGRAM_ID,
    keys,
    data: instructionData,
  });
}

export function serializeCommitPermissionInstructionData(): Buffer {
  const discriminator = [4, 0, 0, 0, 0, 0, 0, 0];
  const buffer = Buffer.alloc(8);

  // Write discriminator (u64)
  for (let i = 0; i < 8; i++) {
    buffer[i] = discriminator[i];
  }

  return buffer;
}
