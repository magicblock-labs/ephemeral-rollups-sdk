import {
  CompressedAccountMeta,
  createRpc,
  DerivationMode,
  deriveAddress,
  deriveAddressSeed,
  featureFlags,
  PackedAccounts,
  PackedAddressTreeInfo,
  packTreeInfos,
  Rpc,
  SystemAccountMetaConfig,
  TreeType,
  ValidityProof,
  VERSION,
} from "@lightprotocol/stateless.js";
import { PublicKey } from "@solana/web3.js";

// Ensure we use V2
featureFlags.version = VERSION.V2;

export const COMPRESSED_DELEGATION_PROGRAM_ID = new PublicKey(
  "DEL2rPzhFaS5qzo8XY9ZNxSzuunWueySq3p2dxJfwPbT",
);
export const BATCHED_MERKLE_TREE = new PublicKey(
  "bmt1LryLZUMmF7ZtqESaw7wifBXLfXHQYoE4GAmrahU",
);
export const ADRESS_TREE = new PublicKey(
  "amt2kaJA14v3urZbZvnc5v2np8jqvc4Z8zDep5wbtzx",
);
export const OUTPUT_QUEUE = new PublicKey(
  "oq1na8gojfdUhsfCpyjNt6h4JaDWtHf1yQj4koBWfto",
);

/**
 * Creates a Photon client
 * @param rpcUrl - The RPC URL
 * @param photonUrl - The Photon URL
 * @param proverUrl - The Prover URL
 * @returns The Photon client
 */
export function createPhotonClient(
  rpcUrl: string,
  photonUrl: string,
  proverUrl: string,
) {
  return createRpc(rpcUrl, photonUrl, proverUrl);
}

/**
 * Derives the compressed derived address for a given delegated account
 * @param delegatedAccount - The delegated account address
 * @returns The compressed derived address
 */
export function deriveCda(delegatedAccount: PublicKey) {
  const addressSeed = deriveAddressSeed([delegatedAccount.toBuffer()]);
  return deriveAddress(
    addressSeed,
    BATCHED_MERKLE_TREE,
    // ADRESS_TREE,
    COMPRESSED_DELEGATION_PROGRAM_ID,
  );
}

export function convertValidityProofToBytes(
  validityProof: ValidityProof | null,
) {
  if (!validityProof) {
    return Buffer.from([0]);
  }
  return Buffer.from([
    1,
    ...validityProof.a,
    ...validityProof.b,
    ...validityProof.c,
  ]);
}

export function convertPackedAddressTreeInfoToBytes(
  packedAddressTreeInfo: PackedAddressTreeInfo,
) {
  const packedAddressTreeInfoBytes = Buffer.alloc(4);
  // rootIndex is two bytes, little-endian
  packedAddressTreeInfoBytes.writeUInt16LE(packedAddressTreeInfo.rootIndex, 0);
  packedAddressTreeInfoBytes.writeUInt8(
    packedAddressTreeInfo.addressMerkleTreePubkeyIndex,
    2,
  );
  packedAddressTreeInfo.addressMerkleTreePubkeyIndex;
  packedAddressTreeInfoBytes.writeUInt8(
    packedAddressTreeInfo.addressQueuePubkeyIndex,
    3,
  );
  return packedAddressTreeInfoBytes;
}

export function convertOutputStateTreeIndexToBytes(
  outputStateTreeIndex: number,
) {
  return Buffer.from([outputStateTreeIndex]);
}

export function convertCompressedAccountMetaToBytes(
  accountMeta: CompressedAccountMeta,
) {
  if (!accountMeta.address) {
    throw new Error(`Compressed account meta address is null`);
  }
  const accountMetaBytes = Buffer.alloc(42);
  accountMetaBytes.writeUInt16LE(accountMeta.treeInfo.rootIndex, 0);
  accountMetaBytes.writeUInt8(accountMeta.treeInfo.proveByIndex ? 1 : 0, 2);
  accountMetaBytes.writeUInt8(accountMeta.treeInfo.merkleTreePubkeyIndex, 3);
  accountMetaBytes.writeUInt8(accountMeta.treeInfo.queuePubkeyIndex, 4);
  accountMetaBytes.writeUInt32LE(accountMeta.treeInfo.leafIndex, 5);
  Buffer.from(accountMeta.address).copy(accountMetaBytes, 9, 0, 32);
  accountMetaBytes.writeUInt8(accountMeta.outputStateTreeIndex, 41);
  return accountMetaBytes;
}

/**
 * Fetches the initialize record data for a given delegated account
 * @param photonClient - The Photon client
 * @param delegatedAccount - The delegated account address
 * @returns The initialize record data
 */
