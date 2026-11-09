import { describe, expect, it } from "vitest";
import {
  hasAuthorizedCompletion,
  parseOutboxAddresses,
  receiptFromLogs,
} from "../utils";

describe("settlement receipt log parsing", () => {
  it("extracts outbox addresses and completion details", () => {
    expect(
      parseOutboxAddresses([
        "Program log: ScheduledCommitSent outbox: address-1",
      ]),
    ).toEqual(["address-1"]);
    const receipt = receiptFromLogs("address-1", [
      "Program log: ScheduledCommitSent signature[0]: base-1",
      "Program log: ScheduledCommitSent error message: action failed",
    ]);
    expect(receipt.signatures).toEqual(["base-1"]);
    expect(receipt.result.errorMessage).toBe("action failed");
  });

  it("requires the validator signer and matching outbox program instruction", () => {
    const tx = {
      transaction: {
        message: {
          accountKeys: [
            "validator",
            "outbox-1",
            "outboxintent1111111111111111111111111111111",
          ],
          header: { numRequiredSignatures: 1 },
          instructions: [{ programIdIndex: 2, accounts: [1] }],
        },
      },
    };
    expect(hasAuthorizedCompletion(tx, "validator", "outbox-1")).toBe(true);
    expect(hasAuthorizedCompletion(tx, "other-validator", "outbox-1")).toBe(
      false,
    );
  });
});
