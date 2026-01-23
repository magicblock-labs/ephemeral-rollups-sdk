import {
  PublicKey,
  TransactionInstruction,
  SystemProgram,
  AccountMeta,
} from "@solana/web3.js";
import { EATA_PROGRAM_ID } from "../../constants";
import { globalVaultPdaWithBumpFromMint } from "../../pda";

export interface InitializeGlobalVaultAccounts {
  payer: PublicKey;
  mint: PublicKey;
}

export function createInitializeGlobalVaultInstruction(
  accounts: InitializeGlobalVaultAccounts,
): TransactionInstruction {
  const [vault, bump] = globalVaultPdaWithBumpFromMint(accounts.mint);

  const keys: AccountMeta[] = [
    { pubkey: vault, isWritable: true, isSigner: false },
    { pubkey: accounts.payer, isWritable: true, isSigner: true },
    { pubkey: accounts.mint, isWritable: false, isSigner: false },
    { pubkey: SystemProgram.programId, isWritable: false, isSigner: false },
  ];

  const buffer = Buffer.alloc(2);
  buffer[0] = 1; // discriminator
  buffer[1] = bump;

  return new TransactionInstruction({
    programId: EATA_PROGRAM_ID,
    keys,
    data: buffer,
  });
}
