import { Commitment, Signature } from "@solana/kit";
import {
  createRecentSignatureConfirmationPromiseFactory,
  getTimeoutPromise,
} from "@solana/transaction-confirmation";

/**
 * Waits for a recent transaction to be confirmed within a time-based blockhash lifetime.
 * Internally uses `raceStrategies` to race between transaction confirmation and timeout.
 *
 * @param config - Configuration for transaction confirmation with timeout.
 */
export async function waitForRecentTransactionConfirmationUntilTimeout(
  config: WaitForRecentTransactionWithTimeBasedLifetimeConfirmationConfig,
): Promise<void> {
  await raceStrategies(
    config.signature,
    config,
    function getSpecificStrategiesForRace({
      abortSignal,
      commitment,
      getTimeoutPromise,
    }) {
      return [
        getTimeoutPromise({
          abortSignal,
          commitment,
        }),
      ];
    },
  );
}

/**
 * Configuration for waiting for a transaction confirmation using
 * a time-based blockhash lifetime.
 */
export interface WaitForRecentTransactionWithTimeBasedLifetimeConfirmationConfig
  extends BaseTransactionConfirmationStrategyConfig {
  /** Factory function for creating a timeout promise for transaction confirmation */
  getTimeoutPromise: typeof getTimeoutPromise;

  /**
   * The transaction signature to confirm.
   * A 64-byte Ed25519 signature, encoded in base-58.
   */
  signature: Signature;
}

/**
 * Base configuration for transaction confirmation strategies.
 */
export interface BaseTransactionConfirmationStrategyConfig {
  /** Optional AbortSignal to cancel confirmation */
  abortSignal?: AbortSignal;

  /** Desired commitment level for confirmation (e.g., "confirmed", "finalized") */
  commitment: Commitment;

  /** Factory function for generating promises that confirm recent transactions */
  getRecentSignatureConfirmationPromise: ReturnType<
    typeof createRecentSignatureConfirmationPromiseFactory
  >;
}

/**
 * Runs multiple transaction confirmation strategies in parallel (racing),
 * ensuring all losing promises are properly settled to prevent memory leaks.
 *
 * @param signature - The transaction signature to confirm.
 * @param config - Base strategy configuration.
 * @param getSpecificStrategiesForRace - A function returning additional promises
 *                                      to race against the main confirmation promise.
 * @returns Resolves when the first promise settles successfully, or rejects if all fail.
 */
export async function raceStrategies<
  TConfig extends BaseTransactionConfirmationStrategyConfig,
>(
  signature: Signature,
  config: TConfig,
  getSpecificStrategiesForRace: (
    config: WithNonNullableAbortSignal<TConfig>,
  ) => ReadonlyArray<Promise<unknown>>,
) {
  const {
    abortSignal: callerAbortSignal,
    commitment,
    getRecentSignatureConfirmationPromise,
  } = config;

  // Immediately abort if caller signal is already aborted
  callerAbortSignal?.throwIfAborted();

  const abortController = new AbortController();

  if (callerAbortSignal) {
    const handleAbort = () => {
      abortController.abort();
    };
    callerAbortSignal.addEventListener("abort", handleAbort, {
      signal: abortController.signal,
    });
  }

  try {
    const specificStrategies = getSpecificStrategiesForRace({
      ...config,
      abortSignal: abortController.signal,
    });

    return await safeRace([
      getRecentSignatureConfirmationPromise({
        abortSignal: abortController.signal,
        commitment,
        signature,
      }),
      ...specificStrategies,
    ]);
  } finally {
    abortController.abort();
  }
}

/** Type helper to ensure `abortSignal` is always non-nullable */
type WithNonNullableAbortSignal<T> = Omit<T, "abortSignal"> &
  Readonly<{ abortSignal: AbortSignal }>;

/** WeakMap records for tracking promises in `safeRace` to avoid memory leaks */
const wm = new WeakMap<
  object,
  { deferreds: Set<Deferred>; settled: boolean }
>();

/**
 * A safe implementation of Promise.race that ensures losing promises are settled.
 * This prevents memory leaks by releasing references to unneeded promises.
 *
 * @param contenders - Array of promises or primitive values to race.
 * @returns Resolves or rejects with the first settled promise value.
 */
export async function safeRace<T extends readonly unknown[] | []>(
  contenders: T,
): Promise<Awaited<T[number]>> {
  let deferred: Deferred;

  const result = new Promise((resolve, reject) => {
    deferred = { resolve, reject };

    for (const contender of contenders) {
      if (!isObject(contender)) {
        // Primitive values are resolved immediately
        Promise.resolve(contender).then(resolve, reject);
        continue;
      }

      let record = wm.get(contender);
      if (!record) {
        record = addRaceContender(contender);
        record.deferreds.add(deferred);
        wm.set(contender, record);
      } else if (record.settled) {
        Promise.resolve(contender).then(resolve, reject);
      } else {
        record.deferreds.add(deferred);
      }
    }
  });

  // The finally callback executes when any value settles, preventing any of
  // the unresolved values from retaining a reference to the resolved value.
  return result.finally(() => {
    // Remove references after settlement to prevent memory leaks
    for (const contender of contenders) {
      if (isObject(contender)) {
        const record = wm.get(contender);
        if (record) {
          record.deferreds.delete(deferred);
        }
      }
    }
  }) as Promise<Awaited<T[number]>>;
}

/** Deferred promise object type used in safeRace */
type Deferred = Readonly<{
  resolve: (value: unknown) => void;
  reject: (reason?: unknown) => void;
}>;

/** Helper to check if value is an object (including functions) */
function isObject(value: unknown): value is object {
  return (
    value !== null && (typeof value === "object" || typeof value === "function")
  );
}

/**
 * Registers a contender promise in the weak map for `safeRace`.
 * Ensures its resolved or rejected value is propagated to all dependent deferreds.
 *
 * @param contender - The promise to track.
 */
function addRaceContender(contender: object) {
  const deferreds = new Set<Deferred>();
  const record = { deferreds, settled: false };

  Promise.resolve(contender).then(
    (value) => {
      for (const { resolve } of deferreds) resolve(value);
      deferreds.clear();
      record.settled = true;
    },
    (err) => {
      for (const { reject } of deferreds) reject(err);
      deferreds.clear();
      record.settled = true;
    },
  );
  return record;
}
