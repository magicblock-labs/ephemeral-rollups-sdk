import { Address, Instruction, AccountMeta, AccountRole } from "@solana/kit";
import { PERMISSION_PROGRAM_ID } from "../../constants";
import { permissionPdaFromAccount } from "../../pda";
import { getMembersArgsEncoder, MembersArgs } from "../../access-control/types";

export const UPDATE_PERMISSION_DISCRIMINATOR = [1, 0, 0, 0, 0, 0, 0, 0];

/**
 * Instruction: UpdatePermission
 * Discriminator: [1, 0, 0, 0, 0, 0, 0, 0]
 *
 * Accounts:
 *   0. `[signer?]` authority - Either this or permissionedAccount must be a signer
 *   1. `[signer?]` permissionedAccount - Either this or authority must be a signer
 *   2. `[writable]` permission
 *
 * Note: The processor validates that at least one of authority or permissionedAccount
 * is authorized (either as a direct signer or as a permission member).
 */
export async function createUpdatePermissionInstruction(
  accounts: {
    authority: [Address, boolean];
    permissionedAccount: [Address, boolean];
  },
  args: MembersArgs,
): Promise<Instruction> {
  const permission = await permissionPdaFromAccount(
    accounts.permissionedAccount[0],
  );

  const accountsMeta: AccountMeta[] = [
    {
      address: accounts.authority[0],
      role: accounts.authority[1]
        ? AccountRole.WRITABLE_SIGNER
        : AccountRole.READONLY,
    },
    {
      address: accounts.permissionedAccount[0],
      role: accounts.permissionedAccount[1]
        ? AccountRole.WRITABLE_SIGNER
        : AccountRole.READONLY,
    },
    { address: permission, role: AccountRole.WRITABLE },
  ];

  const argsBuffer = getMembersArgsEncoder().encode(args);
  const instructionData = Buffer.from([
    ...UPDATE_PERMISSION_DISCRIMINATOR,
    ...argsBuffer,
  ]);

  return {
    accounts: accountsMeta,
    data: instructionData,
    programAddress: PERMISSION_PROGRAM_ID,
  };
}
