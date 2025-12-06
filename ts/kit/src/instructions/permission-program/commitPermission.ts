import {
  Address,
  Instruction,
  AccountMeta,
  AccountRole,
} from "@solana/kit";
import { PERMISSION_PROGRAM_ID, MAGIC_PROGRAM_ID, MAGIC_CONTEXT_ID } from "../../constants";
import { commitStatePdaFromDelegatedAccount } from "../../pda";

/**
 * CommitPermission instruction arguments
 */
// eslint-disable-next-line @typescript-eslint/no-empty-interface
export interface CommitPermissionInstructionArgs {}

/**
 * Instruction: CommitPermission
 * Discriminator: 5
 */
export async function createCommitPermissionInstruction(
  accounts: {
    delegatedAccount: Address;
    permission: Address;
    permissionProgram?: Address;
  },
  args?: CommitPermissionInstructionArgs,
): Promise<Instruction> {

  const accountsMeta: AccountMeta[] = [
    { address: accounts.delegatedAccount, role: AccountRole.WRITABLE_SIGNER },
    { address: accounts.permission, role: AccountRole.WRITABLE },
    { address: MAGIC_PROGRAM_ID, role: AccountRole.READONLY },
    { address: MAGIC_CONTEXT_ID, role: AccountRole.WRITABLE },
  ];

  const [instructionData] = serializeCommitPermissionInstructionData(args);

  return {
    accounts: accountsMeta,
    data: instructionData,
    programAddress: accounts.permissionProgram ?? PERMISSION_PROGRAM_ID,
  };
}

export function serializeCommitPermissionInstructionData(
  args?: CommitPermissionInstructionArgs,
): [Uint8Array] {
  const discriminator = 5;
  const buffer = new ArrayBuffer(1);
  const view = new DataView(buffer);
  let offset = 0;

  // Write discriminator (u8)
  view.setUint8(offset++, discriminator);

  return [new Uint8Array(buffer, 0, offset)];
}
