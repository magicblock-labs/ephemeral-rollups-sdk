import { describe, expect, it } from "vitest";
import {
  AccountRole,
  address,
  getAddressDecoder,
  getAddressEncoder,
  getProgramDerivedAddress,
  type Address,
  type Instruction,
} from "@solana/kit";
import * as nacl from "tweetnacl";
import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  TOKEN_PROGRAM_ID,
  delegateEphemeralAtaIx,
  delegateIx,
  delegateSpl,
  delegateSplWithPrivateTransfer,
  depositSplTokensIx,
  deriveEphemeralAta,
  deriveShuttleEphemeralAta,
  deriveShuttleWalletAta,
  deriveVault,
  initVaultAtaIx,
  initVaultIx,
  mergeShuttleIntoAtaIx,
  transferSpl,
  transferToVaultIx,
  undelegateAndCloseShuttleEphemeralAtaIx,
  undelegateIx,
  withdrawSpl,
  withdrawSplIx,
} from "../index";

const owner = getAddressDecoder().decode(
  nacl.sign.keyPair.fromSeed(new Uint8Array(32).fill(1)).publicKey,
);
const recipient = getAddressDecoder().decode(
  nacl.sign.keyPair.fromSeed(new Uint8Array(32).fill(2)).publicKey,
);
const validator = getAddressDecoder().decode(
  nacl.sign.keyPair.fromSeed(new Uint8Array(32).fill(3)).publicKey,
);
const mint = address("So11111111111111111111111111111111111111112");
const token2022 = address("TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb");
const shuttleId = 42;

async function ata(authority: Address, tokenProgram: Address) {
  const encoder = getAddressEncoder();
  const [derived] = await getProgramDerivedAddress({
    programAddress: ASSOCIATED_TOKEN_PROGRAM_ID,
    seeds: [
      encoder.encode(authority),
      encoder.encode(tokenProgram),
      encoder.encode(mint),
    ],
  });
  return derived;
}

function expectSelectedProgram(instructions: Instruction[], program: Address) {
  expect(
    instructions.some(
      (ix) =>
        ix.programAddress === program ||
        ix.accounts?.some((account) => account.address === program),
    ),
  ).toBe(true);
  if (program !== TOKEN_PROGRAM_ID) {
    for (const ix of instructions) {
      expect(ix.programAddress).not.toBe(TOKEN_PROGRAM_ID);
      expect(ix.accounts?.map((account) => account.address)).not.toContain(
        TOKEN_PROGRAM_ID,
      );
    }
  }
}

