import { describe, expect, it, vi } from "vitest";
import {
  AccountInfoBase,
  AccountInfoWithBase64EncodedData,
  Address,
  getAddressEncoder,
  lamports,
  Rpc,
  SolanaRpcApiDevnet,
} from "@solana/kit";
import { SYSTEM_PROGRAM_ADDRESS } from "@solana-program/system";

import { DELEGATION_PROGRAM_ID } from "../constants.js";
import { delegationRecordPdaFromDelegatedAccount } from "../pda.js";
import {
  DelegationStatus,
  getDelegationRecord,
  parseDelegationRecordAccount,
} from "../resolver.js";

type DelegationAccountInfo = AccountInfoBase & AccountInfoWithBase64EncodedData;

const delegatedAccount =
  "11111111111111111111111111111111" as Address;
const validator = "11111111111111111111111111111112" as Address;

function createDelegationAccountInfo(
  validatorAddress: Address,
  overrides: Partial<DelegationAccountInfo> = {},
): DelegationAccountInfo {
  const data = Buffer.alloc(40);
  data.set(getAddressEncoder().encode(validatorAddress), 8);

  return {
    data: [
      data.toString("base64"),
      "base64",
    ] as AccountInfoWithBase64EncodedData["data"],
    executable: false,
    lamports: lamports(BigInt(1)),
    owner: DELEGATION_PROGRAM_ID,
    space: BigInt(data.length),
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
    expect(
      parseDelegationRecordAccount(
        createDelegationAccountInfo(validator, {
          owner: SYSTEM_PROGRAM_ADDRESS,
        }),
      ),
    ).toEqual({
      status: DelegationStatus.Undelegated,
    });
  });

  it("returns undelegated when the delegation record has zero lamports", () => {
    expect(
      parseDelegationRecordAccount(
        createDelegationAccountInfo(validator, {
          lamports: lamports(BigInt(0)),
        }),
      ),
    ).toEqual({
      status: DelegationStatus.Undelegated,
    });
  });

  it("returns the delegated validator from the delegation record", () => {
    const record = parseDelegationRecordAccount(
      createDelegationAccountInfo(validator),
    );

    expect(record.status).toBe(DelegationStatus.Delegated);
    if (record.status === DelegationStatus.Delegated) {
      expect(record.validator).toBe(validator);
    }
  });
});

describe("getDelegationRecord", () => {
  it("fetches and parses the derived delegation record", async () => {
    const delegationRecord =
      await delegationRecordPdaFromDelegatedAccount(delegatedAccount);
    const getAccountInfo = vi.fn((address, config) => {
      expect(address).toBe(delegationRecord);
      expect(config).toEqual({
        commitment: "processed",
        encoding: "base64",
      });
      return {
        send: vi.fn(async () => ({
          value: createDelegationAccountInfo(validator),
        })),
      };
    });

    const record = await getDelegationRecord(
      { getAccountInfo } as unknown as Rpc<SolanaRpcApiDevnet>,
      delegatedAccount,
      "processed",
    );

    expect(record.status).toBe(DelegationStatus.Delegated);
    if (record.status === DelegationStatus.Delegated) {
      expect(record.validator).toBe(validator);
    }
  });
});
