import { PublicKey, TransactionInstruction } from "@solana/web3.js";
import { MAGIC_PROGRAM_ID, MAGIC_CONTEXT_ID } from "../../constants";

/**
 * Creates a scheduleCommitAndUndelegate instruction for the Magic Program.
 * Schedules the provided accounts to be committed and undelegated.
 * The accounts will no longer be considered delegated after this instruction.
 *
 * @param payer - The payer account (must be signer)
 * @param accountsToCommitAndUndelegate - Array of account addresses to be committed and undelegated
 * @returns TransactionInstruction
 */
export function createCommitAndUndelegateInstruction(
  payer: PublicKey,
  accountsToCommitAndUndelegate: PublicKey[],
): TransactionInstruction {
  const accounts = [
    { pubkey: payer, isSigner: true, isWritable: true },
    { pubkey: MAGIC_CONTEXT_ID, isSigner: false, isWritable: true },
    ...accountsToCommitAndUndelegate.map((account) => ({
      pubkey: account,
      isSigner: false,
      isWritable: false,
    })),
  ];

  // ScheduleCommitAndUndelegate instruction discriminator
  const data = Buffer.alloc(4);
  data.writeUInt32LE(2, 0);

  return new TransactionInstruction({
    keys: accounts,
    programId: MAGIC_PROGRAM_ID,
    data,
  });
}