describe("SPL utilities through the public Kit entry point", () => {
  it("exports explicit deposit and delegation names without replacing existing helpers", async () => {
    expect(depositSplTokensIx).toBe(transferToVaultIx);
    expect(delegateEphemeralAtaIx).toBe(delegateIx);
    const [ephemeralAta] = await deriveEphemeralAta(owner, mint);
    for (const selectedValidator of [undefined, validator]) {
      const instruction = await delegateEphemeralAtaIx(
        owner,
        ephemeralAta,
        selectedValidator,
      );
      expect(instruction.data?.[0]).toBe(4);
      expect(instruction.data?.length).toBe(selectedValidator ? 33 : 1);
      expect(instruction.accounts?.[0]).toEqual({
        address: owner,
        role: AccountRole.WRITABLE_SIGNER,
      });
    }
  });

  it.each([TOKEN_PROGRAM_ID, token2022])(
    "derives and builds a direct deposit and withdrawal using %s",
    async (tokenProgram) => {
      const [ephemeralAta] = await deriveEphemeralAta(owner, mint);
      const [vault] = await deriveVault(mint);
      const [vaultEphemeralAta] = await deriveEphemeralAta(vault, mint);
      const vaultAta = await ata(vault, tokenProgram);
      const ownerAta = await ata(owner, tokenProgram);
      const setup = [
        initVaultIx(
          vault,
          mint,
          owner,
          vaultEphemeralAta,
          vaultAta,
          tokenProgram,
        ),
        initVaultAtaIx(owner, vaultAta, vault, mint, tokenProgram),
      ];
      const deposit = depositSplTokensIx(
        ephemeralAta,
        vault,
        mint,
        ownerAta,
        vaultAta,
        owner,
        0x0102_0304_0506_0708n,
        tokenProgram,
      );
      expect(Array.from(deposit.data ?? [])).toEqual([
        2, 8, 7, 6, 5, 4, 3, 2, 1,
      ]);
      expect(deposit.accounts?.[5]).toEqual({
        address: owner,
        role: AccountRole.READONLY_SIGNER,
      });
      const withdrawal = await withdrawSplIx(owner, mint, 1n, tokenProgram);
      const undelegation = await undelegateIx(owner, mint, tokenProgram);
      expect(undelegation.accounts?.[1].address).toBe(ownerAta);
      expect(withdrawal.accounts?.[4].address).toBe(vaultAta);
      expect(withdrawal.accounts?.[5].address).toBe(ownerAta);
      expect(Array.from(withdrawal.data ?? [])).toEqual([
        3, 1, 0, 0, 0, 0, 0, 0, 0,
      ]);
      expectSelectedProgram([...setup, deposit, withdrawal], tokenProgram);
    },
  );

  it.each([-1n, 0x1_0000_0000_0000_0000n])(
    "rejects out-of-range deposit amounts: %s",
    (amount) => {
      expect(() =>
        depositSplTokensIx(owner, owner, mint, owner, owner, owner, amount),
      ).toThrow();
    },
  );

  it("preserves the close instruction's escrow index with a selected token program", async () => {
    const [shuttle] = await deriveShuttleEphemeralAta(owner, mint, shuttleId);
    const walletAta = await ata(shuttle, token2022);
    expect(await deriveShuttleWalletAta(mint, shuttle, token2022)).toBe(
      walletAta,
    );
    const merge = mergeShuttleIntoAtaIx(
      owner,
      owner,
      shuttle,
      walletAta,
      mint,
      token2022,
    );
    const close = undelegateAndCloseShuttleEphemeralAtaIx(
      owner,
      owner,
      shuttle,
      shuttle,
      walletAta,
      owner,
      255,
      token2022,
    );
    expect(Array.from(close.data ?? [])).toEqual([14, 255]);
    expectSelectedProgram([merge, close], token2022);
    expect(() =>
      undelegateAndCloseShuttleEphemeralAtaIx(
        owner,
        owner,
        shuttle,
        shuttle,
        walletAta,
        owner,
        256,
        token2022,
      ),
    ).toThrow("escrowIndex must fit in u8");
  });

  describe.each([TOKEN_PROGRAM_ID, token2022])(
    "complete flows using %s",
    (tokenProgram) => {
      it.each([false, true])(
        "delegates with idempotent=%s",
        async (idempotent) => {
          const instructions = await delegateSpl(owner, mint, 1n, {
            tokenProgram,
            idempotent,
            shuttleId,
            initVaultIfMissing: true,
            initAtasIfMissing: true,
          });
          const [vault] = await deriveVault(mint);
          const vaultInit = instructions.find(
            (ix) => ix.accounts?.[0].address === vault,
          );
          expect(vaultInit?.accounts?.[4].address).toBe(
            await ata(vault, tokenProgram),
          );
          const deposit = instructions.find(
            (ix) => ix.data?.[0] === (idempotent ? 24 : 2),
          );
          expect(
            deposit?.accounts?.map((account) => account.address),
          ).toContain(await ata(owner, tokenProgram));
          expectSelectedProgram(instructions, tokenProgram);
        },
      );

      it("initializes a zero-balance shuttle using the selected token program", async () => {
        const instructions = await delegateSpl(owner, mint, 0n, {
          tokenProgram,
          shuttleId,
        });
        expect(instructions[instructions.length - 2]?.data?.[0]).toBe(11);
        expectSelectedProgram(instructions, tokenProgram);
      });

      it.each([false, true])(
        "withdraws with idempotent=%s",
        async (idempotent) => {
          const instructions = await withdrawSpl(owner, mint, 1n, {
            tokenProgram,
            idempotent,
            shuttleId,
            initAtasIfMissing: true,
          });
          expect(instructions[0].accounts?.[1].address).toBe(
            await ata(owner, tokenProgram),
          );
          expectSelectedProgram(instructions, tokenProgram);
        },
      );

      it("withdraws a Magic ATA source without creating an eATA", async () => {
        const instructions = await withdrawSpl(owner, mint, 1n, {
          tokenProgram,
          shuttleId,
          magicAtaSource: true,
        });
        expect(instructions).toHaveLength(2);
        expect(instructions[0].accounts?.[1].address).toBe(
          await ata(owner, tokenProgram),
        );
        expectSelectedProgram(instructions, tokenProgram);
      });

      it("delegates and prepares a private transfer with matching vault and queue setup", async () => {
        const instructions = await delegateSplWithPrivateTransfer(
          owner,
          mint,
          1n,
          {
            tokenProgram,
            validator,
            shuttleId,
            initVaultIfMissing: true,
            initAtasIfMissing: true,
            initTransferQueueIfMissing: true,
          },
        );
        expectSelectedProgram(instructions, tokenProgram);
      });

      it.each([
        ["public", "base", "base", false],
        ["public", "ephemeral", "ephemeral", false],
        ["private", "ephemeral", "ephemeral", false],
        ["private", "ephemeral", "base", false],
        ["private", "base", "base", false],
        ["private", "base", "ephemeral", false],
        ["private", "base", "ephemeral", true],
      ] as const)(
        "transfers %s %s -> %s (legacy destination=%s)",
        async (
          visibility,
          fromBalance,
          toBalance,
          legacyCleartextDestination,
        ) => {
          const instructions = await transferSpl(owner, recipient, mint, 1n, {
            tokenProgram,
            validator,
            shuttleId,
            visibility,
            fromBalance,
            toBalance,
            legacyCleartextDestination,
            initIfMissing: true,
            initAtasIfMissing: true,
            initVaultIfMissing: true,
          });
          expectSelectedProgram(instructions, tokenProgram);
          const ownerAta = await ata(owner, tokenProgram);
          expect(
            instructions.some((ix) =>
              ix.accounts?.some((account) => account.address === ownerAta),
            ),
          ).toBe(true);
        },
      );
    },
  );
});
