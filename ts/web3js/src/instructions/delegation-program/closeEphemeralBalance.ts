import {
  PublicKey,
  TransactionInstruction,
  SystemProgram,
  AccountMeta,
} from "@solana/web3.js";
import { DELEGATION_PROGRAM_ID } from "../../constants";

/**
 * CloseEphemeralBalance instruction arguments
 */
export interface CloseEphemeralBalanceInstructionArgs {
  index: number;
}

/**
 * Instruction: CloseEphemeralBalance
 * Discriminator: [11,0,0,0,0,0,0,0]
 */
export function createCloseEscrowInstruction(
  escrow: PublicKey,
  escrowAuthority: PublicKey,
  index?: number,
): TransactionInstruction {
  const keys: AccountMeta[] = [
    { pubkey: escrowAuthority, isWritable: false, isSigner: true },
    { pubkey: escrow, isWritable: true, isSigner: false },
    { pubkey: SystemProgram.programId, isWritable: false, isSigner: false },
  ];

  const instructionData = serializeCloseEphemeralBalanceInstructionData({
    index: index ?? 255,
  });

  return new TransactionInstruction({
    programId: DELEGATION_PROGRAM_ID,
    keys,
    data: instructionData,
  });
}

export function serializeCloseEphemeralBalanceInstructionData(
  args: CloseEphemeralBalanceInstructionArgs,
): Buffer {
  const discriminator = [11, 0, 0, 0, 0, 0, 0, 0];
  const buffer = Buffer.alloc(9);
  let offset = 0;

  // Write discriminator
  for (let i = 0; i < 8; i++) {
    buffer[offset++] = discriminator[i];
  }

  // Write index (u8)
  buffer[offset] = args.index;

  return buffer;
}
