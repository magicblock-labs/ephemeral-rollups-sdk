import bs58 from "bs58";
import { describe, expect, it } from "vitest";
import {
  logsFromProgram,
  parseOutboxAddresses,
  receiptFromCompletion,
  receiptFromLogs,
} from "../utils";

const validator = "11111111111111111111111111111111";
const outbox = "Vote111111111111111111111111111111111111111";
const program = "outboxintent1111111111111111111111111111111";

function makeCompletion(id: number, instructionId = id, err: unknown = null) {
  const instructionData = new Uint8Array(12);
  instructionData[0] = 2;
  instructionData[4] = instructionId;
  return {
    transaction: {
      message: {
        instructions: [
          {
            programId: program,
            accounts: [validator, outbox],
            data: bs58.encode(instructionData),
          },
        ],
      },
    },
    meta: {
      err,
      logMessages: [
        `Program ${program} invoke [1]`,
        `Program log: ScheduledCommitSent id: ${id}, slot: 10, blockhash: hash`,
        "Program log: ScheduledCommitSent signature[0]: real-signature",
        "Program log: ScheduledCommitSent error message: action failed",
        `Program ${program} success`,
        "Program attacker111 invoke [1]",
        "Program log: ScheduledCommitSent signature[0]: forged-signature",
        "Program attacker111 success",
      ],
    },
  };
}

describe("settlement receipt parsing", () => {
  it("extracts outbox addresses from scheduling logs", () => {
    expect(
      parseOutboxAddresses([
        "Program log: ScheduledCommitSent outbox: address-1",
      ]),
    ).toEqual(["address-1"]);
  });

  it("keeps only logs emitted by the named program", () => {
    expect(
      logsFromProgram(
        [
          "Program magic111 invoke [1]",
          "Program log: real",
          "Program attacker111 invoke [2]",
          "Program log: spoofed",
          "Program attacker111 success",
          "Program magic111 success",
        ],
        "magic111",
      ),
    ).toEqual(["Program log: real"]);
  });

  it("extracts completion results without accepting logs from other programs", () => {
    const receipt = receiptFromCompletion(makeCompletion(42), outbox);
    expect(receipt?.signatures).toEqual(["real-signature"]);
    expect(receipt?.result.errorMessage).toBe("action failed");
  });

  it("rejects spoofed logs, mismatched intent IDs, and failed transactions", () => {
    const tx = makeCompletion(42);
    tx.meta.logMessages = [
      "Program attacker111 invoke [1]",
      "Program log: ScheduledCommitSent id: 42, slot: 10, blockhash: hash",
      "Program log: ScheduledCommitSent signature[0]: forged-signature",
      "Program attacker111 success",
    ];
    expect(receiptFromCompletion(tx, outbox)).toBeNull();
    expect(receiptFromCompletion(makeCompletion(42, 43), outbox)).toBeNull();
    expect(
      receiptFromCompletion(
        makeCompletion(42, 42, { InstructionError: [] }),
        outbox,
      ),
    ).toBeNull();
  });

  it("parses a receipt directly from logs", () => {
    const receipt = receiptFromLogs("address-1", [
      "Program log: ScheduledCommitSent signature[0]: base-1",
      "Program log: ScheduledCommitSent error message: action failed",
    ]);
    expect(receipt.signatures).toEqual(["base-1"]);
    expect(receipt.result.errorMessage).toBe("action failed");
  });
});
