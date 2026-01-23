import { Address, Instruction, AccountMeta, AccountRole } from "@solana/kit";
import {
  EATA_PROGRAM_ID,
  MAGIC_CONTEXT_ID,
  MAGIC_PROGRAM_ID,
  PERMISSION_PROGRAM_ID,
} from "../../constants";
import {
  ephemeralAtaPdaFromOwnerAndMint,
  permissionPdaFromAccount,
} from "../../pda";

export interface UndelegateEphemeralAtaPermissionAccounts {
  payer: Address;
  owner: Address;
  mint: Address;
}

export async function createUndelegateEphemeralAtaPermissionInstruction(
  accounts: UndelegateEphemeralAtaPermissionAccounts,
): Promise<Instruction> {
  const ephemeralAta = await ephemeralAtaPdaFromOwnerAndMint(
    accounts.owner,
    accounts.mint,
  );
  const permission = await permissionPdaFromAccount(ephemeralAta);

  const accountsMeta: AccountMeta[] = [
    { address: accounts.payer, role: AccountRole.READONLY_SIGNER },
    { address: ephemeralAta, role: AccountRole.WRITABLE },
    { address: permission, role: AccountRole.WRITABLE },
    { address: PERMISSION_PROGRAM_ID, role: AccountRole.READONLY },
    { address: MAGIC_PROGRAM_ID, role: AccountRole.READONLY },
    { address: MAGIC_CONTEXT_ID, role: AccountRole.WRITABLE },
  ];

  const data = new Uint8Array([8]);

  return {
    accounts: accountsMeta,
    data,
    programAddress: EATA_PROGRAM_ID,
  };
}
