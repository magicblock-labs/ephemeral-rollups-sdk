import {
  assertIsTransactionMessageWithBlockhashLifetime,
  Blockhash,
  Commitment,
  compileTransaction,
  createSolanaRpc,
  createSolanaRpcSubscriptions,
  getBase64EncodedWireTransaction,
  partiallySignTransaction,
  pipe,
  Rpc,
  RpcSubscriptions,
  Signature,
  SolanaRpcSubscriptionsApi,
  TransactionMessage,
  TransactionMessageWithFeePayer,
  setTransactionMessageLifetimeUsingBlockhash,
  SolanaRpcApi,
  TransactionWithBlockhashLifetime,
  TransactionMessageBytes,
  SignaturesMap,
} from "@solana/kit";

import {
  createRecentSignatureConfirmationPromiseFactory,
  getTimeoutPromise,
} from "@solana/transaction-confirmation";
import { waitForRecentTransactionConfirmationUntilTimeout } from "./confirmation";
import {
  getWritableAccounts,
  isRouter,
  parseCommitsLogsMessage,
  parseScheduleCommitsLogsMessage,
} from "./utils";

/** Type representing a recent blockhash and its lifetime validity. */
type LatestBlockhash = Readonly<{
  blockhash: Blockhash;
  lastValidBlockHeight: bigint;
}>;

/**
 * Represents a connection to a Solana cluster (HTTP + WebSocket).
 * Provides RPC and subscription APIs for interacting with Solana nodes.
 *
 * This class abstracts sending transactions, confirming them, and fetching
 * blockhashes — and optionally integrates with router nodes that support
 * `getBlockhashForAccounts`.
 */
export class Connection {
  clusterUrlHttp: string;
  clusterUrlWs: string;
  rpc: Rpc<SolanaRpcApi>;
  rpcSubscriptions: RpcSubscriptions<SolanaRpcSubscriptionsApi>;
  isMagicRouter: boolean;

  private constructor(
    clusterUrlHttp: string,
    clusterUrlWs: string,
    rpc: Rpc<SolanaRpcApi>,
    rpcSubscriptions: RpcSubscriptions<SolanaRpcSubscriptionsApi>,
    isRouter = false,
  ) {
    this.clusterUrlHttp = clusterUrlHttp;
    this.clusterUrlWs = clusterUrlWs;
    this.rpc = rpc;
    this.rpcSubscriptions = rpcSubscriptions;
    this.isMagicRouter = isRouter;
  }

  /**
   * Creates a new `Connection` instance for a given Solana cluster.
   * Automatically detects whether the node supports router extensions.
   *
   * @param clusterUrlHttp - The HTTP RPC endpoint of the cluster.
   * @param clusterUrlWs - (Optional) The WebSocket endpoint. Defaults to
   *                       the same URL with `ws` instead of `http`.
   *
   * @returns A ready-to-use `Connection` instance.
   */
  public static async create(
    clusterUrlHttp: string,
    clusterUrlWs?: string,
  ): Promise<Connection> {
    const isRouterResult = await isRouter(clusterUrlHttp);
    const rpc = createSolanaRpc(clusterUrlHttp);
    const rpcSubscriptions = createSolanaRpcSubscriptions(
      clusterUrlWs ?? clusterUrlHttp.replace(/^http/, "ws"),
    );
    return new Connection(
      clusterUrlHttp,
      clusterUrlWs ?? clusterUrlHttp.replace(/^http/, "ws"),
      rpc,
      rpcSubscriptions,
      isRouterResult,
    );
  }

  /**
   * Waits for a transaction to reach the desired commitment level.
   *
   * @param signature - The transaction signature to confirm.
   * @param options - Confirmation configuration.
   * @param options.commitment - Desired commitment level (e.g., "confirmed", "finalized").
   * @param options.abortSignal - Optional signal to abort waiting after a timeout.
   */
  public async confirmTransaction(
    signature: Signature,
    options?: {
      commitment?: Commitment;
      abortSignal?: AbortSignal;
    },
  ): Promise<void> {
    const {
      commitment = "confirmed",
      abortSignal = AbortSignal.timeout(12000),
    } = options ?? {};
    const solanaRpc = {
      rpc: this.rpc,
      rpcSubscriptions: this.rpcSubscriptions,
    };
    const getRecentSignatureConfirmationPromise =
      createRecentSignatureConfirmationPromiseFactory(solanaRpc);
    await waitForRecentTransactionConfirmationUntilTimeout({
      getTimeoutPromise: async () =>
        getTimeoutPromise({
          abortSignal,
          commitment,
        }),
      getRecentSignatureConfirmationPromise: async () =>
        getRecentSignatureConfirmationPromise({
          abortSignal,
          commitment,
          signature,
        }),
      signature,
      commitment,
    });
  }

