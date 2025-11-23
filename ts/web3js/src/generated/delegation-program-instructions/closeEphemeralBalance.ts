import { PublicKey, TransactionInstruction, AccountMeta, SystemProgram } from "@solana/web3.js";
import { DELEGATION_PROGRAM_ID } from "../../constants";

/**
 * CloseEphemeralBalance instruction arguments
 */
export type CloseEphemeralBalanceInstructionArgs = {
  index: number;
};

/**
 * Instruction: CloseEphemeralBalance
 * Discriminator: [11,0,0,0,0,0,0,0]
 */
export function createCloseEphemeralBalanceInstruction(
  accounts: {
    payer: PublicKey;
    ephemeralBalanceAccount: PublicKey;
  },
  args: CloseEphemeralBalanceInstructionArgs,
  programId = DELEGATION_PROGRAM_ID
): TransactionInstruction {
  const keys: AccountMeta[] = [
    { pubkey: accounts.payer, isWritable: false, isSigner: true },
    { pubkey: accounts.ephemeralBalanceAccount, isWritable: true, isSigner: false },
    { pubkey: SystemProgram.programId, isWritable: false, isSigner: false },
  ];

  const data = serializeCloseEphemeralBalanceInstructionData(args);

  return new TransactionInstruction({
    programId,
    keys,
    data,
  });
}

export function serializeCloseEphemeralBalanceInstructionData(
  args: CloseEphemeralBalanceInstructionArgs
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
