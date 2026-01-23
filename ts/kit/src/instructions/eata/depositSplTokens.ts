import { Address, Instruction, AccountMeta, AccountRole } from "@solana/kit";
import { EATA_PROGRAM_ID } from "../../constants";
import {
  ephemeralAtaPdaFromOwnerAndMint,
  globalVaultPdaFromMint,
} from "../../pda";
import {
  findAssociatedTokenPda,
  TOKEN_PROGRAM_ADDRESS,
} from "@solana-program/token";

export interface DepositSplTokensAccounts {
  owner: Address;
  mint: Address;
  sourceToken: Address;
  authority: Address;
}

export interface DepositSplTokensArgs {
  amount: bigint;
}

export async function createDepositSplTokensInstruction(
  accounts: DepositSplTokensAccounts,
  args: DepositSplTokensArgs,
): Promise<Instruction> {
  const ephemeralAta = await ephemeralAtaPdaFromOwnerAndMint(
    accounts.owner,
    accounts.mint,
  );
  const vault = await globalVaultPdaFromMint(accounts.mint);
  const [vaultToken] = await findAssociatedTokenPda({
    mint: accounts.mint,
    owner: vault,
    tokenProgram: TOKEN_PROGRAM_ADDRESS,
  });

  const accountsMeta: AccountMeta[] = [
    { address: ephemeralAta, role: AccountRole.WRITABLE },
    { address: vault, role: AccountRole.READONLY },
    { address: accounts.mint, role: AccountRole.READONLY },
    { address: accounts.sourceToken, role: AccountRole.WRITABLE },
    { address: vaultToken, role: AccountRole.WRITABLE },
    { address: accounts.authority, role: AccountRole.READONLY_SIGNER },
    { address: TOKEN_PROGRAM_ADDRESS, role: AccountRole.READONLY },
  ];

  const buffer = new ArrayBuffer(9);
  const view = new DataView(buffer);
  let offset = 0;

  view.setUint8(offset++, 2);
  view.setBigUint64(offset, args.amount, true);
  offset += 8;

  return {
    accounts: accountsMeta,
    data: new Uint8Array(buffer, 0, offset),
    programAddress: EATA_PROGRAM_ID,
  };
}
