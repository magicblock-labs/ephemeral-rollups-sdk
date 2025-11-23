import { AccountMeta, Address, AccountRole, Instruction } from "@solana/kit";
import { DELEGATION_PROGRAM_ID } from "../../constants";

/**
 * Delegate instruction arguments
 */
export interface DelegateInstructionArgs {
  commitFrequencyMs: number;
  seeds: Uint8Array[];
  validator: Address | null;
}

/**
 * Instruction: Delegate
 * Discriminator: [0,0,0,0,0,0,0,0]
 */
export function createDelegateInstruction(
  accounts: {
    payer: Address;
    delegatedAccount: Address;
    ownerProgram: Address;
    delegateBuffer: Address;
    delegationRecord: Address;
    delegationMetadata: Address;
    systemProgram: Address;
  },
  args: DelegateInstructionArgs,
): Instruction {
  const [data] = serializeDelegateInstructionData(args);

  const accounts_: AccountMeta[] = [
    {
      address: accounts.payer,
      role: AccountRole.WRITABLE_SIGNER,
    },
    {
      address: accounts.delegatedAccount,
      role: AccountRole.WRITABLE_SIGNER,
    },
    {
      address: accounts.ownerProgram,
      role: AccountRole.READONLY,
    },
    {
      address: accounts.delegateBuffer,
      role: AccountRole.WRITABLE,
    },
    {
      address: accounts.delegationRecord,
      role: AccountRole.WRITABLE,
    },
    {
      address: accounts.delegationMetadata,
      role: AccountRole.WRITABLE,
    },
    {
      address: accounts.systemProgram,
      role: AccountRole.READONLY,
    },
  ];

  return {
    accounts: accounts_,
    data,
    programAddress: DELEGATION_PROGRAM_ID,
  };
}

export function serializeDelegateInstructionData(
  args: DelegateInstructionArgs,
): [Uint8Array] {
  const discriminator = [0, 0, 0, 0, 0, 0, 0, 0];
  let offset = 0;
  const buffer = new ArrayBuffer(1024); // Initial size, will adjust
  const view = new DataView(buffer);

  // Write discriminator
  for (let i = 0; i < 8; i++) {
    view.setUint8(offset++, discriminator[i]);
  }

  // Write commit_frequency_ms (u32)
  view.setUint32(offset, args.commitFrequencyMs, true);
  offset += 4;

  // Write seeds (vec<vec<u8>>)
  // Format: u32 length, then each inner vec as u32 length + data
  view.setUint32(offset, args.seeds.length, true);
  offset += 4;

  for (const seed of args.seeds) {
    view.setUint32(offset, seed.length, true);
    offset += 4;
    const seedBytes = new Uint8Array(buffer, offset, seed.length);
    seedBytes.set(seed);
    offset += seed.length;
  }

  // Write validator (option<pubkey>)
  if (args.validator !== null) {
    view.setUint8(offset, 1); // Some discriminant
    offset += 1;
    // Note: This assumes pubkey is 32 bytes. In practice, you'd need to
    // decode the Address to its bytes representation
    // For now, this is a placeholder
    offset += 32;
  } else {
    view.setUint8(offset, 0); // None discriminant
    offset += 1;
  }

  return [new Uint8Array(buffer, 0, offset)];
}
