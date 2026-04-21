import { describe, it, expect } from "vitest";
import { AccountRole } from "@solana/instructions";
import { SYSTEM_PROGRAM_ADDRESS } from "@solana-program/system";
import bs58 from "bs58";
import * as nacl from "tweetnacl";
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
  address,
  getAddressEncoder,
  getProgramDerivedAddress,
  type Address,
} from "@solana/kit";
import {
  allocateTransferQueueIx,
  delegateEataPermissionIx,
  depositAndQueueTransferIx,
  delegateSpl,
  delegateSplWithPrivateTransfer,
  delegateTransferQueueIx,
  deriveEphemeralAta,
  deriveLamportsPda,
  deriveTransferQueue,
  deriveRentPda,
  deriveVault,
  ensureTransferQueueCrankIx,
  initEphemeralAtaIx,
  initTransferQueueIx,
  initVaultIx,
  initRentPdaIx,
  lamportsDelegatedTransferIx,
  processPendingTransferQueueRefillIx,
  transferSpl,
  undelegateAndCloseShuttleEphemeralAtaIx,
  withdrawSplIx,
  withdrawSpl,
} from "../instructions/ephemeral-spl-token-program";
import {
  DELEGATION_PROGRAM_ID,
  MAGIC_PROGRAM_ID,
  MAGIC_CONTEXT_ID,
  EPHEMERAL_SPL_TOKEN_PROGRAM_ID,
  PERMISSION_PROGRAM_ID,
} from "../constants";
import {
  delegateBufferPdaFromDelegatedAccountAndOwnerProgram,
  delegationMetadataPdaFromDelegatedAccount,
  delegationRecordPdaFromDelegatedAccount,
  permissionPdaFromAccount,
} from "../pda";

function readLengthPrefixedField(
  data: Uint8Array,
  offset: number,
): [Buffer, number] {
  const len = data[offset];
  const start = offset + 1;
  const end = start + len;
  return [Buffer.from(data.subarray(start, end)), end];
}

