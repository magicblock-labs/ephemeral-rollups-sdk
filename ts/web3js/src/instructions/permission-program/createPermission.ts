import {
  PublicKey,
  TransactionInstruction,
  SystemProgram,
  AccountMeta,
} from "@solana/web3.js";
import { PERMISSION_PROGRAM_ID } from "../../constants";

/**
 * CreatePermission instruction arguments
 */
// eslint-disable-next-line @typescript-eslint/no-empty-interface
export interface CreatePermissionInstructionArgs {}

/**
 * Instruction: CreatePermission
 * Discriminator: 1
 */
export function createCreatePermissionInstruction(
  accounts: {
    permission: PublicKey;
    delegatedAccount: PublicKey;
    group: PublicKey;
    payer: PublicKey;
  },
  args?: CreatePermissionInstructionArgs,
): TransactionInstruction {
  const keys: AccountMeta[] = [
    { pubkey: accounts.permission, isWritable: true, isSigner: false },
    { pubkey: accounts.delegatedAccount, isWritable: false, isSigner: true },
    { pubkey: accounts.group, isWritable: false, isSigner: false },
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
  const discriminator = 1;
  const buffer = Buffer.alloc(1);
  let offset = 0;

  // Write discriminator (u8)
  buffer[offset++] = discriminator;

  return buffer.subarray(0, offset);
}
