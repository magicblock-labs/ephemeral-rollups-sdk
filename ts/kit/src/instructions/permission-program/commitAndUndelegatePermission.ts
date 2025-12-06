import {
  Address,
  Instruction,
  AccountMeta,
  AccountRole,
} from "@solana/kit";
import { MAGIC_CONTEXT_ID, MAGIC_PROGRAM_ID, PERMISSION_PROGRAM_ID } from "../../constants";
import { commitStatePdaFromDelegatedAccount } from "../../pda";

/**
 * CommitAndUndelegatePermission instruction arguments
 */
// eslint-disable-next-line @typescript-eslint/no-empty-interface
export interface CommitAndUndelegatePermissionInstructionArgs {}

/**
 * Instruction: CommitAndUndelegatePermission
 * Discriminator: 6
 */
export async function createCommitAndUndelegatePermissionInstruction(
  accounts: {
    delegatedAccount: Address;
    permission: Address;
    permissionProgram?: Address;
  },
  args?: CommitAndUndelegatePermissionInstructionArgs,
): Promise<Instruction> {

  const accountsMeta: AccountMeta[] = [
    { address: accounts.delegatedAccount, role: AccountRole.WRITABLE_SIGNER },
    { address: accounts.permission, role: AccountRole.WRITABLE },
    { address: MAGIC_PROGRAM_ID, role: AccountRole.READONLY },
    { address: MAGIC_CONTEXT_ID, role: AccountRole.WRITABLE },
  ];

  const [instructionData] = serializeCommitAndUndelegatePermissionInstructionData(args);

  return {
    accounts: accountsMeta,
    data: instructionData,
    programAddress: accounts.permissionProgram ?? PERMISSION_PROGRAM_ID,
  };
}

export function serializeCommitAndUndelegatePermissionInstructionData(
  args?: CommitAndUndelegatePermissionInstructionArgs,
): [Uint8Array] {
  const discriminator = 6;
  const buffer = new ArrayBuffer(1);
  const view = new DataView(buffer);
  let offset = 0;

  // Write discriminator (u8)
  view.setUint8(offset++, discriminator);

  return [new Uint8Array(buffer, 0, offset)];
}
