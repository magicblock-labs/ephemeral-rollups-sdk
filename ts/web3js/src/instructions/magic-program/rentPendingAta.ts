import { PublicKey, SYSVAR_RENT_PUBKEY } from "@solana/web3.js";
import { ASSOCIATED_TOKEN_PROGRAM_ID, TOKEN_PROGRAM_ID } from "../../constants";

/**
 * Sentinel close authority marking a token account as a rent-pending ATA.
 * The validator sets close_authority to the rent sysvar on ATAs created
 * inside the ephemeral rollup whose base-layer eATA has not been
 * materialized yet. It is a data marker only, never a signer grant.
 */
export const RENT_PENDING_ATA_CLOSE_AUTHORITY = SYSVAR_RENT_PUBKEY;

/**
 * Derives the canonical ATA address for a rent-pending ATA.
 *
 * @param walletOwner - The wallet that will own the ATA
 * @param mint - The token mint
 * @param tokenProgram - The token program (TOKEN_PROGRAM_ID or TOKEN_2022_PROGRAM_ID)
 * @returns ATA public key
 */
export function rentPendingAtaAddress(
  walletOwner: PublicKey,
  mint: PublicKey,
  tokenProgram: PublicKey = TOKEN_PROGRAM_ID,
): PublicKey {
  const [ata] = PublicKey.findProgramAddressSync(
    [walletOwner.toBuffer(), tokenProgram.toBuffer(), mint.toBuffer()],
    ASSOCIATED_TOKEN_PROGRAM_ID,
  );

  return ata;
}

/**
 * Returns true when the given token account data belongs to a rent-pending
 * ATA, i.e. its close_authority is the rent sysvar sentinel.
 *
 * @param data - Raw SPL token account data (at least 165 bytes)
 */
export function isRentPendingTokenAccount(data: Buffer | Uint8Array): boolean {
  const buf = Buffer.isBuffer(data) ? data : Buffer.from(data);
  if (buf.length < 165) return false;
  // close_authority COption<Pubkey>: u32 LE tag at offset 129, pubkey at 133..165
  return (
    buf.readUInt32LE(129) === 1 &&
    new PublicKey(buf.subarray(133, 165)).equals(
      RENT_PENDING_ATA_CLOSE_AUTHORITY,
    )
  );
}
