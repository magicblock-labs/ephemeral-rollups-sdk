import {
  PublicKey,
  TransactionInstruction,
  AccountMeta,
} from "@solana/web3.js";
import { EATA_PROGRAM_ID } from "../../constants";
import {
  ephemeralAtaPdaFromOwnerAndMint,
  globalVaultPdaFromMint,
} from "../../pda";
import {
  getAssociatedTokenAddressSync,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";

export interface DepositSplTokensAccounts {
  user: PublicKey;
  mint: PublicKey;
  sourceToken: PublicKey;
  authority: PublicKey;
}

export interface DepositSplTokensArgs {
  amount: bigint;
}

export function createDepositSplTokensInstruction(
  accounts: DepositSplTokensAccounts,
  args: DepositSplTokensArgs,
): TransactionInstruction {
  const ephemeralAta = ephemeralAtaPdaFromOwnerAndMint(
    accounts.user,
    accounts.mint,
  );
  const vault = globalVaultPdaFromMint(accounts.mint);
  const vaultToken = getAssociatedTokenAddressSync(accounts.mint, vault);

  const keys: AccountMeta[] = [
    { pubkey: ephemeralAta, isWritable: true, isSigner: false },
    { pubkey: vault, isWritable: false, isSigner: false },
    { pubkey: accounts.mint, isWritable: false, isSigner: false },
    { pubkey: accounts.sourceToken, isWritable: true, isSigner: false },
    { pubkey: vaultToken, isWritable: true, isSigner: false },
    { pubkey: accounts.authority, isWritable: false, isSigner: true },
    { pubkey: TOKEN_PROGRAM_ID, isWritable: false, isSigner: false },
  ];

  const buffer = Buffer.alloc(9);
  buffer[0] = 2; // discriminator
  buffer.writeBigUInt64LE(args.amount, 1);

  return new TransactionInstruction({
    programId: EATA_PROGRAM_ID,
    keys,
    data: buffer,
  });
}
