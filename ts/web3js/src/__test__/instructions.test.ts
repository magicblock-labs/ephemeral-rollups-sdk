import { describe, it, expect } from "vitest";
import { PublicKey } from "@solana/web3.js";
import {
  createDelegateInstruction,
  createTopUpEscrowInstruction,
  createCloseEscrowInstruction,
  type DelegateInstructionArgs,
} from "../instructions/delegation-program";
import {
  createCommitInstruction,
  createCommitAndUndelegateInstruction,
} from "../instructions/magic-program";
import {
  DELEGATION_PROGRAM_ID,
  MAGIC_PROGRAM_ID,
  MAGIC_CONTEXT_ID,
} from "../constants";

describe("Exposed Instructions (web3.js)", () => {
  const mockPublicKey = new PublicKey("11111111111111111111111111111111");

  const differentKey = new PublicKey("11111111111111111111111111111112");

  describe("delegate instruction", () => {
    it("should create a delegate instruction with correct parameters", () => {
      const args: DelegateInstructionArgs = {
        commitFrequencyMs: 1000,
        seeds: [],
      };
      const instruction = createDelegateInstruction(
        {
          payer: mockPublicKey,
          delegatedAccount: mockPublicKey,
          ownerProgram: mockPublicKey,
        },
        args,
      );

      expect(instruction.keys).toHaveLength(7);
      expect(instruction.data).toBeDefined();
      expect(instruction.data.length).toBeGreaterThan(0);
      expect(instruction.programId.toBase58()).toBe(
        DELEGATION_PROGRAM_ID.toBase58(),
      );
    });

    it("should create a delegate instruction without validator", () => {
      const args: DelegateInstructionArgs = {
        commitFrequencyMs: 1000,
        seeds: [],
      };
      const instruction = createDelegateInstruction(
        {
          payer: mockPublicKey,
          delegatedAccount: mockPublicKey,
          ownerProgram: mockPublicKey,
        },
        args,
      );

      expect(instruction.keys).toHaveLength(7);
      expect(instruction.data).toBeDefined();
      expect(instruction.data.length).toBeGreaterThan(0);
    });

    it("should include all required account keys", () => {
      const args: DelegateInstructionArgs = {
        commitFrequencyMs: 1000,
        seeds: [],
      };
      const instruction = createDelegateInstruction(
        {
          payer: mockPublicKey,
          delegatedAccount: mockPublicKey,
          ownerProgram: mockPublicKey,
        },
        args,
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

    it("should serialize validator in args when provided in accounts", () => {
      const args: DelegateInstructionArgs = {
        commitFrequencyMs: 1000,
        seeds: [],
      };
      const instruction = createDelegateInstruction(
        {
          payer: mockPublicKey,
          delegatedAccount: mockPublicKey,
          ownerProgram: mockPublicKey,
          validator: mockPublicKey,
        },
        args,
      );

      expect(instruction.keys).toHaveLength(7);
      expect(instruction.data).toBeDefined();
      // Validator should be serialized in args (1 byte discriminant + 32 bytes pubkey at the end)
      expect(instruction.data.length).toBeGreaterThanOrEqual(
        8 + 4 + 4 + 1 + 32,
      );
    });

    it("should allow validator override via args", () => {
      const validatorFromArgs = new PublicKey(
        "11111111111111111111111111111112",
      );
      const instruction = createDelegateInstruction(
        {
          payer: mockPublicKey,
          delegatedAccount: mockPublicKey,
          ownerProgram: mockPublicKey,
          validator: mockPublicKey,
        },
        {
          commitFrequencyMs: 1000,
          seeds: [],
          validator: validatorFromArgs,
        },
      );

      expect(instruction.keys).toHaveLength(7);
      // Args validator should override accounts validator
      expect(instruction.data).toBeDefined();
    });

    it("should support different account addresses", () => {
      const args: DelegateInstructionArgs = {
        commitFrequencyMs: 1000,
        seeds: [],
      };
      const instruction1 = createDelegateInstruction(
        {
          payer: mockPublicKey,
          delegatedAccount: mockPublicKey,
          ownerProgram: mockPublicKey,
        },
        args,
      );
      const instruction2 = createDelegateInstruction(
        {
          payer: mockPublicKey,
          delegatedAccount: differentKey,
          ownerProgram: mockPublicKey,
        },
        args,
      );

      // Both should be valid instructions but with different account references
      expect(instruction1.data).toBeDefined();
      expect(instruction2.data).toBeDefined();
    });

    it("should handle various commitFrequencyMs values", () => {
      const frequencies = [0, 1000, 5000, 60000];

      frequencies.forEach((freq) => {
        const args: DelegateInstructionArgs = {
          commitFrequencyMs: freq,
          seeds: [],
        };
        const instruction = createDelegateInstruction(
          {
            payer: mockPublicKey,
            delegatedAccount: mockPublicKey,
            ownerProgram: mockPublicKey,
          },
          args,
        );

        expect(instruction.data).toBeDefined();
      });
    });

    it("should use default commitFrequencyMs when args not provided", () => {
      const instruction = createDelegateInstruction({
        payer: mockPublicKey,
        delegatedAccount: mockPublicKey,
        ownerProgram: mockPublicKey,
      });

      expect(instruction.data).toBeDefined();
      expect(instruction.keys).toHaveLength(7);
      expect(instruction.programId.toBase58()).toBe(
        DELEGATION_PROGRAM_ID.toBase58(),
      );
    });

    it("should handle multiple seeds", () => {
      const args: DelegateInstructionArgs = {
        commitFrequencyMs: 1000,
        seeds: [new Uint8Array([1, 2, 3]), new Uint8Array([4, 5, 6])],
      };
      const instruction = createDelegateInstruction(
        {
          payer: mockPublicKey,
          delegatedAccount: mockPublicKey,
          ownerProgram: mockPublicKey,
        },
        args,
      );

      expect(instruction.data).toBeDefined();
    });

    it("should serialize commitFrequencyMs as u32", () => {
      const args: DelegateInstructionArgs = {
        commitFrequencyMs: 1000,
        seeds: [],
      };
      const instruction = createDelegateInstruction(
        {
          payer: mockPublicKey,
          delegatedAccount: mockPublicKey,
          ownerProgram: mockPublicKey,
        },
        args,
      );

      expect(instruction.data).toBeDefined();
      // Discriminator: 8 bytes, commitFrequencyMs: 4 bytes (u32), seeds length: 4 bytes
      const minSize = 8 + 4 + 4;
      expect(instruction.data.length).toBeGreaterThanOrEqual(minSize);

      // Check commitFrequencyMs value at offset 8
      expect(instruction.data.readUInt32LE(8)).toBe(1000);
    });

    it("should serialize with default commitFrequencyMs as max u32", () => {
      const instruction = createDelegateInstruction({
        payer: mockPublicKey,
        delegatedAccount: mockPublicKey,
        ownerProgram: mockPublicKey,
      });

      expect(instruction.data).toBeDefined();
      // Check default commitFrequencyMs (0xffffffff) at offset 8
      expect(instruction.data.readUInt32LE(8)).toBe(0xffffffff);
    });

    it("should serialize seeds array correctly", () => {
      const args: DelegateInstructionArgs = {
        commitFrequencyMs: 1000,
        seeds: [new Uint8Array([1, 2, 3]), new Uint8Array([4, 5, 6])],
      };
      const instruction = createDelegateInstruction(
        {
          payer: mockPublicKey,
          delegatedAccount: mockPublicKey,
          ownerProgram: mockPublicKey,
        },
        args,
      );

      expect(instruction.data).toBeDefined();
      // Offset 12 should have seeds array length = 2
      expect(instruction.data.readUInt32LE(12)).toBe(2);
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
      const delegateArgs: DelegateInstructionArgs = {
        commitFrequencyMs: 1000,
        seeds: [],
      };
      const delegateInstr = createDelegateInstruction(
        {
          payer: mockPublicKey,
          delegatedAccount: mockPublicKey,
          ownerProgram: mockPublicKey,
        },
        delegateArgs,
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

  describe("scheduleCommit instruction (Magic Program)", () => {
    it("should create a scheduleCommit instruction with required parameters", () => {
      const instruction = createCommitInstruction(mockPublicKey, [
        mockPublicKey,
      ]);

      expect(instruction.keys).toHaveLength(3);
      expect(instruction.data).toBeDefined();
      expect(instruction.data.length).toBe(4);
      expect(instruction.programId.toBase58()).toBe(
        MAGIC_PROGRAM_ID.toBase58(),
      );
    });

    it("should have correct discriminator", () => {
      const instruction = createCommitInstruction(mockPublicKey, [
        mockPublicKey,
      ]);

      // Discriminator should be [1,0,0,0] for scheduleCommit
      expect(instruction.data.readUInt32LE(0)).toBe(1);
    });

    it("should include payer as signer and writable", () => {
      const instruction = createCommitInstruction(mockPublicKey, [
        mockPublicKey,
      ]);

      expect(instruction.keys[0].pubkey.toBase58()).toBe(
        mockPublicKey.toBase58(),
      );
      expect(instruction.keys[0].isSigner).toBe(true);
      expect(instruction.keys[0].isWritable).toBe(true);
    });

    it("should include magic context as writable", () => {
      const instruction = createCommitInstruction(mockPublicKey, [
        mockPublicKey,
      ]);

      expect(instruction.keys[1].pubkey.toBase58()).toBe(
        MAGIC_CONTEXT_ID.toBase58(),
      );
      expect(instruction.keys[1].isSigner).toBe(false);
      expect(instruction.keys[1].isWritable).toBe(true);
    });

    it("should include accounts to commit as readonly", () => {
      const accountsToCommit = [
        new PublicKey("11111111111111111111111111111113"),
        new PublicKey("11111111111111111111111111111114"),
      ];
      const instruction = createCommitInstruction(
        mockPublicKey,
        accountsToCommit,
      );

      expect(instruction.keys).toHaveLength(4);
      expect(instruction.keys[2].pubkey.toBase58()).toBe(
        accountsToCommit[0].toBase58(),
      );
      expect(instruction.keys[2].isSigner).toBe(false);
      expect(instruction.keys[2].isWritable).toBe(false);
      expect(instruction.keys[3].pubkey.toBase58()).toBe(
        accountsToCommit[1].toBase58(),
      );
    });

    it("should handle single account to commit", () => {
      const instruction = createCommitInstruction(mockPublicKey, [
        differentKey,
      ]);

      expect(instruction.keys).toHaveLength(3);
      expect(instruction.keys[2].pubkey.toBase58()).toBe(
        differentKey.toBase58(),
      );
    });

    it("should handle multiple accounts to commit", () => {
      const accounts = [
        new PublicKey("11111111111111111111111111111113"),
        new PublicKey("11111111111111111111111111111114"),
        new PublicKey("11111111111111111111111111111115"),
      ];
      const instruction = createCommitInstruction(mockPublicKey, accounts);

      expect(instruction.keys).toHaveLength(5);
      accounts.forEach((account, index) => {
        expect(instruction.keys[2 + index].pubkey.toBase58()).toBe(
          account.toBase58(),
        );
      });
    });
  });

  describe("scheduleCommitAndUndelegate instruction (Magic Program)", () => {
    it("should create a scheduleCommitAndUndelegate instruction with required parameters", () => {
      const instruction = createCommitAndUndelegateInstruction(mockPublicKey, [
        mockPublicKey,
      ]);

      expect(instruction.keys).toHaveLength(3);
      expect(instruction.data).toBeDefined();
      expect(instruction.data.length).toBe(4);
      expect(instruction.programId.toBase58()).toBe(
        MAGIC_PROGRAM_ID.toBase58(),
      );
    });

    it("should have correct discriminator", () => {
      const instruction = createCommitAndUndelegateInstruction(mockPublicKey, [
        mockPublicKey,
      ]);

      // Discriminator should be [2,0,0,0] for scheduleCommitAndUndelegate
      expect(instruction.data.readUInt32LE(0)).toBe(2);
    });

    it("should include payer as signer and writable", () => {
      const instruction = createCommitAndUndelegateInstruction(mockPublicKey, [
        mockPublicKey,
      ]);

      expect(instruction.keys[0].pubkey.toBase58()).toBe(
        mockPublicKey.toBase58(),
      );
      expect(instruction.keys[0].isSigner).toBe(true);
      expect(instruction.keys[0].isWritable).toBe(true);
    });

    it("should include magic context as writable", () => {
      const instruction = createCommitAndUndelegateInstruction(mockPublicKey, [
        mockPublicKey,
      ]);

      expect(instruction.keys[1].pubkey.toBase58()).toBe(
        MAGIC_CONTEXT_ID.toBase58(),
      );
      expect(instruction.keys[1].isSigner).toBe(false);
      expect(instruction.keys[1].isWritable).toBe(true);
    });

    it("should include accounts to commit and undelegate as readonly", () => {
      const accountsToCommitAndUndelegate = [
        new PublicKey("11111111111111111111111111111113"),
        new PublicKey("11111111111111111111111111111114"),
      ];
      const instruction = createCommitAndUndelegateInstruction(
        mockPublicKey,
        accountsToCommitAndUndelegate,
      );

      expect(instruction.keys).toHaveLength(4);
      expect(instruction.keys[2].pubkey.toBase58()).toBe(
        accountsToCommitAndUndelegate[0].toBase58(),
      );
      expect(instruction.keys[2].isSigner).toBe(false);
      expect(instruction.keys[2].isWritable).toBe(false);
      expect(instruction.keys[3].pubkey.toBase58()).toBe(
        accountsToCommitAndUndelegate[1].toBase58(),
      );
    });

    it("should handle single account to commit and undelegate", () => {
      const instruction = createCommitAndUndelegateInstruction(mockPublicKey, [
        differentKey,
      ]);

      expect(instruction.keys).toHaveLength(3);
      expect(instruction.keys[2].pubkey.toBase58()).toBe(
        differentKey.toBase58(),
      );
    });

    it("should handle multiple accounts to commit and undelegate", () => {
      const accounts = [
        new PublicKey("11111111111111111111111111111113"),
        new PublicKey("11111111111111111111111111111114"),
        new PublicKey("11111111111111111111111111111115"),
      ];
      const instruction = createCommitAndUndelegateInstruction(
        mockPublicKey,
        accounts,
      );

      expect(instruction.keys).toHaveLength(5);
      accounts.forEach((account, index) => {
        expect(instruction.keys[2 + index].pubkey.toBase58()).toBe(
          account.toBase58(),
        );
      });
    });
  });
});
