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
 *
 * Accounts:
 *   0. `[writable, signer]` payer
 *   1. `[signer?]` authority - Either this or permissionedAccount must be a signer
 *   2. `[signer?]` permissionedAccount - Either this or authority must be a signer
 *   3. `[writable]` permission
 *   4. `[]` systemProgram
 *   5. `[]` ownerProgram
 *   6. `[writable]` delegateBuffer
 *   7. `[writable]` delegationRecord
 *   8. `[writable]` delegationMetadata
 *   9. `[]` delegationProgram
 *   10. `[optional]` validator
 */
export function createDelegatePermissionInstruction(
  accounts: {
    payer: PublicKey;
    authority: [PublicKey, boolean];
    permissionedAccount: [PublicKey, boolean];
    ownerProgram?: PublicKey;
    validator?: PublicKey | null;
  },
  args?: DelegatePermissionInstructionArgs,
): TransactionInstruction {
  const ownerProgram = accounts.ownerProgram ?? PERMISSION_PROGRAM_ID;
  const permissionPda = permissionPdaFromAccount(
    accounts.permissionedAccount[0],
  );
  const delegateBuffer = delegateBufferPdaFromDelegatedAccountAndOwnerProgram(
    permissionPda,
    ownerProgram,
  );
  const delegationRecord =
    delegationRecordPdaFromDelegatedAccount(permissionPda);
  const delegationMetadata =
    delegationMetadataPdaFromDelegatedAccount(permissionPda);

  const validator = args?.validator ?? accounts.validator;

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
    { pubkey: permissionPda, isWritable: true, isSigner: false },
    { pubkey: SystemProgram.programId, isWritable: false, isSigner: false },
    { pubkey: ownerProgram, isWritable: false, isSigner: false },
    { pubkey: delegateBuffer, isWritable: true, isSigner: false },
    { pubkey: delegationRecord, isWritable: true, isSigner: false },
    { pubkey: delegationMetadata, isWritable: true, isSigner: false },
    { pubkey: DELEGATION_PROGRAM_ID, isWritable: false, isSigner: false },
  ];

  if (validator) {
    keys.push({
      pubkey: validator,
      isWritable: false,
      isSigner: false,
    });
  }

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
