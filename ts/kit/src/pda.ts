import {
  Address,
  getAddressEncoder,
  getProgramDerivedAddress,
} from "@solana/kit";
import {
  DELEGATION_PROGRAM_ID,
  PERMISSION_PROGRAM_ID,
  EATA_PROGRAM_ID,
} from "./constants";

// ============================================================================
// Delegation Program PDAs
// ============================================================================

/**
 * Derives the delegation record PDA for a given delegated account
 * @param delegatedAccount - The delegated account address
 * @returns The delegation record PDA
 */
export async function delegationRecordPdaFromDelegatedAccount(
  delegatedAccount: Address,
) {
  const addressEncoder = getAddressEncoder();
  const [delegationRecordPda] = await getProgramDerivedAddress({
    programAddress: DELEGATION_PROGRAM_ID,
    seeds: [Buffer.from("delegation"), addressEncoder.encode(delegatedAccount)],
  });
  return delegationRecordPda;
}

/**
 * Derives the delegation metadata PDA for a given delegated account
 * @param delegatedAccount - The delegated account address
 * @returns The delegation metadata PDA
 */
export async function delegationMetadataPdaFromDelegatedAccount(
  delegatedAccount: Address,
) {
  const addressEncoder = getAddressEncoder();
  const [delegationMetadataPda] = await getProgramDerivedAddress({
    programAddress: DELEGATION_PROGRAM_ID,
    seeds: [
      Buffer.from("delegation-metadata"),
      addressEncoder.encode(delegatedAccount),
    ],
  });

  return delegationMetadataPda;
}

/**
 * Derives the delegate buffer PDA for a given delegated account and owner program
 * @param delegatedAccount - The delegated account address
 * @param ownerProgramId - The owner program ID
 * @returns The delegate buffer PDA
 */
export async function delegateBufferPdaFromDelegatedAccountAndOwnerProgram(
  delegatedAccount: Address,
  ownerProgramId: Address,
) {
  const addressEncoder = getAddressEncoder();
  const [delegateBufferPda] = await getProgramDerivedAddress({
    programAddress: ownerProgramId,
    seeds: [Buffer.from("buffer"), addressEncoder.encode(delegatedAccount)],
  });
  return delegateBufferPda;
}

/**
 * Derives the escrow PDA from an escrow authority address
 * @param escrowAuthority - The escrow authority address
 * @param index - The index of the ephemeral balance account (0-255)
 * @returns The escrow PDA
 */
export async function escrowPdaFromEscrowAuthority(
  escrowAuthority: Address,
  index: number = 255,
) {
  if (index < 0 || index > 255) {
    throw new Error("Index must be between 0 and 255");
  }
  const addressEncoder = getAddressEncoder();
  const [escrowPda] = await getProgramDerivedAddress({
    programAddress: escrowAuthority,
    seeds: [
      Buffer.from("balance"),
      addressEncoder.encode(escrowAuthority),
      Buffer.from([index]),
    ],
  });
  return escrowPda;
}

/**
 * Derives the undelegate buffer PDA for a given delegated account
 * @param delegatedAccount - The delegated account address
 * @returns The undelegate buffer PDA
 */
export async function undelegateBufferPdaFromDelegatedAccount(
  delegatedAccount: Address,
) {
  const addressEncoder = getAddressEncoder();
  const [undelegateBufferPda] = await getProgramDerivedAddress({
    programAddress: DELEGATION_PROGRAM_ID,
    seeds: [
      Buffer.from("undelegate-buffer"),
      addressEncoder.encode(delegatedAccount),
    ],
  });
  return undelegateBufferPda;
}

/**
 * Derives the fees vault PDA
 * @returns The fees vault PDA
 */
export async function feesVaultPda() {
  const [feesVault] = await getProgramDerivedAddress({
    programAddress: DELEGATION_PROGRAM_ID,
    seeds: [Buffer.from("fees-vault")],
  });
  return feesVault;
}

/**
 * Derives the validator fees vault PDA for a given validator
 * @param validator - The validator address
 * @returns The validator fees vault PDA
 */
