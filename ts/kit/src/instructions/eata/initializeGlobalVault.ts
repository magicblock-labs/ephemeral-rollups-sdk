import { Address, Instruction, AccountMeta, AccountRole } from "@solana/kit";
import { SYSTEM_PROGRAM_ADDRESS } from "@solana-program/system";
import { EATA_PROGRAM_ID } from "../../constants";
import { globalVaultPdaWithBumpFromMint } from "../../pda";

export interface InitializeGlobalVaultAccounts {
  payer: Address;
  mint: Address;
}

export async function createInitializeGlobalVaultInstruction(
  accounts: InitializeGlobalVaultAccounts,
): Promise<Instruction> {
  const [vault, bump] = await globalVaultPdaWithBumpFromMint(accounts.mint);

  const accountsMeta: AccountMeta[] = [
    { address: vault, role: AccountRole.WRITABLE },
    { address: accounts.payer, role: AccountRole.WRITABLE_SIGNER },
    { address: accounts.mint, role: AccountRole.READONLY },
    { address: SYSTEM_PROGRAM_ADDRESS, role: AccountRole.READONLY },
  ];

  const buffer = new ArrayBuffer(2);
  const view = new DataView(buffer);
  let offset = 0;

  view.setUint8(offset++, 1);
  view.setUint8(offset++, bump);

  return {
    accounts: accountsMeta,
    data: new Uint8Array(buffer, 0, offset),
    programAddress: EATA_PROGRAM_ID,
  };
}
