import { Address, Instruction, AccountMeta, AccountRole } from "@solana/kit";
import {
  EATA_PROGRAM_ID,
  MAGIC_CONTEXT_ID,
  MAGIC_PROGRAM_ID,
} from "../../constants";
import { ephemeralAtaPdaFromOwnerAndMint } from "pda";
import {
  findAssociatedTokenPda,
  TOKEN_PROGRAM_ADDRESS,
} from "@solana-program/token";

export interface UndelegateEphemeralAtaAccounts {
  payer: Address;
  owner: Address;
  mint: Address;
}

export async function createUndelegateEphemeralAtaInstruction(
  accounts: UndelegateEphemeralAtaAccounts,
): Promise<Instruction> {
  const [ata] = await findAssociatedTokenPda({
    mint: accounts.mint,
    owner: accounts.owner,
    tokenProgram: TOKEN_PROGRAM_ADDRESS,
  });
  const ephemeralAta = await ephemeralAtaPdaFromOwnerAndMint(
    accounts.owner,
    accounts.mint,
  );
  const accountsMeta: AccountMeta[] = [
    { address: accounts.payer, role: AccountRole.READONLY_SIGNER },
    { address: ata, role: AccountRole.WRITABLE },
    { address: ephemeralAta, role: AccountRole.WRITABLE },
    { address: MAGIC_CONTEXT_ID, role: AccountRole.WRITABLE },
    { address: MAGIC_PROGRAM_ID, role: AccountRole.READONLY },
  ];

  const data = new Uint8Array([5]);

  return {
    accounts: accountsMeta,
    data,
    programAddress: EATA_PROGRAM_ID,
  };
}
