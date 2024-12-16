import { PublicKey, Connection, AccountInfo, Transaction } from "@solana/web3.js";
import { DELEGATION_PROGRAM_ID } from "./constants";
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
enum DelegationStatus {
    Delegated,
    Undelegated
}

/** Type representing a delegation record with status and optional validator information */
type DelegationRecord =
    | { status: DelegationStatus.Delegated; validator: PublicKey }
    | { status: DelegationStatus.Undelegated };


/** Class responsible for resolving connections to Solana validators */
export class Resolver {
    private routes: Map<string, Connection> = new Map();
    private delegations: Map<string, DelegationRecord> = new Map();
    private chain: Connection;
    private ws: Connection;
    private subs: Set<number> = new Set();

    constructor(config: Configuration, routes: Map<string, string>) {
        this.chain = new Connection(config.chain);
        this.ws = new Connection(config.websocket);
        this.routes = new Map([...routes.entries()].map(([k, v]) => [k, new Connection(v)]));
    }

    /**
     * Tracks the delegation status of a Solana account.
     * @param pubkey - The public key of the account to track.
     * @returns The current delegation record of the account.
     */
    public async trackAccount(pubkey: PublicKey): Promise<DelegationRecord> {
        const pubkeyStr = pubkey.toString();
        if (this.delegations.has(pubkeyStr)) {
            return this.delegations.get(pubkeyStr)!;
        }
        const seed = new TextEncoder().encode("delegation");
        const seeds = [seed, pubkey.toBytes()];

        const [delegationRecord] = PublicKey.findProgramAddressSync(seeds, DELEGATION_PROGRAM_ID);

        const id = this.ws.onAccountChange(
            delegationRecord,
            (acc) => this.updateStatus(acc, pubkey),
			{ commitment: "confirmed" }
        );
        this.subs.add(id);

        const accountInfo = await this.chain.getAccountInfo(delegationRecord, "confirmed");
        return this.updateStatus(accountInfo, pubkey);
    }

    /**
     * Resolves the appropriate connection for a given public key.
     * @param pubkey - The public key for which the connection is requested.
     * @returns The connection object or undefined if the connection is unresolvable.
     */
    public async resolve(pubkey: PublicKey): Promise<Connection | undefined> {
        let record = this.delegations.get(pubkey.toString());
        if (!record) {
            record = await this.trackAccount(pubkey);
        }
        return record.status === DelegationStatus.Delegated
            ? this.routes.get(record.validator.toString())
            : this.chain;
    }

    /**
     * Resolves the appropriate connection for a given transaction.
     * @param tx - The transaction requiring connection resolution.
     * @returns The connection object or undefined if the transaction references multiple delegated validators.
     */
    public async resolveForTransaction(tx: Transaction): Promise<Connection | undefined> {
        const validators = new Set<string>();
        for (const { pubkey, isWritable } of tx.instructions.flatMap((i) => i.keys)) {
            if (!isWritable) continue;
			const record = await this.trackAccount(pubkey);
			if (record.status == DelegationStatus.Delegated) {
				validators.add(record.validator.toString());
			}
        }
		const vs = [...validators];
        return vs.length === 1 ? this.routes.get(vs[0]) : validators.size === 0 ? this.chain : undefined;
    }

    /**
     * Terminates all active WebSocket subscriptions.
     * Should be called to clean up resources when the resolver is no longer needed.
     */
    public async terminate() {
        await Promise.all([...this.subs].map((sub) => this.ws.removeAccountChangeListener(sub)));
    }

    private updateStatus(account: AccountInfo<Buffer> | null, pubkey: PublicKey): DelegationRecord {
        const isDelegated = account?.owner.equals(DELEGATION_PROGRAM_ID) && account.lamports !== 0;
        const record: DelegationRecord = isDelegated
            ? { status: DelegationStatus.Delegated, validator: new PublicKey(account!.data.subarray(8, 40)) }
            : { status: DelegationStatus.Undelegated };
        this.delegations.set(pubkey.toString(), record);
        return record;
    }
}

