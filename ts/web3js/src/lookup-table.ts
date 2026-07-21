import {
  AddressLookupTableAccount,
  type Message,
  PUBLIC_KEY_LENGTH,
  SIGNATURE_LENGTH_IN_BYTES,
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

function getShortVecEncodedLength(value: number): number {
  let remaining = value;
  let length = 1;

  while (remaining >= 0x80) {
    remaining = Math.floor(remaining / 0x80);
    length += 1;
  }

  return length;
}

function getLegacyTransactionSize(message: Message): number {
  const instructions = message.compiledInstructions;
  const instructionsSize = instructions.reduce(
    (size, instruction) =>
      size +
      1 +
      getShortVecEncodedLength(instruction.accountKeyIndexes.length) +
      instruction.accountKeyIndexes.length +
      getShortVecEncodedLength(instruction.data.length) +
      instruction.data.length,
    0,
  );
  const messageSize =
    3 +
    getShortVecEncodedLength(message.accountKeys.length) +
    message.accountKeys.length * PUBLIC_KEY_LENGTH +
    PUBLIC_KEY_LENGTH +
    getShortVecEncodedLength(instructions.length) +
    instructionsSize;
  const signatureCount = message.header.numRequiredSignatures;

  return (
    getShortVecEncodedLength(signatureCount) +
    signatureCount * SIGNATURE_LENGTH_IN_BYTES +
    messageSize
  );
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

  const legacySize = getLegacyTransactionSize(transaction.compileMessage());

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
