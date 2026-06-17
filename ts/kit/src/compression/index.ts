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
import {
  Address,
  AccountMeta,
  AccountRole,
  getAddressDecoder,
  getAddressEncoder,
  address,
} from "@solana/kit";
import { PublicKey, type AccountMeta as Web3AccountMeta } from "@solana/web3.js";

// Ensure we use V2
featureFlags.version = VERSION.V2;

const addressEncoder = getAddressEncoder();
const addressDecoder = getAddressDecoder();

export const COMPRESSED_DELEGATION_PROGRAM_ID = address(
  "DEL2rPzhFaS5qzo8XY9ZNxSzuunWueySq3p2dxJfwPbT",
);
export const BATCHED_MERKLE_TREE = address(
  "bmt1LryLZUMmF7ZtqESaw7wifBXLfXHQYoE4GAmrahU",
);
export const ADDRESS_TREE = address(
  "amt2kaJA14v3urZbZvnc5v2np8jqvc4Z8zDep5wbtzx",
);
export const OUTPUT_QUEUE = address(
  "oq1na8gojfdUhsfCpyjNt6h4JaDWtHf1yQj4koBWfto",
);

function toPublicKey(value: Address): PublicKey {
  return new PublicKey(addressEncoder.encode(value));
}

function toAddressBytes(value: Address): Uint8Array {
  return new Uint8Array(addressEncoder.encode(value));
}

function toAddress(value: PublicKey): Address {
  return addressDecoder.decode(value.toBytes());
}

function toKitAccountMetas(metas: Web3AccountMeta[]): AccountMeta[] {
  return metas.map((meta) => ({
    address: toAddress(meta.pubkey),
    role:
      meta.isSigner && meta.isWritable
        ? AccountRole.WRITABLE_SIGNER
        : meta.isSigner
          ? AccountRole.READONLY_SIGNER
          : meta.isWritable
            ? AccountRole.WRITABLE
            : AccountRole.READONLY,
  }));
}

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
 * @param addressTree - The address tree address
 * @returns The compressed derived address
 */
export function deriveCda(delegatedAccount: Address, addressTree: Address) {
  const addressSeed = deriveAddressSeed([toAddressBytes(delegatedAccount)]);
  return toAddress(
    deriveAddress(
      addressSeed,
      toPublicKey(addressTree),
      toPublicKey(COMPRESSED_DELEGATION_PROGRAM_ID),
    ),
  );
}

export function convertValidityProofToBytes(
  validityProof: ValidityProof | null,
): Uint8Array {
  if (!validityProof) {
    return new Uint8Array([0]);
  }
  return new Uint8Array([
    1,
    ...validityProof.a,
    ...validityProof.b,
    ...validityProof.c,
  ]);
}

export function convertPackedAddressTreeInfoToBytes(
  packedAddressTreeInfo: PackedAddressTreeInfo,
): Uint8Array {
  // Wire layout must match the on-chain `CdpPackedAddressTreeInfo` decoder in
  // magicblock-validator's compressed-delegation-api:
  //   byte 0:    address_merkle_tree_pubkey_index (u8)
  //   byte 1:    address_queue_pubkey_index      (u8)
  //   bytes 2-3: root_index                      (u16 LE)
  const bytes = new Uint8Array(4);
  const view = new DataView(bytes.buffer);
  view.setUint8(0, packedAddressTreeInfo.addressMerkleTreePubkeyIndex);
  view.setUint8(1, packedAddressTreeInfo.addressQueuePubkeyIndex);
  view.setUint16(2, packedAddressTreeInfo.rootIndex, true);
  return bytes;
}

export function convertOutputStateTreeIndexToBytes(
  outputStateTreeIndex: number,
): Uint8Array {
  return new Uint8Array([outputStateTreeIndex]);
}

