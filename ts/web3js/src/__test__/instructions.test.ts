import { describe, it, expect } from "vitest";
import { PublicKey } from "@solana/web3.js";
import {
  createDelegateInstruction,
  createTopUpEscrowInstruction,
  createCloseEscrowInstruction,
  type DelegateInstructionData,
} from "../instructions/delegation-program";
import { DELEGATION_PROGRAM_ID } from "../constants";

describe("Exposed Instructions (web3.js)", () => {
  const mockPublicKey = new PublicKey("11111111111111111111111111111111");

  const differentKey = new PublicKey("11111111111111111111111111111112");

  describe("delegate instruction", () => {
    it("should create a delegate instruction with correct parameters", () => {
      const data: DelegateInstructionData = {
        commitFrequencyMs: 1000,
        validator: mockPublicKey,
      };
      const instruction = createDelegateInstruction(
        mockPublicKey,
        [new Uint8Array([1, 2, 3])],
        mockPublicKey,
        mockPublicKey,
        mockPublicKey,
        mockPublicKey,
        mockPublicKey,
        data,
      );

      expect(instruction.keys).toHaveLength(7);
      expect(instruction.data).toBeDefined();
      expect(instruction.data.length).toBeGreaterThan(0);
      expect(instruction.programId.toBase58()).toBe(
        DELEGATION_PROGRAM_ID.toBase58(),
      );
    });

    it("should create a delegate instruction without validator", () => {
      const data: DelegateInstructionData = {
        commitFrequencyMs: 1000,
      };
      const instruction = createDelegateInstruction(
        mockPublicKey,
        [new Uint8Array([1, 2, 3])],
        mockPublicKey,
        mockPublicKey,
        mockPublicKey,
        mockPublicKey,
        mockPublicKey,
        data,
      );

      expect(instruction.keys).toHaveLength(7);
      expect(instruction.data).toBeDefined();
      expect(instruction.data.length).toBeGreaterThan(0);
    });

    it("should include all required account keys", () => {
      const data: DelegateInstructionData = {
        commitFrequencyMs: 1000,
      };
      const instruction = createDelegateInstruction(
        mockPublicKey,
        [new Uint8Array([1, 2, 3])],
        mockPublicKey,
        mockPublicKey,
        mockPublicKey,
        mockPublicKey,
        mockPublicKey,
        data,
      );

      const keyCount = instruction.keys.length;
      expect(keyCount).toBe(7);

      // Verify all keys are AccountMeta objects with proper structure
      instruction.keys.forEach((key) => {
        expect(key.pubkey).toBeDefined();
        expect(key.isSigner).toBeDefined();
        expect(key.isWritable).toBeDefined();
      });
    });

    it("should handle null validator parameter", () => {
      const data: DelegateInstructionData = {
        commitFrequencyMs: 1000,
        validator: null,
      };
      const instruction = createDelegateInstruction(
        mockPublicKey,
        [new Uint8Array([1, 2, 3])],
        mockPublicKey,
        mockPublicKey,
        mockPublicKey,
        mockPublicKey,
        mockPublicKey,
        data,
      );

      expect(instruction.keys).toHaveLength(7);
      expect(instruction.data).toBeDefined();
    });

    it("should support different account addresses", () => {
      const data: DelegateInstructionData = {
        commitFrequencyMs: 1000,
      };
      const instruction1 = createDelegateInstruction(
        mockPublicKey,
        [new Uint8Array([1, 2, 3])],
        mockPublicKey,
        mockPublicKey,
        mockPublicKey,
        mockPublicKey,
        mockPublicKey,
        data,
      );
      const instruction2 = createDelegateInstruction(
        differentKey,
        [new Uint8Array([1, 2, 3])],
        mockPublicKey,
        mockPublicKey,
        mockPublicKey,
        mockPublicKey,
        mockPublicKey,
        data,
      );

      // Both should be valid instructions but with different account references
      expect(instruction1.data).toBeDefined();
      expect(instruction2.data).toBeDefined();
    });

    it("should handle various commitFrequencyMs values", () => {
      const frequencies = [0, 1000, 5000, 60000];

      frequencies.forEach((freq) => {
        const data: DelegateInstructionData = {
          commitFrequencyMs: freq,
        };
        const instruction = createDelegateInstruction(
          mockPublicKey,
          [new Uint8Array([1, 2, 3])],
          mockPublicKey,
          mockPublicKey,
          mockPublicKey,
          mockPublicKey,
          mockPublicKey,
          data,
        );

        expect(instruction.data).toBeDefined();
      });
    });

    it("should handle multiple seeds", () => {
      const data: DelegateInstructionData = {
        commitFrequencyMs: 1000,
      };
      const instruction = createDelegateInstruction(
        mockPublicKey,
        [
          new Uint8Array([1, 2, 3]),
          new Uint8Array([4, 5, 6]),
          new Uint8Array([7, 8, 9]),
        ],
        mockPublicKey,
        mockPublicKey,
        mockPublicKey,
        mockPublicKey,
        mockPublicKey,
        data,
      );

      expect(instruction.data).toBeDefined();
    });

    it("should handle empty seeds array", () => {
      const data: DelegateInstructionData = {
        commitFrequencyMs: 1000,
      };
      const instruction = createDelegateInstruction(
        mockPublicKey,
        [],
        mockPublicKey,
        mockPublicKey,
        mockPublicKey,
        mockPublicKey,
        mockPublicKey,
        data,
      );

      expect(instruction.data).toBeDefined();
    });
  });

  describe("topUpEscrow instruction", () => {
    it("should create a topUpEscrow instruction with all parameters", () => {
      const instruction = createTopUpEscrowInstruction(
        mockPublicKey,
        mockPublicKey,
        mockPublicKey,
        1000000,
        255,
      );

      expect(instruction.keys).toHaveLength(4);
      expect(instruction.data).toBeDefined();
      expect(instruction.data.length).toBe(17);
      expect(instruction.programId.toBase58()).toBe(
        DELEGATION_PROGRAM_ID.toBase58(),
      );
    });

    it("should create a topUpEscrow instruction with default index", () => {
      const instruction = createTopUpEscrowInstruction(
        mockPublicKey,
        mockPublicKey,
        mockPublicKey,
        1000000,
      );

      expect(instruction.keys).toHaveLength(4);
      expect(instruction.data).toBeDefined();
      expect(instruction.data.length).toBe(17);

      // Check discriminator (first 8 bytes)
      expect(instruction.data[0]).toBe(9);
      for (let i = 1; i < 8; i++) {
        expect(instruction.data[i]).toBe(0);
      }

      // Check amount (u64 at offset 8)
      const amount = instruction.data.readBigUInt64LE(8);
      expect(amount).toBe(BigInt(1000000));

      // Check index defaults to 255 (u8 at offset 16)
      expect(instruction.data[16]).toBe(255);
    });

    it("should convert number amount to bigint internally", () => {
      const instruction = createTopUpEscrowInstruction(
        mockPublicKey,
        mockPublicKey,
        mockPublicKey,
        1234567,
      );

      // Check amount is correctly serialized (u64 at offset 8)
      const amount = instruction.data.readBigUInt64LE(8);
      expect(amount).toBe(BigInt(1234567));
    });

    it("should handle custom index values", () => {
      const testIndices = [0, 1, 100, 254, 255];

      testIndices.forEach((index) => {
        const instruction = createTopUpEscrowInstruction(
          mockPublicKey,
          mockPublicKey,
          mockPublicKey,
          1000000,
          index,
        );

        expect(instruction.data[16]).toBe(index);
      });
    });

    it("should handle zero amount", () => {
      const instruction = createTopUpEscrowInstruction(
        mockPublicKey,
        mockPublicKey,
        mockPublicKey,
        0,
      );

      const amount = instruction.data.readBigUInt64LE(8);
      expect(amount).toBe(BigInt(0));
    });

    it("should handle large amounts", () => {
      const largeAmount = 9007199254740991; // Max safe integer in JS

      const instruction = createTopUpEscrowInstruction(
        mockPublicKey,
        mockPublicKey,
        mockPublicKey,
        largeAmount,
      );

      const amount = instruction.data.readBigUInt64LE(8);
      expect(amount).toBe(BigInt(largeAmount));
    });

    it("should include correct account keys", () => {
      const instruction = createTopUpEscrowInstruction(
        mockPublicKey,
        mockPublicKey,
        mockPublicKey,
        1000000,
      );

      expect(instruction.keys.length).toBe(4);
      instruction.keys.forEach((key) => {
        expect(key.pubkey).toBeDefined();
        expect(typeof key.isSigner).toBe("boolean");
        expect(typeof key.isWritable).toBe("boolean");
      });
    });

    it("should use consistent data format for the same params", () => {
      const instruction1 = createTopUpEscrowInstruction(
        mockPublicKey,
        mockPublicKey,
        mockPublicKey,
        1000000,
      );
      const instruction2 = createTopUpEscrowInstruction(
        mockPublicKey,
        mockPublicKey,
        mockPublicKey,
        1000000,
      );

      expect(instruction1.data).toEqual(instruction2.data);
    });
  });

  describe("closeEscrow instruction", () => {
    it("should create a closeEscrow instruction with all parameters", () => {
      const instruction = createCloseEscrowInstruction(
        mockPublicKey,
        mockPublicKey,
        255,
      );

      expect(instruction.keys).toHaveLength(3);
      expect(instruction.data).toBeDefined();
      expect(instruction.data.length).toBe(9);
      expect(instruction.programId.toBase58()).toBe(
        DELEGATION_PROGRAM_ID.toBase58(),
      );
    });

    it("should create a closeEscrow instruction with default index", () => {
      const instruction = createCloseEscrowInstruction(
        mockPublicKey,
        mockPublicKey,
      );

      expect(instruction.keys).toHaveLength(3);
      expect(instruction.data).toBeDefined();
      expect(instruction.data.length).toBe(9);

      // Check discriminator (first 8 bytes)
      expect(instruction.data[0]).toBe(11);
      for (let i = 1; i < 8; i++) {
        expect(instruction.data[i]).toBe(0);
      }

      // Check index defaults to 255 (u8 at offset 8)
      expect(instruction.data[8]).toBe(255);
    });

    it("should handle custom index values", () => {
      const testIndices = [0, 1, 100, 254, 255];

      testIndices.forEach((index) => {
        const instruction = createCloseEscrowInstruction(
          mockPublicKey,
          mockPublicKey,
          index,
        );

        expect(instruction.data[8]).toBe(index);
      });
    });

    it("should include correct account keys", () => {
      const instruction = createCloseEscrowInstruction(
        mockPublicKey,
        mockPublicKey,
      );

      expect(instruction.keys.length).toBe(3);
      instruction.keys.forEach((key) => {
        expect(key.pubkey).toBeDefined();
        expect(typeof key.isSigner).toBe("boolean");
        expect(typeof key.isWritable).toBe("boolean");
      });
    });

    it("should use consistent data format for the same params", () => {
      const instruction1 = createCloseEscrowInstruction(
        mockPublicKey,
        mockPublicKey,
      );
      const instruction2 = createCloseEscrowInstruction(
        mockPublicKey,
        mockPublicKey,
      );

      expect(instruction1.data).toEqual(instruction2.data);
    });

    it("should have correct discriminator", () => {
      const instruction = createCloseEscrowInstruction(
        mockPublicKey,
        mockPublicKey,
      );

      // Discriminator should be 11 for closeEphemeralBalance
      expect(instruction.data[0]).toBe(11);
    });
  });

  describe("Cross-instruction consistency", () => {
    it("should all target the same delegation program", () => {
      const delegateData: DelegateInstructionData = {
        commitFrequencyMs: 1000,
      };
      const delegateInstr = createDelegateInstruction(
        mockPublicKey,
        [],
        mockPublicKey,
        mockPublicKey,
        mockPublicKey,
        mockPublicKey,
        mockPublicKey,
        delegateData,
      );

      const topUpInstr = createTopUpEscrowInstruction(
        mockPublicKey,
        mockPublicKey,
        mockPublicKey,
        1000000,
      );

      const closeInstr = createCloseEscrowInstruction(
        mockPublicKey,
        mockPublicKey,
      );

      const programId = DELEGATION_PROGRAM_ID.toBase58();
      expect(delegateInstr.programId.toBase58()).toBe(programId);
      expect(topUpInstr.programId.toBase58()).toBe(programId);
      expect(closeInstr.programId.toBase58()).toBe(programId);
    });
  });
});
