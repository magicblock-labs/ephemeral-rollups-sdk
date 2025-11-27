import { describe, it, expect } from "vitest";
import { PublicKey } from "@solana/web3.js";
import {
  createCreateGroupInstruction,
  createCreatePermissionInstruction,
  createUpdatePermissionInstruction,
} from "../instructions/permission-program";
import { PERMISSION_PROGRAM_ID } from "../constants";

describe("Permission Program Instructions (web3.js)", () => {
  const mockPublicKey = new PublicKey("11111111111111111111111111111111");
  const differentPublicKey = new PublicKey("11111111111111111111111111111112");

  describe("createGroup instruction", () => {
    it("should create a createGroup instruction with valid parameters", () => {
      const id = mockPublicKey;
      const members = [mockPublicKey, differentPublicKey];

      const instruction = createCreateGroupInstruction(
        {
          group: mockPublicKey,
          payer: mockPublicKey,
        },
        {
          id,
          members,
        },
      );

      expect(instruction.programId.equals(PERMISSION_PROGRAM_ID)).toBe(true);
      expect(instruction.keys).toHaveLength(3);
      expect(instruction.data).toBeDefined();
      expect(instruction.data.length).toBeGreaterThan(0);
    });

    it("should serialize group ID correctly", () => {
      const id = mockPublicKey;
      const instruction = createCreateGroupInstruction(
        {
          group: mockPublicKey,
          payer: mockPublicKey,
        },
        {
          id,
          members: [],
        },
      );

      expect(instruction.data).toBeDefined();
      // First byte is discriminator (0), followed by 32 bytes for pubkey
      expect(instruction.data.length).toBeGreaterThanOrEqual(1 + 32);
    });

    it("should include group account as writable", () => {
      const instruction = createCreateGroupInstruction(
        {
          group: mockPublicKey,
          payer: differentPublicKey,
        },
        {
          id: mockPublicKey,
          members: [],
        },
      );

      const groupAccount = instruction.keys.find((key) =>
        key.pubkey.equals(mockPublicKey),
      );
      expect(groupAccount).toBeDefined();
      expect(groupAccount?.isWritable).toBe(true);
    });

    it("should include payer as writable signer", () => {
      const instruction = createCreateGroupInstruction(
        {
          group: mockPublicKey,
          payer: differentPublicKey,
        },
        {
          id: mockPublicKey,
          members: [],
        },
      );

      const payerAccount = instruction.keys.find((key) =>
        key.pubkey.equals(differentPublicKey),
      );
      expect(payerAccount).toBeDefined();
      expect(payerAccount?.isWritable).toBe(true);
      expect(payerAccount?.isSigner).toBe(true);
    });

    it("should handle empty members list", () => {
      const instruction = createCreateGroupInstruction(
        {
          group: mockPublicKey,
          payer: mockPublicKey,
        },
        {
          id: mockPublicKey,
          members: [],
        },
      );

      expect(instruction.data).toBeDefined();
      // Discriminator (1) + ID (32) + members count (4) = 37 minimum
      expect(instruction.data.length).toBeGreaterThanOrEqual(37);
    });

    it("should handle multiple members", () => {
      const members = [
        mockPublicKey,
        differentPublicKey,
        new PublicKey("11111111111111111111111111111113"),
      ];

      const instruction = createCreateGroupInstruction(
        {
          group: mockPublicKey,
          payer: mockPublicKey,
        },
        {
          id: mockPublicKey,
          members,
        },
      );

      expect(instruction.data).toBeDefined();
      // Should have space for all members
      const expectedSize = 1 + 32 + 4 + members.length * 32;
      expect(instruction.data.length).toBeGreaterThanOrEqual(expectedSize);
    });

    it("should use discriminator 0", () => {
      const instruction = createCreateGroupInstruction(
        {
          group: mockPublicKey,
          payer: mockPublicKey,
        },
        {
          id: mockPublicKey,
          members: [],
        },
      );

      // First byte should be discriminator 0
      expect(instruction.data[0]).toBe(0);
    });
  });

  describe("createPermission instruction", () => {
    it("should create a createPermission instruction with valid parameters", () => {
      const instruction = createCreatePermissionInstruction({
        permission: mockPublicKey,
        delegatedAccount: mockPublicKey,
        group: differentPublicKey,
        payer: mockPublicKey,
      });

      expect(instruction.programId.equals(PERMISSION_PROGRAM_ID)).toBe(true);
      expect(instruction.keys).toHaveLength(5);
      expect(instruction.data).toBeDefined();
    });

    it("should include permission account as writable", () => {
      const instruction = createCreatePermissionInstruction({
        permission: mockPublicKey,
        delegatedAccount: differentPublicKey,
        group: new PublicKey("11111111111111111111111111111113"),
        payer: new PublicKey("11111111111111111111111111111114"),
      });

      const permissionAccount = instruction.keys.find((key) =>
        key.pubkey.equals(mockPublicKey),
      );
      expect(permissionAccount).toBeDefined();
      expect(permissionAccount?.isWritable).toBe(true);
    });

    it("should include delegatedAccount as readonly signer", () => {
      const instruction = createCreatePermissionInstruction({
        permission: mockPublicKey,
        delegatedAccount: differentPublicKey,
        group: mockPublicKey,
        payer: mockPublicKey,
      });

      const delegatedAccount = instruction.keys.find((key) =>
        key.pubkey.equals(differentPublicKey),
      );
      expect(delegatedAccount).toBeDefined();
      expect(delegatedAccount?.isSigner).toBe(true);
      expect(delegatedAccount?.isWritable).toBe(false);
    });

    it("should include payer as writable signer", () => {
      const payerAddress = new PublicKey("11111111111111111111111111111115");
      const instruction = createCreatePermissionInstruction({
        permission: mockPublicKey,
        delegatedAccount: mockPublicKey,
        group: mockPublicKey,
        payer: payerAddress,
      });

      const payerAccount = instruction.keys.find((key) =>
        key.pubkey.equals(payerAddress),
      );
      expect(payerAccount).toBeDefined();
      expect(payerAccount?.isWritable).toBe(true);
      expect(payerAccount?.isSigner).toBe(true);
    });

    it("should use discriminator 1", () => {
      const instruction = createCreatePermissionInstruction({
        permission: mockPublicKey,
        delegatedAccount: mockPublicKey,
        group: mockPublicKey,
        payer: mockPublicKey,
      });

      // First byte should be discriminator 1
      expect(instruction.data[0]).toBe(1);
    });

    it("should have minimal data (just discriminator)", () => {
      const instruction = createCreatePermissionInstruction({
        permission: mockPublicKey,
        delegatedAccount: mockPublicKey,
        group: mockPublicKey,
        payer: mockPublicKey,
      });

      // Should be just 1 byte for discriminator
      expect(instruction.data.length).toBe(1);
    });
  });

  describe("updatePermission instruction", () => {
    it("should create an updatePermission instruction with valid parameters", () => {
      const instruction = createUpdatePermissionInstruction({
        permission: mockPublicKey,
        delegatedAccount: mockPublicKey,
        group: differentPublicKey,
      });

      expect(instruction.programId.equals(PERMISSION_PROGRAM_ID)).toBe(true);
      expect(instruction.keys).toHaveLength(3);
      expect(instruction.data).toBeDefined();
    });

    it("should include permission account as writable", () => {
      const instruction = createUpdatePermissionInstruction({
        permission: mockPublicKey,
        delegatedAccount: differentPublicKey,
        group: new PublicKey("11111111111111111111111111111113"),
      });

      const permissionAccount = instruction.keys.find((key) =>
        key.pubkey.equals(mockPublicKey),
      );
      expect(permissionAccount).toBeDefined();
      expect(permissionAccount?.isWritable).toBe(true);
    });

    it("should include delegatedAccount as readonly signer", () => {
      const instruction = createUpdatePermissionInstruction({
        permission: mockPublicKey,
        delegatedAccount: differentPublicKey,
        group: mockPublicKey,
      });

      const delegatedAccount = instruction.keys.find((key) =>
        key.pubkey.equals(differentPublicKey),
      );
      expect(delegatedAccount).toBeDefined();
      expect(delegatedAccount?.isSigner).toBe(true);
      expect(delegatedAccount?.isWritable).toBe(false);
    });

    it("should include group as readonly", () => {
      const groupAddress = new PublicKey("11111111111111111111111111111114");
      const instruction = createUpdatePermissionInstruction({
        permission: mockPublicKey,
        delegatedAccount: mockPublicKey,
        group: groupAddress,
      });

      const groupAccount = instruction.keys.find((key) =>
        key.pubkey.equals(groupAddress),
      );
      expect(groupAccount).toBeDefined();
      expect(groupAccount?.isWritable).toBe(false);
      expect(groupAccount?.isSigner).toBe(false);
    });

    it("should use discriminator 2", () => {
      const instruction = createUpdatePermissionInstruction({
        permission: mockPublicKey,
        delegatedAccount: mockPublicKey,
        group: mockPublicKey,
      });

      // First byte should be discriminator 2
      expect(instruction.data[0]).toBe(2);
    });

    it("should have minimal data (just discriminator)", () => {
      const instruction = createUpdatePermissionInstruction({
        permission: mockPublicKey,
        delegatedAccount: mockPublicKey,
        group: mockPublicKey,
      });

      // Should be just 1 byte for discriminator
      expect(instruction.data.length).toBe(1);
    });
  });

  describe("Cross-instruction consistency", () => {
    it("should all target the same permission program", () => {
      const createGroupInstr = createCreateGroupInstruction(
        {
          group: mockPublicKey,
          payer: mockPublicKey,
        },
        {
          id: mockPublicKey,
          members: [],
        },
      );

      const createPermissionInstr = createCreatePermissionInstruction({
        permission: mockPublicKey,
        delegatedAccount: mockPublicKey,
        group: mockPublicKey,
        payer: mockPublicKey,
      });

      const updatePermissionInstr = createUpdatePermissionInstruction({
        permission: mockPublicKey,
        delegatedAccount: mockPublicKey,
        group: mockPublicKey,
      });

      expect(createGroupInstr.programId.equals(PERMISSION_PROGRAM_ID)).toBe(
        true,
      );
      expect(
        createPermissionInstr.programId.equals(PERMISSION_PROGRAM_ID),
      ).toBe(true);
      expect(
        updatePermissionInstr.programId.equals(PERMISSION_PROGRAM_ID),
      ).toBe(true);
    });

    it("should have unique discriminators", () => {
      const createGroupInstr = createCreateGroupInstruction(
        {
          group: mockPublicKey,
          payer: mockPublicKey,
        },
        {
          id: mockPublicKey,
          members: [],
        },
      );

      const createPermissionInstr = createCreatePermissionInstruction({
        permission: mockPublicKey,
        delegatedAccount: mockPublicKey,
        group: mockPublicKey,
        payer: mockPublicKey,
      });

      const updatePermissionInstr = createUpdatePermissionInstruction({
        permission: mockPublicKey,
        delegatedAccount: mockPublicKey,
        group: mockPublicKey,
      });

      const disc1 = createGroupInstr.data[0];
      const disc2 = createPermissionInstr.data[0];
      const disc3 = updatePermissionInstr.data[0];

      expect(disc1).not.toBe(disc2);
      expect(disc2).not.toBe(disc3);
      expect(disc1).not.toBe(disc3);
    });
  });
});
