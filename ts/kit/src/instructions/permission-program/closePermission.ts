import {
  Address,
  Instruction,
  AccountMeta,
  AccountRole,
} from "@solana/kit";
import { PERMISSION_PROGRAM_ID } from "../../constants";

/**
 * ClosePermission instruction arguments
 */
// eslint-disable-next-line @typescript-eslint/no-empty-interface
export interface ClosePermissionInstructionArgs {}

/**
 * Instruction: ClosePermission
 * Discriminator: 3
 */
export function createClosePermissionInstruction(
  accounts: {
    permission: Address;
    delegatedAccount: Address;
    permissionProgram?: Address;
  },
  args?: ClosePermissionInstructionArgs,
): Instruction {
  
  const accountsMeta: AccountMeta[] = [
    { address: accounts.permission, role: AccountRole.WRITABLE },
    { address: accounts.delegatedAccount, role: AccountRole.WRITABLE_SIGNER },
  ];

  const [instructionData] = serializeClosePermissionInstructionData(args);

  return {
    accounts: accountsMeta,
    data: instructionData,
    programAddress: accounts.permissionProgram ?? PERMISSION_PROGRAM_ID,
  };
}

export function serializeClosePermissionInstructionData(
  args?: ClosePermissionInstructionArgs,
): [Uint8Array] {
  const discriminator = 3;
  const buffer = new ArrayBuffer(1);
  const view = new DataView(buffer);
  let offset = 0;

  // Write discriminator (u8)
  view.setUint8(offset++, discriminator);

  return [new Uint8Array(buffer, 0, offset)];
}
