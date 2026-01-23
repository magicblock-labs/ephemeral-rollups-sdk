import { Address, Instruction, AccountMeta, AccountRole } from "@solana/kit";
import { EATA_PROGRAM_ID, PERMISSION_PROGRAM_ID } from "../../constants";
import {
  ephemeralAtaPdaWithBumpFromOwnerAndMint,
  permissionPdaFromAccount,
} from "../../pda";

export interface ResetEphemeralAtaPermissionAccounts {
  owner: Address;
  mint: Address;
}

export interface ResetEphemeralAtaPermissionArgs {
  flags: number;
}

export async function createResetEphemeralAtaPermissionInstruction(
  accounts: ResetEphemeralAtaPermissionAccounts,
  args: ResetEphemeralAtaPermissionArgs = { flags: 0 },
): Promise<Instruction> {
  const [ephemeralAta, bump] = await ephemeralAtaPdaWithBumpFromOwnerAndMint(
    accounts.owner,
    accounts.mint,
  );
  const permission = await permissionPdaFromAccount(ephemeralAta);

  const accountsMeta: AccountMeta[] = [
    { address: ephemeralAta, role: AccountRole.WRITABLE },
    { address: permission, role: AccountRole.WRITABLE },
    { address: accounts.owner, role: AccountRole.READONLY_SIGNER },
    { address: PERMISSION_PROGRAM_ID, role: AccountRole.READONLY },
  ];

  const buffer = new ArrayBuffer(3);
  const view = new DataView(buffer);
  let offset = 0;

  view.setUint8(offset++, 9);
  view.setUint8(offset++, bump);
  view.setUint8(offset++, args.flags);

  return {
    accounts: accountsMeta,
    data: new Uint8Array(buffer, 0, offset),
    programAddress: EATA_PROGRAM_ID,
  };
}
