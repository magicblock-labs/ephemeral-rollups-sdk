import {
  Address,
  Instruction,
  AccountMeta,
  AccountRole,
} from "@solana/kit";
import { PERMISSION_PROGRAM_ID } from "../../constants";
import { permissionPdaFromAccount } from "../../pda";

/**
 * Instruction: ClosePermission
 * Discriminator: [2, 0, 0, 0, 0, 0, 0, 0]
 */
export async function createClosePermissionInstruction(
  accounts: {
    payer: Address;
    permissionedAccount: Address;
  },
): Promise<Instruction> {
  const permission = await permissionPdaFromAccount(accounts.permissionedAccount);

  const accountsMeta: AccountMeta[] = [
    { address: accounts.payer, role: AccountRole.WRITABLE_SIGNER },
    { address: accounts.permissionedAccount, role: AccountRole.READONLY_SIGNER },
    { address: permission, role: AccountRole.WRITABLE },
  ];

  const [instructionData] = serializeClosePermissionInstructionData();

  return {
    accounts: accountsMeta,
    data: instructionData,
    programAddress: PERMISSION_PROGRAM_ID,
  };
}

export function serializeClosePermissionInstructionData(): [Uint8Array] {
  const discriminator = [2, 0, 0, 0, 0, 0, 0, 0];
  const buffer = new ArrayBuffer(8);
  const view = new DataView(buffer);

  // Write discriminator (u64)
  for (let i = 0; i < 8; i++) {
    view.setUint8(i, discriminator[i]);
  }

  return [new Uint8Array(buffer, 0, 8)];
}
