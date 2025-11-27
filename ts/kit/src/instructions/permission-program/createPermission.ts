import {
  Address,
  Instruction,
  AccountMeta,
  AccountRole,
} from "@solana/kit";
import { SYSTEM_PROGRAM_ADDRESS } from "@solana-program/system";
import { PERMISSION_PROGRAM_ID } from "../../constants";

/**
 * CreatePermission instruction arguments
 */
export interface CreatePermissionInstructionArgs {}

/**
 * Instruction: CreatePermission
 * Discriminator: 1
 */
export function createCreatePermissionInstruction(
  accounts: {
    permission: Address;
    delegatedAccount: Address;
    group: Address;
    payer: Address;
  },
  args?: CreatePermissionInstructionArgs,
): Instruction {
  const accountsMeta: AccountMeta[] = [
    { address: accounts.permission, role: AccountRole.WRITABLE },
    { address: accounts.delegatedAccount, role: AccountRole.READONLY_SIGNER },
    { address: accounts.group, role: AccountRole.READONLY },
    { address: accounts.payer, role: AccountRole.WRITABLE_SIGNER },
    { address: SYSTEM_PROGRAM_ADDRESS, role: AccountRole.READONLY },
  ];

  const [instructionData] = serializeCreatePermissionInstructionData(args);

  return {
    accounts: accountsMeta,
    data: instructionData,
    programAddress: PERMISSION_PROGRAM_ID,
  };
}

export function serializeCreatePermissionInstructionData(
  args?: CreatePermissionInstructionArgs,
): [Uint8Array] {
  const discriminator = 1;
  let offset = 0;
  const buffer = new ArrayBuffer(1);
  const view = new DataView(buffer);

  // Write discriminator (u8)
  view.setUint8(offset++, discriminator);

  return [new Uint8Array(buffer, 0, offset)];
}