export async function fetchInitializeRecordData(
  photonClient: Rpc,
  delegatedAccount: PublicKey,
) {
  console.log("V2", featureFlags.isV2());
  const addressTree = await photonClient.getAddressTreeInfoV2();
  addressTree.queue = OUTPUT_QUEUE;

  const compressedDerivedAddress = deriveCda(delegatedAccount);

  const systemAccountConfig = SystemAccountMetaConfig.new(
    new PublicKey(COMPRESSED_DELEGATION_PROGRAM_ID),
  );
  const remainingAccounts =
    PackedAccounts.newWithSystemAccountsV2(systemAccountConfig);

  // Try to get the proof of a new address
  const result = await photonClient.getValidityProofV2(
    [],
    [
      {
        address: compressedDerivedAddress.toBytes(),
        treeInfo: addressTree,
      },
    ],
    DerivationMode.standard,
  );

  const addressMerkleTreePubkeyIndex = remainingAccounts.insertOrGet(
    addressTree.tree,
  );
  const outputStateTreeIndex = remainingAccounts.insertOrGet(addressTree.queue);
  const validityProof = result.compressedProof;
  const packedAddressTreeInfo: PackedAddressTreeInfo = {
    rootIndex: result.rootIndices[0],
    addressMerkleTreePubkeyIndex,
    addressQueuePubkeyIndex: outputStateTreeIndex,
  };

  return {
    validityProof,
    packedAddressTreeInfo,
    outputStateTreeIndex,
    remainingAccounts: remainingAccounts.toAccountMetas().remainingAccounts,
  };
}

export async function fetchInitializeRecordDataBytes(
  photonClient: Rpc,
  delegatedAccount: PublicKey,
) {
  let {
    validityProof,
    packedAddressTreeInfo,
    outputStateTreeIndex,
    remainingAccounts,
  } = await fetchInitializeRecordData(photonClient, delegatedAccount);

  const validityProofBytes = convertValidityProofToBytes(validityProof);
  const packedAddressTreeInfoBytes = convertPackedAddressTreeInfoToBytes(
    packedAddressTreeInfo,
  );
  const outputStateTreeIndexBytes =
    convertOutputStateTreeIndexToBytes(outputStateTreeIndex);

  return {
    validityProofBytes,
    packedAddressTreeInfoBytes,
    outputStateTreeIndexBytes,
    remainingAccounts,
  };
}

export async function fetchDelegateCompressedData(
  photonClient: Rpc,
  delegatedAccount: PublicKey,
) {
  const addressTree = await photonClient.getAddressTreeInfoV2();
  addressTree.queue = OUTPUT_QUEUE;

  await photonClient.getStateTreeInfos();
  photonClient.allStateTreeInfos?.push({
    // tree: BATCHED_MERKLE_TREE,
    tree: ADRESS_TREE,
    queue: OUTPUT_QUEUE,
    treeType: TreeType.StateV2,
    nextTreeInfo: null,
  });

  const compressedDerivedAddress = deriveCda(delegatedAccount);

  const compressedDelegatedRecord = await photonClient.getCompressedAccount(
    compressedDerivedAddress.toBytes(),
  );
  if (!compressedDelegatedRecord) {
    throw new Error(`Compressed delegated record not found`);
  }
  const result = await photonClient.getValidityProofV2(
    [
      {
        hash: compressedDelegatedRecord.hash,
        treeInfo: addressTree,
        leafIndex: compressedDelegatedRecord.leafIndex,
        proveByIndex: compressedDelegatedRecord.proveByIndex,
      },
    ],
    [],
    DerivationMode.standard,
  );

  const systemAccountConfig = SystemAccountMetaConfig.new(
    new PublicKey(COMPRESSED_DELEGATION_PROGRAM_ID),
  );
  const remainingAccounts =
    PackedAccounts.newWithSystemAccounts(systemAccountConfig);

  const validityProof = result.compressedProof;

  const packedTreeInfos = packTreeInfos(
    remainingAccounts.toAccountMetas().remainingAccounts.map((a) => a.pubkey),
    [
      {
        hash: compressedDelegatedRecord.hash,
        treeInfo: compressedDelegatedRecord.treeInfo,
        leafIndex: compressedDelegatedRecord.leafIndex,
        rootIndex: result.rootIndices[0],
        proveByIndex: compressedDelegatedRecord.proveByIndex !== null,
      },
    ],
    [],
  );

  const addressMerkleTreePubkeyIndex = remainingAccounts.insertOrGet(
    compressedDelegatedRecord.treeInfo.tree,
  );
  const addressQueuePubkeyIndex = remainingAccounts.insertOrGet(
    compressedDelegatedRecord.treeInfo.queue,
  );

  const accountMeta: CompressedAccountMeta = {
    treeInfo: {
      ...packedTreeInfos.stateTrees!.packedTreeInfos[0],
      merkleTreePubkeyIndex: addressMerkleTreePubkeyIndex,
      queuePubkeyIndex: addressQueuePubkeyIndex,
      leafIndex: compressedDelegatedRecord.leafIndex,
    },
    address: Array.from(compressedDerivedAddress.toBytes()),
    outputStateTreeIndex: addressQueuePubkeyIndex,
    lamports: null,
  };

  return {
    validityProof,
    accountMeta,
    remainingAccounts: remainingAccounts.toAccountMetas().remainingAccounts,
  };
}

export async function fetchDelegateCompressedDataBytes(
  photonClient: Rpc,
  delegatedAccount: PublicKey,
) {
  const { validityProof, accountMeta, remainingAccounts } =
    await fetchDelegateCompressedData(photonClient, delegatedAccount);

  const validityProofBytes = convertValidityProofToBytes(validityProof);
  const accountMetaBytes = convertCompressedAccountMetaToBytes(accountMeta);

  return {
    validityProofBytes,
    accountMetaBytes,
    remainingAccounts,
  };
}
