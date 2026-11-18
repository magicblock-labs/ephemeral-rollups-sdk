import { describe, expect, it } from "vitest";
import { parseScheduleCommitsLogsMessage } from "../utils";
import type { Signature } from "@solana/kit";

describe("parseScheduleCommitsLogsMessage", () => {
  it.each([
    "ScheduledCommitSent signature[0]: indexed-signature",
    "ScheduledCommitSent signature: unindexed-signature",
  ])("parses the scheduling signature from %s", (message) => {
    const result = parseScheduleCommitsLogsMessage([`Program log: ${message}`]);

    expect(result).toBe(message.slice(message.indexOf(": ") + 2) as Signature);
  });

  it("returns null when neither scheduling label is present", () => {
    expect(
      parseScheduleCommitsLogsMessage(["Program log: unrelated"]),
    ).toBeNull();
  });
});
