import { describe, expect, it } from "vitest";
import {
  AddressLookupTableAccount,
  Keypair,
  PublicKey,
  Transaction,
} from "@solana/web3.js";

import { compileLegacyTransactionToV0 } from "../lookup-table.js";
import { transferSpl } from "../instructions/ephemeral-spl-token-program/index.js";

const MAX_DEACTIVATION_SLOT = BigInt("18446744073709551615");

function createLookupTable(
  key: PublicKey,
  addresses: PublicKey[],
): AddressLookupTableAccount {
  return new AddressLookupTableAccount({
    key,
    state: {
      deactivationSlot: MAX_DEACTIVATION_SLOT,
      lastExtendedSlot: 0,
      lastExtendedSlotStartIndex: 0,
      authority: undefined,
      addresses,
    },
  });
}

function collectNonSignerAccounts(transaction: Transaction) {
  const byAddress = new Map<string, PublicKey>();

  for (const instruction of transaction.instructions) {
    byAddress.set(instruction.programId.toBase58(), instruction.programId);

    for (const key of instruction.keys) {
      if (!key.isSigner) {
        byAddress.set(key.pubkey.toBase58(), key.pubkey);
      }
    }
  }

  return [...byAddress.values()];
}

describe("compileLegacyTransactionToV0", () => {
  it("throws when feePayer is missing", () => {
    const transaction = new Transaction();
    transaction.recentBlockhash = "11111111111111111111111111111111";

    expect(() =>
      compileLegacyTransactionToV0({
        transaction,
        lookupTables: [],
      }),
    ).toThrow("transaction.feePayer is required");
  });

  it("throws when recentBlockhash is missing", () => {
    const transaction = new Transaction();
    transaction.feePayer = Keypair.generate().publicKey;

    expect(() =>
      compileLegacyTransactionToV0({
        transaction,
        lookupTables: [],
      }),
    ).toThrow("transaction.recentBlockhash is required");
  });

  it("compiles a prepared legacy transaction to v0 using lookup tables", async () => {
    const from = Keypair.generate().publicKey;
    const to = Keypair.generate().publicKey;
    const mint = Keypair.generate().publicKey;
    const validator = Keypair.generate().publicKey;
    const instructions = await transferSpl(from, to, mint, 25n, {
      visibility: "private",
      fromBalance: "base",
      toBalance: "base",
      validator,
      shuttleId: 7,
      privateTransfer: {
        minDelayMs: 100n,
        maxDelayMs: 300n,
        split: 4,
      },
    });
    const transaction = new Transaction();
    const lookupTableKey = new PublicKey(
      "AddressLookupTab1e1111111111111111111111111",
    );

    transaction.feePayer = from;
    transaction.recentBlockhash = "11111111111111111111111111111111";
    transaction.add(...instructions);

    const lookupTable = createLookupTable(
      lookupTableKey,
      collectNonSignerAccounts(transaction),
    );
    const result = compileLegacyTransactionToV0({
      transaction,
      lookupTables: [lookupTable],
    });

    expect(result.usedLookupTables).toEqual([lookupTableKey.toBase58()]);
    expect(result.transaction.message.addressTableLookups).toHaveLength(1);
    expect(result.bytesSaved).toBeGreaterThan(0);
    expect(result.v0Size).toBeLessThan(result.legacySize);
  });

  it("returns no used lookup tables when none of the addresses match", async () => {
    const from = Keypair.generate().publicKey;
    const to = Keypair.generate().publicKey;
    const mint = Keypair.generate().publicKey;
    const validator = Keypair.generate().publicKey;
    const instructions = await transferSpl(from, to, mint, 25n, {
      visibility: "private",
      fromBalance: "base",
      toBalance: "base",
      validator,
      shuttleId: 7,
      privateTransfer: {
        minDelayMs: 100n,
        maxDelayMs: 300n,
        split: 4,
      },
    });
    const transaction = new Transaction();
    const unrelatedLookupTable = createLookupTable(
      new PublicKey("11111111111111111111111111111111"),
      [Keypair.generate().publicKey],
    );

    transaction.feePayer = from;
    transaction.recentBlockhash = "11111111111111111111111111111111";
    transaction.add(...instructions);

    const result = compileLegacyTransactionToV0({
      transaction,
      lookupTables: [unrelatedLookupTable],
    });

    expect(result.usedLookupTables).toEqual([]);
    expect(result.transaction.message.addressTableLookups).toHaveLength(0);
  });
});
