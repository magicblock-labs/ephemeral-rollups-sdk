import {
  Address,
  Instruction,
  AccountMeta,
  AccountRole,
  getAddressEncoder,
  address,
} from "@solana/kit";
import { SYSTEM_PROGRAM_ADDRESS } from "@solana-program/system";
import { DELEGATION_PROGRAM_ID } from "../../constants";
import {
  delegateBufferPdaFromDelegatedAccountAndOwnerProgram,
  delegationRecordPdaFromDelegatedAccount,
  delegationMetadataPdaFromDelegatedAccount,
} from "../../pda";

// Default validator for delegation
const DEFAULT_VALIDATOR = address(
  "MAS1Dt9qreoRMQ14YQuhg8UTZMMzDdKhmkZMECCzk57",
);

/**
 * Delegate instruction arguments
 */
export interface DelegateInstructionArgs {
  commitFrequencyMs?: number;
  seeds?: Uint8Array[];
  validator?: Address | null;
}

/**
 * Instruction: Delegate
 * Discriminator: [0,0,0,0,0,0,0,0]
 */
export async function createDelegateInstruction(
  accounts: {
    payer: Address;
    delegatedAccount: Address;
    ownerProgram: Address;
    validator?: Address;
  },
  args?: DelegateInstructionArgs,
): Promise<Instruction> {
  const delegateBuffer =
    await delegateBufferPdaFromDelegatedAccountAndOwnerProgram(
      accounts.delegatedAccount,
      accounts.ownerProgram,
    );
  const delegationRecord = await delegationRecordPdaFromDelegatedAccount(
    accounts.delegatedAccount,
  );
  const delegationMetadata = await delegationMetadataPdaFromDelegatedAccount(
    accounts.delegatedAccount,
  );

  const accountsMeta: AccountMeta[] = [
    { address: accounts.payer, role: AccountRole.WRITABLE_SIGNER },
    { address: accounts.delegatedAccount, role: AccountRole.WRITABLE_SIGNER },
    { address: accounts.ownerProgram, role: AccountRole.READONLY },
    { address: delegateBuffer, role: AccountRole.WRITABLE },
    { address: delegationRecord, role: AccountRole.WRITABLE },
    { address: delegationMetadata, role: AccountRole.WRITABLE },
    { address: SYSTEM_PROGRAM_ADDRESS, role: AccountRole.READONLY },
  ];

  const [instructionData] = serializeDelegateInstructionData({
    validator: accounts.validator,
    ...args,
  });

  return {
    accounts: accountsMeta,
    data: instructionData,
    programAddress: DELEGATION_PROGRAM_ID,
  };
}

export function serializeDelegateInstructionData(
  args?: DelegateInstructionArgs,
): [Uint8Array] {
  const delegateInstructionDiscriminator = [0, 0, 0, 0, 0, 0, 0, 0];
  const commitFrequencyMs = args?.commitFrequencyMs ?? 0xffffffff;
  const seeds = args?.seeds ?? [];
  const validator =
    args?.validator !== null && args?.validator !== undefined
      ? args.validator
      : DEFAULT_VALIDATOR;
  let offset = 0;
  const buffer = new ArrayBuffer(1024);
  const view = new DataView(buffer);

  // Write discriminator
  for (let i = 0; i < 8; i++) {
    view.setUint8(offset++, delegateInstructionDiscriminator[i]);
  }

  // Write commit_frequency_ms (u32)
  view.setUint32(offset, commitFrequencyMs, true);
  offset += 4;

  // Write seeds (vec<vec<u8>>)
  view.setUint32(offset, seeds.length, true);
  offset += 4;

  for (const seed of seeds) {
    view.setUint32(offset, seed.length, true);
    offset += 4;
    const seedBytes = new Uint8Array(buffer, offset, seed.length);
    seedBytes.set(seed);
    offset += seed.length;
  }

  // Write validator (Option<Address>)
  if (validator !== null) {
    view.setUint8(offset++, 1); // Some discriminant
    const validatorBytes = new Uint8Array(buffer, offset, 32);
    const addressEncoder = getAddressEncoder();
    const addressBytes = addressEncoder.encode(validator);
    validatorBytes.set(addressBytes);
    offset += 32;
  } else {
    view.setUint8(offset++, 0); // None discriminant
  }

  return [new Uint8Array(buffer, 0, offset)];
}
