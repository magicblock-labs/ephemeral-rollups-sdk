import { PublicKey } from "@solana/web3.js";
import { PERMISSION_PROGRAM_ID } from "./constants";

const PERMISSION_SEED = Buffer.from("permission:");
const GROUP_SEED = Buffer.from("group:");

/**
 * Get the permissionPDA for a given account
 * @param account - The account to get the permission PDA for
 * @returns The permission PDA
 */
export function permissionPdaFromAccount(account: PublicKey) {
  return PublicKey.findProgramAddressSync(
    [PERMISSION_SEED, account.toBuffer()],
    PERMISSION_PROGRAM_ID,
  )[0];
}

/**
 * Get the group PDA for a given ID
 * @param id - The ID to get the group PDA for
 * @returns The group PDA
 */
export function groupPdaFromId(id: PublicKey) {
  return PublicKey.findProgramAddressSync(
    [GROUP_SEED, id.toBuffer()],
    PERMISSION_PROGRAM_ID,
  )[0];
}
