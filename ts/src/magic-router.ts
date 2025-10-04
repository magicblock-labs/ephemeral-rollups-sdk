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
 * Patch Connection prototype with custom method:
 * Get the closest validator info from the router connection.
 */
(Connection.prototype as any).getClosestValidator = async function (): Promise<{
  identity: string;
  fqdn?: string;
}> {
  const response = await fetch(this.rpcEndpoint as string, {
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
  if (identityData === null || identityData.identity === undefined) {
    throw new Error("Invalid response");
  }
  return identityData;
};

/**
 * Patch Connection prototype with custom method:
 * Get delegation status for a given account from the router.
 */
(Connection.prototype as any).getDelegationStatus = async function (
  account: PublicKey | string,
): Promise<{ isDelegated: boolean }> {
  const accountAddress =
    typeof account === "string" ? account : account.toBase58();

  const response = await fetch(
    `${this.rpcEndpoint as string}/getDelegationStatus`,
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

  return (await response.json()).result;
};

/**
 * Patch Connection prototype with custom method:
 * Get the latest blockhash for a transaction based on writable accounts.
 */
(Connection.prototype as any).getLatestBlockhashForTransaction =
  async function (
    transaction: Transaction,
    options?: ConfirmOptions,
  ): Promise<BlockhashWithExpiryBlockHeight> {
    const writableAccounts = getWritableAccounts(transaction);

    const blockHashResponse = await fetch(this.rpcEndpoint as string, {
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
  };

/**
 * Patch Connection prototype with custom method:
 * Prepare a transaction for sending by setting the recent blockhash.
 */
(Connection.prototype as any).prepareTransaction = async function (
  transaction: Transaction,
  options?: ConfirmOptions,
): Promise<Transaction> {
  const blockHashData = await this.getLatestBlockhashForTransaction(
    transaction,
    options,
  );
  transaction.recentBlockhash = blockHashData.blockhash;
  return transaction;
};

/**
 * Patch Connection prototype with custom method:
 * Send a transaction, returning the signature of the transaction.
 * This function is modified to handle the magic transaction sending strategy by getting the latest blockhash based on writable accounts.
 */
(Connection.prototype as any).sendTransaction = async function (
  transaction: Transaction,
  signersOrOptions?: Signer[] | SendOptions,
  options?: SendOptions,
): Promise<TransactionSignature> {
  const sendOpts: SendOptions | undefined = Array.isArray(signersOrOptions)
    ? (options ?? undefined)
    : (signersOrOptions ?? undefined);

  const latestBlockhash = await this.getLatestBlockhashForTransaction(
    transaction,
    sendOpts as ConfirmOptions,
  );
  (transaction as any).recentBlockhash = latestBlockhash.blockhash;
  (transaction as any).lastValidBlockHeight =
    latestBlockhash.lastValidBlockHeight;

  if (Array.isArray(signersOrOptions)) {
    transaction.sign(...signersOrOptions);
  }

  const wireTransaction = transaction.serialize();
  return this.sendRawTransaction(wireTransaction, sendOpts);
};

/**
 * Patch Connection prototype with custom method:
 * Send and confirm a transaction, returning the signature of the transaction.
 * This function is modified to handle the magic transaction sending strategy by getting the latest blockhash based on writable accounts.
 */
(Connection.prototype as any).sendAndConfirmTransaction = async function (
  transaction: Transaction,
  signers: Signer[],
  options?: ConfirmOptions & { abortSignal?: AbortSignal },
): Promise<TransactionSignature> {
  const signature = await this.sendTransaction(transaction, signers, options);
  let status;
  const {
    recentBlockhash,
    lastValidBlockHeight,
    minNonceContextSlot,
    nonceInfo,
  } = transaction;

  if (recentBlockhash !== undefined && lastValidBlockHeight !== undefined) {
    status = (
      await this.confirmTransaction(
        {
          abortSignal: options?.abortSignal,
          signature,
          blockhash: recentBlockhash,
          lastValidBlockHeight,
        },
        options?.commitment,
      )
    ).value;
  } else if (minNonceContextSlot !== undefined && nonceInfo !== undefined) {
    const { nonceInstruction } = nonceInfo;
    const nonceAccountPubkey = nonceInstruction.keys[0].pubkey;
    status = (
      await this.confirmTransaction(
        {
          abortSignal: options?.abortSignal,
          minContextSlot: minNonceContextSlot,
          nonceAccountPubkey,
          nonceValue: nonceInfo.nonce,
          signature,
        },
        options?.commitment,
      )
    ).value;
  } else {
    status = (await this.confirmTransaction(signature, options?.commitment))
      .value;
  }

  if (status.err != null) {
    throw new SendTransactionError({
      action: "send",
      signature,
      transactionMessage: `Status: (${JSON.stringify(status)})`,
    });
  }

  return signature;
};

export { Connection };
