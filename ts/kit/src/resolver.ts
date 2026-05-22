import {
  Address,
  AccountInfoBase,
  Commitment,
  getAddressDecoder,
  lamports,
  getAddressEncoder,
  getProgramDerivedAddress,
  createSolanaRpcSubscriptions,
  TransactionMessage,
  RpcSubscriptions,
  SolanaRpcSubscriptionsApi,
  createSolanaRpc,
  Rpc,
  SolanaRpcApiDevnet,
  AccountInfoWithBase64EncodedData,
  AccountRole,
} from "@solana/kit";
import { DELEGATION_PROGRAM_ID } from "./constants.js";
import { delegationRecordPdaFromDelegatedAccount } from "./pda.js";

/**
 * Interface representing the configuration for the connection resolver.
 */
export interface Configuration {
  /** HTTP endpoint URL for the base layer chain */
  chain: string;
  /** WebSocket endpoint URL for the base layer chain */
  websocket: string;
}

/** Enumeration of possible delegation statuses */
export enum DelegationStatus {
  Delegated,
  Undelegated,
}

/** Type representing a delegation record with status and optional validator information */
export type DelegationRecord =
  | { status: DelegationStatus.Delegated; validator: Address }
  | { status: DelegationStatus.Undelegated };

export function parseDelegationRecordAccount(
  account: (AccountInfoBase & AccountInfoWithBase64EncodedData) | null,
): DelegationRecord {
  const isDelegated =
    account !== null &&
    account.owner === DELEGATION_PROGRAM_ID &&
    account.lamports !== lamports(BigInt(0));

  return isDelegated
    ? {
        status: DelegationStatus.Delegated,
        validator: getAddressDecoder().decode(
          Buffer.from(account.data[0], "base64").subarray(8, 40),
        ),
      }
    : { status: DelegationStatus.Undelegated };
}

export async function getDelegationRecord(
  rpc: Rpc<SolanaRpcApiDevnet>,
  delegatedAccount: Address,
  commitment: Commitment = "confirmed",
): Promise<DelegationRecord> {
  const accountInfo = await rpc
    .getAccountInfo(
      await delegationRecordPdaFromDelegatedAccount(delegatedAccount),
      {
        commitment,
        encoding: "base64",
      },
    )
    .send();
  return parseDelegationRecordAccount(accountInfo.value);
}

/** Class responsible for resolving connections to Solana validators */
export class Resolver {
  private readonly routes = new Map<string, Rpc<SolanaRpcApiDevnet>>();
  private readonly delegations = new Map<string, DelegationRecord>();
  private readonly chain: Rpc<SolanaRpcApiDevnet>;
  private readonly ws: RpcSubscriptions<SolanaRpcSubscriptionsApi>;

  constructor(config: Configuration, routes: Map<string, string>) {
    this.chain = createSolanaRpc(config.chain);
    this.ws = createSolanaRpcSubscriptions(config.websocket);
    this.routes = new Map(
      [...routes.entries()].map(([k, v]) => [k, createSolanaRpc(v)]),
    );
  }

  /**
   * Tracks the delegation status of a Solana account.
   * @param pubkey - The public key of the account to track.
   * @returns The current delegation record of the account.
   */
  public async trackAccount(pubkey: Address): Promise<DelegationRecord> {
    const pubkeyStr = pubkey.toString();
    if (this.delegations.has(pubkeyStr)) {
      const record = this.delegations.get(pubkeyStr);
      if (record !== undefined) {
        return record;
      }
      throw new Error(
        `Expected a delegation record for ${pubkeyStr}, but found undefined.`,
      );
    }

    const addressEncoder = getAddressEncoder();
    const [delegationRecord] = await getProgramDerivedAddress({
      programAddress: DELEGATION_PROGRAM_ID,
      seeds: [Buffer.from("delegation"), addressEncoder.encode(pubkey)],
    });

    const abortController = new AbortController();
    const accountNotifications = await this.ws
      .accountNotifications(delegationRecord, {
        commitment: "confirmed",
        encoding: "base64",
      })
      .subscribe({ abortSignal: abortController.signal });

    for await (const accountNotification of accountNotifications) {
      this.updateStatus(accountNotification.value, pubkey);
      abortController.abort();
    }

    const accountInfo = await this.chain
      .getAccountInfo(delegationRecord, {
        commitment: "confirmed",
        encoding: "base64",
      })
      .send();

    return this.updateStatus(accountInfo.value, pubkey);
  }

  /**
   * Resolves the appropriate connection for a given public key.
   * @param pubkey - The public key for which the connection is requested.
   * @returns The connection object or undefined if the connection is unresolvable.
   */
  public async resolve(
    pubkey: Address,
  ): Promise<Rpc<SolanaRpcApiDevnet> | undefined> {
    let record = this.delegations.get(pubkey);
    if (!record) {
      record = await this.trackAccount(pubkey);
    }
    return record.status === DelegationStatus.Delegated
      ? this.routes.get(record.validator)
      : this.chain;
  }

  /**
   * Resolves the appropriate connection for a given transaction.
   * @param tx - The transaction requiring connection resolution.
   * @returns The connection object or undefined if the transaction references multiple delegated validators.
   */
  public async resolveForTransaction(
    tx: TransactionMessage,
  ): Promise<Rpc<SolanaRpcApiDevnet> | undefined> {
    const validators = new Set<string>();
    for (const account of tx.instructions.flatMap((i) => i.accounts)) {
      if (!account) continue;
      const { address, role } = account;
      if (role === AccountRole.READONLY || role === AccountRole.READONLY_SIGNER)
        continue;
      const record = await this.trackAccount(address);
      if (record.status === DelegationStatus.Delegated) {
        validators.add(record.validator.toString());
      }
    }
    const vs = [...validators];
    return vs.length === 1
      ? this.routes.get(vs[0])
      : validators.size === 0
        ? this.chain
        : undefined;
  }

  private updateStatus(
    account: (AccountInfoBase & AccountInfoWithBase64EncodedData) | null,
    pubkey: Address,
  ): DelegationRecord {
    const record = parseDelegationRecordAccount(account);
    this.delegations.set(pubkey.toString(), record);
    return record;
  }
}
