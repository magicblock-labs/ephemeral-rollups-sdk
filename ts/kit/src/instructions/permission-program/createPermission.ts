import { Address, Instruction, AccountMeta, AccountRole } from "@solana/kit";
import { SYSTEM_PROGRAM_ADDRESS } from "@solana-program/system";
import { PERMISSION_PROGRAM_ID } from "../../constants";
import { permissionPdaFromAccount } from "../../pda";
import { getMembersArgsEncoder, MembersArgs } from "../../access-control/types";

export const CREATE_PERMISSION_DISCRIMINATOR = [0, 0, 0, 0, 0, 0, 0, 0];

/**
 * Instruction: CreatePermission
 * Discriminator: [0, 0, 0, 0, 0, 0, 0, 0]
 */
export async function createCreatePermissionInstruction(
  accounts: {
    permissionedAccount: Address;
    payer: Address;
  },
  args: MembersArgs,
): Promise<Instruction> {
  const permission = await permissionPdaFromAccount(
    accounts.permissionedAccount,
  );

  const accountsMeta: AccountMeta[] = [
    {
      address: accounts.permissionedAccount,
      role: AccountRole.READONLY_SIGNER,
    },
    { address: permission, role: AccountRole.WRITABLE },
    { address: accounts.payer, role: AccountRole.WRITABLE_SIGNER },
    { address: SYSTEM_PROGRAM_ADDRESS, role: AccountRole.READONLY },
  ];

  const argsBuffer = getMembersArgsEncoder().encode(args);
  const instructionData = Buffer.from([
    ...CREATE_PERMISSION_DISCRIMINATOR,
    ...argsBuffer,
  ]);

  return {
    accounts: accountsMeta,
    data: instructionData,
    programAddress: PERMISSION_PROGRAM_ID,
  };
}
