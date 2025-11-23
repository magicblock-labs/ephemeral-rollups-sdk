import { PublicKey, TransactionInstruction, AccountMeta, SystemProgram } from "@solana/web3.js";
import { DELEGATION_PROGRAM_ID } from "../../constants";

/**
 * TopUpEphemeralBalance instruction arguments
 */
export type TopUpEphemeralBalanceInstructionArgs = {
  amount: bigint;
  index: number;
};

/**
 * Instruction: TopUpEphemeralBalance
 * Discriminator: [9,0,0,0,0,0,0,0]
 */
export function createTopUpEphemeralBalanceInstruction(
  accounts: {
    payer: PublicKey;
    pubkey: PublicKey;
    ephemeralBalanceAccount: PublicKey;
  },
  args: TopUpEphemeralBalanceInstructionArgs,
  programId = DELEGATION_PROGRAM_ID
): TransactionInstruction {
  const keys: AccountMeta[] = [
    { pubkey: accounts.payer, isWritable: true, isSigner: true },
    { pubkey: accounts.pubkey, isWritable: false, isSigner: false },
    {
      pubkey: accounts.ephemeralBalanceAccount,
      isWritable: true,
      isSigner: false,
    },
    { pubkey: SystemProgram.programId, isWritable: false, isSigner: false },
  ];

  const data = serializeTopUpEphemeralBalanceInstructionData(args);

  return new TransactionInstruction({
    programId,
    keys,
    data,
  });
}

export function serializeTopUpEphemeralBalanceInstructionData(
  args: TopUpEphemeralBalanceInstructionArgs
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
