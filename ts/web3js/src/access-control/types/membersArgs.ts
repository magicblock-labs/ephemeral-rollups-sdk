import { deserializeMember, serializeMember, type Member } from "./member";

// Member size: 1 byte flags + 32 bytes pubkey
const MEMBER_SIZE = 33;

export interface MembersArgs {
  members: Member[] | null;
}

export function serializeMembersArgs(args: MembersArgs): Buffer {
  // Calculate exact buffer size needed:
  // 1 byte (option) + [4 bytes (count) + (33 bytes per member)] if Some
  let requiredSize = 1;
  if (args.members !== null) {
    requiredSize += 4 + args.members.length * MEMBER_SIZE;
  }

  const buffer = Buffer.alloc(requiredSize);
  let offset = 0;

  // Write members (Option<Vec<Member>>)
  if (args.members === null) {
    buffer[offset++] = 0; // None discriminant
    return buffer.subarray(0, offset);
  }

  buffer[offset++] = 1; // Some discriminant
  // Write vector length
  buffer.writeUInt32LE(args.members.length, offset);
  offset += 4;
  // Write each member
  for (const member of args.members) {
    const memberBuffer = serializeMember(member);
    if (memberBuffer.length !== MEMBER_SIZE) {
      throw new Error(
        `Member serialization mismatch: expected ${MEMBER_SIZE} bytes, got ${memberBuffer.length}`,
      );
    }
    buffer.set(memberBuffer, offset);
    offset += MEMBER_SIZE;
  }

  return buffer.subarray(0, offset);
}

export function deserializeMembersArgs(
  buffer: Buffer,
  offset: number = 0,
): MembersArgs {
  // Read members (Option<Vec<Member>>)
  if (offset + 1 > buffer.length) {
    throw new Error(
      "Buffer underflow: insufficient bytes to read members discriminant",
    );
  }

  const discriminant = buffer[offset++];
  let members: Member[] | null = null;

  if (discriminant === 0) {
    // None variant
    members = null;
  } else if (discriminant === 1) {
    // Some variant
    if (offset + 4 > buffer.length) {
      throw new Error(
        "Buffer underflow: insufficient bytes to read members length",
      );
    }

    const len = buffer.readUInt32LE(offset);
    offset += 4;
    members = [];
    for (let i = 0; i < len; i++) {
      if (offset + MEMBER_SIZE > buffer.length) {
        throw new Error(
          `Buffer underflow: insufficient bytes to read member ${i} (expected ${MEMBER_SIZE} bytes)`,
        );
      }

      const member = deserializeMember(buffer, offset);
      members.push(member);
      offset += MEMBER_SIZE;
    }
  } else {
    throw new Error(
      `Invalid discriminant for MembersArgs: expected 0 (None) or 1 (Some), got ${discriminant}`,
    );
  }

  return { members };
}
