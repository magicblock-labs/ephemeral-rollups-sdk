import { PublicKey, TransactionInstruction, SystemProgram } from "@solana/web3.js";
import { DELEGATION_PROGRAM_ID } from "../../constants";
import {
  createDelegateInstruction as _createDelegateInstruction,
} from "../../generated/delegation-program-instructions";

export interface DelegateInstructionData {
  commitFrequencyMs: number;
  validator?: PublicKey | null;
}

/**
 * Creates a delegate instruction with simplified parameters.
 * Delegation program and system program are automatically included.
 *
 * @param delegatedAccount - The delegated account
 * @param seeds - Array of seeds
 * @param ownerProgram - The owner program
 * @param payer - The payer account
 * @param delegateBuffer - The delegate buffer account
 * @param delegationRecord - The delegation record account
 * @param delegationMetadata - The delegation metadata account
 * @param data - Instruction data containing commitFrequencyMs and optional validator
 * @returns TransactionInstruction
 */
export function createDelegateInstruction(
  delegatedAccount: PublicKey,
  seeds: Uint8Array[],
  ownerProgram: PublicKey,
  payer: PublicKey,
  delegateBuffer: PublicKey,
  delegationRecord: PublicKey,
  delegationMetadata: PublicKey,
  data: DelegateInstructionData
): TransactionInstruction {
  return _createDelegateInstruction(
    {
      payer,
      delegatedAccount,
      ownerProgram,
      delegateBuffer,
      delegationRecord,
      delegationMetadata,
      systemProgram: SystemProgram.programId,
    },
    {
      commitFrequencyMs: data.commitFrequencyMs,
      seeds,
      validator: data.validator ?? null,
    },
    DELEGATION_PROGRAM_ID
  );
}
