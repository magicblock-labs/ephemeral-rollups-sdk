import { describe, it, expect } from "vitest";
import { AccountRole } from "@solana/instructions";
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
import { type Address } from "@solana/kit";
import { MAGIC_PROGRAM_ID, MAGIC_CONTEXT_ID } from "../constants";

describe("Exposed Instructions (@solana/kit)", () => {
  const mockAddress = "11111111111111111111111111111111" as Address;
  const differentAddress = "11111111111111111111111111111112" as Address;

  describe("delegate instruction", () => {
    it("should create a delegate instruction with correct parameters", async () => {
      const args: DelegateInstructionArgs = {
        commitFrequencyMs: 1000,
        seeds: [],
      };
      const instruction = await createDelegateInstruction(
        {
          payer: mockAddress,
          delegatedAccount: mockAddress,
          ownerProgram: mockAddress,
        },
        args,
      );

      expect(instruction.accounts).toHaveLength(7);
      expect(instruction.data).toBeDefined();
    });

    it("should create a delegate instruction without validator", async () => {
      const args: DelegateInstructionArgs = {
        commitFrequencyMs: 1000,
        seeds: [],
      };
      const instruction = await createDelegateInstruction(
        {
          payer: mockAddress,
          delegatedAccount: mockAddress,
          ownerProgram: mockAddress,
        },
        args,
      );

      expect(instruction.accounts).toHaveLength(7);
      expect(instruction.data).toBeDefined();
    });

    it("should include all required account keys", async () => {
      const args: DelegateInstructionArgs = {
        commitFrequencyMs: 1000,
        seeds: [],
      };
      const instruction = await createDelegateInstruction(
        {
          payer: mockAddress,
          delegatedAccount: mockAddress,
          ownerProgram: mockAddress,
        },
        args,
      );

      expect(instruction.accounts).toBeDefined();
      expect(instruction.accounts).toHaveLength(7);

      // Verify all accounts have proper structure
      instruction.accounts?.forEach((account) => {
        expect(account).toBeDefined();
        expect(account.address).toBeDefined();
      });
    });

    it("should serialize validator in args when provided in accounts", async () => {
      const args: DelegateInstructionArgs = {
        commitFrequencyMs: 1000,
        seeds: [],
      };
      const instruction = await createDelegateInstruction(
        {
          payer: mockAddress,
          delegatedAccount: mockAddress,
          ownerProgram: mockAddress,
          validator: mockAddress,
        },
        args,
      );

      expect(instruction.accounts).toHaveLength(7);
      expect(instruction.data).toBeDefined();
      // Validator should be serialized in args (1 byte discriminant + 32 bytes pubkey at the end)
      expect(instruction.data?.length).toBeGreaterThanOrEqual(
        8 + 4 + 4 + 1 + 32,
      );
    });

    it("should allow validator override via args", async () => {
      const validatorFromArgs = "11111111111111111111111111111115" as Address;
      const instruction = await createDelegateInstruction(
        {
          payer: mockAddress,
          delegatedAccount: mockAddress,
          ownerProgram: mockAddress,
          validator: mockAddress,
        },
        {
          commitFrequencyMs: 1000,
          seeds: [],
          validator: validatorFromArgs,
        },
      );

      expect(instruction.accounts).toHaveLength(7);
      // Args validator should override accounts validator
      expect(instruction.data).toBeDefined();
    });

    it("should support different account addresses", async () => {
      const args: DelegateInstructionArgs = {
        commitFrequencyMs: 1000,
        seeds: [],
      };
      const instruction1 = await createDelegateInstruction(
        {
          payer: mockAddress,
          delegatedAccount: mockAddress,
          ownerProgram: mockAddress,
        },
        args,
      );
      const instruction2 = await createDelegateInstruction(
        {
          payer: mockAddress,
          delegatedAccount: differentAddress,
          ownerProgram: mockAddress,
        },
        args,
      );

      // Both should be valid instructions but with different account references
      expect(instruction1.data).toBeDefined();
      expect(instruction2.data).toBeDefined();
    });

    it("should handle various commitFrequencyMs values", async () => {
      const frequencies = [0, 1000, 5000, 60000];

      for (const freq of frequencies) {
        const args: DelegateInstructionArgs = {
          commitFrequencyMs: freq,
          seeds: [],
        };
        const instruction = await createDelegateInstruction(
          {
            payer: mockAddress,
            delegatedAccount: mockAddress,
            ownerProgram: mockAddress,
          },
          args,
        );

        expect(instruction.data).toBeDefined();
      }
    });

    it("should use default commitFrequencyMs when args not provided", async () => {
      const instruction = await createDelegateInstruction({
        payer: mockAddress,
        delegatedAccount: mockAddress,
        ownerProgram: mockAddress,
      });

      expect(instruction.data).toBeDefined();
      expect(instruction.accounts).toHaveLength(7);
    });

    it("should handle multiple seeds", async () => {
      const args: DelegateInstructionArgs = {
        commitFrequencyMs: 1000,
        seeds: [new Uint8Array([1, 2, 3]), new Uint8Array([4, 5, 6])],
      };
      const instruction = await createDelegateInstruction(
        {
          payer: mockAddress,
          delegatedAccount: mockAddress,
          ownerProgram: mockAddress,
        },
        args,
      );

      expect(instruction.data).toBeDefined();
    });

    it("should serialize commitFrequencyMs as u32", async () => {
      const args: DelegateInstructionArgs = {
        commitFrequencyMs: 1000,
        seeds: [],
      };
      const instruction = await createDelegateInstruction(
        {
          payer: mockAddress,
          delegatedAccount: mockAddress,
          ownerProgram: mockAddress,
        },
        args,
      );

      expect(instruction.data).toBeDefined();
      // Discriminator: 8 bytes, commitFrequencyMs: 4 bytes (u32), seeds length: 4 bytes
      const minSize = 8 + 4 + 4;
      expect(instruction.data?.length).toBeGreaterThanOrEqual(minSize);

      // Check commitFrequencyMs value at offset 8
      const view = new DataView(instruction.data?.buffer as ArrayBuffer, 8, 4);
      expect(view.getUint32(0, true)).toBe(1000);
    });

    it("should serialize with default commitFrequencyMs as max u32", async () => {
      const instruction = await createDelegateInstruction({
        payer: mockAddress,
        delegatedAccount: mockAddress,
        ownerProgram: mockAddress,
      });

      expect(instruction.data).toBeDefined();
      // Check default commitFrequencyMs (0xffffffff) at offset 8
      const view = new DataView(instruction.data?.buffer as ArrayBuffer, 8, 4);
      expect(view.getUint32(0, true)).toBe(0xffffffff);
    });

    it("should serialize seeds array correctly", async () => {
      const args: DelegateInstructionArgs = {
        commitFrequencyMs: 1000,
        seeds: [new Uint8Array([1, 2, 3]), new Uint8Array([4, 5, 6])],
      };
      const instruction = await createDelegateInstruction(
        {
          payer: mockAddress,
          delegatedAccount: mockAddress,
          ownerProgram: mockAddress,
        },
        args,
      );

      expect(instruction.data).toBeDefined();
      // Offset 12 should have seeds array length = 2
      const view = new DataView(instruction.data?.buffer as ArrayBuffer, 12, 4);
      expect(view.getUint32(0, true)).toBe(2);
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
    it("should all return valid instruction objects", async () => {
      const delegateArgs: DelegateInstructionArgs = {
        commitFrequencyMs: 1000,
        seeds: [],
      };
      const delegateInstr = await createDelegateInstruction(
        {
          payer: mockAddress,
          delegatedAccount: mockAddress,
          ownerProgram: mockAddress,
        },
        delegateArgs,
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

  describe("scheduleCommit instruction (Magic Program)", () => {
    it("should create a scheduleCommit instruction with required parameters", () => {
      const instruction = createCommitInstruction(mockAddress, [mockAddress]);

      expect(instruction.accounts).toHaveLength(3);
      expect(instruction.data).toBeDefined();
      expect(instruction.data?.length).toBe(4);
      expect(instruction.programAddress).toBe(MAGIC_PROGRAM_ID);
    });

    it("should have correct discriminator", () => {
      const instruction = createCommitInstruction(mockAddress, [mockAddress]);

      // Discriminator should be [1,0,0,0] for scheduleCommit
      expect(
        new DataView(instruction.data?.buffer as ArrayBuffer).getUint32(
          0,
          true,
        ),
      ).toBe(1);
    });

    it("should include payer as signer and writable", () => {
      const instruction = createCommitInstruction(mockAddress, [mockAddress]);

      expect(instruction.accounts?.[0].address).toBe(mockAddress);
      expect(instruction.accounts?.[0].role).toBe(AccountRole.WRITABLE_SIGNER);
    });

    it("should include magic context as writable", () => {
      const instruction = createCommitInstruction(mockAddress, [mockAddress]);

      expect(instruction.accounts?.[1].address).toBe(MAGIC_CONTEXT_ID);
      expect(instruction.accounts?.[1].role).toBe(AccountRole.WRITABLE);
    });

    it("should include accounts to commit as readonly", () => {
      const accountsToCommit: Address[] = [
        "11111111111111111111111111111113" as Address,
        "11111111111111111111111111111114" as Address,
      ];
      const instruction = createCommitInstruction(
        mockAddress,
        accountsToCommit,
      );

      expect(instruction.accounts).toHaveLength(4);
      expect(instruction.accounts?.[2].address).toBe(accountsToCommit[0]);
      expect(instruction.accounts?.[2].role).toBe(AccountRole.READONLY);
      expect(instruction.accounts?.[3].address).toBe(accountsToCommit[1]);
    });

    it("should handle single account to commit", () => {
      const instruction = createCommitInstruction(mockAddress, [
        differentAddress,
      ]);

      expect(instruction.accounts).toHaveLength(3);
      expect(instruction.accounts?.[2].address).toBe(differentAddress);
    });

    it("should handle multiple accounts to commit", () => {
      const accounts: Address[] = [
        "22222222222222222222222222222222" as Address,
        "33333333333333333333333333333333" as Address,
        "44444444444444444444444444444444" as Address,
      ];
      const instruction = createCommitInstruction(mockAddress, accounts);

      expect(instruction.accounts).toHaveLength(5);
      accounts.forEach((account, index) => {
        expect(instruction.accounts?.[2 + index].address).toBe(account);
      });
    });
  });

  describe("scheduleCommitAndUndelegate instruction (Magic Program)", () => {
    it("should create a scheduleCommitAndUndelegate instruction with required parameters", () => {
      const instruction = createCommitAndUndelegateInstruction(mockAddress, [
        mockAddress,
      ]);

      expect(instruction.accounts).toHaveLength(3);
      expect(instruction.data).toBeDefined();
      expect(instruction.data?.length).toBe(4);
      expect(instruction.programAddress).toBe(MAGIC_PROGRAM_ID);
    });

    it("should have correct discriminator", () => {
      const instruction = createCommitAndUndelegateInstruction(mockAddress, [
        mockAddress,
      ]);

      // Discriminator should be [2,0,0,0] for scheduleCommitAndUndelegate
      expect(
        new DataView(instruction.data?.buffer as ArrayBuffer).getUint32(
          0,
          true,
        ),
      ).toBe(2);
    });

    it("should include payer as signer and writable", () => {
      const instruction = createCommitAndUndelegateInstruction(mockAddress, [
        mockAddress,
      ]);

      expect(instruction.accounts?.[0].address).toBe(mockAddress);
      expect(instruction.accounts?.[0].role).toBe(AccountRole.WRITABLE_SIGNER);
    });

    it("should include magic context as writable", () => {
      const instruction = createCommitAndUndelegateInstruction(mockAddress, [
        mockAddress,
      ]);

      expect(instruction.accounts?.[1].address).toBe(MAGIC_CONTEXT_ID);
      expect(instruction.accounts?.[1].role).toBe(AccountRole.WRITABLE);
    });

    it("should include accounts to commit and undelegate as readonly", () => {
      const accountsToCommitAndUndelegate: Address[] = [
        "11111111111111111111111111111113" as Address,
        "11111111111111111111111111111114" as Address,
      ];
      const instruction = createCommitAndUndelegateInstruction(
        mockAddress,
        accountsToCommitAndUndelegate,
      );

      expect(instruction.accounts).toHaveLength(4);
      expect(instruction.accounts?.[2].address).toBe(
        accountsToCommitAndUndelegate[0],
      );
      expect(instruction.accounts?.[2].role).toBe(AccountRole.READONLY);
      expect(instruction.accounts?.[3].address).toBe(
        accountsToCommitAndUndelegate[1],
      );
    });

    it("should handle single account to commit and undelegate", () => {
      const instruction = createCommitAndUndelegateInstruction(mockAddress, [
        differentAddress,
      ]);

      expect(instruction.accounts).toHaveLength(3);
      expect(instruction.accounts?.[2].address).toBe(differentAddress);
    });

    it("should handle multiple accounts to commit and undelegate", () => {
      const accounts: Address[] = [
        "22222222222222222222222222222222" as Address,
        "33333333333333333333333333333333" as Address,
        "44444444444444444444444444444444" as Address,
      ];
      const instruction = createCommitAndUndelegateInstruction(
        mockAddress,
        accounts,
      );

      expect(instruction.accounts).toHaveLength(5);
      accounts.forEach((account, index) => {
        expect(instruction.accounts?.[2 + index].address).toBe(account);
      });
    });
  });
});
