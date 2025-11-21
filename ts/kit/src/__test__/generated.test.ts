import { describe, it, expect } from "vitest";
import {
  createDelegateInstruction,
  serializeDelegateInstructionData,
  createTopUpEphemeralBalanceInstruction,
  serializeTopUpEphemeralBalanceInstructionData,
  createCloseEphemeralBalanceInstruction,
  serializeCloseEphemeralBalanceInstructionData,
} from "../generated/delegation-program-instructions";
import { getAddressFromPublicKey } from "@solana/kit";

describe("Generated Instructions (@solana/kit)", () => {
  const mockAddress = "11111111111111111111111111111111" as any;

  describe("delegate instruction", () => {
    it("should create a delegate instruction", () => {
      const instruction = createDelegateInstruction(
        {
          payer: mockAddress,
          delegatedAccount: mockAddress,
          ownerProgram: mockAddress,
          delegateBuffer: mockAddress,
          delegationRecord: mockAddress,
          delegationMetadata: mockAddress,
          systemProgram: mockAddress,
        },
        {
          commitFrequencyMs: 1000,
          seeds: [new Uint8Array([1, 2, 3])],
          validator: null,
        }
      );

      expect(instruction.accounts).toHaveLength(7);
      expect(instruction.data).toBeDefined();
    });

    it("should serialize delegate instruction data with correct discriminator", () => {
      const [data] = serializeDelegateInstructionData({
        commitFrequencyMs: 1000,
        seeds: [new Uint8Array([1, 2, 3])],
        validator: null,
      });

      // Check discriminator
      expect(data[0]).toBe(0);
      expect(data[1]).toBe(0);
      expect(data[2]).toBe(0);
      expect(data[3]).toBe(0);
      expect(data[4]).toBe(0);
      expect(data[5]).toBe(0);
      expect(data[6]).toBe(0);
      expect(data[7]).toBe(0);

      // Check commit_frequency_ms (u32 at offset 8)
      const frequency = new DataView(data.buffer, 8, 4).getUint32(0, true);
      expect(frequency).toBe(1000);
    });

    it("should include validator in serialized data when provided", () => {
      const validatorKey = mockAddress;
      const [data] = serializeDelegateInstructionData({
        commitFrequencyMs: 500,
        seeds: [],
        validator: validatorKey,
      });

      expect(data.length).toBeGreaterThan(17);
    });
  });

  describe("topUpEphemeralBalance instruction", () => {
    it("should create a topUpEphemeralBalance instruction", () => {
      const instruction = createTopUpEphemeralBalanceInstruction(
        {
          payer: mockAddress,
          pubkey: mockAddress,
          ephemeralBalanceAccount: mockAddress,
          systemProgram: mockAddress,
        },
        {
          amount: BigInt(1000000),
          index: 0,
        }
      );

      expect(instruction.accounts).toHaveLength(4);
      expect(instruction.data).toBeDefined();
      expect(instruction.data?.length).toBe(17);
    });

    it("should serialize topUpEphemeralBalance instruction data correctly", () => {
      const [data] = serializeTopUpEphemeralBalanceInstructionData({
        amount: BigInt(5000000),
        index: 255,
      });

      expect(data.length).toBe(17);

      // Check discriminator
      expect(data[0]).toBe(9);
      for (let i = 1; i < 8; i++) {
        expect(data[i]).toBe(0);
      }

      // Check amount (u64 at offset 8)
      const amount = new DataView(data.buffer, 8, 8).getBigUint64(0, true);
      expect(amount).toBe(BigInt(5000000));

      // Check index (u8 at offset 16)
      expect(data[16]).toBe(255);
    });
  });

  describe("closeEphemeralBalance instruction", () => {
    it("should create a closeEphemeralBalance instruction", () => {
      const instruction = createCloseEphemeralBalanceInstruction(
        {
          payer: mockAddress,
          ephemeralBalanceAccount: mockAddress,
          systemProgram: mockAddress,
        },
        {
          index: 42,
        }
      );

      expect(instruction.accounts).toHaveLength(3);
      expect(instruction.data).toBeDefined();
      expect(instruction.data?.length).toBe(9);
    });

    it("should serialize closeEphemeralBalance instruction data correctly", () => {
      const [data] = serializeCloseEphemeralBalanceInstructionData({
        index: 100,
      });

      expect(data.length).toBe(9);

      // Check discriminator
      expect(data[0]).toBe(11);
      for (let i = 1; i < 8; i++) {
        expect(data[i]).toBe(0);
      }

      // Check index (u8 at offset 8)
      expect(data[8]).toBe(100);
    });
  });
});