export async function validatorFeesVaultPdaFromValidator(validator: Address) {
  const addressEncoder = getAddressEncoder();
  const [validatorFeesVault] = await getProgramDerivedAddress({
    programAddress: DELEGATION_PROGRAM_ID,
    seeds: [Buffer.from("v-fees-vault"), addressEncoder.encode(validator)],
  });
  return validatorFeesVault;
}

/**
 * Derives the commit state PDA for a given delegated account
 * @param delegatedAccount - The delegated account address
 * @returns The commit state PDA
 */
export async function commitStatePdaFromDelegatedAccount(
  delegatedAccount: Address,
) {
  const addressEncoder = getAddressEncoder();
  const [commitStatePda] = await getProgramDerivedAddress({
    programAddress: DELEGATION_PROGRAM_ID,
    seeds: [Buffer.from("state-diff"), addressEncoder.encode(delegatedAccount)],
  });
  return commitStatePda;
}

/**
 * Derives the commit record PDA for a given delegated account
 * @param delegatedAccount - The delegated account address
 * @returns The commit record PDA
 */
export async function commitRecordPdaFromDelegatedAccount(
  delegatedAccount: Address,
) {
  const addressEncoder = getAddressEncoder();
  const [commitRecordPda] = await getProgramDerivedAddress({
    programAddress: DELEGATION_PROGRAM_ID,
    seeds: [
      Buffer.from("commit-state-record"),
      addressEncoder.encode(delegatedAccount),
    ],
  });
  return commitRecordPda;
}

// ============================================================================
// Permission Program PDAs
// ============================================================================

export const PERMISSION_SEED = Buffer.from("permission:");

/**
 * Derives the permission PDA for a given account
 * @param permissionedAccount - The account address permissioned
 * @returns The permission PDA
 */
export async function permissionPdaFromAccount(permissionedAccount: Address) {
  const addressEncoder = getAddressEncoder();
  const [permissionPda] = await getProgramDerivedAddress({
    programAddress: PERMISSION_PROGRAM_ID,
    seeds: [PERMISSION_SEED, addressEncoder.encode(permissionedAccount)],
  });
  return permissionPda;
}

// ============================================================================
// EATA Program PDAs
// ============================================================================

/**
 * Derives the ephemeral ATA PDA for a given owner and mint
 * @param owner - The owner address
 * @param mint - The mint address
 * @returns The ephemeral ATA PDA and bump
 */
export async function ephemeralAtaPdaWithBumpFromOwnerAndMint(
  owner: Address,
  mint: Address,
): Promise<[Address, number]> {
  const addressEncoder = getAddressEncoder();
  const [pda, bump] = await getProgramDerivedAddress({
    programAddress: EATA_PROGRAM_ID,
    seeds: [addressEncoder.encode(owner), addressEncoder.encode(mint)],
  });
  return [pda, bump];
}

/**
 * Derives the ephemeral ATA PDA for a given owner and mint
 * @param owner - The owner address
 * @param mint - The mint address
 * @returns The ephemeral ATA PDA
 */
export async function ephemeralAtaPdaFromOwnerAndMint(
  owner: Address,
  mint: Address,
): Promise<Address> {
  const [pda] = await ephemeralAtaPdaWithBumpFromOwnerAndMint(owner, mint);
  return pda;
}

/**
 * Derives the global vault PDA for a given mint
 * @param mint - The mint address
 * @returns The global vault PDA and bump
 */
export async function globalVaultPdaWithBumpFromMint(
  mint: Address,
): Promise<[Address, number]> {
  const addressEncoder = getAddressEncoder();
  const [pda, bump] = await getProgramDerivedAddress({
    programAddress: EATA_PROGRAM_ID,
    seeds: [addressEncoder.encode(mint)],
  });
  return [pda, bump];
}

/**
 * Derives the global vault PDA for a given mint
 * @param mint - The mint address
 * @returns The global vault PDA
 */
export async function globalVaultPdaFromMint(mint: Address): Promise<Address> {
  const [pda] = await globalVaultPdaWithBumpFromMint(mint);
  return pda;
}
