import {
  PublicKey,
  TransactionInstruction,
  AccountMeta,
} from "@solana/web3.js";
import { EATA_PROGRAM_ID } from "../../constants";
import {
  ephemeralAtaPdaWithBumpFromOwnerAndMint,
  globalVaultPdaWithBumpFromMint,
} from "../../pda";
import {
  getAssociatedTokenAddressSync,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";

export interface WithdrawSplTokensAccounts {
  owner: PublicKey;
  mint: PublicKey;
  userDest: PublicKey;
}

export interface WithdrawSplTokensArgs {
  amount: bigint;
}

export function createWithdrawSplTokensInstruction(
  accounts: WithdrawSplTokensAccounts,
  args: WithdrawSplTokensArgs,
): TransactionInstruction {
  const [ephemeralAta, bump] = ephemeralAtaPdaWithBumpFromOwnerAndMint(
    accounts.owner,
    accounts.mint,
  );
  const vault = globalVaultPdaWithBumpFromMint(accounts.mint)[0];
  const vaultSource = getAssociatedTokenAddressSync(accounts.mint, vault);

  const keys: AccountMeta[] = [
    { pubkey: accounts.owner, isWritable: false, isSigner: true },
    { pubkey: ephemeralAta, isWritable: true, isSigner: false },
    { pubkey: vault, isWritable: false, isSigner: false },
    { pubkey: accounts.mint, isWritable: false, isSigner: false },
    { pubkey: vaultSource, isWritable: true, isSigner: false },
    { pubkey: accounts.userDest, isWritable: true, isSigner: false },
    { pubkey: TOKEN_PROGRAM_ID, isWritable: false, isSigner: false },
  ];

  const buffer = Buffer.alloc(10);
  buffer[0] = 3; // discriminator
  buffer.writeBigUInt64LE(args.amount, 1);
  buffer[9] = bump;

  return new TransactionInstruction({
    programId: EATA_PROGRAM_ID,
    keys,
    data: buffer,
  });
}
