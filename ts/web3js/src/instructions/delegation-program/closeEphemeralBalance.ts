import { PublicKey, TransactionInstruction } from "@solana/web3.js";
import { DELEGATION_PROGRAM_ID } from "../../constants";
import { createCloseEphemeralBalanceInstruction as _createCloseEphemeralBalanceInstruction } from "../../generated/delegation-program-instructions";

/**
 * Creates a closeEscrow instruction with simplified parameters.
 * System program is automatically included.
 *
 * @param escrow - The escrow account
 * @param escrowAuthority - The escrowAuthority account
 * @param index - Optional index (defaults to 255)
 * @returns TransactionInstruction
 */
export function createCloseEscrowInstruction(
  escrow: PublicKey,
  escrowAuthority: PublicKey,
  index?: number,
): TransactionInstruction {
  return _createCloseEphemeralBalanceInstruction(
    {
      payer: escrowAuthority,
      ephemeralBalanceAccount: escrow,
    },
    {
      index: index ?? 255,
    },
    DELEGATION_PROGRAM_ID,
  );
}
