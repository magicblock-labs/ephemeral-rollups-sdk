import { describe, expect, it } from "vitest";
import { parseOutboxAddresses, receiptFromLogs } from "../utils";

describe("settlement receipt log parsing", () => {
  it("extracts outbox addresses and completion details", () => {
    expect(parseOutboxAddresses(["Program log: ScheduledCommitSent outbox: address-1"])).toEqual(["address-1"]);
    const receipt = receiptFromLogs("address-1", [
      "Program log: ScheduledCommitSent signature[0]: base-1",
      "Program log: ScheduledCommitSent error message: action failed",
    ]);
    expect(receipt.signatures).toEqual(["base-1"]);
    expect(receipt.result.errorMessage).toBe("action failed");
  });
});
