import { Address, Instruction, AccountMeta, AccountRole } from "@solana/kit";
import {
  PERMISSION_PROGRAM_ID,
  MAGIC_PROGRAM_ID,
  MAGIC_CONTEXT_ID,
} from "../../constants";
import { permissionPdaFromAccount } from "../../pda";

/**
 * Instruction: CommitPermission
 * Discriminator: [4, 0, 0, 0, 0, 0, 0, 0]
 *
 * Accounts:
 *   0. `[signer?]` authority - Either this or permissionedAccount must be a signer
 *   1. `[writable, signer?]` permissionedAccount - Either this or authority must be a signer
 *   2. `[writable]` permission
 *   3. `[]` magic_program
 *   4. `[writable]` magic_context
 *
 * Note: The processor validates that at least one of authority or permissionedAccount
 * is authorized (either as a direct signer or as a permission member).
 */
export async function createCommitPermissionInstruction(accounts: {
  authority: [Address, boolean];
  permissionedAccount: [Address, boolean];
}): Promise<Instruction> {
  const permission = await permissionPdaFromAccount(
    accounts.permissionedAccount[0],
  );

  // Either authority or permissionedAccount must be a signer
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
        : AccountRole.WRITABLE,
    },
    { address: permission, role: AccountRole.WRITABLE },
    { address: MAGIC_PROGRAM_ID, role: AccountRole.READONLY },
    { address: MAGIC_CONTEXT_ID, role: AccountRole.WRITABLE },
  ];

  const [instructionData] = serializeCommitPermissionInstructionData();

  return {
    accounts: accountsMeta,
    data: instructionData,
    programAddress: PERMISSION_PROGRAM_ID,
  };
}

export function serializeCommitPermissionInstructionData(): [Uint8Array] {
  const discriminator = [4, 0, 0, 0, 0, 0, 0, 0];
  const buffer = new ArrayBuffer(8);
  const view = new DataView(buffer);

  // Write discriminator (u64)
  for (let i = 0; i < 8; i++) {
    view.setUint8(i, discriminator[i]);
  }

  return [new Uint8Array(buffer, 0, 8)];
}
