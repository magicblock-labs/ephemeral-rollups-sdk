import { AccountMeta, Address, AccountRole, Instruction } from "@solana/kit";
import { DELEGATION_PROGRAM_ID } from "../../constants";
import { SYSTEM_PROGRAM_ADDRESS } from "@solana-program/system";

/**
 * TopUpEphemeralBalance instruction arguments
 */
export interface TopUpEphemeralBalanceInstructionArgs {
  amount: bigint;
  index?: number; // defaults to 255
}

/**
 * Instruction: TopUpEphemeralBalance
 * Discriminator: [9,0,0,0,0,0,0,0]
 */
export function createTopUpEphemeralBalanceInstruction(
  accounts: {
    payer: Address;
    pubkey: Address;
    ephemeralBalanceAccount: Address;
  },
  args: TopUpEphemeralBalanceInstructionArgs,
): Instruction {
  const [data] = serializeTopUpEphemeralBalanceInstructionData(args);

  const accounts_: AccountMeta[] = [
    {
      address: accounts.payer,
      role: AccountRole.WRITABLE_SIGNER,
    },
    {
      address: accounts.pubkey,
      role: AccountRole.READONLY,
    },
    {
      address: accounts.ephemeralBalanceAccount,
      role: AccountRole.WRITABLE,
    },
    {
      address: SYSTEM_PROGRAM_ADDRESS,
      role: AccountRole.READONLY,
    },
  ];

  return {
    accounts: accounts_,
    data,
    programAddress: DELEGATION_PROGRAM_ID,
  };
}

export function serializeTopUpEphemeralBalanceInstructionData(
  args: TopUpEphemeralBalanceInstructionArgs,
): [Uint8Array] {
  const discriminator = [9, 0, 0, 0, 0, 0, 0, 0];
  const data = new Uint8Array(17);
  let offset = 0;

  // Write discriminator
  for (let i = 0; i < 8; i++) {
    data[offset++] = discriminator[i];
  }

  // Write amount as u64 little-endian
  const amountView = new DataView(data.buffer, offset, 8);
  amountView.setBigUint64(0, args.amount, true);
  offset += 8;

  // Write index as u8
  data[offset] = args.index ?? 255;

  return [data];
}
