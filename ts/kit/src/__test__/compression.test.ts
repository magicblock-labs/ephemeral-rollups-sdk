import { describe, expect, it } from "vitest";
import type { CompressedAccountMeta, PackedAddressTreeInfo } from "@lightprotocol/stateless.js";
import {
  convertCompressedAccountMetaToBytes,
  convertOutputStateTreeIndexToBytes,
  convertPackedAddressTreeInfoToBytes,
  convertValidityProofToBytes,
} from "../compression/index.js";

describe("compression byte converters", () => {
  it("convertValidityProofToBytes encodes null as [0]", () => {
    expect(Array.from(convertValidityProofToBytes(null))).toEqual([0]);
  });

  it("convertValidityProofToBytes encodes proof with leading tag byte", () => {
    const proof = {
      a: [1, 2],
      b: [3, 4],
      c: [5, 6],
    };
    expect(Array.from(convertValidityProofToBytes(proof))).toEqual([
      1, 1, 2, 3, 4, 5, 6,
    ]);
  });

  it("convertPackedAddressTreeInfoToBytes matches wire layout", () => {
    const info: PackedAddressTreeInfo = {
      addressMerkleTreePubkeyIndex: 7,
      addressQueuePubkeyIndex: 9,
      rootIndex: 0x1234,
    };
    const bytes = convertPackedAddressTreeInfoToBytes(info);
    expect(bytes.length).toBe(4);
    expect(bytes[0]).toBe(7);
    expect(bytes[1]).toBe(9);
    expect(bytes[2]).toBe(0x34);
    expect(bytes[3]).toBe(0x12);
  });

  it("convertOutputStateTreeIndexToBytes encodes a single byte", () => {
    expect(Array.from(convertOutputStateTreeIndexToBytes(42))).toEqual([42]);
  });

  it("convertCompressedAccountMetaToBytes matches wire layout", () => {
    const accountMeta: CompressedAccountMeta = {
      treeInfo: {
        rootIndex: 0xabcd,
        proveByIndex: true,
        merkleTreePubkeyIndex: 3,
        queuePubkeyIndex: 4,
        leafIndex: 0x01020304,
      },
      address: Array.from({ length: 32 }, (_, i) => i),
      outputStateTreeIndex: 5,
      lamports: null,
    };

    const bytes = convertCompressedAccountMetaToBytes(accountMeta);
    expect(bytes.length).toBe(42);
    expect(bytes[2]).toBe(1);
    expect(bytes[3]).toBe(3);
    expect(bytes[4]).toBe(4);
    expect(bytes[5]).toBe(0x04);
    expect(bytes[6]).toBe(0x03);
    expect(bytes[7]).toBe(0x02);
    expect(bytes[8]).toBe(0x01);
    expect(Array.from(bytes.slice(9, 41))).toEqual(accountMeta.address);
    expect(bytes[41]).toBe(5);
  });

  it("convertCompressedAccountMetaToBytes rejects null address", () => {
    const accountMeta = {
      treeInfo: {
        rootIndex: 0,
        proveByIndex: false,
        merkleTreePubkeyIndex: 0,
        queuePubkeyIndex: 0,
        leafIndex: 0,
      },
      address: null,
      outputStateTreeIndex: 0,
      lamports: null,
    } as CompressedAccountMeta;

    expect(() => convertCompressedAccountMetaToBytes(accountMeta)).toThrow(
      "Compressed account meta address is null",
    );
  });
});
