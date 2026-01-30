import {
  PublicKey,
  TransactionInstruction,
  SystemProgram,
  AccountMeta,
} from "@solana/web3.js";
import { PERMISSION_PROGRAM_ID } from "../../constants";
import { permissionPdaFromAccount } from "../../pda";
import {
  serializeMembersArgs,
  type MembersArgs,
} from "../../access-control/types";

export const CREATE_PERMISSION_DISCRIMINATOR = [0, 0, 0, 0, 0, 0, 0, 0];

/**
 * Instruction: CreatePermission
 * Discriminator: [0, 0, 0, 0, 0, 0, 0, 0]
 */
export function createCreatePermissionInstruction(
  accounts: {
    permissionedAccount: PublicKey;
    payer: PublicKey;
  },
  args: MembersArgs,
): TransactionInstruction {
  const permission = permissionPdaFromAccount(accounts.permissionedAccount);

  const keys: AccountMeta[] = [
    { pubkey: accounts.permissionedAccount, isWritable: false, isSigner: true },
    { pubkey: permission, isWritable: true, isSigner: false },
    { pubkey: accounts.payer, isWritable: true, isSigner: true },
    { pubkey: SystemProgram.programId, isWritable: false, isSigner: false },
  ];

  const argsBuffer = serializeMembersArgs(args);
  const instructionData = Buffer.from([...CREATE_PERMISSION_DISCRIMINATOR, ...argsBuffer]);

  return new TransactionInstruction({
    programId: PERMISSION_PROGRAM_ID,
    keys,
    data: instructionData,
  });
}
