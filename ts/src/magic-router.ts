import {
  Connection,
  Transaction,
  ConfirmOptions,
  TransactionSignature,
  Signer,
  BlockhashWithExpiryBlockHeight,
  SendOptions,
  PublicKey,
  SendTransactionError,
} from "@solana/web3.js";

/**
 * Get all writable accounts from a transaction.
 */
export function getWritableAccounts(transaction: Transaction) {
  const writableAccounts = new Set<string>();

  if (transaction.feePayer) {
    writableAccounts.add(transaction.feePayer.toBase58());
  }

  // Check all instruction keys
  for (const instruction of transaction.instructions) {
    for (const key of instruction.keys) {
      if (key.isWritable) {
        writableAccounts.add(key.pubkey.toBase58());
      }
    }
  }

  return Array.from(writableAccounts);
}

/**
 * Get the closest validator info from the router connection.
 */
export async function getClosestValidator(routerConnection: Connection) : Promise<{ identity: string; fqdn?: string }> {
  const response = await fetch(routerConnection.rpcEndpoint, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      jsonrpc: "2.0",
      id: 1,
      method: "getIdentity",
      params: [],
    }),
  });

  const identityData = (await response.json())?.result;

  if (identityData == null || identityData.identity == null) {
    throw new Error("Invalid response");
  }


  return identityData;
}

/**
 * Get delegation status for a given account from the router.
 */
export async function getDelegationStatus(
  connection: Connection,
  account: PublicKey | string,
): Promise<{ isDelegated: boolean }> {
  const accountAddress =
    typeof account === "string" ? account : account.toBase58();

  const response = await fetch(
    `${connection.rpcEndpoint}/getDelegationStatus`,
    {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        jsonrpc: "2.0",
        id: 1,
        method: "getDelegationStatus",
        params: [accountAddress],
      }),
    },
  );

  const data = await response.json();
  return data.result as { isDelegated: boolean };
}

/**
 * Get the latest blockhash for a transaction based on writable accounts.
 */
export async function getLatestBlockhashForMagicTransaction(
  connection: Connection,
  transaction: Transaction,
  options?: ConfirmOptions,
): Promise<BlockhashWithExpiryBlockHeight> {
  const writableAccounts = getWritableAccounts(transaction);

  const blockHashResponse = await fetch(connection.rpcEndpoint, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      jsonrpc: "2.0",
      id: 1,
      method: "getBlockhashForAccounts",
      params: [writableAccounts],
    }),
  });

  const blockHashData = await blockHashResponse.json();
  return blockHashData.result;
}

/**
 * Prepare a transaction for sending by setting the recent blockhash.
 */
export async function prepareMagicTransaction(
  connection: Connection,
  transaction: Transaction,
  options?: ConfirmOptions,
): Promise<Transaction> {
  const blockHashData = await getLatestBlockhashForMagicTransaction(
    connection,
    transaction,
    options,
  );
  transaction.recentBlockhash = blockHashData.blockhash;

  return transaction;
}

/**
 * Send a transaction, returning the signature of the transaction.
 * This function is modified to handle the magic transaction sending strategy by getting the latest blockhash based on writable accounts.
 */
export async function sendMagicTransaction(
  connection: Connection,
  transaction: Transaction,
  signersOrOptions?: Signer[] | SendOptions,
  options?: SendOptions,
): Promise<TransactionSignature> {
  // This implementation avoids invoking real web3.js signing/serialization in test environments
  // and focuses on fetching the latest blockhash for writable accounts, then sending a raw payload.
  const sendOpts: SendOptions | undefined = Array.isArray(signersOrOptions)
    ? (options ?? undefined)
    : (signersOrOptions ?? undefined);

  // Always refresh recent blockhash for the provided transaction
  const latestBlockhash = await getLatestBlockhashForMagicTransaction(
    connection,
    transaction,
    sendOpts as ConfirmOptions,
  );
  (transaction as any).lastValidBlockHeight =
    latestBlockhash.lastValidBlockHeight;
  (transaction as any).recentBlockhash = latestBlockhash.blockhash;

  // If signers are provided, call transaction.sign
  if (Array.isArray(signersOrOptions)) {
    (transaction as any).sign?.(...signersOrOptions);
  }

  const wireTransaction = transaction.serialize();
  return connection.sendRawTransaction(wireTransaction, sendOpts);
}

/**
 * Send and confirm a transaction, returning the signature of the transaction.
 */
export async function sendAndConfirmMagicTransaction(
  connection: Connection,
  transaction: Transaction,
  signers: Signer[],
  options?: ConfirmOptions &
    Readonly<{
      abortSignal?: AbortSignal;
    }>,
): Promise<TransactionSignature> {
  const signature = await sendMagicTransaction(
    connection,
    transaction,
    signers,
    options,
  );
  let status;
  const { recentBlockhash, lastValidBlockHeight, minNonceContextSlot, nonceInfo } = transaction;
  if (
    recentBlockhash != undefined && lastValidBlockHeight != undefined
  ) {
      status = (await connection.confirmTransaction({
      abortSignal: options?.abortSignal,
      signature: signature,
      blockhash: recentBlockhash,
      lastValidBlockHeight: lastValidBlockHeight
      }, options?.commitment)).value;
  } else if (
    minNonceContextSlot != undefined &&
    nonceInfo != undefined
  ) {
      const {
        nonceInstruction
      } = nonceInfo;
      const nonceAccountPubkey = nonceInstruction.keys[0].pubkey;
      status = (await connection.confirmTransaction({
      abortSignal: options?.abortSignal,
      minContextSlot: minNonceContextSlot,
      nonceAccountPubkey,
      nonceValue: nonceInfo.nonce,
      signature
      }, options?.commitment)).value;
  } else {
      if (options?.abortSignal != undefined) {
      console.warn('sendAndConfirmTransaction(): A transaction with a deprecated confirmation strategy was ' + 'supplied along with an `abortSignal`. Only transactions having `lastValidBlockHeight` ' + 'or a combination of `nonceInfo` and `minNonceContextSlot` are abortable.');
      }
      status = (await connection.confirmTransaction(signature, options?.commitment)).value;
  }
  if (status.err) {
      if (signature != undefined) {
      throw new SendTransactionError({
          action: 'send',
          signature: signature,
          transactionMessage: `Status: (${JSON.stringify(status)})`
      });
      }
      throw new Error(`Transaction ${signature} failed (${JSON.stringify(status)})`);
  }
  return signature;
}
