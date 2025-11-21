import {
  AccountMeta,
  Address,
  AccountRole,
  Instruction,
} from "@solana/kit";
import { DELEGATION_PROGRAM_ID } from "../../constants";

/**
 * CloseEphemeralBalance instruction arguments
 */
export type CloseEphemeralBalanceInstructionArgs = {
  index: number;
};

/**
 * Instruction: CloseEphemeralBalance
 * Discriminator: [11,0,0,0,0,0,0,0]
 */
export function createCloseEphemeralBalanceInstruction(
  accounts: {
    payer: Address;
    ephemeralBalanceAccount: Address;
    systemProgram: Address;
  },
  args: CloseEphemeralBalanceInstructionArgs
): Instruction {
  const [data] = serializeCloseEphemeralBalanceInstructionData(args);

  const accounts_: AccountMeta[] = [
    {
      address: accounts.payer,
      role: AccountRole.READONLY_SIGNER,
    },
    {
      address: accounts.ephemeralBalanceAccount,
      role: AccountRole.WRITABLE,
    },
    {
      address: accounts.systemProgram,
      role: AccountRole.READONLY,
    },
  ];

  return {
    accounts: accounts_,
    data,
    programAddress: DELEGATION_PROGRAM_ID,
  };
}

export function serializeCloseEphemeralBalanceInstructionData(
  args: CloseEphemeralBalanceInstructionArgs
): [Uint8Array] {
  const discriminator = [11, 0, 0, 0, 0, 0, 0, 0];
  const data = new Uint8Array(9);
  let offset = 0;

  // Write discriminator
  for (let i = 0; i < 8; i++) {
    data[offset++] = discriminator[i];
  }

  // Write index as u8
  data[offset] = args.index;

  return [data];
}
