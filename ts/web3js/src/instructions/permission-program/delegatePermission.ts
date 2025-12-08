import {
  PublicKey,
  TransactionInstruction,
  SystemProgram,
  AccountMeta,
} from "@solana/web3.js";
import { PERMISSION_PROGRAM_ID, DELEGATION_PROGRAM_ID } from "../../constants";
import {
  delegationRecordPdaFromDelegatedAccount,
  delegationMetadataPdaFromDelegatedAccount,
  delegateBufferPdaFromDelegatedAccountAndOwnerProgram,
} from "../../pda";

/**
 * DelegatePermission instruction arguments
 */
// eslint-disable-next-line @typescript-eslint/no-empty-interface
export interface DelegatePermissionInstructionArgs {}

/**
 * Instruction: DelegatePermission
 * Discriminator: 4
 */
export function createDelegatePermissionInstruction(
  accounts: {
    payer: PublicKey;
    delegatedAccount: PublicKey;
    permission: PublicKey;
    permissionProgram?: PublicKey;
  },
  args?: DelegatePermissionInstructionArgs,
): TransactionInstruction {

  const delegationBuffer = delegateBufferPdaFromDelegatedAccountAndOwnerProgram(
    accounts.permission,
    accounts.permissionProgram ?? PERMISSION_PROGRAM_ID,
  );
  const delegationRecord = delegationRecordPdaFromDelegatedAccount(
    accounts.permission,
  );
  const delegationMetadata = delegationMetadataPdaFromDelegatedAccount(
    accounts.permission,
  );

  const keys: AccountMeta[] = [
    { pubkey: accounts.payer, isWritable: true, isSigner: true },
    { pubkey: accounts.delegatedAccount, isWritable: false, isSigner: true },
    { pubkey: SystemProgram.programId, isWritable: false, isSigner: false },
    { pubkey: accounts.permission, isWritable: true, isSigner: false },
    { pubkey: accounts.permissionProgram ?? PERMISSION_PROGRAM_ID, isWritable: false, isSigner: false },
    { pubkey: delegationBuffer, isWritable: true, isSigner: false },
    { pubkey: delegationRecord, isWritable: true, isSigner: false },
    { pubkey: delegationMetadata, isWritable: true, isSigner: false },
    { pubkey: DELEGATION_PROGRAM_ID, isWritable: false, isSigner: false },
  ];

  const instructionData = serializeDelegatePermissionInstructionData(args);

  return new TransactionInstruction({
    programId: PERMISSION_PROGRAM_ID,
    keys,
    data: instructionData,
  });
}

export function serializeDelegatePermissionInstructionData(
  args?: DelegatePermissionInstructionArgs,
): Buffer {
  const discriminator = 4;
  const buffer = Buffer.alloc(1);
  let offset = 0;

  // Write discriminator (u8)
  buffer[offset++] = discriminator;

  return buffer.subarray(0, offset);
}
