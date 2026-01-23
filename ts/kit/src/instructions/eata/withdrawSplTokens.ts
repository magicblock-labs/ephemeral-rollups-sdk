import { Address, Instruction, AccountMeta, AccountRole } from "@solana/kit";
import { EATA_PROGRAM_ID } from "../../constants";
import {
  ephemeralAtaPdaWithBumpFromOwnerAndMint,
  globalVaultPdaFromMint,
} from "../../pda";
import {
  findAssociatedTokenPda,
  TOKEN_PROGRAM_ADDRESS,
} from "@solana-program/token";

export interface WithdrawSplTokensAccounts {
  owner: Address;
  mint: Address;
  userDest: Address;
}

export interface WithdrawSplTokensArgs {
  amount: bigint;
}

export async function createWithdrawSplTokensInstruction(
  accounts: WithdrawSplTokensAccounts,
  args: WithdrawSplTokensArgs,
): Promise<Instruction> {
  const [ephemeralAta, bump] = await ephemeralAtaPdaWithBumpFromOwnerAndMint(
    accounts.owner,
    accounts.mint,
  );
  const vault = await globalVaultPdaFromMint(accounts.mint);
  const [vaultSource] = await findAssociatedTokenPda({
    mint: accounts.mint,
    owner: vault,
    tokenProgram: TOKEN_PROGRAM_ADDRESS,
  });

  const accountsMeta: AccountMeta[] = [
    { address: accounts.owner, role: AccountRole.READONLY_SIGNER },
    { address: ephemeralAta, role: AccountRole.WRITABLE },
    { address: vault, role: AccountRole.READONLY },
    { address: accounts.mint, role: AccountRole.READONLY },
    { address: vaultSource, role: AccountRole.WRITABLE },
    { address: accounts.userDest, role: AccountRole.WRITABLE },
    { address: TOKEN_PROGRAM_ADDRESS, role: AccountRole.READONLY },
  ];

  const buffer = new ArrayBuffer(10);
  const view = new DataView(buffer);
  let offset = 0;

  view.setUint8(offset++, 3);
  view.setBigUint64(offset, args.amount, true);
  offset += 8;
  view.setUint8(offset++, bump);

  return {
    accounts: accountsMeta,
    data: new Uint8Array(buffer, 0, offset),
    programAddress: EATA_PROGRAM_ID,
  };
}
