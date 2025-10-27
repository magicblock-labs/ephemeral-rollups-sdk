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
  VersionedTransaction,
} from "@solana/web3.js";

/**
 * Get all writable accounts from a transaction.
 * @param {Transaction} transaction
 * @returns {string[]}
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
 * Extended Connection class that adds custom RPC router methods.
 */
export class ConnectionMagicRouter extends Connection {
  /**
   * Get the closest validator info from the router connection.
   * @returns {Promise<{identity: string, fqdn?: string}>}
   */
  async getClosestValidator(): Promise<{ identity: string; fqdn?: string }> {
    const response = await fetch(this.rpcEndpoint, {
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
  }

  /**
   * Get delegation status for a given account from the router.
   * @param {PublicKey | string} account
   * @returns {Promise<{isDelegated: boolean}>}
   */
  async getDelegationStatus(
    account: PublicKey | string,
  ): Promise<{ isDelegated: boolean }> {
    const accountAddress =
      typeof account === "string" ? account : account.toBase58();

    const response = await fetch(`${this.rpcEndpoint}/getDelegationStatus`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        jsonrpc: "2.0",
        id: 1,
        method: "getDelegationStatus",
        params: [accountAddress],
      }),
    });

    return (await response.json()).result;
  }

  /**
   * Get the latest blockhash for a transaction based on writable accounts.
   * @param {Transaction} transaction
   * @param {ConfirmOptions} [options]
   * @returns {Promise<BlockhashWithExpiryBlockHeight>}
   */
  async getLatestBlockhashForTransaction(
    transaction: Transaction,
    options?: ConfirmOptions,
  ): Promise<BlockhashWithExpiryBlockHeight> {
    const writableAccounts = getWritableAccounts(transaction);

    const blockHashResponse = await fetch(this.rpcEndpoint, {
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
   * @param {Transaction} transaction
   * @param {ConfirmOptions} [options]
   * @returns {Promise<Transaction>}
   */
  async prepareTransaction(
    transaction: Transaction,
    options?: ConfirmOptions,
  ): Promise<Transaction> {
    const blockHashData = await this.getLatestBlockhashForTransaction(
      transaction,
      options,
    );
    transaction.recentBlockhash = blockHashData.blockhash;
    return transaction;
  }

  /**
   * Send a transaction, returning the signature of the transaction.
   * Modified to handle magic transaction sending strategy.
   * @param {Transaction | VersionedTransaction} transaction
   * @param {Signer[] | SendOptions} [signersOrOptions]
   * @param {SendOptions} [options]
   * @returns {Promise<TransactionSignature>}
   */
  async sendTransaction(
    transaction: Transaction | VersionedTransaction,
    signersOrOptions?: Signer[] | SendOptions,
    options?: SendOptions,
  ): Promise<TransactionSignature> {
    // ✅ Implementation goes here
    // You can check the type dynamically:
    if (transaction instanceof Transaction) {
      // Legacy
      const latestBlockhash =
        await this.getLatestBlockhashForTransaction(transaction);
      transaction.recentBlockhash = latestBlockhash.blockhash;
      transaction.lastValidBlockHeight = latestBlockhash.lastValidBlockHeight;

      if (Array.isArray(signersOrOptions)) {
        transaction.sign(...signersOrOptions);
      }

      const wireTx = transaction.serialize();
      return this.sendRawTransaction(wireTx, options);
    } else {
      // VersionedTransaction
      return super.sendTransaction(
        transaction,
        signersOrOptions as SendOptions,
      );
    }
  }

  /**
   * Send and confirm a transaction, returning the signature of the transaction.
   * Modified to handle the magic transaction sending strategy.
   * @param {Transaction} transaction
   * @param {Signer[]} signers
   * @param {ConfirmOptions & {abortSignal?: AbortSignal}} [options]
   * @returns {Promise<TransactionSignature>}
   */
  async sendAndConfirmTransaction(
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
  }
}
