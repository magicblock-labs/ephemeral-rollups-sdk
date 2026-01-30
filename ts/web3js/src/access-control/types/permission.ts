import { PublicKey } from "@solana/web3.js";
import { deserializeMember, Member } from "./member";

export interface Permission {
  discriminator: number;
  bump: number;
  permissionedAccount: PublicKey;
  members?: Member[];
}

export function serializePermission(permission: Permission): Buffer {
  const buffer = Buffer.alloc(567);
  let offset = 0;

  buffer[offset++] = permission.discriminator;
  buffer[offset++] = permission.bump;
  buffer.set(permission.permissionedAccount.toBuffer(), offset);
  offset += 32;

  if (permission.members !== undefined) {
    buffer[offset++] = 1;

    buffer.writeUInt32LE(permission.members?.length ?? 0, offset);
    offset += 4;
    for (const member of permission.members ?? []) {
      buffer.set(member.pubkey.toBuffer(), offset);
      offset += 32;
    }
  } else {
    buffer[offset++] = 0;
  }

  return buffer.subarray(0, offset);
}

export function deserializePermission(
  buffer: Buffer,
  offset: number = 0,
): Permission {
  const discriminator = buffer[offset];
  offset += 1;
  const bump = buffer[offset];
  offset += 1;
  const permissionedAccount = new PublicKey(
    buffer.subarray(offset, offset + 32),
  );
  offset += 32;

  let members;
  const hasMembers = buffer[offset++];
  if (hasMembers) {
    const membersCount = buffer.readUInt32LE(offset);
    offset += 4;
    members = [];
    for (let i = 0; i < membersCount; i++) {
      members.push(deserializeMember(buffer.subarray(offset)));
      offset += 33;
    }
  }
  return { discriminator, bump, permissionedAccount, members };
}
