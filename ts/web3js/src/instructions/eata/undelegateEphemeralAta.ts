import {
  PublicKey,
  TransactionInstruction,
  AccountMeta,
} from "@solana/web3.js";
import {
  EATA_PROGRAM_ID,
  MAGIC_PROGRAM_ID,
  MAGIC_CONTEXT_ID,
} from "../../constants";
import { ephemeralAtaPdaWithBumpFromOwnerAndMint } from "../../pda";

export interface UndelegateEphemeralAtaAccounts {
  payer: PublicKey;
  user: PublicKey;
  mint: PublicKey;
}

export function createUndelegateEphemeralAtaInstruction(
  accounts: UndelegateEphemeralAtaAccounts,
): TransactionInstruction {
  const [ephemeralAta] = ephemeralAtaPdaWithBumpFromOwnerAndMint(
    accounts.user,
    accounts.mint,
  );

  const keys: AccountMeta[] = [
    { pubkey: accounts.payer, isWritable: false, isSigner: true },
    { pubkey: ephemeralAta, isWritable: true, isSigner: false },
    { pubkey: ephemeralAta, isWritable: true, isSigner: false },
    { pubkey: MAGIC_CONTEXT_ID, isWritable: true, isSigner: false },
    { pubkey: MAGIC_PROGRAM_ID, isWritable: false, isSigner: false },
  ];

  const buffer = Buffer.alloc(1);
  buffer[0] = 5; // discriminator

  return new TransactionInstruction({
    programId: EATA_PROGRAM_ID,
    keys,
    data: buffer,
  });
}
