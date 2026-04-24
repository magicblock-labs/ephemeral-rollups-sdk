/**
 * Wire-format helpers shared by the ephemeral SPL token-program builders.
 * Kept in one place so the encoding stays in lockstep across builders.
 */

export function encodeLengthPrefixedBytes(bytes: Uint8Array): Buffer {
  if (bytes.length > 0xff) {
    throw new Error("encrypted private transfer payload exceeds u8 length");
  }
  return Buffer.concat([Buffer.from([bytes.length]), Buffer.from(bytes)]);
}

export function packPrivateTransferSuffix(
  minDelayMs: bigint,
  maxDelayMs: bigint,
  split: number,
  clientRefId?: bigint,
): Buffer {
  const suffix = Buffer.alloc(
    clientRefId === undefined ? 8 + 8 + 4 : 8 + 8 + 4 + 8,
  );
  suffix.writeBigUInt64LE(minDelayMs, 0);
  suffix.writeBigUInt64LE(maxDelayMs, 8);
  suffix.writeUInt32LE(split, 16);
  if (clientRefId !== undefined) {
    suffix.writeBigUInt64LE(clientRefId, 20);
  }
  return suffix;
}

export function u32leBuffer(value: number): Buffer {
  const out = Buffer.alloc(4);
  out.writeUInt32LE(value, 0);
  return out;
}

export function u64leBuffer(value: bigint): Buffer {
  const out = Buffer.alloc(8);
  out.writeBigUInt64LE(value, 0);
  return out;
}
