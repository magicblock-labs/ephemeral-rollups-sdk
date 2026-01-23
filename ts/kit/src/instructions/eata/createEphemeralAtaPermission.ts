import { Address, Instruction, AccountMeta, AccountRole } from "@solana/kit";
import { SYSTEM_PROGRAM_ADDRESS } from "@solana-program/system";
import { EATA_PROGRAM_ID, PERMISSION_PROGRAM_ID } from "../../constants";
import {
  ephemeralAtaPdaWithBumpFromOwnerAndMint,
  permissionPdaFromAccount,
} from "../../pda";

export interface CreateEphemeralAtaPermissionAccounts {
  owner: Address;
  mint: Address;
  payer: Address;
}

export interface CreateEphemeralAtaPermissionArgs {
  flags: number;
}

export async function createCreateEphemeralAtaPermissionInstruction(
  accounts: CreateEphemeralAtaPermissionAccounts,
  args: CreateEphemeralAtaPermissionArgs = { flags: 0 },
): Promise<Instruction> {
  const [ephemeralAta, bump] = await ephemeralAtaPdaWithBumpFromOwnerAndMint(
    accounts.owner,
    accounts.mint,
  );
  const permission = await permissionPdaFromAccount(ephemeralAta);

  const accountsMeta: AccountMeta[] = [
    { address: ephemeralAta, role: AccountRole.WRITABLE },
    { address: permission, role: AccountRole.WRITABLE },
    { address: accounts.payer, role: AccountRole.WRITABLE_SIGNER },
    { address: SYSTEM_PROGRAM_ADDRESS, role: AccountRole.READONLY },
    { address: PERMISSION_PROGRAM_ID, role: AccountRole.READONLY },
  ];

  const buffer = new ArrayBuffer(3);
  const view = new DataView(buffer);
  let offset = 0;

  view.setUint8(offset++, 6);
  view.setUint8(offset++, bump);
  view.setUint8(offset++, args.flags);

  return {
    accounts: accountsMeta,
    data: new Uint8Array(buffer, 0, offset),
    programAddress: EATA_PROGRAM_ID,
  };
}
