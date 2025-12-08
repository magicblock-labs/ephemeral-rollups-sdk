import {
  Address,
  Instruction,
  AccountMeta,
  AccountRole,
} from "@solana/kit";
import { SYSTEM_PROGRAM_ADDRESS } from "@solana-program/system";
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
export async function createDelegatePermissionInstruction(
  accounts: {
    payer: Address
    delegatedAccount: Address;
    permission: Address;
    permissionProgram?: Address;
  },
  args?: DelegatePermissionInstructionArgs,
): Promise<Instruction> {

  const delegationBuffer = await delegateBufferPdaFromDelegatedAccountAndOwnerProgram(
    accounts.permission,
    accounts.permissionProgram ?? PERMISSION_PROGRAM_ID,
  );
  const delegationRecord = await delegationRecordPdaFromDelegatedAccount(
    accounts.permission,
  );
  const delegationMetadata = await delegationMetadataPdaFromDelegatedAccount(
    accounts.permission,
  );

  const accountsMeta: AccountMeta[] = [
    { address: accounts.payer, role: AccountRole.WRITABLE_SIGNER },
    { address: accounts.delegatedAccount, role: AccountRole.READONLY_SIGNER },
    { address: SYSTEM_PROGRAM_ADDRESS, role: AccountRole.READONLY },
    { address: accounts.permission, role: AccountRole.WRITABLE },
    { address: accounts.permissionProgram ?? PERMISSION_PROGRAM_ID, role: AccountRole.READONLY },
    { address: delegationBuffer, role: AccountRole.WRITABLE },
    { address: delegationRecord, role: AccountRole.WRITABLE },
    { address: delegationMetadata, role: AccountRole.WRITABLE },
    { address: DELEGATION_PROGRAM_ID, role: AccountRole.READONLY },
  ];

  const [instructionData] = serializeDelegatePermissionInstructionData(args);

  return {
    accounts: accountsMeta,
    data: instructionData,
    programAddress: PERMISSION_PROGRAM_ID,
  };
}

export function serializeDelegatePermissionInstructionData(
  args?: DelegatePermissionInstructionArgs,
): [Uint8Array] {
  const discriminator = 4;
  const buffer = new ArrayBuffer(1);
  const view = new DataView(buffer);
  let offset = 0;

  // Write discriminator (u8)
  view.setUint8(offset++, discriminator);

  return [new Uint8Array(buffer, 0, offset)];
}