  /**
   * Prepares a transaction by fetching the latest blockhash for its accounts
   * and setting the transaction’s blockhash lifetime accordingly.
   *
   * This ensures that the transaction is valid for the current Solana network state
   * before signing or sending it.
   *
   * @param transaction - The transaction message to prepare, which must include a fee payer.
   * @returns A new `TransactionMessage` with the blockhash lifetime set, ready for signing or sending.
   *
   * @example
   * ```ts
   * const preparedTx = await connection.prepareTransactionWithLatestBlockhash(txMessage);
   * const signature = await connection.sendTransaction(preparedTx, [signer]);
   * ```
   */
  public async prepareTransactionWithLatestBlockhash(
    transaction: TransactionMessage & TransactionMessageWithFeePayer,
  ) {
    const latestBlockhash =
      await this.getLatestBlockhashForTransaction(transaction);
    transaction = pipe(transaction, (tx) =>
      setTransactionMessageLifetimeUsingBlockhash(latestBlockhash, tx),
    );
    return transaction;
  }

  /**
   * Sends a transaction message to the network.
   *
   * This method:
   * 1. Fetches the latest blockhash for the writable accounts.
   * 2. Sets it as the transaction’s lifetime.
   * 3. Signs and serializes the transaction.
   * 4. Sends it to the cluster.
   *
   * @param transaction - The transaction message to send.
   * @param signers - Array of signers to partially sign the transaction.
   * @param options - (Optional) Transaction sending configuration.
   * @returns The signature of the sent transaction.
   */
  public async sendTransaction(
    transaction: TransactionMessage & TransactionMessageWithFeePayer,
    signers: CryptoKeyPair[],
    options?: {
      skipPreflight?: boolean;
      preflightCommitment?: Commitment;
    },
  ): Promise<Signature> {
    const { skipPreflight = true, preflightCommitment = "confirmed" } =
      options ?? {};
    const hasBlockhash =
      "lifetimeConstraint" in transaction &&
      typeof transaction.lifetimeConstraint === "object" &&
      transaction.lifetimeConstraint !== null &&
      "blockhash" in transaction.lifetimeConstraint &&
      (transaction.lifetimeConstraint as { blockhash?: unknown }).blockhash !==
        undefined;

    if (!hasBlockhash) {
      transaction =
        await this.prepareTransactionWithLatestBlockhash(transaction);
    }
    const isAlreadySigned: boolean =
      (transaction as any).signatures &&
      Object.keys((transaction as any).signatures).length > 0;

    const signedTransaction =
      isAlreadySigned &&
      "messageBytes" in transaction &&
      "signatures" in transaction
        ? (transaction as Readonly<{
            messageBytes: TransactionMessageBytes;
            signatures: SignaturesMap;
          }>)
        : await this.partiallySignTransaction(signers, transaction);
    const serializedTransaction =
      getBase64EncodedWireTransaction(signedTransaction);
    const signature = await this.rpc
      .sendTransaction(serializedTransaction, {
        encoding: "base64",
        skipPreflight,
        preflightCommitment,
        ...options,
      })
      .send();
    return signature;
  }

  /**
   * Partially signs a compiled transaction using the provided keypair signers.
   *
   * This method takes a `TransactionMessage` (which includes a fee payer and a valid blockhash lifetime),
   * compiles it, and applies partial signatures from one or more signers.
   *
   * The resulting transaction can be further signed by additional parties before being sent to the network.
   *
   * @param signers - An array of {@link CryptoKeyPair} objects used to partially sign the transaction.
   * @param transaction - The transaction message to sign, which must include a fee payer and a blockhash lifetime.
   *
   * @returns A read-only, partially signed transaction that includes:
   * - The compiled message bytes.
   * - A signatures map.
   * - The blockhash lifetime information.
   *
   * @example
   * ```ts
   * const tx = await connection.prepareTransactionWithLatestBlockhash(transaction);
   * const partiallySigned = await connection.partiallySignTransaction([signer1, signer2], tx);
   *
   * // Later, another party can add their signature:
   * const fullySigned = await connection.fullySignTransaction([signer3], partiallySigned);
   * ```
   */
  public async partiallySignTransaction(
    signers: CryptoKeyPair[],
    transaction: TransactionMessage & TransactionMessageWithFeePayer<string>,
  ): Promise<
    Readonly<
      TransactionWithBlockhashLifetime &
        Readonly<{
          messageBytes: TransactionMessageBytes;
          signatures: SignaturesMap;
        }>
    >
  > {
    assertIsTransactionMessageWithBlockhashLifetime(transaction);
    const compiledTransaction = compileTransaction(transaction);
    const signedTransaction = await partiallySignTransaction(
      signers,
      compiledTransaction,
    );
    return signedTransaction;
  }

