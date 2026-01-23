import {
  Address,
  Instruction,
  AccountMeta,
  AccountRole,
  getAddressEncoder,
} from "@solana/kit";
import { SYSTEM_PROGRAM_ADDRESS } from "@solana-program/system";
import { EATA_PROGRAM_ID, DELEGATION_PROGRAM_ID } from "../../constants";
import {
  delegateBufferPdaFromDelegatedAccountAndOwnerProgram,
  delegationRecordPdaFromDelegatedAccount,
  delegationMetadataPdaFromDelegatedAccount,
  ephemeralAtaPdaWithBumpFromOwnerAndMint,
} from "../../pda";

export interface DelegateEphemeralAtaAccounts {
  payer: Address;
  owner: Address;
  mint: Address;
  validator?: Address;
}

export async function createDelegateEphemeralAtaInstruction(
  accounts: DelegateEphemeralAtaAccounts,
): Promise<Instruction> {
  const [ephemeralAta, bump] = await ephemeralAtaPdaWithBumpFromOwnerAndMint(
    accounts.owner,
    accounts.mint,
  );
  const buffer = await delegateBufferPdaFromDelegatedAccountAndOwnerProgram(
    ephemeralAta,
    EATA_PROGRAM_ID,
  );
  const delegationRecord =
    await delegationRecordPdaFromDelegatedAccount(ephemeralAta);
  const delegationMetadata =
    await delegationMetadataPdaFromDelegatedAccount(ephemeralAta);

  const accountsMeta: AccountMeta[] = [
    { address: accounts.payer, role: AccountRole.WRITABLE_SIGNER },
    { address: ephemeralAta, role: AccountRole.WRITABLE },
    { address: EATA_PROGRAM_ID, role: AccountRole.READONLY },
    { address: buffer, role: AccountRole.WRITABLE },
    { address: delegationRecord, role: AccountRole.WRITABLE },
    { address: delegationMetadata, role: AccountRole.WRITABLE },
    { address: DELEGATION_PROGRAM_ID, role: AccountRole.READONLY },
    { address: SYSTEM_PROGRAM_ADDRESS, role: AccountRole.READONLY },
  ];

  const data = serializeDelegateEphemeralAtaData(bump, accounts.validator);

  return {
    accounts: accountsMeta,
    data,
    programAddress: EATA_PROGRAM_ID,
  };
}

function serializeDelegateEphemeralAtaData(
  bump: number,
  validator?: Address,
): Uint8Array {
  const dataBuffer = new ArrayBuffer(64);
  const view = new DataView(dataBuffer);
  let offset = 0;

  view.setUint8(offset++, 4);
  view.setUint8(offset++, bump);

  if (validator) {
    view.setUint8(offset++, 1);
    const addressEncoder = getAddressEncoder();
    const validatorBytes = addressEncoder.encode(validator);
    const validatorView = new Uint8Array(dataBuffer, offset, 32);
    validatorView.set(validatorBytes);
    offset += 32;
  } else {
    view.setUint8(offset++, 0);
  }

  return new Uint8Array(dataBuffer, 0, offset);
}
