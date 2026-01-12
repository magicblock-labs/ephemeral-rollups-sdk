import {
  Address,
  Instruction,
  AccountMeta,
  AccountRole,
  getAddressEncoder,
} from "@solana/kit";
import { PERMISSION_PROGRAM_ID } from "../../constants";
import { permissionPdaFromAccount } from "../../pda";
import type { Member } from "../../access-control/types";

/**
 * Update permission instruction arguments
 */
export interface UpdatePermissionInstructionArgs {
  members?: Member[];
}

/**
 * Instruction: UpdatePermission
 * Discriminator: [1, 0, 0, 0, 0, 0, 0, 0]
 */
export async function createUpdatePermissionInstruction(
  accounts: {
    authority: Address;
    permissionedAccount: Address;
  },
  args?: UpdatePermissionInstructionArgs,
): Promise<Instruction> {
  const permission = await permissionPdaFromAccount(
    accounts.permissionedAccount,
  );

  const accountsMeta: AccountMeta[] = [
    { address: accounts.authority, role: AccountRole.READONLY_SIGNER },
    {
      address: accounts.permissionedAccount,
      role: AccountRole.READONLY_SIGNER,
    },
    { address: permission, role: AccountRole.WRITABLE },
  ];

  const [instructionData] = serializeUpdatePermissionInstructionData(args);

  return {
    accounts: accountsMeta,
    data: instructionData,
    programAddress: PERMISSION_PROGRAM_ID,
  };
}

export function serializeUpdatePermissionInstructionData(
  args?: UpdatePermissionInstructionArgs,
): [Uint8Array] {
  const discriminator = [1, 0, 0, 0, 0, 0, 0, 0];
  const members = args?.members ?? [];

  // Calculate exact buffer size needed:
  // 8 bytes (discriminator) + 4 bytes (members count) + (32 bytes + 1 byte) per member
  let requiredSize = 8 + 4;
  for (let i = 0; i < members.length; i++) {
    requiredSize += 32 + 1;
  }

  const buffer = new ArrayBuffer(requiredSize);
  const view = new DataView(buffer);
  let offset = 0;

  // Write discriminator (u64)
  for (let i = 0; i < 8; i++) {
    view.setUint8(offset++, discriminator[i]);
  }

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
