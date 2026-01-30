import {
  PublicKey,
  TransactionInstruction,
  AccountMeta,
} from "@solana/web3.js";
import { PERMISSION_PROGRAM_ID } from "../../constants";
import { permissionPdaFromAccount } from "../../pda";
import {
  serializeMembersArgs,
  type MembersArgs,
} from "../../access-control/types";

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
export function createUpdatePermissionInstruction(
  accounts: {
    authority: [PublicKey, boolean];
    permissionedAccount: [PublicKey, boolean];
  },
  args: MembersArgs,
): TransactionInstruction {
  const permission = permissionPdaFromAccount(accounts.permissionedAccount[0]);

  const keys: AccountMeta[] = [
    {
      pubkey: accounts.authority[0],
      isWritable: false,
      isSigner: accounts.authority[1],
    },
    {
      pubkey: accounts.permissionedAccount[0],
      isWritable: false,
      isSigner: accounts.permissionedAccount[1],
    },
    { pubkey: permission, isWritable: true, isSigner: false },
  ];

  const argsBuffer = serializeMembersArgs(args);
  const instructionData = Buffer.from([...UPDATE_PERMISSION_DISCRIMINATOR, ...argsBuffer]);

  return new TransactionInstruction({
    programId: PERMISSION_PROGRAM_ID,
    keys,
    data: instructionData,
  });
}
