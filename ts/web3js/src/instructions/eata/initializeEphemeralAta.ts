import {
  PublicKey,
  TransactionInstruction,
  SystemProgram,
  AccountMeta,
} from "@solana/web3.js";
import { EATA_PROGRAM_ID } from "../../constants";
import { ephemeralAtaPdaWithBumpFromOwnerAndMint } from "../../pda";

export interface InitializeEphemeralAtaAccounts {
  payer: PublicKey;
  user: PublicKey;
  mint: PublicKey;
}

export function createInitializeEphemeralAtaInstruction(
  accounts: InitializeEphemeralAtaAccounts,
): TransactionInstruction {
  const [ephemeralAta, bump] = ephemeralAtaPdaWithBumpFromOwnerAndMint(
    accounts.user,
    accounts.mint,
  );

  const keys: AccountMeta[] = [
    { pubkey: ephemeralAta, isWritable: true, isSigner: false },
    { pubkey: accounts.payer, isWritable: true, isSigner: true },
    { pubkey: accounts.user, isWritable: false, isSigner: false },
    { pubkey: accounts.mint, isWritable: false, isSigner: false },
    { pubkey: SystemProgram.programId, isWritable: false, isSigner: false },
  ];

  const buffer = Buffer.alloc(2);
  buffer[0] = 0; // discriminator
  buffer[1] = bump;

  return new TransactionInstruction({
    programId: EATA_PROGRAM_ID,
    keys,
    data: buffer,
  });
}
