import {
  deserializeMember,
  serializeMember,
  type Member,
  type MemberArgs,
} from "./member";

export interface MembersArgs {
  members: Member[] | null;
}

export interface MembersArgsArgs {
  members: MemberArgs[] | null;
}

export function serializeMembersArgs(args: MembersArgsArgs): Buffer {
  const buffer = Buffer.alloc(4096); // Allocate enough space
  let offset = 0;

  // Write members (Option<Vec<Member>>)
  if (args.members === null) {
    buffer[offset++] = 0; // None discriminant
  } else {
    buffer[offset++] = 1; // Some discriminant
    // Write vector length
    buffer.writeUInt32LE(args.members.length, offset);
    offset += 4;
    // Write each member
    for (const member of args.members) {
      const memberBuffer = serializeMember(member);
      buffer.set(memberBuffer, offset);
      offset += memberBuffer.length;
    }
  }

  return buffer.subarray(0, offset);
}

export function deserializeMembersArgs(
  buffer: Buffer,
  offset: number = 0,
): MembersArgs {
  // Read members (Option<Vec<Member>>)
  const discriminant = buffer[offset++];
  let members: Member[] | null = null;

  if (discriminant === 1) {
    // Some variant
    const len = buffer.readUInt32LE(offset);
    offset += 4;
    members = [];
    for (let i = 0; i < len; i++) {
      const member = deserializeMember(buffer, offset);
      members.push(member);
      offset += 33; // 1 byte flags + 32 bytes pubkey
    }
  }

  return { members };
}
