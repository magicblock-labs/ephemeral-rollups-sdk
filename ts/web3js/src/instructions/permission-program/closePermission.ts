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
 */
export function createClosePermissionInstruction(accounts: {
  payer: PublicKey;
  permissionedAccount: PublicKey;
}): TransactionInstruction {
  const permission = permissionPdaFromAccount(accounts.permissionedAccount);

  const keys: AccountMeta[] = [
    { pubkey: accounts.payer, isWritable: true, isSigner: true },
    { pubkey: accounts.permissionedAccount, isWritable: false, isSigner: true },
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
