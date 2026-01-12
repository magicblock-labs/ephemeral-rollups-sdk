import { Address, Instruction, AccountMeta, AccountRole } from "@solana/kit";
import { SYSTEM_PROGRAM_ADDRESS } from "@solana-program/system";
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
export async function createUndelegatePermissionInstruction(
  accounts: {
    delegatedPermission: Address;
    delegationBuffer: Address;
    validator: Address;
  },
  args?: UndelegatePermissionInstructionArgs,
): Promise<Instruction> {
  const accountsMeta: AccountMeta[] = [
    { address: accounts.delegatedPermission, role: AccountRole.WRITABLE },
    { address: accounts.delegationBuffer, role: AccountRole.WRITABLE },
    { address: accounts.validator, role: AccountRole.READONLY_SIGNER },
    { address: SYSTEM_PROGRAM_ADDRESS, role: AccountRole.READONLY },
  ];

  const [instructionData] = serializeUndelegatePermissionInstructionData(args);

  return {
    accounts: accountsMeta,
    data: instructionData,
    programAddress: PERMISSION_PROGRAM_ID,
  };
}

export function serializeUndelegatePermissionInstructionData(
  args?: UndelegatePermissionInstructionArgs,
): [Uint8Array] {
  const MAX_SEED_LENGTH = 32; // Solana PDA seed maximum length
  // Discriminator for UndelegatePermission: 12048014319693667524 in little-endian
  const discriminator = [0xa4, 0xa7, 0x5c, 0xcc, 0x04, 0x8a, 0xa9, 0xa6];
  const pdaSeeds = args?.pdaSeeds ?? [];

  // Validate seed lengths
  for (let i = 0; i < pdaSeeds.length; i++) {
    if (pdaSeeds[i].length > MAX_SEED_LENGTH) {
      throw new Error(
        `PDA seed ${i} exceeds maximum length of ${MAX_SEED_LENGTH} bytes (got ${pdaSeeds[i].length})`,
      );
    }
  }

  // Calculate exact buffer size needed:
  // 8 bytes (discriminator) + 4 bytes (vec length) + (4 bytes + seed length) per seed
  let requiredSize = 8 + 4;
  for (const seed of pdaSeeds) {
    requiredSize += 4 + seed.length;
  }

  const buffer = new ArrayBuffer(requiredSize);
  const view = new DataView(buffer);
  let offset = 0;

  // Write discriminator (u64)
  for (let i = 0; i < 8; i++) {
    view.setUint8(offset++, discriminator[i]);
  }

  // Write pda_seeds (vec<vec<u8>>)
  view.setUint32(offset, pdaSeeds.length, true);
  offset += 4;

  for (const seed of pdaSeeds) {
    view.setUint32(offset, seed.length, true);
    offset += 4;
    const seedBytes = new Uint8Array(buffer, offset, seed.length);
    seedBytes.set(seed);
    offset += seed.length;
  }

  return [new Uint8Array(buffer, 0, offset)];
}
