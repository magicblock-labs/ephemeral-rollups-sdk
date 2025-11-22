import {
  AccountMeta,
  Address,
  AccountRole,
  Instruction,
} from "@solana/kit";
import { DELEGATION_PROGRAM_ID } from "../../constants";
<<<<<<< HEAD

/**
 * TopUpEscrow instruction arguments
 */
export type TopUpEscrowInstructionArgs = {
=======
import { SYSTEM_PROGRAM_ADDRESS } from "@solana-program/system";

/**
 * TopUpEphemeralBalance instruction arguments
 */
export type TopUpEphemeralBalanceInstructionArgs = {
>>>>>>> jonasXchen/action-escrow
  amount: bigint;
  index?: number; // defaults to 255
};

/**
<<<<<<< HEAD
 * Instruction: TopUpEscrow
 * Discriminator: [9,0,0,0,0,0,0,0]
 */
export function createTopUpEscrowInstruction(
=======
 * Instruction: TopUpEphemeralBalance
 * Discriminator: [9,0,0,0,0,0,0,0]
 */
export function createTopUpEphemeralBalanceInstruction(
>>>>>>> jonasXchen/action-escrow
  accounts: {
    payer: Address;
    pubkey: Address;
    ephemeralBalanceAccount: Address;
<<<<<<< HEAD
    systemProgram: Address;
  },
  args: TopUpEscrowInstructionArgs
): Instruction {
  const [data] = serializeTopUpEscrowInstructionData(args);
=======
  },
  args: TopUpEphemeralBalanceInstructionArgs
): Instruction {
  const [data] = serializeTopUpEphemeralBalanceInstructionData(args);
>>>>>>> jonasXchen/action-escrow

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
<<<<<<< HEAD
      address: accounts.systemProgram,
=======
      address: SYSTEM_PROGRAM_ADDRESS,
>>>>>>> jonasXchen/action-escrow
      role: AccountRole.READONLY,
    },
  ];

  return {
    accounts: accounts_,
    data,
    programAddress: DELEGATION_PROGRAM_ID,
  };
}

<<<<<<< HEAD
export function serializeTopUpEscrowInstructionData(
  args: TopUpEscrowInstructionArgs
=======
export function serializeTopUpEphemeralBalanceInstructionData(
  args: TopUpEphemeralBalanceInstructionArgs
>>>>>>> jonasXchen/action-escrow
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
