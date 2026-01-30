import { PublicKey } from "@solana/web3.js";

// Flags for Member
export const AUTHORITY_FLAG: number = 1 << 0; // Member has authority privileges
export const TX_LOGS_FLAG: number = 1 << 1; // Member can see transaction logs
export const TX_BALANCES_FLAG: number = 1 << 2; // Member can see transaction balances
export const TX_MESSAGE_FLAG: number = 1 << 3; // Member can see transaction messages
export const ACCOUNT_SIGNATURES_FLAG: number = 1 << 4; // Member can see account signatures

export interface Member {
  flags: number;
  pubkey: PublicKey;
}

export function serializeMember(member: Member): Buffer {
  const buffer = Buffer.alloc(33); // 1 byte for flags + 32 bytes for pubkey
  let offset = 0;

  // Write flags (u8)
  buffer[offset++] = member.flags;

  // Write pubkey (PublicKey)
  buffer.set(member.pubkey.toBuffer(), offset);
  offset += 32;

  return buffer.subarray(0, offset);
}

export function deserializeMember(buffer: Buffer, offset: number = 0): Member {
  // Read flags (u8)
  const flags = buffer[offset];
  offset += 1;

  // Read pubkey (PublicKey)
  const pubkey = new PublicKey(buffer.subarray(offset, offset + 32));
  offset += 32;

  return { flags, pubkey };
}

/**
 * Check if a member is an authority for a given user
 */
export function isAuthority(member: Member, user: PublicKey): boolean {
  return (member.flags & AUTHORITY_FLAG) !== 0 && member.pubkey.equals(user);
}

/**
 * Check if a member can see transaction logs for a given user
 */
export function canSeeTxLogs(member: Member, user: PublicKey): boolean {
  return (member.flags & TX_LOGS_FLAG) !== 0 && member.pubkey.equals(user);
}

/**
 * Check if a member can see transaction balances for a given user
 */
export function canSeeTxBalances(member: Member, user: PublicKey): boolean {
  return (member.flags & TX_BALANCES_FLAG) !== 0 && member.pubkey.equals(user);
}

/**
 * Check if a member can see transaction messages for a given user
 */
export function canSeeTxMessages(member: Member, user: PublicKey): boolean {
  return (member.flags & TX_MESSAGE_FLAG) !== 0 && member.pubkey.equals(user);
}

/**
 * Check if a member can see account signatures for a given user
 */
export function canSeeAccountSignatures(
  member: Member,
  user: PublicKey,
): boolean {
  return (
    (member.flags & ACCOUNT_SIGNATURES_FLAG) !== 0 && member.pubkey.equals(user)
  );
}
