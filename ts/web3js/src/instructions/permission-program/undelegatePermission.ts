import {
  PublicKey,
  TransactionInstruction,
  SystemProgram,
  AccountMeta,
} from "@solana/web3.js";
import { PERMISSION_PROGRAM_ID } from "../../constants";

/**
 * Undelegate permission instruction arguments
 */
export interface UndelegatePermissionInstructionArgs {
  pdaSeeds?: Uint8Array[];
}

/**
 * Instruction: UndelegatePermission
 * Discriminator: [0xA4, 0xA7, 0x5C, 0xCC, 0x04, 0x8A, 0xA9, 0xA6] (little-endian for 12048014319693667524)
 */
export function createUndelegatePermissionInstruction(
  accounts: {
    delegatedPermission: PublicKey;
    delegationBuffer: PublicKey;
    validator: PublicKey;
  },
  args?: UndelegatePermissionInstructionArgs,
): TransactionInstruction {
  const keys: AccountMeta[] = [
    { pubkey: accounts.delegatedPermission, isWritable: true, isSigner: false },
    { pubkey: accounts.delegationBuffer, isWritable: true, isSigner: false },
    { pubkey: accounts.validator, isWritable: false, isSigner: true },
    { pubkey: SystemProgram.programId, isWritable: false, isSigner: false },
  ];

  const instructionData = serializeUndelegatePermissionInstructionData(args);

  return new TransactionInstruction({
    programId: PERMISSION_PROGRAM_ID,
    keys,
    data: instructionData,
  });
}

export function serializeUndelegatePermissionInstructionData(
  args?: UndelegatePermissionInstructionArgs,
): Buffer {
  // Discriminator for UndelegatePermission: 12048014319693667524 in little-endian
  const discriminator = [0xa4, 0xa7, 0x5c, 0xcc, 0x04, 0x8a, 0xa9, 0xa6];
  const pdaSeeds = args?.pdaSeeds ?? [];

  // Calculate exact buffer size needed:
  // 8 bytes (discriminator) + 4 bytes (vec length) + (4 bytes + seed length) per seed
  let requiredSize = 8 + 4;
  for (const seed of pdaSeeds) {
    requiredSize += 4 + seed.length;
  }

  const buffer = Buffer.alloc(requiredSize);
  let offset = 0;

  // Write discriminator (u64)
  for (let i = 0; i < 8; i++) {
    buffer[offset++] = discriminator[i];
  }

  // Write pda_seeds (vec<vec<u8>>)
  buffer.writeUInt32LE(pdaSeeds.length, offset);
  offset += 4;

  for (const seed of pdaSeeds) {
    buffer.writeUInt32LE(seed.length, offset);
    offset += 4;
    buffer.set(seed, offset);
    offset += seed.length;
  }

  return buffer;
}
