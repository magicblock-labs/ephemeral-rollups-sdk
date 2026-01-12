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
 */
export async function createDelegatePermissionInstruction(
  accounts: {
    payer: Address;
    permissionedAccount: Address;
    ownerProgram?: Address;
    validator?: Address | null;
  },
  args?: DelegatePermissionInstructionArgs,
): Promise<Instruction> {
  const ownerProgram = accounts.ownerProgram ?? PERMISSION_PROGRAM_ID;
  const permissionPda = await permissionPdaFromAccount(
    accounts.permissionedAccount,
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
    { address: accounts.permissionedAccount, role: AccountRole.READONLY },
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
