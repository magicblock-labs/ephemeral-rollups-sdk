import bs58 from "bs58";
import { Connection, PublicKey } from "@solana/web3.js";
import { describe, expect, it, vi } from "vitest";
import { MAGIC_PROGRAM_ID } from "../constants";
import {
  GetCommitmentSignature,
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
        `Program log: Settlement receipt intent id: ${id}, slot: 10, blockhash: hash`,
        "Program log: Settlement receipt signature[0]: real-signature",
        "Program log: Settlement receipt error message: action failed",
        `Program ${program.toBase58()} success`,
        "Program attacker111 invoke [1]",
        "Program log: Settlement receipt signature[0]: forged-signature",
        "Program attacker111 success",
      ],
    },
  };
}

describe("settlement receipt parsing", () => {
  it("retries a missing scheduling transaction and parses an unprefixed Magic program log", async () => {
    const magicProgram = MAGIC_PROGRAM_ID.toBase58();
    const outboxAddress = new PublicKey(
      "3m1fa5yaCJv9AhE227MvxACew5sWy1uMdZWNZFDUzuww",
    );
    const transactionData = new Uint8Array(12);
    transactionData[0] = 2;
    const connection = new Connection("http://localhost");
    const getSignatures = vi
      .spyOn(connection, "getSignaturesForAddress")
      .mockResolvedValue([{ signature: "completion" }] as unknown as Awaited<
        ReturnType<typeof connection.getSignaturesForAddress>
      >);

    const schedulingTransaction = {
      meta: {
        err: null,
        logMessages: [
          "Program parent111 invoke [1]",
          `Program ${magicProgram} invoke [2]`,
          "ScheduleCommit: parent program id: parent111",
          "Scheduled intent id: 0",
          `Outbox for scheduled intent: ${outboxAddress.toBase58()}`,
          "Program attacker111 invoke [3]",
          "Outbox for scheduled intent: spoofed-address",
          "Program attacker111 success",
          `Program ${magicProgram} success`,
          "Program parent111 success",
        ],
      },
    } as unknown as Awaited<ReturnType<typeof connection.getTransaction>>;
    const getTransaction = vi
      .spyOn(connection, "getTransaction")
      .mockResolvedValueOnce(null)
      .mockResolvedValue(schedulingTransaction);
    vi.spyOn(connection, "getParsedTransaction").mockResolvedValue({
      transaction: {
        message: {
          instructions: [
            {
              programId: program,
              accounts: [validator, outboxAddress],
              data: bs58.encode(transactionData),
            },
          ],
        },
      },
      meta: {
        err: null,
        logMessages: [
          `Program ${program.toBase58()} invoke [1]`,
          "Program log: Settlement receipt intent id: 0, slot: 10, blockhash: hash",
          "Program log: Settlement receipt signature[0]: base-signature",
          `Program ${program.toBase58()} success`,
        ],
      },
    } as unknown as Awaited<
      ReturnType<typeof connection.getParsedTransaction>
    >);

    await expect(
      GetCommitmentSignature("scheduling", connection, 1_500),
    ).resolves.toBe("base-signature");
    expect(getTransaction).toHaveBeenCalledTimes(2);
    expect(getSignatures.mock.calls[0][0].toBase58()).toBe(
      outboxAddress.toBase58(),
    );
  });

  it("does not accept legacy signature-only scheduling logs", async () => {
    const magicProgram = MAGIC_PROGRAM_ID.toBase58();
    const connection = new Connection("http://localhost");
    const schedulingTransaction = {
      meta: {
        err: null,
        logMessages: [
          `Program ${magicProgram} invoke [1]`,
          "Program log: ScheduledCommitSent signature: scheduled-commit",
          `Program ${magicProgram} success`,
        ],
      },
    } as unknown as Awaited<ReturnType<typeof connection.getTransaction>>;
    const getTransaction = vi
      .spyOn(connection, "getTransaction")
      .mockResolvedValueOnce(schedulingTransaction);

    await expect(
      GetCommitmentSignature("scheduling", connection, 1_500),
    ).rejects.toThrow("Settlement outbox address not found in scheduling logs");
    expect(getTransaction).toHaveBeenCalledTimes(1);
  });

  it("times out when the transaction lookup keeps returning null", async () => {
    const connection = new Connection("http://localhost");
    const getTransaction = vi
      .spyOn(connection, "getTransaction")
      .mockResolvedValue(null);

    await expect(
      GetCommitmentSignature("scheduling", connection, 10),
    ).rejects.toThrow("Transaction not found after waiting 10ms");
    expect(getTransaction).toHaveBeenCalled();
  });

  it("does not retry a transaction whose metadata is null", async () => {
    const connection = new Connection("http://localhost");
    const getTransaction = vi
      .spyOn(connection, "getTransaction")
      .mockResolvedValue({
        meta: null,
      } as unknown as Awaited<ReturnType<typeof connection.getTransaction>>);

    await expect(
      GetCommitmentSignature("scheduling", connection, 10_000),
    ).rejects.toThrow("Transaction not found or meta is null");
    expect(getTransaction).toHaveBeenCalledTimes(1);
  });

  it("does not retry a failed transaction lookup RPC", async () => {
    const connection = new Connection("http://localhost");
    const rpcError = new Error("RPC unavailable");
    const getTransaction = vi
      .spyOn(connection, "getTransaction")
      .mockRejectedValue(rpcError);

    await expect(
      GetCommitmentSignature("scheduling", connection, 10_000),
    ).rejects.toBe(rpcError);
    expect(getTransaction).toHaveBeenCalledTimes(1);
  });

  it("extracts result details from a new-format receipt", () => {
    const receipt = receiptFromLogs("outbox", [
      "Program log: Settlement receipt signature[0]: base-1",
      "Program log: Settlement receipt signature[1]: base-2",
      "Program log: Settlement receipt error message: action failed",
    ]);
    expect(receipt.outboxAddress).toBe("outbox");
    // Keep the previous public field available for existing consumers.
    expect(receipt.intentAddress).toBe("outbox");
    expect(receipt.signatures).toEqual(["base-1", "base-2"]);
    expect(receipt.result.errorMessage).toBe("action failed");
  });

  it("ignores old signature labels", () => {
    const receipt = receiptFromLogs(null, [
      "Program log: ScheduledCommitSent signature[0]: base-signature",
    ]);

    expect(receipt.outboxAddress).toBeNull();
    expect(receipt.intentAddress).toBeNull();
    expect(receipt.signatures).toEqual([]);
  });

  it("accepts only matching outbox completion logs", () => {
    const receipt = receiptFromCompletion(makeCompletion(42), outbox);
    expect(receipt?.outboxAddress).toBe(outbox.toBase58());
    expect(receipt?.signatures).toEqual(["real-signature"]);
    expect(receipt?.result.errorMessage).toBe("action failed");
  });

  it("rejects spoofed logs, mismatched intent IDs, and failed transactions", () => {
    const tx = makeCompletion(42);
    tx.meta.logMessages = [
      "Program attacker111 invoke [1]",
      "Program log: Settlement receipt intent id: 42, slot: 10, blockhash: hash",
      "Program log: Settlement receipt signature[0]: forged-signature",
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
