import { describe, it, expect } from "vitest";
import { Keypair, PublicKey } from "@solana/web3.js";
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
  delegateEataPermissionIx,
  depositAndQueueTransferIx,
  delegateSpl,
  delegateSplWithPrivateTransfer,
  delegateTransferQueueIx,
  deriveEphemeralAta,
  deriveRentPda,
  deriveShuttleAta,
  deriveShuttleEphemeralAta,
  deriveVault,
  ensureTransferQueueCrankIx,
  initEphemeralAtaIx,
  initVaultIx,
  initRentPdaIx,
  transferSpl,
  withdrawSplIx,
  withdrawSpl,
} from "../instructions/ephemeral-spl-token-program";
import {
  DELEGATION_PROGRAM_ID,
  EPHEMERAL_SPL_TOKEN_PROGRAM_ID,
  MAGIC_PROGRAM_ID,
  MAGIC_CONTEXT_ID,
} from "../constants";

function readLengthPrefixedField(
  data: Uint8Array,
  offset: number,
): [Buffer, number] {
  const len = data[offset];
  const start = offset + 1;
  const end = start + len;
  return [Buffer.from(data.subarray(start, end)), end];
}

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

  describe("initRentPdaIx (Ephemeral SPL Token Program)", () => {
    it("should derive and initialize the global rent PDA", () => {
      const [rentPda] = deriveRentPda();
      const instruction = initRentPdaIx(mockPublicKey, rentPda);

      expect(instruction.programId.toBase58()).toBe(
        EPHEMERAL_SPL_TOKEN_PROGRAM_ID.toBase58(),
      );
      expect(instruction.keys).toHaveLength(3);
      expect(instruction.keys[0].pubkey.toBase58()).toBe(
        mockPublicKey.toBase58(),
      );
      expect(instruction.keys[0].isSigner).toBe(true);
      expect(instruction.keys[1].pubkey.toBase58()).toBe(rentPda.toBase58());
      expect(instruction.data).toEqual(Buffer.from([23]));
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

  describe("delegateSpl (Ephemeral SPL Token Program)", () => {
    const owner = new PublicKey("11111111111111111111111111111113");
    const mint = new PublicKey("11111111111111111111111111111114");
    const validator = new PublicKey("11111111111111111111111111111115");

    it("should delegate the vault eata when initializing the vault in legacy flow", async () => {
      const [vault] = deriveVault(mint);
      const [vaultEphemeralAta] = deriveEphemeralAta(vault, mint);

      const instructions = await delegateSpl(owner, mint, 1n, {
        validator,
        initIfMissing: true,
        initVaultIfMissing: true,
        idempotent: false,
      });

      expect(instructions[3].keys[1].pubkey.toBase58()).toBe(
        vaultEphemeralAta.toBase58(),
      );
      expect(instructions[3].data[0]).toBe(4);
      expect(
        Buffer.from(instructions[3].data.subarray(1)).equals(
          validator.toBuffer(),
        ),
      ).toBe(true);
    });

    it("should delegate the vault eata when initializing the vault in idempotent flow", async () => {
      const [vault] = deriveVault(mint);
      const [vaultEphemeralAta] = deriveEphemeralAta(vault, mint);

      const instructions = await delegateSpl(owner, mint, 1n, {
        validator,
        initVaultIfMissing: true,
        shuttleId: 7,
      });

      expect(instructions[2].keys[1].pubkey.toBase58()).toBe(
        vaultEphemeralAta.toBase58(),
      );
      expect(instructions[2].data[0]).toBe(4);
      expect(
        Buffer.from(instructions[2].data.subarray(1)).equals(
          validator.toBuffer(),
        ),
      ).toBe(true);
    });

    it("should use setup_and_delegate_shuttle_with_merge in idempotent flow when amount is nonzero", async () => {
      const instructions = await delegateSpl(owner, mint, 1n, {
        validator,
        shuttleId: 7,
      });

      const setupAndDelegateInstruction = instructions.find(
        (ix) => ix.data[0] === 24,
      );

      expect(setupAndDelegateInstruction).toBeDefined();
      if (setupAndDelegateInstruction == null) {
        throw new Error("Expected setup_and_delegate instruction");
      }
      expect(setupAndDelegateInstruction?.keys).toHaveLength(19);
      expect(instructions.find((ix) => ix.data[0] === 11)).toBeUndefined();
      expect(
        Buffer.from(setupAndDelegateInstruction.data).readBigUInt64LE(5),
      ).toBe(1n);
      expect(
        Buffer.from(setupAndDelegateInstruction.data.subarray(13)).equals(
          validator.toBuffer(),
        ),
      ).toBe(true);
    });

    it("should keep the shuttle eata writable in the zero-amount shuttle setup flow", async () => {
      const [shuttleEphemeralAta] = deriveShuttleEphemeralAta(owner, mint, 7);
      const [shuttleAta] = deriveShuttleAta(shuttleEphemeralAta, mint);

      const instructions = await delegateSpl(owner, mint, 0n, {
        validator,
        shuttleId: 7,
      });

      const initShuttleInstruction = instructions.find(
        (ix) => ix.data[0] === 11,
      );
      const delegateShuttleInstruction = instructions.find(
        (ix) => ix.data[0] === 13,
      );

      expect(initShuttleInstruction).toBeDefined();
      expect(delegateShuttleInstruction).toBeDefined();
      expect(initShuttleInstruction?.keys[2].pubkey.toBase58()).toBe(
        shuttleAta.toBase58(),
      );
      expect(delegateShuttleInstruction?.keys[2].pubkey.toBase58()).toBe(
        shuttleAta.toBase58(),
      );
      expect(initShuttleInstruction?.keys[2].isWritable).toBe(true);
      expect(delegateShuttleInstruction?.keys[2].isWritable).toBe(true);
    });
  });

  describe("delegateSplWithPrivateTransfer (Ephemeral SPL Token Program)", () => {
    const owner = new PublicKey("11111111111111111111111111111113");
    const mint = new PublicKey("11111111111111111111111111111114");
    const validator = Keypair.generate().publicKey;

    it("should use the private transfer shuttle flow", async () => {
      const instructions = await delegateSplWithPrivateTransfer(
        owner,
        mint,
        1n,
        {
          validator,
          shuttleId: 7,
          initTransferQueueIfMissing: true,
          minDelayMs: 100n,
          maxDelayMs: 300n,
          split: 4,
        },
      );

      const privateTransferInstruction = instructions.find(
        (ix) => ix.data[0] === 25,
      );

      expect(instructions.find((ix) => ix.data[0] === 12)).toBeDefined();
      expect(privateTransferInstruction).toBeDefined();
      if (privateTransferInstruction == null) {
        throw new Error("Expected private transfer instruction");
      }
      expect(privateTransferInstruction?.keys).toHaveLength(19);
      const data = Buffer.from(privateTransferInstruction.data);
      expect(data.readUInt32LE(1)).toBe(7);
      expect(data.readBigUInt64LE(5)).toBe(1n);

      const [validatorField, nextOffset] = readLengthPrefixedField(data, 13);
      const [destinationField, suffixOffset] = readLengthPrefixedField(
        data,
        nextOffset,
      );
      const [suffixField, endOffset] = readLengthPrefixedField(
        data,
        suffixOffset,
      );

      expect(validatorField.equals(validator.toBuffer())).toBe(true);
      expect(destinationField).toHaveLength(80);
      expect(suffixField).toHaveLength(69);
      expect(endOffset).toBe(data.length);
    });
  });

  describe("withdrawSpl (Ephemeral SPL Token Program)", () => {
    const owner = new PublicKey("11111111111111111111111111111113");
    const mint = new PublicKey("11111111111111111111111111111114");
    const validator = new PublicKey("11111111111111111111111111111115");

    it("should use the delegated shuttle withdrawal flow when idempotent", async () => {
      const instructions = await withdrawSpl(owner, mint, 1n, {
        validator,
        shuttleId: 7,
      });

      const withdrawInstruction = instructions.find((ix) => ix.data[0] === 26);

      expect(withdrawInstruction).toBeDefined();
      expect(withdrawInstruction?.keys).toHaveLength(16);
      expect(instructions.find((ix) => ix.data[0] === 3)).toBeUndefined();
    });

    it("should fall back to the legacy withdraw instruction when idempotent is false", async () => {
      const instructions = await withdrawSpl(owner, mint, 1n, {
        idempotent: false,
      });

      expect(instructions).toHaveLength(1);
      expect(instructions[0].data[0]).toBe(3);
    });
  });

  describe("transferSpl (Ephemeral SPL Token Program)", () => {
    const from = Keypair.generate().publicKey;
    const to = Keypair.generate().publicKey;
    const mint = Keypair.generate().publicKey;
    const validator = Keypair.generate().publicKey;

    it("should use the shuttle private transfer instruction for private base-to-base transfers", async () => {
      const instructions = await transferSpl(from, to, mint, 25n, {
        visibility: "private",
        fromBalance: "base",
        toBalance: "base",
        validator,
        shuttleId: 7,
        privateTransfer: {
          minDelayMs: 100n,
          maxDelayMs: 300n,
          split: 4,
        },
      });

      expect(instructions).toHaveLength(1);
      const data = Buffer.from(instructions[0].data);
      expect(data[0]).toBe(25);
      expect(instructions[0].keys).toHaveLength(19);
      expect(data.readUInt32LE(1)).toBe(7);
      expect(data.readBigUInt64LE(5)).toBe(25n);

      const [validatorField, nextOffset] = readLengthPrefixedField(data, 13);
      const [destinationField, suffixOffset] = readLengthPrefixedField(
        data,
        nextOffset,
      );
      const [suffixField, endOffset] = readLengthPrefixedField(
        data,
        suffixOffset,
      );

      expect(validatorField.equals(validator.toBuffer())).toBe(true);
      expect(destinationField).toHaveLength(80);
      expect(suffixField).toHaveLength(69);
      expect(endOffset).toBe(data.length);
    });

    it("should initialize the destination ATA and vault when requested", async () => {
      const [vault] = deriveVault(mint);
      const [vaultEphemeralAta] = deriveEphemeralAta(vault, mint);

      const instructions = await transferSpl(from, to, mint, 25n, {
        visibility: "private",
        fromBalance: "base",
        toBalance: "base",
        validator,
        shuttleId: 7,
        initIfMissing: true,
        initVaultIfMissing: true,
        privateTransfer: {
          minDelayMs: 100n,
          maxDelayMs: 300n,
          split: 4,
        },
      });

      expect(instructions).toHaveLength(4);
      expect(instructions[2].keys[1].pubkey.toBase58()).toBe(
        vaultEphemeralAta.toBase58(),
      );
      expect(instructions[2].data[0]).toBe(4);
      expect(instructions[3].data[0]).toBe(25);
    });

    it("should prepend source ATA creation when initAtasIfMissing is set on base-source transfers", async () => {
      const instructions = await transferSpl(from, to, mint, 25n, {
        visibility: "public",
        fromBalance: "base",
        toBalance: "base",
        initAtasIfMissing: true,
      });

      expect(instructions).toHaveLength(2);
      expect(instructions[0].data[0]).toBe(1);
      expect(instructions[0].keys[2].pubkey.toBase58()).toBe(from.toBase58());
      expect(instructions[1].data[0]).toBe(3);
    });

    it("should use the shuttle merge instruction for private base-to-ephemeral transfers", async () => {
      const instructions = await transferSpl(from, to, mint, 25n, {
        visibility: "private",
        fromBalance: "base",
        toBalance: "ephemeral",
        validator,
        shuttleId: 7,
      });

      expect(instructions).toHaveLength(1);
      expect(instructions[0].data[0]).toBe(24);
      expect(instructions[0].keys).toHaveLength(19);
      expect(Buffer.from(instructions[0].data).readBigUInt64LE(5)).toBe(25n);
    });

    it("should initialize and delegate the receiver eata for private base-to-ephemeral transfers when requested", async () => {
      const [toEphemeralAta] = deriveEphemeralAta(to, mint);

      const instructions = await transferSpl(from, to, mint, 25n, {
        visibility: "private",
        fromBalance: "base",
        toBalance: "ephemeral",
        validator,
        shuttleId: 7,
        initIfMissing: true,
      });

      expect(instructions).toHaveLength(4);
      expect(instructions[0].data[0]).toBe(1);
      expect(instructions[0].keys[2].pubkey.toBase58()).toBe(to.toBase58());
      expect(instructions[1].data[0]).toBe(0);
      expect(instructions[1].keys[0].pubkey.toBase58()).toBe(
        toEphemeralAta.toBase58(),
      );
      expect(instructions[2].data[0]).toBe(4);
      expect(instructions[2].keys[1].pubkey.toBase58()).toBe(
        toEphemeralAta.toBase58(),
      );
      expect(instructions[3].data[0]).toBe(24);
    });

    it("should ignore initAtasIfMissing on ephemeral-source transfers", async () => {
      const instructions = await transferSpl(from, to, mint, 25n, {
        visibility: "private",
        fromBalance: "ephemeral",
        toBalance: "base",
        initAtasIfMissing: true,
        privateTransfer: {
          minDelayMs: 100n,
          maxDelayMs: 300n,
          split: 4,
        },
      });

      expect(instructions).toHaveLength(1);
      expect(instructions[0].data[0]).toBe(16);
    });

    it("should use depositAndQueueTransferIx for private ephemeral-to-base transfers", async () => {
      const instructions = await transferSpl(from, to, mint, 25n, {
        visibility: "private",
        fromBalance: "ephemeral",
        toBalance: "base",
        initIfMissing: true,
        initVaultIfMissing: true,
        privateTransfer: {
          minDelayMs: 100n,
          maxDelayMs: 300n,
          split: 4,
        },
      });

      expect(instructions).toHaveLength(1);
      expect(instructions[0].data[0]).toBe(16);
      expect(instructions[0].keys).toHaveLength(8);
      expect(Buffer.from(instructions[0].data).readBigUInt64LE(1)).toBe(25n);
      expect(Buffer.from(instructions[0].data).readBigUInt64LE(9)).toBe(100n);
      expect(Buffer.from(instructions[0].data).readBigUInt64LE(17)).toBe(300n);
      expect(Buffer.from(instructions[0].data).readUInt32LE(25)).toBe(4);
    });

    it("should use a normal transfer for public base-to-base transfers", async () => {
      const instructions = await transferSpl(from, to, mint, 25n, {
        visibility: "public",
        fromBalance: "base",
        toBalance: "base",
      });

      expect(instructions).toHaveLength(1);
      expect(instructions[0].data[0]).toBe(3);
      expect(instructions[0].keys).toHaveLength(3);
      expect(Buffer.from(instructions[0].data).readBigUInt64LE(1)).toBe(25n);
    });

    it("should use a normal transfer for public ephemeral-to-ephemeral transfers", async () => {
      const instructions = await transferSpl(from, to, mint, 25n, {
        visibility: "public",
        fromBalance: "ephemeral",
        toBalance: "ephemeral",
      });

      expect(instructions).toHaveLength(1);
      expect(instructions[0].data[0]).toBe(3);
      expect(instructions[0].keys).toHaveLength(3);
      expect(Buffer.from(instructions[0].data).readBigUInt64LE(1)).toBe(25n);
    });

    it("should use a normal transfer for private ephemeral-to-ephemeral transfers", async () => {
      const instructions = await transferSpl(from, to, mint, 25n, {
        visibility: "private",
        fromBalance: "ephemeral",
        toBalance: "ephemeral",
        initIfMissing: true,
        initVaultIfMissing: true,
      });

      expect(instructions).toHaveLength(1);
      expect(instructions[0].data[0]).toBe(3);
      expect(instructions[0].keys).toHaveLength(3);
      expect(Buffer.from(instructions[0].data).readBigUInt64LE(1)).toBe(25n);
    });

    it("should reject unsupported routes", async () => {
      await expect(
        transferSpl(from, to, mint, 25n, {
          visibility: "public",
          fromBalance: "base",
          toBalance: "ephemeral",
        }),
      ).rejects.toThrow(
        "transferSpl route not implemented: visibility=public, fromBalance=base, toBalance=ephemeral",
      );
    });
  });

  describe("ensureTransferQueueCrankIx (Ephemeral SPL Token Program)", () => {
    const payer = mockPublicKey;
    const queue = differentKey;

    it("should include queue, magic context, and magic program in order", () => {
      const instruction = ensureTransferQueueCrankIx(payer, queue);

      expect(instruction.keys).toHaveLength(4);
      expect(instruction.keys[0].pubkey.toBase58()).toBe(payer.toBase58());
      expect(instruction.keys[1].pubkey.toBase58()).toBe(queue.toBase58());
      expect(instruction.keys[2].pubkey.toBase58()).toBe(
        MAGIC_CONTEXT_ID.toBase58(),
      );
      expect(instruction.keys[3].pubkey.toBase58()).toBe(
        MAGIC_PROGRAM_ID.toBase58(),
      );
    });
  });

  describe("depositAndQueueTransferIx (Ephemeral SPL Token Program)", () => {
    const queue = differentKey;
    const vault = new PublicKey("11111111111111111111111111111113");
    const mint = new PublicKey("11111111111111111111111111111114");
    const source = new PublicKey("11111111111111111111111111111115");
    const vaultAta = new PublicKey("11111111111111111111111111111116");
    const destination = new PublicKey("11111111111111111111111111111117");

    it("should serialize min/max delay ms and split", () => {
      const instruction = depositAndQueueTransferIx(
        queue,
        vault,
        mint,
        source,
        vaultAta,
        destination,
        mockPublicKey,
        25n,
        100n,
        300n,
        4,
      );

      expect(instruction.keys).toHaveLength(8);
      expect(Array.from(instruction.data)).toEqual([
        16,
        ...Array.from(
          Buffer.from(
            [25n, 100n, 300n].flatMap((value) => {
              const out = Buffer.alloc(8);
              out.writeBigUInt64LE(value);
              return Array.from(out);
            }),
          ),
        ),
        4,
        0,
        0,
        0,
      ]);
    });
  });

  describe("delegateTransferQueueIx (Ephemeral SPL Token Program)", () => {
    const payer = mockPublicKey;
    const queue = differentKey;

    it("should serialize discriminator 19 for the delegated transfer queue opcode", () => {
      const instruction = delegateTransferQueueIx(queue, payer, mockPublicKey);

      expect(instruction.keys).toHaveLength(9);
      expect(Array.from(instruction.data)).toEqual([19]);
    });
  });

  describe("delegateEataPermissionIx (Ephemeral SPL Token Program)", () => {
    it("should serialize only the discriminator", () => {
      const instruction = delegateEataPermissionIx(
        mockPublicKey,
        differentKey,
        mockPublicKey,
      );

      expect(Array.from(instruction.data)).toEqual([7]);
    });
  });

  describe("initEphemeralAtaIx (Ephemeral SPL Token Program)", () => {
    it("should serialize only the discriminator", () => {
      const instruction = initEphemeralAtaIx(
        mockPublicKey,
        differentKey,
        mockPublicKey,
        differentKey,
      );

      expect(Array.from(instruction.data)).toEqual([0]);
    });
  });

  describe("initVaultIx (Ephemeral SPL Token Program)", () => {
    it("should serialize only the discriminator", () => {
      const vault = new PublicKey("11111111111111111111111111111113");
      const mint = new PublicKey("11111111111111111111111111111114");
      const payer = new PublicKey("11111111111111111111111111111115");
      const instruction = initVaultIx(vault, mint, payer);

      expect(Array.from(instruction.data)).toEqual([1]);
      expect(instruction.keys[3].pubkey.toBase58()).toBe(
        deriveEphemeralAta(vault, mint)[0].toBase58(),
      );
    });
  });

  describe("withdrawSplIx (Ephemeral SPL Token Program)", () => {
    it("should encode only discriminator plus amount", () => {
      const owner = new PublicKey("11111111111111111111111111111113");
      const mint = new PublicKey("11111111111111111111111111111114");
      const instruction = withdrawSplIx(owner, mint, 1n);

      expect(instruction.data).toHaveLength(9);
      expect(instruction.data[0]).toBe(3);
      expect(Buffer.from(instruction.data).readBigUInt64LE(1)).toBe(1n);
    });
  });
});
