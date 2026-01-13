import {
  Address,
  Instruction,
  AccountMeta,
  AccountRole,
  getAddressEncoder,
} from "@solana/kit";
import { SYSTEM_PROGRAM_ADDRESS } from "@solana-program/system";
import { PERMISSION_PROGRAM_ID } from "../../constants";
import { permissionPdaFromAccount } from "../../pda";
import type { Member } from "../../access-control/types";

/**
 * Create permission instruction arguments
 */
export interface CreatePermissionInstructionArgs {
  members?: Member[];
}

/**
 * Instruction: CreatePermission
 * Discriminator: [0, 0, 0, 0, 0, 0, 0, 0]
 */
export async function createCreatePermissionInstruction(
  accounts: {
    permissionedAccount: Address;
    payer: Address;
  },
  args?: CreatePermissionInstructionArgs,
): Promise<Instruction> {
  const permission = await permissionPdaFromAccount(
    accounts.permissionedAccount,
  );

  const accountsMeta: AccountMeta[] = [
    {
      address: accounts.permissionedAccount,
      role: AccountRole.READONLY_SIGNER,
    },
    { address: permission, role: AccountRole.WRITABLE },
    { address: accounts.payer, role: AccountRole.WRITABLE_SIGNER },
    { address: SYSTEM_PROGRAM_ADDRESS, role: AccountRole.READONLY },
  ];

  const [instructionData] = serializeCreatePermissionInstructionData(args);

  return {
    accounts: accountsMeta,
    data: instructionData,
    programAddress: PERMISSION_PROGRAM_ID,
  };
}

export function serializeCreatePermissionInstructionData(
  args?: CreatePermissionInstructionArgs,
): [Uint8Array] {
  const discriminator = [0, 0, 0, 0, 0, 0, 0, 0];
  const members = args?.members ?? [];
  let offset = 0;
  const buffer = new ArrayBuffer(2048);
  const view = new DataView(buffer);

  // Write discriminator (u64)
  for (let i = 0; i < 8; i++) {
    view.setUint8(offset++, discriminator[i]);
  }

  // Write option discriminant (u8) - 1 if members are present
  view.setUint8(offset++, members.length > 0 ? 1 : 0);

  // Write members count (u32)
  view.setUint32(offset, members.length, true);
  offset += 4;

  // Write members
  const addressEncoder = getAddressEncoder();
  for (const member of members) {
    const addressBytes = addressEncoder.encode(member.pubkey);
    const memberBytes = new Uint8Array(buffer, offset, 33);
    memberBytes.set(addressBytes);
    offset += 32;

    // Write flags (u8)
    view.setUint8(offset++, member.flags);
  }

  return [new Uint8Array(buffer, 0, offset)];
}
