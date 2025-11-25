import { PublicKey, TransactionInstruction } from "@solana/web3.js";
import { MAGIC_PROGRAM_ID, MAGIC_CONTEXT_ID } from "../../constants";

/**
 * Creates a scheduleCommit instruction for the Magic Program.
 * Schedules the provided accounts to be committed.
 *
 * @param payer - The payer account (must be signer)
 * @param accountsToBCommit - Array of account addresses to be committed
 * @returns TransactionInstruction
 */
export function createCommitInstruction(
  payer: PublicKey,
  accountsToCommit: PublicKey[],
): TransactionInstruction {
  const accounts = [
    { pubkey: payer, isSigner: true, isWritable: true },
    { pubkey: MAGIC_CONTEXT_ID, isSigner: false, isWritable: true },
    ...accountsToCommit.map((account) => ({
      pubkey: account,
      isSigner: false,
      isWritable: false,
    })),
  ];

  // ScheduleCommit instruction discriminator
  const data = Buffer.alloc(4);
  data.writeUInt32LE(1, 0);

  return new TransactionInstruction({
    keys: accounts,
    programId: MAGIC_PROGRAM_ID,
    data,
  });
}
