import { Address, Instruction, AccountMeta, AccountRole } from "@solana/kit";
import { SYSTEM_PROGRAM_ADDRESS } from "@solana-program/system";
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
  validator?: Address | null;
}

/**
 * Instruction: DelegatePermission
 * Discriminator: [3, 0, 0, 0, 0, 0, 0, 0]
 *
 * Accounts:
 *   0. `[writable, signer]` payer
 *   1. `[signer?]` authority - Either this or permissionedAccount must be a signer
 *   2. `[writable, signer?]` permissionedAccount - Either this or authority must be a signer
 *   3. `[writable]` permission
 *   4. `[]` systemProgram
 *   5. `[]` ownerProgram
 *   6. `[writable]` delegateBuffer
 *   7. `[writable]` delegationRecord
 *   8. `[writable]` delegationMetadata
 *   9. `[]` delegationProgram
 *   10. `[optional]` validator
 */
export async function createDelegatePermissionInstruction(
  accounts: {
    payer: Address;
    authority: [Address, boolean];
    permissionedAccount: [Address, boolean];
    ownerProgram?: Address;
    validator?: Address | null;
  },
  args?: DelegatePermissionInstructionArgs,
): Promise<Instruction> {
  const ownerProgram = accounts.ownerProgram ?? PERMISSION_PROGRAM_ID;
  const permissionPda = await permissionPdaFromAccount(
    accounts.permissionedAccount[0],
  );
  const delegateBuffer =
    await delegateBufferPdaFromDelegatedAccountAndOwnerProgram(
      permissionPda,
      ownerProgram,
    );
  const delegationRecord =
    await delegationRecordPdaFromDelegatedAccount(permissionPda);
  const delegationMetadata =
    await delegationMetadataPdaFromDelegatedAccount(permissionPda);

  const validator = args?.validator ?? accounts.validator;

  const accountsMeta: AccountMeta[] = [
    { address: accounts.payer, role: AccountRole.WRITABLE_SIGNER },
    {
      address: accounts.authority[0],
      role: accounts.authority[1]
        ? AccountRole.READONLY_SIGNER
        : AccountRole.READONLY,
    },
    {
      address: accounts.permissionedAccount[0],
      role: accounts.permissionedAccount[1]
        ? AccountRole.WRITABLE_SIGNER
        : AccountRole.WRITABLE,
    },
    { address: permissionPda, role: AccountRole.WRITABLE },
    { address: SYSTEM_PROGRAM_ADDRESS, role: AccountRole.READONLY },
    { address: ownerProgram, role: AccountRole.READONLY },
    { address: delegateBuffer, role: AccountRole.WRITABLE },
    { address: delegationRecord, role: AccountRole.WRITABLE },
    { address: delegationMetadata, role: AccountRole.WRITABLE },
    { address: DELEGATION_PROGRAM_ID, role: AccountRole.READONLY },
  ];

  if (validator) {
    accountsMeta.push({
      address: validator,
      role: AccountRole.READONLY,
    });
  }

  const [instructionData] = serializeDelegatePermissionInstructionData();

  return {
    accounts: accountsMeta,
    data: instructionData,
    programAddress: PERMISSION_PROGRAM_ID,
  };
}

export function serializeDelegatePermissionInstructionData(): [Uint8Array] {
  const discriminator = [3, 0, 0, 0, 0, 0, 0, 0];
  const buffer = new ArrayBuffer(8);
  const view = new DataView(buffer);

  // Write discriminator (u64)
  for (let i = 0; i < 8; i++) {
    view.setUint8(i, discriminator[i]);
  }

  return [new Uint8Array(buffer, 0, 8)];
}
