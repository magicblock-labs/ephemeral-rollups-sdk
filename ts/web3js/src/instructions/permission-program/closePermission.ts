import {
  PublicKey,
  TransactionInstruction,
  AccountMeta,
} from "@solana/web3.js";
import { PERMISSION_PROGRAM_ID } from "../../constants";
import { permissionPdaFromAccount } from "../../pda";

/**
 * Instruction: ClosePermission
 * Discriminator: [2, 0, 0, 0, 0, 0, 0, 0]
 *
 * Accounts:
 *   0. `[writable, signer]` payer
 *   1. `[signer?]` authority - Either this or permissionedAccount must be a signer
 *   2. `[signer?]` permissionedAccount - Either this or authority must be a signer
 *   3. `[writable]` permission
 *
 * Note: The processor validates that at least one of authority or permissionedAccount
 * is authorized (either as a direct signer or as a permission member).
 */
export function createClosePermissionInstruction(accounts: {
  payer: PublicKey;
  authority: [PublicKey, boolean];
  permissionedAccount: [PublicKey, boolean];
}): TransactionInstruction {
  const permission = permissionPdaFromAccount(accounts.permissionedAccount[0]);

  const keys: AccountMeta[] = [
    { pubkey: accounts.payer, isWritable: true, isSigner: true },
    {
      pubkey: accounts.authority[0],
      isWritable: accounts.authority[1],
      isSigner: accounts.authority[1],
    },
    {
      pubkey: accounts.permissionedAccount[0],
      isWritable: accounts.permissionedAccount[1],
      isSigner: accounts.permissionedAccount[1],
    },
    { pubkey: permission, isWritable: true, isSigner: false },
  ];

  const instructionData = serializeClosePermissionInstructionData();

  return new TransactionInstruction({
    programId: PERMISSION_PROGRAM_ID,
    keys,
    data: instructionData,
  });
}

export function serializeClosePermissionInstructionData(): Buffer {
  const discriminator = [2, 0, 0, 0, 0, 0, 0, 0];
  const buffer = Buffer.alloc(8);

  // Write discriminator (u64)
  for (let i = 0; i < 8; i++) {
    buffer[i] = discriminator[i];
  }

  return buffer;
}
