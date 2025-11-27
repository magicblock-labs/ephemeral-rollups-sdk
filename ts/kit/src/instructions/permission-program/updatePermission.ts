import { Address, Instruction, AccountMeta, AccountRole } from "@solana/kit";
import { PERMISSION_PROGRAM_ID } from "../../constants";

/**
 * UpdatePermission instruction arguments
 */
// eslint-disable-next-line @typescript-eslint/no-empty-interface
export interface UpdatePermissionInstructionArgs {}

/**
 * Instruction: UpdatePermission
 * Discriminator: 2
 */
export function createUpdatePermissionInstruction(
  accounts: {
    permission: Address;
    delegatedAccount: Address;
    group: Address;
  },
  args?: UpdatePermissionInstructionArgs,
): Instruction {
  const accountsMeta: AccountMeta[] = [
    { address: accounts.permission, role: AccountRole.WRITABLE },
    { address: accounts.delegatedAccount, role: AccountRole.READONLY_SIGNER },
    { address: accounts.group, role: AccountRole.READONLY },
  ];

  const [instructionData] = serializeUpdatePermissionInstructionData(args);

  return {
    accounts: accountsMeta,
    data: instructionData,
    programAddress: PERMISSION_PROGRAM_ID,
  };
}

export function serializeUpdatePermissionInstructionData(
  args?: UpdatePermissionInstructionArgs,
): [Uint8Array] {
  const discriminator = 2;
  let offset = 0;
  const buffer = new ArrayBuffer(1);
  const view = new DataView(buffer);

  // Write discriminator (u8)
  view.setUint8(offset++, discriminator);

  return [new Uint8Array(buffer, 0, offset)];
}
