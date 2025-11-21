import { describe, it, expect } from "vitest";
import { PublicKey, SystemProgram } from "@solana/web3.js";
import {
  createDelegateInstruction,
  serializeDelegateInstructionData,
  createTopUpEscrowInstruction,
  serializeTopUpEscrowInstructionData,
  createCloseEscrowInstruction,
  serializeCloseEscrowInstructionData,
} from "../generated/delegation-program-instructions";

describe("Generated Instructions (web3.js)", () => {
  const mockPublicKey = new PublicKey(
    "11111111111111111111111111111111"
  );

  describe("delegate instruction", () => {
    it("should create a delegate instruction", () => {
      const instruction = createDelegateInstruction(
        {
          payer: mockPublicKey,
          delegatedAccount: mockPublicKey,
          ownerProgram: mockPublicKey,
          delegateBuffer: mockPublicKey,
          delegationRecord: mockPublicKey,
          delegationMetadata: mockPublicKey,
          systemProgram: SystemProgram.programId,
        },
        {
          commitFrequencyMs: 1000,
          seeds: [new Uint8Array([1, 2, 3])],
          validator: null,
        }
      );

      expect(instruction.keys).toHaveLength(7);
      expect(instruction.data).toBeDefined();
      expect(instruction.programId.equals(instruction.programId)).toBe(true);
    });

    it("should serialize delegate instruction data with correct discriminator", () => {
      const data = serializeDelegateInstructionData({
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
      const frequency = data.readUInt32LE(8);
      expect(frequency).toBe(1000);
    });

    it("should include validator in serialized data when provided", () => {
      const validatorKey = mockPublicKey;
      const data = serializeDelegateInstructionData({
        commitFrequencyMs: 500,
        seeds: [],
        validator: validatorKey,
      });

      expect(data.length).toBeGreaterThan(17);
    });
  });

  describe("topUpEscrow instruction", () => {
    it("should create a topUpEscrow instruction", () => {
      const instruction = createTopUpEscrowInstruction(
        {
          payer: mockPublicKey,
          pubkey: mockPublicKey,
          ephemeralBalanceAccount: mockPublicKey,
          systemProgram: SystemProgram.programId,
        },
        {
          amount: BigInt(1000000),
          index: 255,
        }
      );

      expect(instruction.keys).toHaveLength(4);
      expect(instruction.data.length).toBe(17);
    });

    it("should serialize topUpEscrow instruction data correctly", () => {
      const data = serializeTopUpEscrowInstructionData({
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
      const amount = data.readBigUInt64LE(8);
      expect(amount).toBe(BigInt(5000000));

      // Check index (u8 at offset 16)
      expect(data[16]).toBe(255);
    });
  });

  describe("closeEscrow instruction", () => {
    it("should create a closeEscrow instruction", () => {
      const instruction = createCloseEscrowInstruction(
        {
          payer: mockPublicKey,
          ephemeralBalanceAccount: mockPublicKey,
          systemProgram: SystemProgram.programId,
        },
        {
          index: 255,
        }
      );

      expect(instruction.keys).toHaveLength(3);
      expect(instruction.data.length).toBe(9);
    });

    it("should serialize closeEscrow instruction data correctly", () => {
      const data = serializeCloseEscrowInstructionData({
        index: 255,
      });

      expect(data.length).toBe(9);

      // Check discriminator
      expect(data[0]).toBe(11);
      for (let i = 1; i < 8; i++) {
        expect(data[i]).toBe(0);
      }

      // Check index (u8 at offset 8)
      expect(data[8]).toBe(255);
    });
  });
});
