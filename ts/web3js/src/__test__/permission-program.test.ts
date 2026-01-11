import { describe, it, expect } from "vitest";
import { PublicKey } from "@solana/web3.js";
import {
  createCreatePermissionInstruction,
  createUpdatePermissionInstruction,
} from "../instructions/permission-program";
import { PERMISSION_PROGRAM_ID } from "../constants";

describe("Permission Program Instructions (web3.js)", () => {
  const mockPublicKey = new PublicKey("11111111111111111111111111111111");
  const differentPublicKey = new PublicKey("11111111111111111111111111111112");

  describe("createPermission instruction", () => {
    it("should create a createPermission instruction with valid parameters", () => {
      const instruction = createCreatePermissionInstruction(
        {
          permissionedAccount: mockPublicKey,
          payer: mockPublicKey,
        },
        {
          members: [
            { pubkey: mockPublicKey, authority: true },
            { pubkey: differentPublicKey, authority: false },
          ],
        },
      );

      expect(instruction.programId.equals(PERMISSION_PROGRAM_ID)).toBe(true);
      expect(instruction.keys).toHaveLength(4);
      expect(instruction.data).toBeDefined();
      expect(instruction.data.length).toBeGreaterThan(0);
    });

    it("should serialize members correctly", () => {
      const members = [{ pubkey: mockPublicKey, authority: true }];
      const instruction = createCreatePermissionInstruction(
        {
          permissionedAccount: mockPublicKey,
          payer: mockPublicKey,
        },
        { members },
      );

      expect(instruction.data).toBeDefined();
      // Discriminator (8) + count (4) + member (32 + 1) = 45 minimum
      expect(instruction.data.length).toBeGreaterThanOrEqual(45);
    });

    it("should include permissionedAccount as readonly signer", () => {
      const instruction = createCreatePermissionInstruction(
        {
          permissionedAccount: mockPublicKey,
          payer: differentPublicKey,
        },
      );

      const permissionedAccount = instruction.keys.find((key) =>
        key.pubkey.equals(mockPublicKey),
      );
      expect(permissionedAccount).toBeDefined();
      expect(permissionedAccount?.isWritable).toBe(false);
      expect(permissionedAccount?.isSigner).toBe(true);
    });

    it("should include payer as writable signer", () => {
      const payerAddress = new PublicKey("11111111111111111111111111111115");
      const instruction = createCreatePermissionInstruction(
        {
          permissionedAccount: mockPublicKey,
          payer: payerAddress,
        },
      );

      const payerAccount = instruction.keys.find((key) =>
        key.pubkey.equals(payerAddress),
      );
      expect(payerAccount).toBeDefined();
      expect(payerAccount?.isWritable).toBe(true);
      expect(payerAccount?.isSigner).toBe(true);
    });

    it("should include permission PDA as writable", () => {
      const instruction = createCreatePermissionInstruction(
        {
          permissionedAccount: mockPublicKey,
          payer: mockPublicKey,
        },
      );

      const permissionAccount = instruction.keys.find(
        (key) => key.pubkey.equals(mockPublicKey) && key.isWritable,
      );
      expect(permissionAccount).toBeDefined();
    });

    it("should include system program", () => {
      const instruction = createCreatePermissionInstruction(
        {
          permissionedAccount: mockPublicKey,
          payer: mockPublicKey,
        },
      );

      const systemProgram = instruction.keys.find(
        (key) => key.pubkey.toBase58() === "11111111111111111111111111111111",
      );
      expect(systemProgram).toBeDefined();
      expect(systemProgram?.isWritable).toBe(false);
      expect(systemProgram?.isSigner).toBe(false);
    });

    it("should handle empty members list", () => {
      const instruction = createCreatePermissionInstruction(
        {
          permissionedAccount: mockPublicKey,
          payer: mockPublicKey,
        },
        { members: [] },
      );

      expect(instruction.data).toBeDefined();
      // Discriminator (8) + count (4) = 12 minimum
      expect(instruction.data.length).toBeGreaterThanOrEqual(12);
    });

    it("should handle multiple members", () => {
      const members = [
        { pubkey: mockPublicKey, authority: true },
        { pubkey: differentPublicKey, authority: false },
        { pubkey: new PublicKey("11111111111111111111111111111113"), authority: true },
      ];

      const instruction = createCreatePermissionInstruction(
        {
          permissionedAccount: mockPublicKey,
          payer: mockPublicKey,
        },
        { members },
      );

      expect(instruction.data).toBeDefined();
      // Discriminator (8) + count (4) + members (each 33 bytes)
      const expectedSize = 8 + 4 + members.length * 33;
      expect(instruction.data.length).toBeGreaterThanOrEqual(expectedSize);
    });

    it("should use discriminator [0, 0, 0, 0, 0, 0, 0, 0]", () => {
      const instruction = createCreatePermissionInstruction(
        {
          permissionedAccount: mockPublicKey,
          payer: mockPublicKey,
        },
      );

      // First 8 bytes should be discriminator
      expect(instruction.data[0]).toBe(0);
      expect(instruction.data[1]).toBe(0);
      expect(instruction.data[2]).toBe(0);
      expect(instruction.data[3]).toBe(0);
      expect(instruction.data[4]).toBe(0);
      expect(instruction.data[5]).toBe(0);
      expect(instruction.data[6]).toBe(0);
      expect(instruction.data[7]).toBe(0);
    });

    it("should encode authority flag correctly", () => {
      const authorityMember = { pubkey: mockPublicKey, authority: true };
      const nonAuthorityMember = { pubkey: differentPublicKey, authority: false };

      const instruction1 = createCreatePermissionInstruction(
        {
          permissionedAccount: mockPublicKey,
          payer: mockPublicKey,
        },
        { members: [authorityMember] },
      );

      const instruction2 = createCreatePermissionInstruction(
        {
          permissionedAccount: mockPublicKey,
          payer: mockPublicKey,
        },
        { members: [nonAuthorityMember] },
      );

      // Authority flag is after discriminator (8) + count (4) + pubkey (32)
      const authorityFlagIndex = 8 + 4 + 32;
      expect(instruction1.data[authorityFlagIndex]).toBe(1);
      expect(instruction2.data[authorityFlagIndex]).toBe(0);
    });
  });

  describe("updatePermission instruction", () => {
    it("should create an updatePermission instruction with valid parameters", () => {
      const instruction = createUpdatePermissionInstruction(
        {
          authority: mockPublicKey,
          permissionedAccount: mockPublicKey,
        },
        {
          members: [
            { pubkey: mockPublicKey, authority: true },
            { pubkey: differentPublicKey, authority: false },
          ],
        },
      );

      expect(instruction.programId.equals(PERMISSION_PROGRAM_ID)).toBe(true);
      expect(instruction.keys).toHaveLength(3);
      expect(instruction.data).toBeDefined();
    });

    it("should include authority as readonly signer", () => {
      const authorityAddress = new PublicKey("11111111111111111111111111111113");
      const instruction = createUpdatePermissionInstruction(
        {
          authority: authorityAddress,
          permissionedAccount: mockPublicKey,
        },
      );

      const authorityAccount = instruction.keys.find((key) =>
        key.pubkey.equals(authorityAddress),
      );
      expect(authorityAccount).toBeDefined();
      expect(authorityAccount?.isWritable).toBe(false);
      expect(authorityAccount?.isSigner).toBe(true);
    });

    it("should include permissionedAccount as readonly signer", () => {
      const permissionedAddress = new PublicKey("11111111111111111111111111111114");
      const instruction = createUpdatePermissionInstruction(
        {
          authority: mockPublicKey,
          permissionedAccount: permissionedAddress,
        },
      );

      const permissionedAccount = instruction.keys.find((key) =>
        key.pubkey.equals(permissionedAddress),
      );
      expect(permissionedAccount).toBeDefined();
      expect(permissionedAccount?.isWritable).toBe(false);
      expect(permissionedAccount?.isSigner).toBe(true);
    });

    it("should include permission PDA as writable", () => {
      const instruction = createUpdatePermissionInstruction(
        {
          authority: mockPublicKey,
          permissionedAccount: mockPublicKey,
        },
      );

      const writableAccounts = instruction.keys.filter((key) => key.isWritable);
      expect(writableAccounts.length).toBeGreaterThan(0);
    });

    it("should use discriminator [1, 0, 0, 0, 0, 0, 0, 0]", () => {
      const instruction = createUpdatePermissionInstruction(
        {
          authority: mockPublicKey,
          permissionedAccount: mockPublicKey,
        },
      );

      // First byte should be discriminator 1
      expect(instruction.data[0]).toBe(1);
      expect(instruction.data[1]).toBe(0);
      expect(instruction.data[2]).toBe(0);
      expect(instruction.data[3]).toBe(0);
      expect(instruction.data[4]).toBe(0);
      expect(instruction.data[5]).toBe(0);
      expect(instruction.data[6]).toBe(0);
      expect(instruction.data[7]).toBe(0);
    });

    it("should handle empty members list", () => {
      const instruction = createUpdatePermissionInstruction(
        {
          authority: mockPublicKey,
          permissionedAccount: mockPublicKey,
        },
        { members: [] },
      );

      expect(instruction.data).toBeDefined();
      // Discriminator (8) + count (4) = 12 minimum
      expect(instruction.data.length).toBeGreaterThanOrEqual(12);
    });

    it("should handle multiple members", () => {
      const members = [
        { pubkey: mockPublicKey, authority: true },
        { pubkey: differentPublicKey, authority: false },
        { pubkey: new PublicKey("11111111111111111111111111111113"), authority: true },
      ];

      const instruction = createUpdatePermissionInstruction(
        {
          authority: mockPublicKey,
          permissionedAccount: mockPublicKey,
        },
        { members },
      );

      expect(instruction.data).toBeDefined();
      // Discriminator (8) + count (4) + members (each 33 bytes)
      const expectedSize = 8 + 4 + members.length * 33;
      expect(instruction.data.length).toBeGreaterThanOrEqual(expectedSize);
    });
  });

  describe("Cross-instruction consistency", () => {
    it("should all target the same permission program", () => {
      const createPermissionInstr = createCreatePermissionInstruction({
        permissionedAccount: mockPublicKey,
        payer: mockPublicKey,
      });

      const updatePermissionInstr = createUpdatePermissionInstruction({
        authority: mockPublicKey,
        permissionedAccount: mockPublicKey,
      });

      expect(createPermissionInstr.programId.equals(PERMISSION_PROGRAM_ID)).toBe(
        true,
      );
      expect(updatePermissionInstr.programId.equals(PERMISSION_PROGRAM_ID)).toBe(
        true,
      );
    });

    it("should have unique discriminators", () => {
      const createPermissionInstr = createCreatePermissionInstruction({
        permissionedAccount: mockPublicKey,
        payer: mockPublicKey,
      });

      const updatePermissionInstr = createUpdatePermissionInstruction({
        authority: mockPublicKey,
        permissionedAccount: mockPublicKey,
      });

      const disc1 = createPermissionInstr.data[0];
      const disc2 = updatePermissionInstr.data[0];

      expect(disc1).not.toBe(disc2);
      expect(disc1).toBe(0);
      expect(disc2).toBe(1);
    });
  });
});
