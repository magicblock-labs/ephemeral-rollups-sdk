import {
  address,
  Address,
  getAddressEncoder,
  getProgramDerivedAddress,
} from "@solana/kit";
import { ASSOCIATED_TOKEN_PROGRAM_ID, TOKEN_PROGRAM_ID } from "../../constants";

/**
 * Sentinel close authority marking a token account as a rent-pending ATA.
 * The validator sets close_authority to the rent sysvar on ATAs created
 * inside the ephemeral rollup whose base-layer eATA has not been
 * materialized yet. It is a data marker only, never a signer grant.
 */
export const RENT_PENDING_ATA_CLOSE_AUTHORITY = address(
  "SysvarRent111111111111111111111111111111111",
);

/**
 * Derives the canonical ATA address for a rent-pending ATA.
 *
 * @param walletOwner - The wallet that will own the ATA
 * @param mint - The token mint
 * @param tokenProgram - The token program (TOKEN_PROGRAM_ID or TOKEN_2022_PROGRAM_ID)
 * @returns ATA address
 */
export async function rentPendingAtaAddress(
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
 * Returns true when the given token account data belongs to a rent-pending
 * ATA, i.e. its close_authority is the rent sysvar sentinel.
 *
 * @param data - Raw SPL token account data (at least 165 bytes)
 */
export function isRentPendingTokenAccount(data: Uint8Array): boolean {
  if (data.length < 165) return false;
  // close_authority COption<Pubkey>: u32 LE tag at offset 129, pubkey at 133..165
  const view = new DataView(data.buffer, data.byteOffset, data.byteLength);
  if (view.getUint32(129, true) !== 1) return false;

  const addressEncoder = getAddressEncoder();
  const sentinel = addressEncoder.encode(RENT_PENDING_ATA_CLOSE_AUTHORITY);
  return sentinel.every((byte, index) => data[133 + index] === byte);
}
