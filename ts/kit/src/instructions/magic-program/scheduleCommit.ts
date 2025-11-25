import { Address, Instruction } from "@solana/kit";
import { AccountRole } from "@solana/instructions";
import { MAGIC_PROGRAM_ID, MAGIC_CONTEXT_ID } from "../../constants";

/**
 * Creates a scheduleCommit instruction for the Magic Program.
 * Schedules the provided accounts to be committed.
 *
 * @param payer - The payer account (must be signer)
 * @param accountsToCommit - Array of account addresses to be committed
 * @returns Instruction
 */
export function createCommitInstruction(
  payer: Address,
  accountsToCommit: Address[],
): Instruction {
  const accounts = [
    { address: payer, role: AccountRole.WRITABLE_SIGNER },
    { address: MAGIC_CONTEXT_ID, role: AccountRole.WRITABLE },
    ...accountsToCommit.map((account) => ({
      address: account,
      role: AccountRole.READONLY,
    })),
  ] as const;

  // ScheduleCommit instruction discriminator
  const data = new Uint8Array(4);
  const view = new DataView(data.buffer);
  view.setUint32(0, 1, true);

  return {
    accounts,
    programAddress: MAGIC_PROGRAM_ID,
    data,
  };
}
