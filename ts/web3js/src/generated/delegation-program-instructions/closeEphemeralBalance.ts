import { PublicKey, TransactionInstruction, AccountMeta } from "@solana/web3.js";
import { DELEGATION_PROGRAM_ID } from "../../constants";

/**
 * CloseEscrow instruction arguments
 */
export type CloseEscrowInstructionArgs = {
  index?: number; // defaults to 255
};

/**
 * Instruction: CloseEscrow
 * Discriminator: [11,0,0,0,0,0,0,0]
 */
export function createCloseEscrowInstruction(
  accounts: {
    payer: PublicKey;
    ephemeralBalanceAccount: PublicKey;
    systemProgram: PublicKey;
  },
  args?: CloseEscrowInstructionArgs,
  programId = DELEGATION_PROGRAM_ID
): TransactionInstruction {
  const keys: AccountMeta[] = [
    { pubkey: accounts.payer, isWritable: false, isSigner: true },
    { pubkey: accounts.ephemeralBalanceAccount, isWritable: true, isSigner: false },
    { pubkey: accounts.systemProgram, isWritable: false, isSigner: false },
  ];

  const data = serializeCloseEscrowInstructionData(args ?? {});

  return new TransactionInstruction({
    programId,
    keys,
    data,
  });
}

export function serializeCloseEscrowInstructionData(
  args?: CloseEscrowInstructionArgs
): Buffer {
  const discriminator = [11, 0, 0, 0, 0, 0, 0, 0];
  const buffer = Buffer.alloc(9);
  let offset = 0;

  // Write discriminator
  for (let i = 0; i < 8; i++) {
    buffer[offset++] = discriminator[i];
  }

  // Write index (u8)
  buffer[offset] = args.index ?? 255;

  return buffer;
}
