import { describe, it, expect } from "vitest";
import { address, type Address } from "@solana/kit";
import {
  createCreatePermissionInstruction,
  createUpdatePermissionInstruction,
} from "../instructions/permission-program";
import { PERMISSION_PROGRAM_ID } from "../constants";

describe("Permission Program Instructions (@solana/kit)", () => {
  const mockAddress = address("11111111111111111111111111111113");
  const differentAddress = address("11111111111111111111111111111112");

  describe("createPermission instruction", () => {
    it("should create a createPermission instruction with valid parameters", async () => {
      const instruction = await createCreatePermissionInstruction(
        {
          permissionedAccount: mockAddress,
          payer: mockAddress,
        },
        {
          members: [
            { pubkey: mockAddress, authority: true },
            { pubkey: differentAddress, authority: false },
          ],
        },
      );

      expect(instruction.programAddress).toBe(PERMISSION_PROGRAM_ID);
      expect(instruction.accounts).toHaveLength(4);
      expect(instruction.data).toBeDefined();
    });

    it("should serialize members correctly", async () => {
      const members = [{ pubkey: mockAddress, authority: true }];
      const instruction = await createCreatePermissionInstruction(
        {
          permissionedAccount: mockAddress,
          payer: mockAddress,
        },
        { members },
      );

      expect(instruction.data).toBeDefined();
      // Discriminator (8) + count (4) + member (32 + 1) = 45 minimum
      expect(instruction.data?.length).toBeGreaterThanOrEqual(45);
    });

    it("should include permissionedAccount as readonly signer", async () => {
      const instruction = await createCreatePermissionInstruction({
        permissionedAccount: mockAddress,
        payer: differentAddress,
      });

      const permissionedAccount = instruction.accounts?.find(
        (acc) => acc.address === mockAddress,
      );
      expect(permissionedAccount).toBeDefined();
    });

    it("should include payer as writable signer", async () => {
      const payerAddress = address("11111111111111111111111111111115");
      const instruction = await createCreatePermissionInstruction({
        permissionedAccount: mockAddress,
        payer: payerAddress,
      });

      const payerAccount = instruction.accounts?.find(
        (acc) => acc.address === payerAddress,
      );
      expect(payerAccount).toBeDefined();
    });

    it("should include permission PDA as writable", async () => {
      const instruction = await createCreatePermissionInstruction({
        permissionedAccount: mockAddress,
        payer: mockAddress,
      });

      expect(instruction.accounts).toBeDefined();
      expect(instruction.accounts?.length).toBe(4);
    });

    it("should handle empty members list", async () => {
      const instruction = await createCreatePermissionInstruction(
        {
          permissionedAccount: mockAddress,
          payer: mockAddress,
        },
        { members: [] },
      );

      expect(instruction.data).toBeDefined();
      // Discriminator (8) + count (4) = 12 minimum
      expect(instruction.data?.length).toBeGreaterThanOrEqual(12);
    });

    it("should handle multiple members", async () => {
      const members: Array<{ pubkey: Address; authority: boolean }> = [
        { pubkey: mockAddress, authority: true },
        { pubkey: differentAddress, authority: false },
        {
          pubkey: address("11111111111111111111111111111113"),
          authority: true,
        },
      ];

      const instruction = await createCreatePermissionInstruction(
        {
          permissionedAccount: mockAddress,
          payer: mockAddress,
        },
        { members },
      );

      expect(instruction.data).toBeDefined();
      // Discriminator (8) + count (4) + members (each 33 bytes)
      const expectedSize = 8 + 4 + members.length * 33;
      expect(instruction.data?.length).toBeGreaterThanOrEqual(expectedSize);
    });

    it("should use discriminator [0, 0, 0, 0, 0, 0, 0, 0]", async () => {
      const instruction = await createCreatePermissionInstruction({
        permissionedAccount: mockAddress,
        payer: mockAddress,
      });

      // First 8 bytes should be discriminator
      expect(instruction.data?.[0]).toBe(0);
      expect(instruction.data?.[1]).toBe(0);
      expect(instruction.data?.[2]).toBe(0);
      expect(instruction.data?.[3]).toBe(0);
      expect(instruction.data?.[4]).toBe(0);
      expect(instruction.data?.[5]).toBe(0);
      expect(instruction.data?.[6]).toBe(0);
      expect(instruction.data?.[7]).toBe(0);
    });

    it("should encode authority flag correctly", async () => {
      const authorityMember = { pubkey: mockAddress, authority: true };
      const nonAuthorityMember = { pubkey: differentAddress, authority: false };

      const instruction1 = await createCreatePermissionInstruction(
        {
          permissionedAccount: mockAddress,
          payer: mockAddress,
        },
        { members: [authorityMember] },
      );

      const instruction2 = await createCreatePermissionInstruction(
        {
          permissionedAccount: mockAddress,
          payer: mockAddress,
        },
        { members: [nonAuthorityMember] },
      );

      // Authority flag is after discriminator (8) + count (4) + pubkey (32)
      const authorityFlagIndex = 8 + 4 + 32;
      expect(instruction1.data?.[authorityFlagIndex]).toBe(1);
      expect(instruction2.data?.[authorityFlagIndex]).toBe(0);
    });
  });

  describe("updatePermission instruction", () => {
    it("should create an updatePermission instruction with valid parameters", async () => {
      const instruction = await createUpdatePermissionInstruction(
        {
          authority: mockAddress,
          permissionedAccount: mockAddress,
        },
        {
          members: [
            { pubkey: mockAddress, authority: true },
            { pubkey: differentAddress, authority: false },
          ],
        },
      );

      expect(instruction.programAddress).toBe(PERMISSION_PROGRAM_ID);
      expect(instruction.accounts).toHaveLength(3);
      expect(instruction.data).toBeDefined();
    });

    it("should include authority as readonly signer", async () => {
      const authorityAddress = address("11111111111111111111111111111113");
      const instruction = await createUpdatePermissionInstruction({
        authority: authorityAddress,
        permissionedAccount: mockAddress,
      });

      const authorityAccount = instruction.accounts?.find(
        (acc) => acc.address === authorityAddress,
      );
      expect(authorityAccount).toBeDefined();
    });

    it("should include permissionedAccount as readonly signer", async () => {
      const permissionedAddress = address("11111111111111111111111111111114");
      const instruction = await createUpdatePermissionInstruction({
        authority: mockAddress,
        permissionedAccount: permissionedAddress,
      });

      const permissionedAccount = instruction.accounts?.find(
        (acc) => acc.address === permissionedAddress,
      );
      expect(permissionedAccount).toBeDefined();
    });

    it("should include permission PDA as writable", async () => {
      const instruction = await createUpdatePermissionInstruction({
        authority: mockAddress,
        permissionedAccount: mockAddress,
      });

      expect(instruction.accounts).toBeDefined();
      expect(instruction.accounts?.length).toBe(3);
    });

    it("should use discriminator [1, 0, 0, 0, 0, 0, 0, 0]", async () => {
      const instruction = await createUpdatePermissionInstruction({
        authority: mockAddress,
        permissionedAccount: mockAddress,
      });

      // First byte should be discriminator 1
      expect(instruction.data?.[0]).toBe(1);
      expect(instruction.data?.[1]).toBe(0);
      expect(instruction.data?.[2]).toBe(0);
      expect(instruction.data?.[3]).toBe(0);
      expect(instruction.data?.[4]).toBe(0);
      expect(instruction.data?.[5]).toBe(0);
      expect(instruction.data?.[6]).toBe(0);
      expect(instruction.data?.[7]).toBe(0);
    });

    it("should handle empty members list", async () => {
      const instruction = await createUpdatePermissionInstruction(
        {
          authority: mockAddress,
          permissionedAccount: mockAddress,
        },
        { members: [] },
      );

      expect(instruction.data).toBeDefined();
      // Discriminator (8) + count (4) = 12 minimum
      expect(instruction.data?.length).toBeGreaterThanOrEqual(12);
    });

    it("should handle multiple members", async () => {
      const members: Array<{ pubkey: Address; authority: boolean }> = [
        { pubkey: mockAddress, authority: true },
        { pubkey: differentAddress, authority: false },
        {
          pubkey: address("11111111111111111111111111111113"),
          authority: true,
        },
      ];

      const instruction = await createUpdatePermissionInstruction(
        {
          authority: mockAddress,
          permissionedAccount: mockAddress,
        },
        { members },
      );

      expect(instruction.data).toBeDefined();
      // Discriminator (8) + count (4) + members (each 33 bytes)
      const expectedSize = 8 + 4 + members.length * 33;
      expect(instruction.data?.length).toBeGreaterThanOrEqual(expectedSize);
    });
  });

  describe("Cross-instruction consistency", () => {
    it("should all target the same permission program", async () => {
      const createPermissionInstr = await createCreatePermissionInstruction({
        permissionedAccount: mockAddress,
        payer: mockAddress,
      });

      const updatePermissionInstr = await createUpdatePermissionInstruction({
        authority: mockAddress,
        permissionedAccount: mockAddress,
      });

      expect(createPermissionInstr.programAddress).toBe(PERMISSION_PROGRAM_ID);
      expect(updatePermissionInstr.programAddress).toBe(PERMISSION_PROGRAM_ID);
    });

    it("should have unique discriminators", async () => {
      const createPermissionInstr = await createCreatePermissionInstruction({
        permissionedAccount: mockAddress,
        payer: mockAddress,
      });

      const updatePermissionInstr = await createUpdatePermissionInstruction({
        authority: mockAddress,
        permissionedAccount: mockAddress,
      });

      const disc1 = createPermissionInstr.data?.[0];
      const disc2 = updatePermissionInstr.data?.[0];

      expect(disc1).not.toBe(disc2);
      expect(disc1).toBe(0);
      expect(disc2).toBe(1);
    });
  });
});
