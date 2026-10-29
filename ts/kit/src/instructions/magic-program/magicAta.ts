import {
  address,
  Address,
  getAddressEncoder,
  getProgramDerivedAddress,
  Instruction,
} from "@solana/kit";
import { AccountRole } from "@solana/instructions";
import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  MAGIC_PROGRAM_ID,
  TOKEN_PROGRAM_ID,
} from "../../constants";

/**
 * Sentinel close authority marking a token account as a Magic ATA.
 * The validator sets close_authority to the rent sysvar on ATAs created
 * inside the ephemeral rollup whose base-layer eATA has not been
 * materialized yet. It is a data marker only, never a signer grant.
 */
export const MAGIC_ATA_CLOSE_AUTHORITY = address(
  "SysvarRent111111111111111111111111111111111",
);

/**
 * Derives the canonical ATA address for a Magic ATA.
 *
 * @param walletOwner - The wallet that will own the ATA
 * @param mint - The token mint
 * @param tokenProgram - The token program (TOKEN_PROGRAM_ID or TOKEN_2022_PROGRAM_ID)
 * @returns ATA address
 */
export async function magicAtaAddress(
  walletOwner: Address,
  mint: Address,
  tokenProgram: Address = TOKEN_PROGRAM_ID,
): Promise<Address> {
  const addressEncoder = getAddressEncoder();
  const [ata] = await getProgramDerivedAddress({
    programAddress: ASSOCIATED_TOKEN_PROGRAM_ID,
    seeds: [
      addressEncoder.encode(walletOwner),
      addressEncoder.encode(tokenProgram),
      addressEncoder.encode(mint),
    ],
  });

  return ata;
}

/**
 * Creates a Magic ATA through the Magic Program.
 *
 * The instruction is idempotent for an existing matching Magic ATA or
 * projected ATA. The ATA must end the transaction with a positive token
 * balance. Requires a validator that supports Magic ATA materialization.
 *
 * @param payer - Payer/sponsor account (must sign; the wallet owner does not sign)
 * @param walletOwner - The wallet that will own the ATA
 * @param mint - The token mint
 * @param tokenProgram - The token program (TOKEN_PROGRAM_ID or TOKEN_2022_PROGRAM_ID)
 * @returns Instruction
 */
export async function createMagicAtaInstruction(
  payer: Address,
  walletOwner: Address,
  mint: Address,
  tokenProgram: Address = TOKEN_PROGRAM_ID,
): Promise<Instruction> {
  const addressEncoder = getAddressEncoder();
  const ata = await magicAtaAddress(walletOwner, mint, tokenProgram);
  const data = new Uint8Array(36);
  new DataView(data.buffer).setUint32(0, 15, true);
  data.set(addressEncoder.encode(walletOwner), 4);

  return {
    accounts: [
      { address: payer, role: AccountRole.READONLY_SIGNER },
      { address: ata, role: AccountRole.WRITABLE },
      { address: mint, role: AccountRole.READONLY },
      { address: tokenProgram, role: AccountRole.READONLY },
    ],
    programAddress: MAGIC_PROGRAM_ID,
    data,
  };
}

/**
 * Closes a drained Magic ATA through the Magic Program.
 *
 * No-op unless the ATA matches the Magic ATA marker for the signing
 * owner and holds zero tokens, so it can be appended unconditionally to
 * withdrawal flows.
 *
 * @param owner - The wallet owning the Magic ATA (must sign)
 * @param mint - The token mint
 * @param tokenProgram - The token program (TOKEN_PROGRAM_ID or TOKEN_2022_PROGRAM_ID)
 * @returns Instruction
 */
export async function closeMagicAtaInstruction(
  owner: Address,
  mint: Address,
  tokenProgram: Address = TOKEN_PROGRAM_ID,
): Promise<Instruction> {
  const ata = await magicAtaAddress(owner, mint, tokenProgram);
  const data = new Uint8Array(4);
  new DataView(data.buffer).setUint32(0, 26, true);

  return {
    accounts: [
      { address: owner, role: AccountRole.READONLY_SIGNER },
      { address: ata, role: AccountRole.WRITABLE },
    ],
    programAddress: MAGIC_PROGRAM_ID,
    data,
  };
}

/**
 * Returns true when the given token account data belongs to a Magic ATA
 * ATA, i.e. its close_authority is the rent sysvar sentinel.
 *
 * @param data - Raw SPL token account data (at least 165 bytes)
 */
export function isMagicAtaTokenAccount(data: Uint8Array): boolean {
  if (data.length < 165) return false;
  // close_authority COption<Pubkey>: u32 LE tag at offset 129, pubkey at 133..165
  const view = new DataView(data.buffer, data.byteOffset, data.byteLength);
  if (view.getUint32(129, true) !== 1) return false;

  const addressEncoder = getAddressEncoder();
  const sentinel = addressEncoder.encode(MAGIC_ATA_CLOSE_AUTHORITY);
  return sentinel.every((byte, index) => data[133 + index] === byte);
}
