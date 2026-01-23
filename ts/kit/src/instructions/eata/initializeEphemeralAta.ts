import { Address, Instruction, AccountMeta, AccountRole } from "@solana/kit";
import { SYSTEM_PROGRAM_ADDRESS } from "@solana-program/system";
import { EATA_PROGRAM_ID } from "../../constants";
import { ephemeralAtaPdaWithBumpFromOwnerAndMint } from "../../pda";

export interface InitializeEphemeralAtaAccounts {
  payer: Address;
  user: Address;
  mint: Address;
}

export async function createInitializeEphemeralAtaInstruction(
  accounts: InitializeEphemeralAtaAccounts,
): Promise<Instruction> {
  const [ephemeralAta, bump] = await ephemeralAtaPdaWithBumpFromOwnerAndMint(
    accounts.user,
    accounts.mint,
  );

  const accountsMeta: AccountMeta[] = [
    { address: ephemeralAta, role: AccountRole.WRITABLE },
    { address: accounts.payer, role: AccountRole.WRITABLE_SIGNER },
    { address: accounts.user, role: AccountRole.READONLY },
    { address: accounts.mint, role: AccountRole.READONLY },
    { address: SYSTEM_PROGRAM_ADDRESS, role: AccountRole.READONLY },
  ];

  const buffer = new ArrayBuffer(2);
  const view = new DataView(buffer);
  let offset = 0;

  view.setUint8(offset++, 0);
  view.setUint8(offset++, bump);

  return {
    accounts: accountsMeta,
    data: new Uint8Array(buffer, 0, offset),
    programAddress: EATA_PROGRAM_ID,
  };
}
