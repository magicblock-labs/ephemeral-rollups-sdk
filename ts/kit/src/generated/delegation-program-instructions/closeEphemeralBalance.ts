import {
  AccountMeta,
  Address,
  AccountRole,
  Instruction,
} from "@solana/kit";
import { DELEGATION_PROGRAM_ID } from "../../constants";

/**
 * CloseEscrow instruction arguments
 */
export type CloseEscrowInstructionArgs = {
  index?: number; // defaults to 255
};

/**
 * Instruction: CloseEscrow
 * Discriminator: [11,0,0,0,0,0,0,0]
 */
export function createCloseEscrowInstruction(
  accounts: {
    payer: Address;
    ephemeralBalanceAccount: Address;
    systemProgram: Address;
  },
  args?: CloseEscrowInstructionArgs
): Instruction {
  const [data] = serializeCloseEscrowInstructionData(args ?? {});

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

export function serializeCloseEscrowInstructionData(
  args?: CloseEscrowInstructionArgs
): [Uint8Array] {
  const discriminator = [11, 0, 0, 0, 0, 0, 0, 0];
  const data = new Uint8Array(9);
  let offset = 0;

  // Write discriminator
  for (let i = 0; i < 8; i++) {
    data[offset++] = discriminator[i];
  }

  // Write index as u8
  data[offset] = args?.index ?? 255;

  return [data];
}
