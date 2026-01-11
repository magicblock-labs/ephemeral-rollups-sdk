export type UndelegateArgs = { pdaSeeds: Array<Array<number>> };

export type UndelegateArgsArgs = UndelegateArgs;

export function serializeUndelegateArgs(args: UndelegateArgsArgs): Buffer {
  const buffer = Buffer.alloc(4096); // Allocate enough space
  let offset = 0;

  // Write pdaSeeds (Vec<Vec<u8>>)
  buffer.writeUInt32LE(args.pdaSeeds.length, offset);
  offset += 4;

  for (const seed of args.pdaSeeds) {
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
  const seedsLen = buffer.readUInt32LE(offset);
  offset += 4;
  const pdaSeeds: Array<Array<number>> = [];

  for (let i = 0; i < seedsLen; i++) {
    const seedLen = buffer.readUInt32LE(offset);
    offset += 4;
    const seed: Array<number> = [];
    for (let j = 0; j < seedLen; j++) {
      seed.push(buffer[offset++]);
    }
    pdaSeeds.push(seed);
  }

  return { pdaSeeds };
}
