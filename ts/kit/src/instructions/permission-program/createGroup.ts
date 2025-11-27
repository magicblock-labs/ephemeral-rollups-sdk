import {
  Address,
  Instruction,
  AccountMeta,
  AccountRole,
  getAddressEncoder,
} from "@solana/kit";
import { SYSTEM_PROGRAM_ADDRESS } from "@solana-program/system";
import { PERMISSION_PROGRAM_ID } from "../../constants";

/**
 * CreateGroup instruction arguments
 */
export interface CreateGroupInstructionArgs {
  id: Address;
  members: Address[];
}

/**
 * Instruction: CreateGroup
 * Discriminator: 0
 */
export function createCreateGroupInstruction(
  accounts: {
    group: Address;
    payer: Address;
  },
  args: CreateGroupInstructionArgs,
): Instruction {
  const accountsMeta: AccountMeta[] = [
    { address: accounts.group, role: AccountRole.WRITABLE },
    { address: accounts.payer, role: AccountRole.WRITABLE_SIGNER },
    { address: SYSTEM_PROGRAM_ADDRESS, role: AccountRole.READONLY },
  ];

  const [instructionData] = serializeCreateGroupInstructionData(args);

  return {
    accounts: accountsMeta,
    data: instructionData,
    programAddress: PERMISSION_PROGRAM_ID,
  };
}

export function serializeCreateGroupInstructionData(
  args: CreateGroupInstructionArgs,
): [Uint8Array] {
  const discriminator = 0;
  const addressEncoder = getAddressEncoder();
  let offset = 0;
  const buffer = new ArrayBuffer(10000);
  const view = new DataView(buffer);

  // Write discriminator (u8)
  view.setUint8(offset++, discriminator);

  // Write id (PublicKey)
  const idBytes = addressEncoder.encode(args.id);
  const idView = new Uint8Array(buffer, offset, 32);
  idView.set(idBytes);
  offset += 32;

  // Write members count (u32)
  view.setUint32(offset, args.members.length, true);
  offset += 4;

  // Write members (vec<PublicKey>)
  for (const member of args.members) {
    const memberBytes = addressEncoder.encode(member);
    const memberView = new Uint8Array(buffer, offset, 32);
    memberView.set(memberBytes);
    offset += 32;
  }

  return [new Uint8Array(buffer, 0, offset)];
}
