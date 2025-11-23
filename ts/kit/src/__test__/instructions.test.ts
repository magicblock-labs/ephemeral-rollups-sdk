import { describe, it, expect } from "vitest";
import {
  createDelegateInstruction,
  createTopUpEscrowInstruction,
  createCloseEscrowInstruction,
  type DelegateInstructionData,
} from "../instructions/delegation-program";
import { type Address } from "@solana/kit";

describe("Exposed Instructions (@solana/kit)", () => {
  const mockAddress = "11111111111111111111111111111111" as Address;
  const differentAddress = "11111111111111111111111111111112" as Address;

  describe("delegate instruction", () => {
    it("should create a delegate instruction with correct parameters", () => {
      const data: DelegateInstructionData = {
        commitFrequencyMs: 1000,
        validator: mockAddress,
      };
      const instruction = createDelegateInstruction(
        mockAddress,
        [new Uint8Array([1, 2, 3])],
        mockAddress,
        mockAddress,
        mockAddress,
        mockAddress,
        mockAddress,
        data,
      );

      expect(instruction.accounts).toHaveLength(7);
      expect(instruction.data).toBeDefined();
    });

    it("should create a delegate instruction without validator", () => {
      const data: DelegateInstructionData = {
        commitFrequencyMs: 1000,
      };
      const instruction = createDelegateInstruction(
        mockAddress,
        [new Uint8Array([1, 2, 3])],
        mockAddress,
        mockAddress,
        mockAddress,
        mockAddress,
        mockAddress,
        data,
      );

      expect(instruction.accounts).toHaveLength(7);
      expect(instruction.data).toBeDefined();
    });

    it("should include all required account keys", () => {
      const data: DelegateInstructionData = {
        commitFrequencyMs: 1000,
      };
      const instruction = createDelegateInstruction(
        mockAddress,
        [new Uint8Array([1, 2, 3])],
        mockAddress,
        mockAddress,
        mockAddress,
        mockAddress,
        mockAddress,
        data,
      );

      expect(instruction.accounts).toBeDefined();
      expect(instruction.accounts).toHaveLength(7);

      // Verify all accounts have proper structure
      instruction.accounts?.forEach((account) => {
        expect(account).toBeDefined();
        expect(account.address).toBeDefined();
      });
    });

    it("should handle null validator parameter", () => {
      const data: DelegateInstructionData = {
        commitFrequencyMs: 1000,
        validator: null,
      };
      const instruction = createDelegateInstruction(
        mockAddress,
        [new Uint8Array([1, 2, 3])],
        mockAddress,
        mockAddress,
        mockAddress,
        mockAddress,
        mockAddress,
        data,
      );

      expect(instruction.accounts).toHaveLength(7);
      expect(instruction.data).toBeDefined();
    });

    it("should support different account addresses", () => {
      const data: DelegateInstructionData = {
        commitFrequencyMs: 1000,
      };
      const instruction1 = createDelegateInstruction(
        mockAddress,
        [new Uint8Array([1, 2, 3])],
        mockAddress,
        mockAddress,
        mockAddress,
        mockAddress,
        mockAddress,
        data,
      );
      const instruction2 = createDelegateInstruction(
        differentAddress,
        [new Uint8Array([1, 2, 3])],
        mockAddress,
        mockAddress,
        mockAddress,
        mockAddress,
        mockAddress,
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
          mockAddress,
          [new Uint8Array([1, 2, 3])],
          mockAddress,
          mockAddress,
          mockAddress,
          mockAddress,
          mockAddress,
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
        mockAddress,
        [
          new Uint8Array([1, 2, 3]),
          new Uint8Array([4, 5, 6]),
          new Uint8Array([7, 8, 9]),
        ],
        mockAddress,
        mockAddress,
        mockAddress,
        mockAddress,
        mockAddress,
        data,
      );

      expect(instruction.data).toBeDefined();
    });

    it("should handle empty seeds array", () => {
      const data: DelegateInstructionData = {
        commitFrequencyMs: 1000,
      };
      const instruction = createDelegateInstruction(
        mockAddress,
        [],
        mockAddress,
        mockAddress,
        mockAddress,
        mockAddress,
        mockAddress,
        data,
      );

      expect(instruction.data).toBeDefined();
    });
  });

  describe("topUpEscrow instruction", () => {
    it("should create a topUpEscrow instruction with all parameters", () => {
      const instruction = createTopUpEscrowInstruction(
        mockAddress,
        mockAddress,
        mockAddress,
        1000000,
        255,
      );

      expect(instruction.accounts).toHaveLength(4);
      expect(instruction.data).toBeDefined();
      expect(instruction.data?.length).toBe(17);
    });

    it("should create a topUpEscrow instruction with default index", () => {
      const instruction = createTopUpEscrowInstruction(
        mockAddress,
        mockAddress,
        mockAddress,
        1000000,
      );

      expect(instruction.data).toBeDefined();
      expect(instruction.data?.length).toBe(17);

      // Check discriminator (first 8 bytes)
      expect(instruction.data?.[0]).toBe(9);
      for (let i = 1; i < 8; i++) {
        expect(instruction.data?.[i]).toBe(0);
      }

      // Check amount (u64 at offset 8)
      const buffer = instruction.data?.buffer as ArrayBuffer | undefined;
      expect(buffer).toBeDefined();
      // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
      const amount = new DataView(buffer!, 8, 8).getBigUint64(0, true);
      expect(amount).toBe(BigInt(1000000));

      // Check index defaults to 255 (u8 at offset 16)
      expect(instruction.data?.[16]).toBe(255);
    });

    it("should convert number amount to bigint internally", () => {
      const instruction = createTopUpEscrowInstruction(
        mockAddress,
        mockAddress,
        mockAddress,
        1234567,
      );

      // Check amount is correctly serialized (u64 at offset 8)
      const buffer2 = instruction.data?.buffer as ArrayBuffer | undefined;
      expect(buffer2).toBeDefined();
      // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
      const amount = new DataView(buffer2!, 8, 8).getBigUint64(0, true);
      expect(amount).toBe(BigInt(1234567));
    });

    it("should handle custom index values", () => {
      const testIndices = [0, 1, 100, 254, 255];

      testIndices.forEach((index) => {
        const instruction = createTopUpEscrowInstruction(
          mockAddress,
          mockAddress,
          mockAddress,
          1000000,
          index,
        );

        expect(instruction.data?.[16]).toBe(index);
      });
    });

    it("should handle zero amount", () => {
      const instruction = createTopUpEscrowInstruction(
        mockAddress,
        mockAddress,
        mockAddress,
        0,
      );

      const buffer3 = instruction.data?.buffer as ArrayBuffer | undefined;
      expect(buffer3).toBeDefined();
      // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
      const amount = new DataView(buffer3!, 8, 8).getBigUint64(0, true);
      expect(amount).toBe(BigInt(0));
    });

    it("should handle large amounts", () => {
      const largeAmount = 9007199254740991; // Max safe integer in JS

      const instruction = createTopUpEscrowInstruction(
        mockAddress,
        mockAddress,
        mockAddress,
        largeAmount,
      );

      const buffer4 = instruction.data?.buffer as ArrayBuffer | undefined;
      expect(buffer4).toBeDefined();
      // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
      const amount = new DataView(buffer4!, 8, 8).getBigUint64(0, true);
      expect(amount).toBe(BigInt(largeAmount));
    });

    it("should include correct account keys", () => {
      const instruction = createTopUpEscrowInstruction(
        mockAddress,
        mockAddress,
        mockAddress,
        1000000,
      );

      expect(instruction.accounts).toBeDefined();
      expect(instruction.accounts).toHaveLength(4);
      instruction.accounts?.forEach((account) => {
        expect(account).toBeDefined();
        expect(account.address).toBeDefined();
      });
    });

    it("should use consistent data format for the same params", () => {
      const instruction1 = createTopUpEscrowInstruction(
        mockAddress,
        mockAddress,
        mockAddress,
        1000000,
      );
      const instruction2 = createTopUpEscrowInstruction(
        mockAddress,
        mockAddress,
        mockAddress,
        1000000,
      );

      expect(instruction1.data).toEqual(instruction2.data);
    });
  });

  describe("closeEscrow instruction", () => {
    it("should create a closeEscrow instruction with all parameters", () => {
      const instruction = createCloseEscrowInstruction(
        mockAddress,
        mockAddress,
        255,
      );

      expect(instruction.accounts).toHaveLength(3);
      expect(instruction.data).toBeDefined();
      expect(instruction.data?.length).toBe(9);
    });

    it("should create a closeEscrow instruction with default index", () => {
      const instruction = createCloseEscrowInstruction(
        mockAddress,
        mockAddress,
      );

      expect(instruction.data).toBeDefined();
      expect(instruction.data?.length).toBe(9);

      // Check discriminator (first 8 bytes)
      expect(instruction.data?.[0]).toBe(11);
      for (let i = 1; i < 8; i++) {
        expect(instruction.data?.[i]).toBe(0);
      }

      // Check index defaults to 255 (u8 at offset 8)
      expect(instruction.data?.[8]).toBe(255);
    });

    it("should handle custom index values", () => {
      const testIndices = [0, 1, 100, 254, 255];

      testIndices.forEach((index) => {
        const instruction = createCloseEscrowInstruction(
          mockAddress,
          mockAddress,
          index,
        );

        expect(instruction.data?.[8]).toBe(index);
      });
    });

    it("should include correct account keys", () => {
      const instruction = createCloseEscrowInstruction(
        mockAddress,
        mockAddress,
      );

      expect(instruction.accounts).toBeDefined();
      expect(instruction.accounts).toHaveLength(3);
      instruction.accounts?.forEach((account) => {
        expect(account).toBeDefined();
        expect(account.address).toBeDefined();
      });
    });

    it("should use consistent data format for the same params", () => {
      const instruction1 = createCloseEscrowInstruction(
        mockAddress,
        mockAddress,
      );
      const instruction2 = createCloseEscrowInstruction(
        mockAddress,
        mockAddress,
      );

      expect(instruction1.data).toEqual(instruction2.data);
    });

    it("should have correct discriminator", () => {
      const instruction = createCloseEscrowInstruction(
        mockAddress,
        mockAddress,
      );

      // Discriminator should be 11 for closeEphemeralBalance
      expect(instruction.data?.[0]).toBe(11);
    });
  });

  describe("Cross-instruction consistency", () => {
    it("should all return valid instruction objects", () => {
      const delegateData: DelegateInstructionData = {
        commitFrequencyMs: 1000,
      };
      const delegateInstr = createDelegateInstruction(
        mockAddress,
        [],
        mockAddress,
        mockAddress,
        mockAddress,
        mockAddress,
        mockAddress,
        delegateData,
      );

      const topUpInstr = createTopUpEscrowInstruction(
        mockAddress,
        mockAddress,
        mockAddress,
        1000000,
      );

      const closeInstr = createCloseEscrowInstruction(mockAddress, mockAddress);

      expect(delegateInstr.accounts).toBeDefined();
      expect(delegateInstr.data).toBeDefined();
      expect(topUpInstr.accounts).toBeDefined();
      expect(topUpInstr.data).toBeDefined();
      expect(closeInstr.accounts).toBeDefined();
      expect(closeInstr.data).toBeDefined();
    });
  });
});
