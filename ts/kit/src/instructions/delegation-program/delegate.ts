import { Address, Instruction } from "@solana/kit";
import { SYSTEM_PROGRAM_ADDRESS } from "@solana-program/system";
import { createDelegateInstruction as _createDelegateInstruction } from "../../generated/delegation-program-instructions";

export interface DelegateInstructionData {
  commitFrequencyMs: number;
  validator?: Address | null;
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
 * @returns Instruction
 */
export function createDelegateInstruction(
  delegatedAccount: Address,
  seeds: Uint8Array[],
  ownerProgram: Address,
  payer: Address,
  delegateBuffer: Address,
  delegationRecord: Address,
  delegationMetadata: Address,
  data: DelegateInstructionData,
): Instruction {
  return _createDelegateInstruction(
    {
      payer,
      delegatedAccount,
      ownerProgram,
      delegateBuffer,
      delegationRecord,
      delegationMetadata,
      systemProgram: SYSTEM_PROGRAM_ADDRESS,
    },
    {
      commitFrequencyMs: data.commitFrequencyMs,
      seeds,
      validator: data.validator ?? null,
    },
  );
}
