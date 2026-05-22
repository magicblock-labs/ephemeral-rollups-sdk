import { describe, expect, it, vi } from "vitest";
import {
  AccountInfo,
  Connection,
  Keypair,
  PublicKey,
  SystemProgram,
} from "@solana/web3.js";

import { DELEGATION_PROGRAM_ID } from "../constants.js";
import { delegationRecordPdaFromDelegatedAccount } from "../pda.js";
import {
  DelegationStatus,
  getDelegationRecord,
  parseDelegationRecordAccount,
} from "../resolver.js";

function createDelegationAccountInfo(
  validator: PublicKey,
  overrides: Partial<AccountInfo<Buffer>> = {},
): AccountInfo<Buffer> {
  const data = Buffer.alloc(40);
  validator.toBuffer().copy(data, 8);

  return {
    data,
    executable: false,
    lamports: 1,
    owner: DELEGATION_PROGRAM_ID,
    rentEpoch: 0,
    ...overrides,
  };
}

describe("parseDelegationRecordAccount", () => {
  it("returns undelegated when the delegation record is missing", () => {
    expect(parseDelegationRecordAccount(null)).toEqual({
      status: DelegationStatus.Undelegated,
    });
  });

  it("returns undelegated when the account has the wrong owner", () => {
    const validator = Keypair.generate().publicKey;

    expect(
      parseDelegationRecordAccount(
        createDelegationAccountInfo(validator, {
          owner: SystemProgram.programId,
        }),
      ),
    ).toEqual({
      status: DelegationStatus.Undelegated,
    });
  });

  it("returns undelegated when the delegation record has zero lamports", () => {
    const validator = Keypair.generate().publicKey;

    expect(
      parseDelegationRecordAccount(
        createDelegationAccountInfo(validator, {
          lamports: 0,
        }),
      ),
    ).toEqual({
      status: DelegationStatus.Undelegated,
    });
  });

  it("returns the delegated validator from the delegation record", () => {
    const validator = Keypair.generate().publicKey;
    const record = parseDelegationRecordAccount(
      createDelegationAccountInfo(validator),
    );

    expect(record.status).toBe(DelegationStatus.Delegated);
    if (record.status === DelegationStatus.Delegated) {
      expect(record.validator.toBase58()).toBe(validator.toBase58());
    }
  });
});

describe("getDelegationRecord", () => {
  it("fetches and parses the derived delegation record", async () => {
    const delegatedAccount = Keypair.generate().publicKey;
    const validator = Keypair.generate().publicKey;
    const delegationRecord =
      delegationRecordPdaFromDelegatedAccount(delegatedAccount);
    const getAccountInfo = vi.fn(async (address, commitment) => {
      expect(address.toBase58()).toBe(delegationRecord.toBase58());
      expect(commitment).toBe("processed");
      return createDelegationAccountInfo(validator);
    });

    const record = await getDelegationRecord(
      { getAccountInfo } as unknown as Connection,
      delegatedAccount,
      "processed",
    );

    expect(record.status).toBe(DelegationStatus.Delegated);
    if (record.status === DelegationStatus.Delegated) {
      expect(record.validator.toBase58()).toBe(validator.toBase58());
    }
  });
});
