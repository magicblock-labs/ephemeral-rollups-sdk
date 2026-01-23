import { Address, Instruction, AccountMeta, AccountRole } from "@solana/kit";
import { SYSTEM_PROGRAM_ADDRESS } from "@solana-program/system";
import {
  EATA_PROGRAM_ID,
  DELEGATION_PROGRAM_ID,
  PERMISSION_PROGRAM_ID,
} from "../../constants";
import {
  delegateBufferPdaFromDelegatedAccountAndOwnerProgram,
  delegationRecordPdaFromDelegatedAccount,
  delegationMetadataPdaFromDelegatedAccount,
  permissionPdaFromAccount,
  ephemeralAtaPdaWithBumpFromOwnerAndMint,
} from "../../pda";

export interface DelegateEphemeralAtaPermissionAccounts {
  payer: Address;
  owner: Address;
  mint: Address;
  validator?: Address;
}

export async function createDelegateEphemeralAtaPermissionInstruction(
  accounts: DelegateEphemeralAtaPermissionAccounts,
): Promise<Instruction> {
  const [ephemeralAta, bump] = await ephemeralAtaPdaWithBumpFromOwnerAndMint(
    accounts.owner,
    accounts.mint,
  );
  const permission = await permissionPdaFromAccount(ephemeralAta);
  const buffer = await delegateBufferPdaFromDelegatedAccountAndOwnerProgram(
    permission,
    PERMISSION_PROGRAM_ID,
  );
  const record = await delegationRecordPdaFromDelegatedAccount(permission);
  const metadata = await delegationMetadataPdaFromDelegatedAccount(permission);

  const accountsMeta: AccountMeta[] = [
    { address: accounts.payer, role: AccountRole.WRITABLE_SIGNER },
    { address: ephemeralAta, role: AccountRole.WRITABLE },
    { address: PERMISSION_PROGRAM_ID, role: AccountRole.READONLY },
    { address: permission, role: AccountRole.WRITABLE },
    { address: SYSTEM_PROGRAM_ADDRESS, role: AccountRole.READONLY },
    { address: buffer, role: AccountRole.WRITABLE },
    { address: record, role: AccountRole.WRITABLE },
    { address: metadata, role: AccountRole.WRITABLE },
    { address: DELEGATION_PROGRAM_ID, role: AccountRole.READONLY },
  ];

  if (accounts.validator) {
    accountsMeta.push({
      address: accounts.validator,
      role: AccountRole.READONLY,
    });
  }

  const dataBuffer = new ArrayBuffer(2);
  const view = new DataView(dataBuffer);
  let offset = 0;

  view.setUint8(offset++, 7);
  view.setUint8(offset++, bump);

  return {
    accounts: accountsMeta,
    data: new Uint8Array(dataBuffer, 0, offset),
    programAddress: EATA_PROGRAM_ID,
  };
}
