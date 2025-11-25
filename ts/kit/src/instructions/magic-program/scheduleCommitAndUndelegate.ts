import { Address, Instruction } from "@solana/kit";
import { AccountRole } from "@solana/instructions";
import { MAGIC_PROGRAM_ID, MAGIC_CONTEXT_ID } from "../../constants";

/**
 * Creates a scheduleCommitAndUndelegate instruction for the Magic Program.
 * Schedules the provided accounts to be committed and undelegated.
 * The accounts will no longer be considered delegated after this instruction.
 *
 * @param payer - The payer account (must be signer)
 * @param accountsToCommitAndUndelegate - Array of account addresses to be committed and undelegated
 * @returns Instruction
 */
export function createCommitAndUndelegateInstruction(
  payer: Address,
  accountsToCommitAndUndelegate: Address[],
): Instruction {
  const accounts = [
    { address: payer, role: AccountRole.WRITABLE_SIGNER },
    { address: MAGIC_CONTEXT_ID, role: AccountRole.WRITABLE },
    ...accountsToCommitAndUndelegate.map((account) => ({
      address: account,
      role: AccountRole.READONLY,
    })),
  ] as const;

  // ScheduleCommitAndUndelegate instruction discriminator
  const data = new Uint8Array(4);
  const view = new DataView(data.buffer);
  view.setUint32(0, 2, true);

  return {
    accounts,
    programAddress: MAGIC_PROGRAM_ID,
    data,
  };
}
