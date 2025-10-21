import { Address, getAddressEncoder, getProgramDerivedAddress} from "@solana/kit"
import { PERMISSION_PROGRAM_ID } from "./constants";

const PERMISSION_SEED = Buffer.from("permission:");
const GROUP_SEED = Buffer.from("group:");

/**
 * Get the permissionPDA for a given account
 * @param account - The account to get the permission PDA for
 * @returns The permission PDA
 */
export async function permissionPdaFromAccount(account: Address) {
  const addressEncoder = getAddressEncoder();
  const [permissionPda, bump] = await getProgramDerivedAddress({
    programAddress: PERMISSION_PROGRAM_ID,
    seeds: [
        PERMISSION_SEED,
        addressEncoder.encode(account)
    ],
  });
  return permissionPda
}

/**
 * Get the group PDA for a given ID
 * @param id - The ID to get the group PDA for
 * @returns The group PDA
 */
export async function groupPdaFromId(id: Address) {
  const addressEncoder = getAddressEncoder();
  const [groupPda, bump] = await getProgramDerivedAddress({
    programAddress: PERMISSION_PROGRAM_ID,
    seeds: [
        GROUP_SEED,
        addressEncoder.encode(id)
    ],
  });
  return groupPda
}
