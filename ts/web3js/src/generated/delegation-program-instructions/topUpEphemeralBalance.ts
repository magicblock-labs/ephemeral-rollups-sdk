import { PublicKey, TransactionInstruction, AccountMeta } from "@solana/web3.js";
import { DELEGATION_PROGRAM_ID } from "../../constants";

/**
 * TopUpEscrow instruction arguments
 */
export type TopUpEscrowInstructionArgs = {
  amount: bigint;
  index?: number; // defaults to 255
};

/**
 * Instruction: TopUpEscrow
 * Discriminator: [9,0,0,0,0,0,0,0]
 */
export function createTopUpEscrowInstruction(
  accounts: {
    payer: PublicKey;
    pubkey: PublicKey;
    ephemeralBalanceAccount: PublicKey;
    systemProgram: PublicKey;
  },
  args: TopUpEscrowInstructionArgs,
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
    { pubkey: accounts.systemProgram, isWritable: false, isSigner: false },
  ];

  const data = serializeTopUpEscrowInstructionData(args);

  return new TransactionInstruction({
    programId,
    keys,
    data,
  });
}

export function serializeTopUpEscrowInstructionData(
  args: TopUpEscrowInstructionArgs
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
  buffer[offset] = args.index ?? 255;

  return buffer;
}
