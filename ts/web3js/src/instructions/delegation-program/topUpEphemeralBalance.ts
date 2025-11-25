import {
  PublicKey,
  TransactionInstruction,
  SystemProgram,
  AccountMeta,
} from "@solana/web3.js";
import { DELEGATION_PROGRAM_ID } from "../../constants";

/**
 * TopUpEphemeralBalance instruction arguments
 */
export interface TopUpEphemeralBalanceInstructionArgs {
  amount: bigint;
  index: number;
}

/**
 * Instruction: TopUpEphemeralBalance
 * Discriminator: [9,0,0,0,0,0,0,0]
 */
export function createTopUpEscrowInstruction(
  escrow: PublicKey,
  escrowAuthority: PublicKey,
  payer: PublicKey,
  amount: number,
  index?: number,
): TransactionInstruction {
  const keys: AccountMeta[] = [
    { pubkey: payer, isWritable: true, isSigner: true },
    { pubkey: escrowAuthority, isWritable: false, isSigner: false },
    { pubkey: escrow, isWritable: true, isSigner: false },
    { pubkey: SystemProgram.programId, isWritable: false, isSigner: false },
  ];

  const instructionData = serializeTopUpEphemeralBalanceInstructionData({
    amount: BigInt(amount),
    index: index ?? 255,
  });

  return new TransactionInstruction({
    programId: DELEGATION_PROGRAM_ID,
    keys,
    data: instructionData,
  });
}

export function serializeTopUpEphemeralBalanceInstructionData(
  args: TopUpEphemeralBalanceInstructionArgs,
): Buffer {
  const discriminator = [9, 0, 0, 0, 0, 0, 0, 0];
  const buffer = Buffer.alloc(17);
  let offset = 0;

  // Write discriminator
  for (let i = 0; i < 8; i++) {
    buffer[offset++] = discriminator[i];
  }

  // Write amount (u64)
  buffer.writeBigUInt64LE(args.amount, offset);
  offset += 8;

  // Write index (u8)
  buffer[offset] = args.index;

  return buffer;
}
