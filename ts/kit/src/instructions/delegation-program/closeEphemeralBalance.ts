import { Address, Instruction } from "@solana/kit";
import { createCloseEphemeralBalanceInstruction as _createCloseEphemeralBalanceInstruction } from "../../generated/delegation-program-instructions";

/**
 * Creates a closeEscrow instruction with simplified parameters.
 * System program is automatically included.
 *
 * @param escrow - The escrow account
 * @param escrowAuthority - The escrowAuthority account
 * @param index - Optional index (defaults to 255)
 * @returns Instruction
 */
export function createCloseEscrowInstruction(
  escrow: Address,
  escrowAuthority: Address,
  index?: number,
): Instruction {
  return _createCloseEphemeralBalanceInstruction(
    {
      payer: escrowAuthority,
      ephemeralBalanceAccount: escrow,
    },
    {
      index: index ?? 255,
    },
  );
}
