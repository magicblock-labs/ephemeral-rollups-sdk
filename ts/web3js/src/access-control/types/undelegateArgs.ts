export interface UndelegateArgs {
  pdaSeeds: number[][];
}

export type UndelegateArgsArgs = UndelegateArgs;

export function serializeUndelegateArgs(args: UndelegateArgsArgs): Buffer {
  const MAX_BUFFER_SIZE = 4096;
  const buffer = Buffer.alloc(MAX_BUFFER_SIZE);
  let offset = 0;

  // Write pdaSeeds (Vec<Vec<u8>>)
  buffer.writeUInt32LE(args.pdaSeeds.length, offset);
  offset += 4;

  for (const seed of args.pdaSeeds) {
    if (offset + 4 + seed.length > MAX_BUFFER_SIZE) {
      throw new Error(
        `Serialized pdaSeeds exceed maximum buffer size (${MAX_BUFFER_SIZE} bytes)`,
      );
    }
    buffer.writeUInt32LE(seed.length, offset);
    offset += 4;
    for (const byte of seed) {
      buffer[offset++] = byte;
    }
  }

  return buffer.subarray(0, offset);
}

export function deserializeUndelegateArgs(
  buffer: Buffer,
  offset: number = 0,
): UndelegateArgs {
  // Read pdaSeeds (Vec<Vec<u8>>)
  if (offset + 4 > buffer.length) {
    throw new Error(
      "Buffer underflow: insufficient bytes to read pdaSeeds length",
    );
  }

  const seedsLen = buffer.readUInt32LE(offset);
  offset += 4;
  const pdaSeeds: number[][] = [];

  for (let i = 0; i < seedsLen; i++) {
    if (offset + 4 > buffer.length) {
      throw new Error(
        `Buffer underflow: insufficient bytes to read seed ${i} length`,
      );
    }

    const seedLen = buffer.readUInt32LE(offset);
    offset += 4;

    if (offset + seedLen > buffer.length) {
      throw new Error(
        `Buffer underflow: insufficient bytes to read seed ${i} data (expected ${seedLen} bytes)`,
      );
    }

    const seed: number[] = [];
    for (let j = 0; j < seedLen; j++) {
      seed.push(buffer[offset++]);
    }
    pdaSeeds.push(seed);
  }

  return { pdaSeeds };
}
