import bs58 from "bs58";
import { PublicKey } from "@solana/web3.js";
import { describe, expect, it } from "vitest";
import {
  parseCommitsLogsMessage,
  parseScheduleCommitsLogsMessage,
  receiptFromCompletion,
  receiptFromLogs,
} from "../utils";

const validator = new PublicKey("11111111111111111111111111111111");
const outbox = new PublicKey("Vote111111111111111111111111111111111111111");
const program = new PublicKey("outboxintent1111111111111111111111111111111");

function makeCompletion(id: number, instructionId = id, err: unknown = null) {
  const instructionData = new Uint8Array(12);
  instructionData[0] = 2; // OutboxIntentInstruction::ScheduledCommitSent
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
        `Program ${program.toBase58()} invoke [1]`,
        `Program log: ScheduledCommitSent id: ${id}, slot: 10, blockhash: hash`,
        "Program log: ScheduledCommitSent signature[0]: real-signature",
        "Program log: ScheduledCommitSent error message: action failed",
        `Program ${program.toBase58()} success`,
        "Program attacker111 invoke [1]",
        "Program log: ScheduledCommitSent signature[0]: forged-signature",
        "Program attacker111 success",
      ],
    },
  };
}

describe("settlement receipt parsing", () => {
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

  it("extracts result details from a new-format receipt", () => {
    const receipt = receiptFromLogs("outbox", [
      "Program log: ScheduledCommitSent signature[0]: base-1",
      "Program log: ScheduledCommitSent signature[1]: base-2",
      "Program log: ScheduledCommitSent error message: action failed",
    ]);
    expect(receipt.intentAddress).toBe("outbox");
    expect(receipt.signatures).toEqual(["base-1", "base-2"]);
    expect(receipt.result.errorMessage).toBe("action failed");
  });

  it("accepts only matching outbox completion logs", () => {
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
});
