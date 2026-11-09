import { PublicKey } from "@solana/web3.js";
import { describe, expect, it } from "vitest";
import {
  hasAuthorizedCompletion,
  parseCommitsLogsMessage,
  parseScheduleCommitsLogsMessage,
  receiptFromLogs,
} from "../utils";

describe("settlement receipt log parsing", () => {
  it("keeps the legacy signature receipt parsers working", () => {
    expect(
      parseScheduleCommitsLogsMessage([
        "Program log: ScheduledCommitSent signature: old",
      ]),
    ).toBe("old");
    expect(
      parseCommitsLogsMessage([
        "Program log: ScheduledCommitSent signature[0]: base",
      ]),
    ).toBe("base");
  });

  it("extracts all base-chain signatures and result details from an address receipt", () => {
    const receipt = receiptFromLogs("outbox", [
      "Program log: ScheduledCommitSent signature[0]: base-1",
      "Program log: ScheduledCommitSent signature[1]: base-2",
      "Program log: ScheduledCommitSent error message: action failed",
    ]);
    expect(receipt.intentAddress).toBe("outbox");
    expect(receipt.signatures).toEqual(["base-1", "base-2"]);
    expect(receipt.result.errorMessage).toBe("action failed");
  });

  it("requires the validator signer and the matching outbox program instruction", () => {
    const validator = new PublicKey("11111111111111111111111111111111");
    const outbox = new PublicKey("Vote111111111111111111111111111111111111111");
    const program = new PublicKey(
      "outboxintent1111111111111111111111111111111",
    );
    const tx = {
      transaction: {
        message: {
          staticAccountKeys: [validator, outbox, program],
          header: { numRequiredSignatures: 1 },
          compiledInstructions: [{ programIdIndex: 2, accountKeyIndexes: [1] }],
        },
      },
    };
    expect(hasAuthorizedCompletion(tx, validator, outbox)).toBe(true);
    expect(
      hasAuthorizedCompletion(
        tx,
        new PublicKey("SysvarRent111111111111111111111111111111111"),
        outbox,
      ),
    ).toBe(false);
  });
});
