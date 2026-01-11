import {
  PublicKey,
  TransactionInstruction,
  SystemProgram,
  AccountMeta,
} from "@solana/web3.js";
import { PERMISSION_PROGRAM_ID, DELEGATION_PROGRAM_ID } from "../../constants";
import {
  permissionPdaFromAccount,
  delegateBufferPdaFromDelegatedAccountAndOwnerProgram,
  delegationRecordPdaFromDelegatedAccount,
  delegationMetadataPdaFromDelegatedAccount,
} from "../../pda";

/**
 * Delegate permission instruction arguments
 */
export interface DelegatePermissionInstructionArgs {
  validator?: PublicKey | null;
}

/**
 * Instruction: DelegatePermission
 * Discriminator: [3, 0, 0, 0, 0, 0, 0, 0]
 */
export function createDelegatePermissionInstruction(
  accounts: {
    payer: PublicKey;
    permissionedAccount: PublicKey;
    ownerProgram?: PublicKey;
    validator?: PublicKey | null;
  },
  args?: DelegatePermissionInstructionArgs,
): TransactionInstruction {
  const ownerProgram = accounts.ownerProgram ?? PERMISSION_PROGRAM_ID;
  const permissionPda = permissionPdaFromAccount(accounts.permissionedAccount);
  const delegateBuffer = delegateBufferPdaFromDelegatedAccountAndOwnerProgram(
    permissionPda,
    ownerProgram,
  );
  const delegationRecord =
    delegationRecordPdaFromDelegatedAccount(permissionPda);
  const delegationMetadata =
    delegationMetadataPdaFromDelegatedAccount(permissionPda);

  const validator = args?.validator ?? accounts.validator;

  if (!validator) {
    throw new Error("validator is required");
  }

  const keys: AccountMeta[] = [
    { pubkey: accounts.payer, isWritable: true, isSigner: true },
    {
      pubkey: accounts.permissionedAccount,
      isWritable: false,
      isSigner: false,
    },
    { pubkey: permissionPda, isWritable: true, isSigner: false },
    { pubkey: SystemProgram.programId, isWritable: false, isSigner: false },
    { pubkey: ownerProgram, isWritable: false, isSigner: false },
    { pubkey: delegateBuffer, isWritable: true, isSigner: false },
    { pubkey: delegationRecord, isWritable: true, isSigner: false },
    { pubkey: delegationMetadata, isWritable: true, isSigner: false },
    { pubkey: DELEGATION_PROGRAM_ID, isWritable: false, isSigner: false },
    {
      pubkey: validator,
      isWritable: false,
      isSigner: false,
    },
  ];

  const instructionData = serializeDelegatePermissionInstructionData();

  return new TransactionInstruction({
    programId: PERMISSION_PROGRAM_ID,
    keys,
    data: instructionData,
  });
}

export function serializeDelegatePermissionInstructionData(): Buffer {
  const discriminator = [3, 0, 0, 0, 0, 0, 0, 0];
  const buffer = Buffer.alloc(8);

  // Write discriminator (u64)
  for (let i = 0; i < 8; i++) {
    buffer[i] = discriminator[i];
  }

  return buffer;
}
