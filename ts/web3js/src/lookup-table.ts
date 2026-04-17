import {
  AddressLookupTableAccount,
  Transaction,
  TransactionMessage,
  VersionedTransaction,
} from "@solana/web3.js";

export interface CompileLegacyTransactionToV0Input {
  transaction: Transaction;
  lookupTables: AddressLookupTableAccount[];
}

export interface CompileLegacyTransactionToV0Result {
  transaction: VersionedTransaction;
  legacySize: number;
  v0Size: number;
  bytesSaved: number;
  usedLookupTables: string[];
}

/**
 * Compile a prepared legacy transaction into a v0 transaction using the
 * provided address lookup tables.
 *
 * The input transaction must already have a fee payer and recent blockhash.
 * This helper performs no RPC calls and does not fetch lookup tables.
 */
export function compileLegacyTransactionToV0({
  transaction,
  lookupTables,
}: CompileLegacyTransactionToV0Input): CompileLegacyTransactionToV0Result {
  if (transaction.feePayer == null) {
    throw new Error("transaction.feePayer is required");
  }

  if (transaction.recentBlockhash == null) {
    throw new Error("transaction.recentBlockhash is required");
  }

  const legacySize = transaction.serialize({
    requireAllSignatures: false,
    verifySignatures: false,
  }).length;

  const message = new TransactionMessage({
    payerKey: transaction.feePayer,
    recentBlockhash: transaction.recentBlockhash,
    instructions: transaction.instructions,
  }).compileToV0Message(lookupTables);

  const versionedTransaction = new VersionedTransaction(message);
  const v0Size = versionedTransaction.serialize().length;

  return {
    transaction: versionedTransaction,
    legacySize,
    v0Size,
    bytesSaved: legacySize - v0Size,
    usedLookupTables: message.addressTableLookups.map((lookup) =>
      lookup.accountKey.toBase58(),
    ),
  };
}
