import { describe, it, expect } from "vitest";
import { PublicKey } from "@solana/web3.js";
import {
  createCreatePermissionInstruction,
  createUpdatePermissionInstruction,
} from "../instructions/permission-program";
import { PERMISSION_PROGRAM_ID } from "../constants";
import { permissionPdaFromAccount } from "../pda";
import { AUTHORITY_FLAG } from "../access-control/types";

describe("Permission Program Instructions (web3.js)", () => {
  const testAuthority = new PublicKey("11111111111111111111111111111113");
  const testMember = new PublicKey("11111111111111111111111111111112");

  describe("createPermission instruction", () => {
    it("should create a createPermission instruction with valid parameters", () => {
      const instruction = createCreatePermissionInstruction(
        {
          permissionedAccount: testAuthority,
          payer: testAuthority,
        },
        {
          members: [
            { pubkey: testAuthority, flags: AUTHORITY_FLAG },
            { pubkey: testMember, flags: 0 },
          ],
        },
      );

      expect(instruction.programId.equals(PERMISSION_PROGRAM_ID)).toBe(true);
      expect(instruction.keys).toHaveLength(4);
      expect(instruction.data).toBeDefined();
      expect(instruction.data.length).toBeGreaterThan(0);
    });

    it("should serialize members correctly", () => {
      const members = [{ pubkey: testAuthority, flags: AUTHORITY_FLAG }];
      const instruction = createCreatePermissionInstruction(
        {
          permissionedAccount: testAuthority,
          payer: testAuthority,
        },
        { members },
      );

      expect(instruction.data).toBeDefined();
      // Discriminator (8) + Option discriminant (1) + count (4) + member (32 + 1) = 46 minimum
      expect(instruction.data.length).toBeGreaterThanOrEqual(46);
    });

    it("should include permissionedAccount as readonly signer", () => {
      const instruction = createCreatePermissionInstruction({
        permissionedAccount: testAuthority,
        payer: testMember,
      }, { members: null });

      const permissionedAccount = instruction.keys.find((key) =>
        key.pubkey.equals(testAuthority),
      );
      expect(permissionedAccount).toBeDefined();
      expect(permissionedAccount?.isWritable).toBe(false);
      expect(permissionedAccount?.isSigner).toBe(true);
    });

    it("should include payer as writable signer", () => {
      const payerAddress = new PublicKey("11111111111111111111111111111115");
      const instruction = createCreatePermissionInstruction({
        permissionedAccount: testAuthority,
        payer: payerAddress,
      }, { members: null });

      const payerAccount = instruction.keys.find((key) =>
        key.pubkey.equals(payerAddress),
      );
      expect(payerAccount).toBeDefined();
      expect(payerAccount?.isWritable).toBe(true);
      expect(payerAccount?.isSigner).toBe(true);
    });

    it("should include permission PDA as writable", () => {
      const permissionedAccountAddress = new PublicKey(
        "11111111111111111111111111111116",
      );
      const instruction = createCreatePermissionInstruction({
        permissionedAccount: permissionedAccountAddress,
        payer: testMember,
      }, { members: null });

      const expectedPda = permissionPdaFromAccount(permissionedAccountAddress);

      // Verify the permission PDA is at the expected index (1)
      const permissionAccount = instruction.keys[1];
      expect(permissionAccount).toBeDefined();
      expect(permissionAccount.pubkey.equals(expectedPda)).toBe(true);
      expect(permissionAccount.isWritable).toBe(true);
      expect(permissionAccount.isSigner).toBe(false);
    });

    it("should include system program", () => {
      const instruction = createCreatePermissionInstruction({
        permissionedAccount: testAuthority,
        payer: testAuthority,
      }, { members: null });

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
          permissionedAccount: testAuthority,
          payer: testAuthority,
        },
        { members: [] },
      );

      expect(instruction.data).toBeDefined();
      // Discriminator (8) + Option discriminant (1) + count (4) = 13 minimum
      expect(instruction.data.length).toBeGreaterThanOrEqual(13);
    });

    it("should handle multiple members", () => {
      const members = [
        { pubkey: testAuthority, flags: AUTHORITY_FLAG },
        { pubkey: testMember, flags: 0 },
        {
          pubkey: new PublicKey("11111111111111111111111111111111"),
          flags: AUTHORITY_FLAG,
        },
      ];

      const instruction = createCreatePermissionInstruction(
        {
          permissionedAccount: testAuthority,
          payer: testAuthority,
        },
        { members },
      );

      expect(instruction.data).toBeDefined();
      // Discriminator (8) + Option discriminant (1) + count (4) + members (each 33 bytes)
      const expectedSize = 8 + 1 + 4 + members.length * 33;
      expect(instruction.data.length).toBeGreaterThanOrEqual(expectedSize);
    });

    it("should use discriminator [0, 0, 0, 0, 0, 0, 0, 0]", () => {
      const instruction = createCreatePermissionInstruction({
        permissionedAccount: testAuthority,
        payer: testAuthority,
      }, { members: null });

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
      const authorityMember = {
        pubkey: testAuthority,
        flags: AUTHORITY_FLAG,
      };
      const nonAuthorityMember = {
        pubkey: testMember,
        flags: 0,
      };

      const instruction1 = createCreatePermissionInstruction(
        {
          permissionedAccount: testAuthority,
          payer: testAuthority,
        },
        { members: [authorityMember] },
      );

      const instruction2 = createCreatePermissionInstruction(
        {
          permissionedAccount: testAuthority,
          payer: testAuthority,
        },
        { members: [nonAuthorityMember] },
      );

      // Authority flag is after discriminator (8) + option discriminant (1) + count (4)
      const authorityFlagIndex = 8 + 1 + 4;
      expect(instruction1.data[authorityFlagIndex]).toBe(1);
      expect(instruction2.data[authorityFlagIndex]).toBe(0);
    });
  });

  describe("updatePermission instruction", () => {
    it("should create an updatePermission instruction with valid parameters", () => {
      const instruction = createUpdatePermissionInstruction(
        {
          authority: [testAuthority, true],
          permissionedAccount: [testAuthority, true],
        },
        {
          members: [
            { pubkey: testAuthority, flags: AUTHORITY_FLAG },
            { pubkey: testMember, flags: 0 },
          ],
        },
      );

      expect(instruction.programId.equals(PERMISSION_PROGRAM_ID)).toBe(true);
      expect(instruction.keys).toHaveLength(3);
      expect(instruction.data).toBeDefined();
    });

    it("should include authority as signer", () => {
      const authorityAddress = new PublicKey(
        "11111111111111111111111111111113",
      );
      const instruction = createUpdatePermissionInstruction({
        authority: [authorityAddress, true],
        permissionedAccount: [testAuthority, false],
      }, { members: null });

      const authorityAccount = instruction.keys.find((key) =>
        key.pubkey.equals(authorityAddress),
      );
      expect(authorityAccount).toBeDefined();
      expect(authorityAccount?.isWritable).toBe(false);
      expect(authorityAccount?.isSigner).toBe(true);
    });

    it("should not include permissionedAccount as writable signer", () => {
      const permissionedAddress = new PublicKey(
        "11111111111111111111111111111114",
      );
      const instruction = createUpdatePermissionInstruction({
        authority: [testAuthority, false],
        permissionedAccount: [permissionedAddress, true],
      }, { members: null });

      const permissionedAccount = instruction.keys.find((key) =>
        key.pubkey.equals(permissionedAddress),
      );
      expect(permissionedAccount).toBeDefined();
      expect(permissionedAccount?.isWritable).toBe(false);
      expect(permissionedAccount?.isSigner).toBe(true);
    });

    it("should include permission PDA as writable at index 2", () => {
      const permissionedAccountAddress = new PublicKey(
        "11111111111111111111111111111117",
      );
      const instruction = createUpdatePermissionInstruction({
        authority: [testAuthority, false],
        permissionedAccount: [permissionedAccountAddress, true],
      }, { members: null });

      const expectedPda = permissionPdaFromAccount(permissionedAccountAddress);

      // Verify the permission PDA is at the expected index (2)
      const permissionAccount = instruction.keys[2];
      expect(permissionAccount).toBeDefined();
      expect(permissionAccount.pubkey.equals(expectedPda)).toBe(true);
      expect(permissionAccount.isWritable).toBe(true);
      expect(permissionAccount.isSigner).toBe(false);
    });

    it("should use discriminator [1, 0, 0, 0, 0, 0, 0, 0]", () => {
      const instruction = createUpdatePermissionInstruction({
        authority: [testAuthority, true],
        permissionedAccount: [testAuthority, true],
      }, { members: null });

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
          authority: [testAuthority, true],
          permissionedAccount: [testAuthority, true],
        },
        { members: [] },
      );

      expect(instruction.data).toBeDefined();
      // Discriminator (8) + count (4) = 12 minimum
      expect(instruction.data.length).toBeGreaterThanOrEqual(12);
    });

    it("should handle multiple members", () => {
      const members = [
        { pubkey: testAuthority, flags: AUTHORITY_FLAG },
        { pubkey: testMember, flags: 0 },
        {
          pubkey: new PublicKey("11111111111111111111111111111113"),
          flags: AUTHORITY_FLAG,
        },
      ];

      const instruction = createUpdatePermissionInstruction(
        {
          authority: [testAuthority, true],
          permissionedAccount: [testAuthority, true],
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
        permissionedAccount: testAuthority,
        payer: testAuthority,
      }, { members: null });

      const updatePermissionInstr = createUpdatePermissionInstruction({
        authority: [testAuthority, true],
        permissionedAccount: [testAuthority, true],
      }, { members: null });

      expect(
        createPermissionInstr.programId.equals(PERMISSION_PROGRAM_ID),
      ).toBe(true);
      expect(
        updatePermissionInstr.programId.equals(PERMISSION_PROGRAM_ID),
      ).toBe(true);
    });

    it("should have unique discriminators", () => {
      const createPermissionInstr = createCreatePermissionInstruction({
        permissionedAccount: testAuthority,
        payer: testAuthority,
      }, { members: null });

      const updatePermissionInstr = createUpdatePermissionInstruction({
        authority: [testAuthority, true],
        permissionedAccount: [testAuthority, true],
      }, { members: null });

      const disc1 = createPermissionInstr.data[0];
      const disc2 = updatePermissionInstr.data[0];

      expect(disc1).not.toBe(disc2);
      expect(disc1).toBe(0);
      expect(disc2).toBe(1);
    });
  });
});
