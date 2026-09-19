import {
  PublicKey,
  SYSVAR_RENT_PUBKEY,
  TransactionInstruction,
} from "@solana/web3.js";
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
export const MAGIC_ATA_CLOSE_AUTHORITY = SYSVAR_RENT_PUBKEY;

/**
 * Derives the canonical ATA address for a Magic ATA.
 *
 * @param walletOwner - The wallet that will own the ATA
 * @param mint - The token mint
 * @param tokenProgram - The token program (TOKEN_PROGRAM_ID or TOKEN_2022_PROGRAM_ID)
 * @returns ATA public key
 */
export function magicAtaAddress(
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
 * @returns TransactionInstruction
 */
export function createMagicAtaInstruction(
  payer: PublicKey,
  walletOwner: PublicKey,
  mint: PublicKey,
  tokenProgram: PublicKey = TOKEN_PROGRAM_ID,
): TransactionInstruction {
  const ata = magicAtaAddress(walletOwner, mint, tokenProgram);
  const data = Buffer.alloc(36);
  data.writeUInt32LE(15, 0);
  walletOwner.toBuffer().copy(data, 4);

  return new TransactionInstruction({
    keys: [
      { pubkey: payer, isSigner: true, isWritable: false },
      { pubkey: ata, isSigner: false, isWritable: true },
      { pubkey: mint, isSigner: false, isWritable: false },
      { pubkey: tokenProgram, isSigner: false, isWritable: false },
    ],
    programId: MAGIC_PROGRAM_ID,
    data,
  });
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
 * @returns TransactionInstruction
 */
export function closeMagicAtaInstruction(
  owner: PublicKey,
  mint: PublicKey,
  tokenProgram: PublicKey = TOKEN_PROGRAM_ID,
): TransactionInstruction {
  const ata = magicAtaAddress(owner, mint, tokenProgram);
  const data = Buffer.alloc(4);
  data.writeUInt32LE(26, 0);

  return new TransactionInstruction({
    keys: [
      { pubkey: owner, isSigner: true, isWritable: false },
      { pubkey: ata, isSigner: false, isWritable: true },
    ],
    programId: MAGIC_PROGRAM_ID,
    data,
  });
}

/**
 * Returns true when the given token account data belongs to a Magic ATA
 * ATA, i.e. its close_authority is the rent sysvar sentinel.
 *
 * @param data - Raw SPL token account data (at least 165 bytes)
 */
export function isMagicAtaTokenAccount(data: Buffer | Uint8Array): boolean {
  const buf = Buffer.isBuffer(data) ? data : Buffer.from(data);
  if (buf.length < 165) return false;
  // close_authority COption<Pubkey>: u32 LE tag at offset 129, pubkey at 133..165
  return (
    buf.readUInt32LE(129) === 1 &&
    new PublicKey(buf.subarray(133, 165)).equals(MAGIC_ATA_CLOSE_AUTHORITY)
  );
}
