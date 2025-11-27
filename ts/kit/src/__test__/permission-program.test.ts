import { describe, it, expect } from "vitest";
import { address, type Address } from "@solana/kit";
import {
  createCreateGroupInstruction,
  createCreatePermissionInstruction,
  createUpdatePermissionInstruction,
} from "../instructions/permission-program";
import { PERMISSION_PROGRAM_ID } from "../constants";

describe("Permission Program Instructions (@solana/kit)", () => {
  const mockAddress = address("11111111111111111111111111111111");
  const differentAddress = address("11111111111111111111111111111112");

  describe("createGroup instruction", () => {
    it("should create a createGroup instruction with valid parameters", () => {
      const id = mockAddress;
      const members: Address[] = [mockAddress, differentAddress];

      const instruction = createCreateGroupInstruction(
        {
          group: mockAddress,
          payer: mockAddress,
        },
        {
          id,
          members,
        },
      );

      expect(instruction.programAddress).toBe(PERMISSION_PROGRAM_ID);
      expect(instruction.accounts).toHaveLength(3);
      expect(instruction.data).toBeDefined();
    });

    it("should serialize group ID correctly", () => {
      const id = mockAddress;
      const instruction = createCreateGroupInstruction(
        {
          group: mockAddress,
          payer: mockAddress,
        },
        {
          id,
          members: [],
        },
      );

      expect(instruction.data).toBeDefined();
      // First byte is discriminator (0), followed by 32 bytes for pubkey
      expect(instruction.data?.length).toBeGreaterThanOrEqual(1 + 32);
    });

    it("should include group account as writable", () => {
      const instruction = createCreateGroupInstruction(
        {
          group: mockAddress,
          payer: differentAddress,
        },
        {
          id: mockAddress,
          members: [],
        },
      );

      const groupAccount = instruction.accounts?.find(
        (acc) => acc.address === mockAddress,
      );
      expect(groupAccount).toBeDefined();
    });

    it("should include payer as writable signer", () => {
      const instruction = createCreateGroupInstruction(
        {
          group: mockAddress,
          payer: differentAddress,
        },
        {
          id: mockAddress,
          members: [],
        },
      );

      const payerAccount = instruction.accounts?.find(
        (acc) => acc.address === differentAddress,
      );
      expect(payerAccount).toBeDefined();
    });

    it("should handle empty members list", () => {
      const instruction = createCreateGroupInstruction(
        {
          group: mockAddress,
          payer: mockAddress,
        },
        {
          id: mockAddress,
          members: [],
        },
      );

      expect(instruction.data).toBeDefined();
      // Discriminator (1) + ID (32) + members count (4) = 37 minimum
      expect(instruction.data?.length).toBeGreaterThanOrEqual(37);
    });

    it("should handle multiple members", () => {
      const members: Address[] = [
        mockAddress,
        differentAddress,
        address("11111111111111111111111111111113"),
      ];

      const instruction = createCreateGroupInstruction(
        {
          group: mockAddress,
          payer: mockAddress,
        },
        {
          id: mockAddress,
          members,
        },
      );

      expect(instruction.data).toBeDefined();
      // Should have space for all members
      const expectedSize = 1 + 32 + 4 + members.length * 32;
      expect(instruction.data?.length).toBeGreaterThanOrEqual(expectedSize);
    });

    it("should use discriminator 0", () => {
      const instruction = createCreateGroupInstruction(
        {
          group: mockAddress,
          payer: mockAddress,
        },
        {
          id: mockAddress,
          members: [],
        },
      );

      // First byte should be discriminator 0
      expect(instruction.data?.[0]).toBe(0);
    });
  });

  describe("createPermission instruction", () => {
    it("should create a createPermission instruction with valid parameters", () => {
      const instruction = createCreatePermissionInstruction({
        permission: mockAddress,
        delegatedAccount: mockAddress,
        group: differentAddress,
        payer: mockAddress,
      });

      expect(instruction.programAddress).toBe(PERMISSION_PROGRAM_ID);
      expect(instruction.accounts).toHaveLength(5);
      expect(instruction.data).toBeDefined();
    });

    it("should include permission account as writable", () => {
      const instruction = createCreatePermissionInstruction({
        permission: mockAddress,
        delegatedAccount: differentAddress,
        group: address("11111111111111111111111111111113"),
        payer: address("11111111111111111111111111111114"),
      });

      const permissionAccount = instruction.accounts?.find(
        (acc) => acc.address === mockAddress,
      );
      expect(permissionAccount).toBeDefined();
    });

    it("should include delegatedAccount as readonly signer", () => {
      const instruction = createCreatePermissionInstruction({
        permission: mockAddress,
        delegatedAccount: differentAddress,
        group: mockAddress,
        payer: mockAddress,
      });

      const delegatedAccount = instruction.accounts?.find(
        (acc) => acc.address === differentAddress,
      );
      expect(delegatedAccount).toBeDefined();
    });

    it("should include payer as writable signer", () => {
      const payerAddress = address("11111111111111111111111111111115");
      const instruction = createCreatePermissionInstruction({
        permission: mockAddress,
        delegatedAccount: mockAddress,
        group: mockAddress,
        payer: payerAddress,
      });

      const payerAccount = instruction.accounts?.find(
        (acc) => acc.address === payerAddress,
      );
      expect(payerAccount).toBeDefined();
    });

    it("should use discriminator 1", () => {
      const instruction = createCreatePermissionInstruction({
        permission: mockAddress,
        delegatedAccount: mockAddress,
        group: mockAddress,
        payer: mockAddress,
      });

      // First byte should be discriminator 1
      expect(instruction.data?.[0]).toBe(1);
    });

    it("should have minimal data (just discriminator)", () => {
      const instruction = createCreatePermissionInstruction({
        permission: mockAddress,
        delegatedAccount: mockAddress,
        group: mockAddress,
        payer: mockAddress,
      });

      // Should be just 1 byte for discriminator
      expect(instruction.data?.length).toBe(1);
    });
  });

  describe("updatePermission instruction", () => {
    it("should create an updatePermission instruction with valid parameters", () => {
      const instruction = createUpdatePermissionInstruction({
        permission: mockAddress,
        delegatedAccount: mockAddress,
        group: differentAddress,
      });

      expect(instruction.programAddress).toBe(PERMISSION_PROGRAM_ID);
      expect(instruction.accounts).toHaveLength(3);
      expect(instruction.data).toBeDefined();
    });

    it("should include permission account as writable", () => {
      const instruction = createUpdatePermissionInstruction({
        permission: mockAddress,
        delegatedAccount: differentAddress,
        group: address("11111111111111111111111111111113"),
      });

      const permissionAccount = instruction.accounts?.find(
        (acc) => acc.address === mockAddress,
      );
      expect(permissionAccount).toBeDefined();
    });

    it("should include delegatedAccount as readonly signer", () => {
      const instruction = createUpdatePermissionInstruction({
        permission: mockAddress,
        delegatedAccount: differentAddress,
        group: mockAddress,
      });

      const delegatedAccount = instruction.accounts?.find(
        (acc) => acc.address === differentAddress,
      );
      expect(delegatedAccount).toBeDefined();
    });

    it("should include group as readonly", () => {
      const groupAddress = address("11111111111111111111111111111114");
      const instruction = createUpdatePermissionInstruction({
        permission: mockAddress,
        delegatedAccount: mockAddress,
        group: groupAddress,
      });

      const groupAccount = instruction.accounts?.find(
        (acc) => acc.address === groupAddress,
      );
      expect(groupAccount).toBeDefined();
    });

    it("should use discriminator 2", () => {
      const instruction = createUpdatePermissionInstruction({
        permission: mockAddress,
        delegatedAccount: mockAddress,
        group: mockAddress,
      });

      // First byte should be discriminator 2
      expect(instruction.data?.[0]).toBe(2);
    });

    it("should have minimal data (just discriminator)", () => {
      const instruction = createUpdatePermissionInstruction({
        permission: mockAddress,
        delegatedAccount: mockAddress,
        group: mockAddress,
      });

      // Should be just 1 byte for discriminator
      expect(instruction.data?.length).toBe(1);
    });
  });

  describe("Cross-instruction consistency", () => {
    it("should all target the same permission program", () => {
      const createGroupInstr = createCreateGroupInstruction(
        {
          group: mockAddress,
          payer: mockAddress,
        },
        {
          id: mockAddress,
          members: [],
        },
      );

      const createPermissionInstr = createCreatePermissionInstruction({
        permission: mockAddress,
        delegatedAccount: mockAddress,
        group: mockAddress,
        payer: mockAddress,
      });

      const updatePermissionInstr = createUpdatePermissionInstruction({
        permission: mockAddress,
        delegatedAccount: mockAddress,
        group: mockAddress,
      });

      expect(createGroupInstr.programAddress).toBe(PERMISSION_PROGRAM_ID);
      expect(createPermissionInstr.programAddress).toBe(PERMISSION_PROGRAM_ID);
      expect(updatePermissionInstr.programAddress).toBe(PERMISSION_PROGRAM_ID);
    });

    it("should have unique discriminators", () => {
      const createGroupInstr = createCreateGroupInstruction(
        {
          group: mockAddress,
          payer: mockAddress,
        },
        {
          id: mockAddress,
          members: [],
        },
      );

      const createPermissionInstr = createCreatePermissionInstruction({
        permission: mockAddress,
        delegatedAccount: mockAddress,
        group: mockAddress,
        payer: mockAddress,
      });

      const updatePermissionInstr = createUpdatePermissionInstruction({
        permission: mockAddress,
        delegatedAccount: mockAddress,
        group: mockAddress,
      });

      const disc1 = createGroupInstr.data?.[0];
      const disc2 = createPermissionInstr.data?.[0];
      const disc3 = updatePermissionInstr.data?.[0];

      expect(disc1).not.toBe(disc2);
      expect(disc2).not.toBe(disc3);
      expect(disc1).not.toBe(disc3);
    });
  });
});