describe("Exposed Instructions (@solana/kit)", () => {
  const mockAddress = "11111111111111111111111111111111" as Address;
  const differentAddress = "11111111111111111111111111111112" as Address;
  const addressEncoder = getAddressEncoder();

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

  describe("initRentPdaIx (Ephemeral SPL Token Program)", () => {
    it("should derive and initialize the global rent PDA", async () => {
      const [rentPda] = await deriveRentPda();
      const instruction = initRentPdaIx(mockAddress, rentPda);

      expect(instruction.programAddress).toBe(EPHEMERAL_SPL_TOKEN_PROGRAM_ID);
      expect(instruction.accounts).toHaveLength(3);
      expect(instruction.accounts?.[0]).toEqual({
        address: mockAddress,
        role: AccountRole.WRITABLE_SIGNER,
      });
      expect(instruction.accounts?.[1]).toEqual({
        address: rentPda,
        role: AccountRole.WRITABLE,
      });
      expect(instruction.data).toEqual(new Uint8Array([23]));
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

    it("should include accounts to commit and undelegate as writable", () => {
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
      expect(instruction.accounts?.[2].role).toBe(AccountRole.WRITABLE);
      expect(instruction.accounts?.[3].address).toBe(
        accountsToCommitAndUndelegate[1],
      );
      expect(instruction.accounts?.[3].role).toBe(AccountRole.WRITABLE);
    });

    it("should mark every delegated PDA writable across multiple accounts", () => {
      const delegatedAccounts: Address[] = [
        "22222222222222222222222222222222" as Address,
        "33333333333333333333333333333333" as Address,
        "44444444444444444444444444444444" as Address,
      ];
      const instruction = createCommitAndUndelegateInstruction(
        mockAddress,
        delegatedAccounts,
      );

      expect(instruction.accounts).toHaveLength(5);
      delegatedAccounts.forEach((_, index) => {
        expect(instruction.accounts?.[2 + index].role).toBe(
          AccountRole.WRITABLE,
        );
      });
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

  describe("delegateSpl (Ephemeral SPL Token Program)", () => {
    const owner = address("11111111111111111111111111111113");
    const mint = address("11111111111111111111111111111114");
    const validator = address("11111111111111111111111111111115");

    it("should delegate the vault eata when initializing the vault in legacy flow", async () => {
      const [vault] = await deriveVault(mint);
      const [vaultEphemeralAta] = await deriveEphemeralAta(vault, mint);

      const instructions = await delegateSpl(owner, mint, 1n, {
        validator,
        initIfMissing: true,
        initVaultIfMissing: true,
        idempotent: false,
      });

      expect(instructions[3].accounts?.[1].address).toBe(vaultEphemeralAta);
      expect(instructions[3].data?.[0]).toBe(4);
      expect(Array.from(instructions[3].data?.subarray(1) ?? [])).toEqual(
        Array.from(addressEncoder.encode(validator)),
      );
    });

    it("should delegate the vault eata when initializing the vault in idempotent flow", async () => {
      const [vault] = await deriveVault(mint);
      const [vaultEphemeralAta] = await deriveEphemeralAta(vault, mint);

      const instructions = await delegateSpl(owner, mint, 1n, {
        validator,
        initVaultIfMissing: true,
        shuttleId: 7,
      });

      expect(instructions[2].accounts?.[1].address).toBe(vaultEphemeralAta);
      expect(instructions[2].data?.[0]).toBe(4);
      expect(Array.from(instructions[2].data?.subarray(1) ?? [])).toEqual(
        Array.from(addressEncoder.encode(validator)),
      );
    });

    it("should use setup_and_delegate_shuttle_with_merge across the idempotent shuttle flow", async () => {
      const instructions = await delegateSpl(owner, mint, 1n, {
        validator,
        shuttleId: 7,
      });

      const setupAndDelegateInstruction = instructions.find(
        (ix) => ix.data?.[0] === 24,
      );

      expect(setupAndDelegateInstruction).toBeDefined();
      expect(setupAndDelegateInstruction?.accounts).toHaveLength(19);
      expect(instructions.find((ix) => ix.data?.[0] === 11)).toBeUndefined();

      const setupAndDelegateData = setupAndDelegateInstruction?.data;
      expect(setupAndDelegateData).toBeDefined();

      if (setupAndDelegateData === undefined) {
        throw new Error("Expected setup-and-delegate instruction data");
      }

      expect(
        new DataView(
          setupAndDelegateData.buffer,
          setupAndDelegateData.byteOffset,
          setupAndDelegateData.byteLength,
        ).getBigUint64(5, true),
      ).toBe(1n);
      expect(Array.from(setupAndDelegateData.subarray(13))).toEqual(
        Array.from(addressEncoder.encode(validator)),
      );
    });

    it("should skip ephemeral ATA init in idempotent flow when initIfMissing is false", async () => {
      const [ephemeralAta] = await deriveEphemeralAta(owner, mint);
      const instructions = await delegateSpl(owner, mint, 1n, {
        validator,
        shuttleId: 7,
        initIfMissing: false,
      });

      const initInstruction = instructions.find(
        (ix) => ix.data?.[0] === 0 && ix.accounts?.[0].address === ephemeralAta,
      );
      const delegateInstruction = instructions.find(
        (ix) => ix.data?.[0] === 4 && ix.accounts?.[1].address === ephemeralAta,
      );

      expect(initInstruction).toBeUndefined();
      expect(delegateInstruction).toBeDefined();
    });
  });

  describe("delegateSplWithPrivateTransfer (Ephemeral SPL Token Program)", () => {
    const owner = address("11111111111111111111111111111113");
    const mint = address("11111111111111111111111111111114");
    const validator = address(bs58.encode(nacl.sign.keyPair().publicKey));

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
        (ix) => ix.data?.[0] === 25,
      );

      expect(instructions.find((ix) => ix.data?.[0] === 12)).toBeDefined();
      expect(privateTransferInstruction).toBeDefined();
      expect(privateTransferInstruction?.accounts).toHaveLength(19);

      const privateTransferData = privateTransferInstruction?.data;
      expect(privateTransferData).toBeDefined();

      if (privateTransferData === undefined) {
        throw new Error("Expected private transfer instruction data");
      }

      const data = Buffer.from(privateTransferData);
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

      expect(
        validatorField.equals(Buffer.from(addressEncoder.encode(validator))),
      ).toBe(true);
      expect(destinationField).toHaveLength(80);
      expect(suffixField).toHaveLength(68);
      expect(endOffset).toBe(data.length);
    });
  });

  describe("withdrawSpl (Ephemeral SPL Token Program)", () => {
    const owner = address("11111111111111111111111111111113");
    const mint = address("11111111111111111111111111111114");
    const validator = address("11111111111111111111111111111115");

    it("should use the delegated shuttle withdrawal flow when idempotent", async () => {
      const instructions = await withdrawSpl(owner, mint, 1n, {
        validator,
        shuttleId: 7,
      });

      const withdrawInstruction = instructions.find(
        (ix) => ix.data?.[0] === 26,
      );

      expect(withdrawInstruction).toBeDefined();
      expect(withdrawInstruction?.accounts).toHaveLength(16);
      expect(instructions.find((ix) => ix.data?.[0] === 3)).toBeUndefined();
    });

    it("should fall back to the legacy withdraw instruction when idempotent is false", async () => {
      const instructions = await withdrawSpl(owner, mint, 1n, {
        idempotent: false,
      });

      expect(instructions).toHaveLength(1);
      expect(instructions[0].data?.[0]).toBe(3);
    });
  });

  describe("lamportsDelegatedTransferIx (Ephemeral SPL Token Program)", () => {
    const payer = address(bs58.encode(nacl.sign.keyPair().publicKey));
    const destination = address(bs58.encode(nacl.sign.keyPair().publicKey));
    const salt = new Uint8Array(Array.from({ length: 32 }, (_, i) => i));

    it("should derive the lamports PDA and encode the sponsored delegated transfer instruction", async () => {
      const [rentPda] = await deriveRentPda();
      const [lamportsPda] = await deriveLamportsPda(payer, destination, salt);
      const destinationDelegationRecord =
        await delegationRecordPdaFromDelegatedAccount(destination);

      const instruction = await lamportsDelegatedTransferIx(
        payer,
        destination,
        25n,
        salt,
      );

      expect(instruction.programAddress).toBe(EPHEMERAL_SPL_TOKEN_PROGRAM_ID);
      expect(instruction.accounts).toHaveLength(11);
      expect(instruction.accounts?.[0]).toEqual({
        address: payer,
        role: AccountRole.WRITABLE_SIGNER,
      });
      expect(instruction.accounts?.[1]).toEqual({
        address: rentPda,
        role: AccountRole.WRITABLE,
      });
      expect(instruction.accounts?.[2]).toEqual({
        address: lamportsPda,
        role: AccountRole.WRITABLE,
      });
      expect(instruction.accounts?.[9]).toEqual({
        address: destination,
        role: AccountRole.WRITABLE,
      });
      expect(instruction.accounts?.[10]).toEqual({
        address: destinationDelegationRecord,
        role: AccountRole.READONLY,
      });

      const data = Buffer.from(instruction.data ?? []);
      expect(data[0]).toBe(20);
      expect(data.readBigUInt64LE(1)).toBe(25n);
      expect(Buffer.from(data.subarray(9, 41)).equals(Buffer.from(salt))).toBe(
        true,
      );
      expect(data).toHaveLength(41);
    });
  });

  describe("transferSpl (Ephemeral SPL Token Program)", () => {
    const from = address("11111111111111111111111111111113");
    const to = address("11111111111111111111111111111114");
    const mint = address("11111111111111111111111111111115");
    const validator = address(bs58.encode(nacl.sign.keyPair().publicKey));

    it("should use the shuttle private transfer instruction for private base-to-base transfers", async () => {
      const [queue] = await deriveTransferQueue(mint, validator);
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

      expect(instructions).toHaveLength(2);
      expect(instructions[0].data?.[0]).toBe(28);
      expect(instructions[0].accounts?.[1].address).toBe(queue);
      const data = Buffer.from(instructions[1].data ?? []);
      expect(data[0]).toBe(25);
      expect(instructions[1].accounts).toHaveLength(19);
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

      expect(
        validatorField.equals(Buffer.from(addressEncoder.encode(validator))),
      ).toBe(true);
      expect(destinationField).toHaveLength(80);
      expect(suffixField).toHaveLength(68);
      expect(endOffset).toBe(data.length);
    });

    it("should append clientRefId to the encrypted private transfer suffix when provided", async () => {
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
          clientRefId: 42n,
        },
      });

      const data = Buffer.from(instructions[1].data ?? []);
      const [, nextOffset] = readLengthPrefixedField(data, 13);
      const [, suffixOffset] = readLengthPrefixedField(data, nextOffset);
      const [suffixField] = readLengthPrefixedField(data, suffixOffset);

      expect(suffixField).toHaveLength(76);
    });

    it("should initialize the destination ATA and vault when requested", async () => {
      const [vault] = await deriveVault(mint);
      const [vaultEphemeralAta] = await deriveEphemeralAta(vault, mint);

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

      expect(instructions).toHaveLength(5);
      expect(instructions[2].accounts?.[1].address).toBe(vaultEphemeralAta);
      expect(instructions[2].data?.[0]).toBe(4);
      expect(instructions[3].data?.[0]).toBe(28);
      expect(instructions[4].data?.[0]).toBe(25);
    });

    it("should prepend source ATA creation when initAtasIfMissing is set on base-source transfers", async () => {
      const instructions = await transferSpl(from, to, mint, 25n, {
        visibility: "public",
        fromBalance: "base",
        toBalance: "base",
        initAtasIfMissing: true,
      });

      expect(instructions).toHaveLength(2);
      expect(instructions[0].data?.[0]).toBe(1);
      expect(instructions[0].accounts?.[2].address).toBe(from);
      expect(instructions[1].data?.[0]).toBe(3);
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
      expect(instructions[0].data?.[0]).toBe(24);
      expect(instructions[0].accounts).toHaveLength(19);
      expect(Buffer.from(instructions[0].data ?? []).readBigUInt64LE(5)).toBe(
        25n,
      );
    });

    it("should initialize and delegate the receiver eata for private base-to-ephemeral transfers when requested", async () => {
      const [toEphemeralAta] = await deriveEphemeralAta(to, mint);

      const instructions = await transferSpl(from, to, mint, 25n, {
        visibility: "private",
        fromBalance: "base",
        toBalance: "ephemeral",
        validator,
        shuttleId: 7,
        initIfMissing: true,
      });

      expect(instructions).toHaveLength(4);
      expect(instructions[0].data?.[0]).toBe(1);
      expect(instructions[0].accounts?.[2].address).toBe(to);
      expect(instructions[1].data?.[0]).toBe(0);
      expect(instructions[1].accounts?.[0].address).toBe(toEphemeralAta);
      expect(instructions[2].data?.[0]).toBe(4);
      expect(instructions[2].accounts?.[1].address).toBe(toEphemeralAta);
      expect(instructions[3].data?.[0]).toBe(24);
    });

    it("should ignore initAtasIfMissing on ephemeral-source transfers", async () => {
      const instructions = await transferSpl(from, to, mint, 25n, {
        visibility: "private",
        fromBalance: "ephemeral",
        toBalance: "base",
        validator,
        initAtasIfMissing: true,
        privateTransfer: {
          minDelayMs: 100n,
          maxDelayMs: 300n,
          split: 4,
        },
      });

      expect(instructions).toHaveLength(1);
      expect(instructions[0].data?.[0]).toBe(16);
    });

    it("should use depositAndQueueTransferIx for private ephemeral-to-base transfers", async () => {
      const instructions = await transferSpl(from, to, mint, 25n, {
        visibility: "private",
        fromBalance: "ephemeral",
        toBalance: "base",
        validator,
        initIfMissing: true,
        initVaultIfMissing: true,
        privateTransfer: {
          minDelayMs: 100n,
          maxDelayMs: 300n,
          split: 4,
        },
      });

      expect(instructions).toHaveLength(1);
      expect(instructions[0].data?.[0]).toBe(16);
      expect(instructions[0].accounts).toHaveLength(9);
      expect(instructions[0].accounts?.[5].address).toBe(to);
      expect(instructions[0].accounts?.[8].address).toBe(
        instructions[0].accounts?.[3].address,
      );
      expect(Buffer.from(instructions[0].data ?? []).readBigUInt64LE(1)).toBe(
        25n,
      );
      expect(Buffer.from(instructions[0].data ?? []).readBigUInt64LE(9)).toBe(
        100n,
      );
      expect(Buffer.from(instructions[0].data ?? []).readBigUInt64LE(17)).toBe(
        300n,
      );
      expect(Buffer.from(instructions[0].data ?? []).readUInt32LE(25)).toBe(4);
    });

    it("should require validator for private ephemeral-to-base transfers", async () => {
      await expect(
        transferSpl(from, to, mint, 25n, {
          visibility: "private",
          fromBalance: "ephemeral",
          toBalance: "base",
        }),
      ).rejects.toThrow(
        "validator is required for private ephemeral-to-base transfers",
      );
    });

    it("should reject private base-to-base transfers when maxDelayMs is less than minDelayMs", async () => {
      await expect(
        transferSpl(from, to, mint, 25n, {
          visibility: "private",
          fromBalance: "base",
          toBalance: "base",
          shuttleId: 7,
          privateTransfer: {
            minDelayMs: 300n,
            maxDelayMs: 100n,
            split: 4,
          },
        }),
      ).rejects.toThrow(
        "maxDelayMs must be greater than or equal to minDelayMs",
      );
    });

    it("should use a normal transfer for public base-to-base transfers", async () => {
      const instructions = await transferSpl(from, to, mint, 25n, {
        visibility: "public",
        fromBalance: "base",
        toBalance: "base",
      });

      expect(instructions).toHaveLength(1);
      expect(instructions[0].data?.[0]).toBe(3);
      expect(instructions[0].accounts).toHaveLength(3);
      expect(Buffer.from(instructions[0].data ?? []).readBigUInt64LE(1)).toBe(
        25n,
      );
    });

    it("should not prepend refill for public base-to-base transfers even when validator is provided", async () => {
      const instructions = await transferSpl(from, to, mint, 25n, {
        visibility: "public",
        fromBalance: "base",
        toBalance: "base",
        validator,
      });

      expect(instructions).toHaveLength(1);
      expect(instructions[0].data?.[0]).toBe(3);
      expect(instructions[0].accounts).toHaveLength(3);
    });

    it("should use a normal transfer for public ephemeral-to-ephemeral transfers", async () => {
      const instructions = await transferSpl(from, to, mint, 25n, {
        visibility: "public",
        fromBalance: "ephemeral",
        toBalance: "ephemeral",
      });

      expect(instructions).toHaveLength(1);
      expect(instructions[0].data?.[0]).toBe(3);
      expect(instructions[0].accounts).toHaveLength(3);
      expect(Buffer.from(instructions[0].data ?? []).readBigUInt64LE(1)).toBe(
        25n,
      );
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
      expect(instructions[0].data?.[0]).toBe(3);
      expect(instructions[0].accounts).toHaveLength(3);
      expect(Buffer.from(instructions[0].data ?? []).readBigUInt64LE(1)).toBe(
        25n,
      );
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

  describe("initVaultIx (Ephemeral SPL Token Program)", () => {
    it("should use the provided vault ephemeral ATA synchronously", async () => {
      const mint = address("11111111111111111111111111111114");
      const payer = address("11111111111111111111111111111115");
      const [vault] = await deriveVault(mint);
      const [vaultEphemeralAta] = await deriveEphemeralAta(vault, mint);
      const vaultAta = address("11111111111111111111111111111116");

      const instruction = initVaultIx(
        vault,
        mint,
        payer,
        vaultEphemeralAta,
        vaultAta,
      );

      expect(instruction.accounts?.[0].address).toBe(vault);
      expect(instruction.accounts?.[3].address).toBe(vaultEphemeralAta);
      expect(instruction.accounts?.[4].address).toBe(vaultAta);
      expect(Array.from(instruction.data ?? [])).toEqual([1]);
    });
  });

  describe("ensureTransferQueueCrankIx (Ephemeral SPL Token Program)", () => {
    const payer = mockAddress;
    const queue = differentAddress;
    const magicFeeVault = address("11111111111111111111111111111113");

    it("should include queue, magic fee vault, magic context, and magic program in order", () => {
      const instruction = ensureTransferQueueCrankIx(
        payer,
        queue,
        magicFeeVault,
      );

      expect(instruction.accounts).toHaveLength(5);
      expect(instruction.accounts?.[0].address).toBe(payer);
      expect(instruction.accounts?.[1].address).toBe(queue);
      expect(instruction.accounts?.[2].address).toBe(magicFeeVault);
      expect(instruction.accounts?.[3].address).toBe(MAGIC_CONTEXT_ID);
      expect(instruction.accounts?.[4].address).toBe(MAGIC_PROGRAM_ID);
    });
  });

  describe("depositAndQueueTransferIx (Ephemeral SPL Token Program)", () => {
    const queue = differentAddress;
    const vault = address("11111111111111111111111111111113");
    const mint = address("11111111111111111111111111111114");
    const source = address("11111111111111111111111111111115");
    const vaultAta = address("11111111111111111111111111111116");
    const destination = address("11111111111111111111111111111117");

    it("should serialize min/max delay ms and split", () => {
      const instruction = depositAndQueueTransferIx(
        queue,
        vault,
        mint,
        source,
        vaultAta,
        destination,
        mockAddress,
        25n,
        100n,
        300n,
        4,
      );

      expect(instruction.accounts).toHaveLength(9);
      expect(instruction.accounts?.[8].address).toBe(source);
      expect(instruction.accounts?.[8].role).toBe(AccountRole.WRITABLE);
      expect(Array.from(instruction.data ?? [])).toEqual([
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

    it("should allow overriding the reimbursement token account", () => {
      const reimbursementTokenInfo = address(
        "11111111111111111111111111111118",
      );
      const instruction = depositAndQueueTransferIx(
        queue,
        vault,
        mint,
        source,
        vaultAta,
        destination,
        mockAddress,
        25n,
        100n,
        300n,
        4,
        reimbursementTokenInfo,
      );

      expect(instruction.accounts?.[8].address).toBe(reimbursementTokenInfo);
    });

    it("should append clientRefId when provided", () => {
      const instruction = depositAndQueueTransferIx(
        queue,
        vault,
        mint,
        source,
        vaultAta,
        destination,
        mockAddress,
        25n,
        100n,
        300n,
        4,
        source,
        42n,
      );

      expect(Array.from(instruction.data ?? [])).toEqual([
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
        ...Array.from(
          (() => {
            const out = Buffer.alloc(8);
            out.writeBigUInt64LE(42n);
            return out;
          })(),
        ),
      ]);
    });
  });

  describe("undelegateAndCloseShuttleEphemeralAtaIx (Ephemeral SPL Token Program)", () => {
    it("should include rent reimbursement and destination ATA accounts", () => {
      const rentReimbursement = address("11111111111111111111111111111113");
      const shuttleEphemeralAta = address("11111111111111111111111111111114");
      const shuttleAta = address("11111111111111111111111111111115");
      const shuttleWalletAta = address("11111111111111111111111111111116");
      const destinationAta = address("11111111111111111111111111111117");
      const instruction = undelegateAndCloseShuttleEphemeralAtaIx(
        mockAddress,
        rentReimbursement,
        shuttleEphemeralAta,
        shuttleAta,
        shuttleWalletAta,
        destinationAta,
        3,
      );

      expect(instruction.accounts).toHaveLength(9);
      expect(instruction.accounts?.[1].address).toBe(rentReimbursement);
      expect(instruction.accounts?.[1].role).toBe(AccountRole.WRITABLE);
      expect(instruction.accounts?.[5].address).toBe(destinationAta);
      expect(instruction.accounts?.[5].role).toBe(AccountRole.WRITABLE);
      expect(Array.from(instruction.data ?? [])).toEqual([14, 3]);
    });
  });

  describe("delegateTransferQueueIx (Ephemeral SPL Token Program)", () => {
    const payer = mockAddress;
    const queue = differentAddress;

    it("should serialize discriminator 19 for the delegated transfer queue opcode", async () => {
      const instruction = await delegateTransferQueueIx(
        queue,
        payer,
        mockAddress,
      );

      expect(instruction.accounts).toHaveLength(9);
      expect(instruction.data).toEqual(new Uint8Array([19]));
    });
  });

  describe("transfer queue helpers (Ephemeral SPL Token Program)", () => {
    const mint = address("11111111111111111111111111111113");
    const validator = address("11111111111111111111111111111114");

    it("should derive validator-scoped transfer queue PDAs", async () => {
      const [queueA] = await deriveTransferQueue(mint, validator);
      const [queueB] = await deriveTransferQueue(mint, mockAddress);

      expect(queueA).not.toBe(queueB);
    });

    it("should include validator and requested item count in initTransferQueueIx", async () => {
      const [queue] = await deriveTransferQueue(mint, validator);
      const instruction = await initTransferQueueIx(
        mockAddress,
        queue,
        mint,
        validator,
        92,
      );

      expect(instruction.accounts).toHaveLength(7);
      expect(instruction.accounts?.[2].address).toBe(
        await permissionPdaFromAccount(queue),
      );
      expect(instruction.accounts?.[4].address).toBe(validator);
      expect(instruction.accounts?.[6].address).toBe(PERMISSION_PROGRAM_ID);
      expect(Array.from(instruction.data ?? [])).toEqual([12, 92, 0, 0, 0]);
    });

    it("should serialize discriminator 27 for allocateTransferQueueIx", async () => {
      const [queue] = await deriveTransferQueue(mint, validator);
      const instruction = allocateTransferQueueIx(queue);

      expect(instruction.accounts).toHaveLength(2);
      expect(Array.from(instruction.data ?? [])).toEqual([27]);
    });

    it("should derive the sponsored refill accounts for processPendingTransferQueueRefillIx", async () => {
      const [queue] = await deriveTransferQueue(mint, validator);
      const instruction = await processPendingTransferQueueRefillIx(queue);
      const [rentPda] = await deriveRentPda();
      const [refillState] = await getProgramDerivedAddress({
        programAddress: EPHEMERAL_SPL_TOKEN_PROGRAM_ID,
        seeds: [Buffer.from("queue-refill"), addressEncoder.encode(queue)],
      });
      const [lamportsPda] = await deriveLamportsPda(
        rentPda,
        queue,
        Uint8Array.from(addressEncoder.encode(queue)),
      );

      expect(instruction.accounts).toHaveLength(11);
      expect(instruction.accounts?.[0].address).toBe(refillState);
      expect(instruction.accounts?.[1].address).toBe(queue);
      expect(instruction.accounts?.[2].address).toBe(rentPda);
      expect(instruction.accounts?.[3].address).toBe(lamportsPda);
      expect(instruction.accounts?.[4].address).toBe(
        EPHEMERAL_SPL_TOKEN_PROGRAM_ID,
      );
      expect(instruction.accounts?.[5].address).toBe(
        await delegateBufferPdaFromDelegatedAccountAndOwnerProgram(
          lamportsPda,
          EPHEMERAL_SPL_TOKEN_PROGRAM_ID,
        ),
      );
      expect(instruction.accounts?.[6].address).toBe(
        await delegationRecordPdaFromDelegatedAccount(lamportsPda),
      );
      expect(instruction.accounts?.[7].address).toBe(
        await delegationMetadataPdaFromDelegatedAccount(lamportsPda),
      );
      expect(instruction.accounts?.[8].address).toBe(DELEGATION_PROGRAM_ID);
      expect(instruction.accounts?.[9].address).toBe(SYSTEM_PROGRAM_ADDRESS);
      expect(instruction.accounts?.[10].address).toBe(
        await delegationRecordPdaFromDelegatedAccount(queue),
      );
      expect(Array.from(instruction.data ?? [])).toEqual([28]);
    });
  });

  describe("delegateEataPermissionIx (Ephemeral SPL Token Program)", () => {
    it("should serialize only the discriminator", async () => {
      const instruction = await delegateEataPermissionIx(
        mockAddress,
        differentAddress,
        mockAddress,
      );

      expect(Array.from(instruction.data ?? [])).toEqual([7]);
    });
  });

  describe("initEphemeralAtaIx (Ephemeral SPL Token Program)", () => {
    it("should serialize only the discriminator", () => {
      const instruction = initEphemeralAtaIx(
        mockAddress,
        differentAddress,
        mockAddress,
        differentAddress,
      );

      expect(Array.from(instruction.data ?? [])).toEqual([0]);
    });
  });

  describe("withdrawSplIx (Ephemeral SPL Token Program)", () => {
    it("should encode only discriminator plus amount", async () => {
      const owner = address("11111111111111111111111111111113");
      const mint = address("11111111111111111111111111111114");
      const instruction = await withdrawSplIx(owner, mint, 1n);

      expect(instruction.data).toHaveLength(9);
      expect(instruction.data?.[0]).toBe(3);

      const data = instruction.data;
      expect(data).toBeDefined();

      if (data === undefined) {
        throw new Error("Expected withdraw instruction data");
      }

      expect(
        new DataView(
          data.buffer,
          data.byteOffset,
          data.byteLength,
        ).getBigUint64(1, true),
      ).toBe(1n);
    });
  });
});
