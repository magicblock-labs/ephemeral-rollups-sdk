import { Address, Instruction, AccountMeta, AccountRole } from "@solana/kit";
import {
  PERMISSION_PROGRAM_ID,
  MAGIC_PROGRAM_ID,
  MAGIC_CONTEXT_ID,
} from "../../constants";
import { permissionPdaFromAccount } from "../../pda";

/**
 * Instruction: CommitAndUndelegatePermission
 * Discriminator: [5, 0, 0, 0, 0, 0, 0, 0]
 */
export async function createCommitAndUndelegatePermissionInstruction(accounts: {
  authority: Address;
  permissionedAccount: Address;
}): Promise<Instruction> {
  const permission = await permissionPdaFromAccount(
    accounts.permissionedAccount,
  );

  const accountsMeta: AccountMeta[] = [
    { address: accounts.authority, role: AccountRole.READONLY_SIGNER },
    {
      address: accounts.permissionedAccount,
      role: AccountRole.WRITABLE_SIGNER,
    },
    { address: permission, role: AccountRole.WRITABLE },
    { address: MAGIC_PROGRAM_ID, role: AccountRole.READONLY },
    { address: MAGIC_CONTEXT_ID, role: AccountRole.WRITABLE },
  ];

  const [instructionData] =
    serializeCommitAndUndelegatePermissionInstructionData();

  return {
    accounts: accountsMeta,
    data: instructionData,
    programAddress: PERMISSION_PROGRAM_ID,
  };
}

export function serializeCommitAndUndelegatePermissionInstructionData(): [
  Uint8Array,
] {
  const discriminator = [5, 0, 0, 0, 0, 0, 0, 0];
  const buffer = new ArrayBuffer(8);
  const view = new DataView(buffer);

  // Write discriminator (u64)
  for (let i = 0; i < 8; i++) {
    view.setUint8(i, discriminator[i]);
  }

  return [new Uint8Array(buffer, 0, 8)];
}