export function convertCompressedAccountMetaToBytes(
  accountMeta: CompressedAccountMeta,
): Uint8Array {
  if (!accountMeta.address) {
    throw new Error(`Compressed account meta address is null`);
  }
  const bytes = new Uint8Array(42);
  const view = new DataView(bytes.buffer);
  view.setUint16(0, accountMeta.treeInfo.rootIndex, true);
  view.setUint8(2, accountMeta.treeInfo.proveByIndex ? 1 : 0);
  view.setUint8(3, accountMeta.treeInfo.merkleTreePubkeyIndex);
  view.setUint8(4, accountMeta.treeInfo.queuePubkeyIndex);
  view.setUint32(5, accountMeta.treeInfo.leafIndex, true);
  bytes.set(accountMeta.address, 9);
  view.setUint8(41, accountMeta.outputStateTreeIndex);
  return bytes;
}

/**
 * Fetches the initialize record data for a given delegated account
 * @param photonClient - The Photon client
 * @param delegatedAccount - The delegated account address
 * @returns The initialize record data
 */
export async function fetchInitializeRecordData(
  photonClient: Rpc,
  delegatedAccount: Address,
) {
  const addressTreeInfo = await photonClient.getAddressTreeInfoV2();
  const addressTree = { ...addressTreeInfo, queue: toPublicKey(OUTPUT_QUEUE) };

  const compressedDerivedAddress = deriveCda(
    delegatedAccount,
    toAddress(addressTree.tree),
  );

  const systemAccountConfig = SystemAccountMetaConfig.new(
    toPublicKey(COMPRESSED_DELEGATION_PROGRAM_ID),
  );
  const remainingAccounts =
    PackedAccounts.newWithSystemAccountsV2(systemAccountConfig);

  // Try to get the proof of a new address
  const result = await photonClient.getValidityProofV2(
    [],
    [
      {
        address: toAddressBytes(compressedDerivedAddress),
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
    remainingAccounts: toKitAccountMetas(
      remainingAccounts.toAccountMetas().remainingAccounts,
    ),
  };
}

export async function fetchInitializeRecordDataBytes(
  photonClient: Rpc,
  delegatedAccount: Address,
) {
  const {
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
  delegatedAccount: Address,
) {
  const addressTreeInfo = await photonClient.getAddressTreeInfoV2();
  const addressTree = { ...addressTreeInfo, queue: toPublicKey(OUTPUT_QUEUE) };

  await photonClient.getStateTreeInfos();
  photonClient.allStateTreeInfos?.push({
    tree: toPublicKey(BATCHED_MERKLE_TREE),
    queue: toPublicKey(OUTPUT_QUEUE),
    treeType: TreeType.StateV2,
    nextTreeInfo: null,
  });

  const compressedDerivedAddress = deriveCda(
    delegatedAccount,
    toAddress(addressTree.tree),
  );

  const compressedDelegatedRecord = await photonClient.getCompressedAccount(
    toAddressBytes(compressedDerivedAddress),
  );
  if (!compressedDelegatedRecord) {
    throw new Error(`Compressed delegated record not found`);
  }
  const result = await photonClient.getValidityProofV2(
    [
      {
        hash: compressedDelegatedRecord.hash,
        treeInfo: compressedDelegatedRecord.treeInfo,
        leafIndex: compressedDelegatedRecord.leafIndex,
        proveByIndex: compressedDelegatedRecord.proveByIndex,
      },
    ],
    [],
    DerivationMode.standard,
  );

  const systemAccountConfig = SystemAccountMetaConfig.new(
    toPublicKey(COMPRESSED_DELEGATION_PROGRAM_ID),
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

  if (!packedTreeInfos.stateTrees) {
    throw new Error(`Packed tree infos state trees is null`);
  }

  const accountMeta: CompressedAccountMeta = {
    treeInfo: {
      ...packedTreeInfos.stateTrees.packedTreeInfos[0],
      merkleTreePubkeyIndex: addressMerkleTreePubkeyIndex,
      queuePubkeyIndex: addressQueuePubkeyIndex,
      leafIndex: compressedDelegatedRecord.leafIndex,
    },
    address: Array.from(toAddressBytes(compressedDerivedAddress)),
    outputStateTreeIndex: addressQueuePubkeyIndex,
    lamports: null,
  };

  return {
    validityProof,
    accountMeta,
    remainingAccounts: toKitAccountMetas(
      remainingAccounts.toAccountMetas().remainingAccounts,
    ),
  };
}

export async function fetchDelegateCompressedDataBytes(
  photonClient: Rpc,
  delegatedAccount: Address,
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
