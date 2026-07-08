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
 * Sentinel close authority marking a token account as a rent-pending ATA.
 * The validator sets close_authority to the rent sysvar on ATAs created
 * inside the ephemeral rollup whose base-layer eATA has not been
 * materialized yet. It is a data marker only, never a signer grant.
 */
export const RENT_PENDING_ATA_CLOSE_AUTHORITY = address(
  "SysvarRent111111111111111111111111111111111",
);

/**
 * Creates a CreateRentPendingAta instruction for the Magic Program.
 * Initializes the canonical ATA for (walletOwner, mint) directly inside the
 * ephemeral rollup, without paying base-layer rent upfront. The base eATA is
 * materialized later, when the account is committed/undelegated.
 *
 * The instruction is idempotent: it succeeds if the ATA already exists as a
 * rent-pending or projected ATA. A transaction that creates a rent-pending
 * ATA and leaves it with a zero balance is rolled back, so this instruction
 * must be bundled with a funding transfer in the same transaction.
 *
 * @param payer - The payer/sponsor account (must be signer; the wallet owner does not sign)
 * @param walletOwner - The wallet that will own the ATA
 * @param mint - The token mint
 * @param tokenProgram - The token program (TOKEN_PROGRAM_ID or TOKEN_2022_PROGRAM_ID)
 * @returns Instruction
 */
export async function createRentPendingAtaInstruction(
  payer: Address,
  walletOwner: Address,
  mint: Address,
  tokenProgram: Address = TOKEN_PROGRAM_ID,
): Promise<Instruction> {
  const addressEncoder = getAddressEncoder();
  const [ata] = await getProgramDerivedAddress({
    programAddress: ASSOCIATED_TOKEN_PROGRAM_ID,
    seeds: [
      addressEncoder.encode(walletOwner),
      addressEncoder.encode(tokenProgram),
      addressEncoder.encode(mint),
    ],
  });

  // bincode: u32 LE enum discriminant (15 = CreateRentPendingAta)
  // + wallet_owner (32) + mint (32) + token_program (32)
  const data = new Uint8Array(100);
  const view = new DataView(data.buffer);
  view.setUint32(0, 15, true);
  data.set(addressEncoder.encode(walletOwner), 4);
  data.set(addressEncoder.encode(mint), 36);
  data.set(addressEncoder.encode(tokenProgram), 68);

  return {
    accounts: [
      { address: payer, role: AccountRole.WRITABLE_SIGNER },
      { address: ata, role: AccountRole.WRITABLE },
      { address: mint, role: AccountRole.READONLY },
      { address: tokenProgram, role: AccountRole.READONLY },
    ],
    programAddress: MAGIC_PROGRAM_ID,
    data,
  };
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
