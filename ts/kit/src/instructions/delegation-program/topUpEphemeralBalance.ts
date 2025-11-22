import { Address, Instruction } from "@solana/kit";
import { DELEGATION_PROGRAM_ID } from "../../constants";
import {
  createTopUpEphemeralBalanceInstruction as _createTopUpEphemeralBalanceInstruction,
} from "../../generated/delegation-program-instructions";

/**
 * Creates a topUpEscrow instruction with simplified parameters.
 * System program is automatically included.
 *
 * @param escrow - The escrow account
 * @param escrowAuthority - The escrowAuthority account
 * @param payer - The payer account
 * @param amount - The amount to top up
 * @param index - Optional index (defaults to 255)
 * @returns Instruction
 */
export function createTopUpEscrowInstruction(
  escrow: Address,
  escrowAuthority: Address,
  payer: Address,
  amount: number,
  index?: number
): Instruction {
  return _createTopUpEphemeralBalanceInstruction(
    {
      payer,
      pubkey: escrowAuthority,
      ephemeralBalanceAccount: escrow,
    },
    {
      amount: BigInt(amount),
      index: index ?? 255,
    }
  );
}
