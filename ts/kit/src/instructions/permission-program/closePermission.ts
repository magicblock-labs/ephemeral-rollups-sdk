import { Address, Instruction, AccountMeta, AccountRole } from "@solana/kit";
import { PERMISSION_PROGRAM_ID } from "../../constants";
import { permissionPdaFromAccount } from "../../pda";

/**
 * Instruction: ClosePermission
 * Discriminator: [2, 0, 0, 0, 0, 0, 0, 0]
 *
 * Accounts:
 *   0. `[writable, signer]` payer
 *   1. `[signer?]` authority - Either this or permissionedAccount must be a signer
 *   2. `[signer?]` permissionedAccount - Either this or authority must be a signer
 *   3. `[writable]` permission
 *
 * Note: The processor validates that at least one of authority or permissionedAccount
 * is authorized (either as a direct signer or as a permission member).
 */
export async function createClosePermissionInstruction(accounts: {
  payer: Address;
  authority: [Address, boolean];
  permissionedAccount: [Address, boolean];
}): Promise<Instruction> {
  const permission = await permissionPdaFromAccount(
    accounts.permissionedAccount[0],
  );

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
        ? AccountRole.READONLY_SIGNER
        : AccountRole.READONLY,
    },
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