  /**
   * Sends and confirms a transaction atomically.
   * Internally combines {@link sendTransaction} and {@link confirmTransaction}.
   *
   * @param transaction - The transaction message to send.
   * @param signers - Array of signers for the transaction.
   * @param config - Optional confirmation configuration.
   * @returns The confirmed transaction signature.
   */
  public async sendAndConfirmTransaction(
    transaction: TransactionMessage & TransactionMessageWithFeePayer,
    signers: CryptoKeyPair[],
    config: SendAndConfirmTransactionConfig,
  ): Promise<Signature> {
    const signature = await this.sendTransaction(transaction, signers, config);
    await this.confirmTransaction(signature, config);
    return signature;
  }

  /**
   * Fetches the latest blockhash for a transaction.
   * Uses a router endpoint if available; otherwise falls back to standard RPC.
   *
   * @param transaction - Transaction message to inspect for writable accounts.
   * @returns The latest blockhash and last valid block height.
   */
  public async getLatestBlockhashForTransaction(
    transaction: TransactionMessage,
  ): Promise<Readonly<LatestBlockhash>> {
    const writableAccounts = getWritableAccounts(transaction);

    if (this.isMagicRouter) {
      const blockHashResponse = await fetch(this.clusterUrlHttp, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          jsonrpc: "2.0",
          id: 1,
          method: "getBlockhashForAccounts",
          params: [writableAccounts],
        }),
      });

      const { result } = (await blockHashResponse.json()) as {
        result: { blockhash: string; lastValidBlockHeight: number };
      };

      if (!result?.blockhash || !result?.lastValidBlockHeight) {
        throw new Error(`Invalid RPC response: ${JSON.stringify(result)}`);
      }

      return {
        blockhash: result.blockhash as Blockhash,
        lastValidBlockHeight: BigInt(result.lastValidBlockHeight),
      };
    }

    const blockhashResponse = await this.rpc.getLatestBlockhash().send();

    if (
      !blockhashResponse?.value?.blockhash ||
      !blockhashResponse?.value?.lastValidBlockHeight
    ) {
      throw new Error(
        `Invalid blockhash response: ${JSON.stringify(blockhashResponse)}`,
      );
    }

    return blockhashResponse.value;
  }

  /**
   * Retrieves the final commitment signature for a given base layer transaction.
   *
   * ⚠️ **Important:** This method must be called using a `Connection` instance
   * configured for an **ephemeral rollup** (not a standard Solana RPC connection).
   * The ephemeral rollup connection is required to access the scheduling and commit logs.
   *
   * @param signature - The transaction signature of the accounts commitment transaction to the base layer.
   * @returns A `Promise<string>` that resolves with the commitment signature found in the commit logs.
   *
   * @throws {Error} If the transaction or its metadata cannot be found, or if the
   *                 expected log messages are missing from either scheduling or commit stages.
   */
  public async getCommitmentSignature(
    signature: Signature,
  ): Promise<Signature> {
    const txSchedulingSgn = await this.rpc
      .getTransaction(signature, {
        encoding: "base64",
        maxSupportedTransactionVersion: 0,
      })
      .send();

    if (txSchedulingSgn?.meta == null) {
      throw new Error("Transaction not found or meta is null");
    }
    const scheduledCommitSgn = parseScheduleCommitsLogsMessage(
      txSchedulingSgn.meta.logMessages ?? [],
    );
    if (scheduledCommitSgn == null) {
      throw new Error("ScheduledCommitSent signature not found");
    }
    await this.confirmTransaction(scheduledCommitSgn);

    const txCommitInfo = await this.rpc
      .getTransaction(scheduledCommitSgn, {
        encoding: "base64",
        maxSupportedTransactionVersion: 0,
      })
      .send();

    if (txCommitInfo?.meta == null) {
      throw new Error("Transaction not found or meta is null");
    }

    const commitSignature = parseCommitsLogsMessage(
      txCommitInfo.meta.logMessages ?? [],
    );
    if (commitSignature == null) {
      throw new Error("Unable to find Commitment signature");
    }
    return commitSignature;
  }
}

export interface SendAndConfirmTransactionConfig {
  /** Commitment level for transaction confirmation */
  commitment?: Commitment;
  /** Signal to abort waiting for confirmation */
  abortSignal?: AbortSignal;
  /** Whether to skip preflight checks */
  skipPreflight?: boolean;
  /** Commitment level for preflight checks */
  preflightCommitment?: Commitment;
  /** Optional maximum retries */
  maxRetries?: number;
  /** Optional timeout in milliseconds */
  timeout?: number;
}
