import {
  AccountMeta,
  PublicKey,
  SystemProgram,
  TransactionInstruction,
} from "@solana/web3.js";

import {
  DELEGATION_PROGRAM_ID,
  EPHEMERAL_SPL_TOKEN_PROGRAM_ID,
  MAGIC_CONTEXT_ID,
  MAGIC_PROGRAM_ID,
  PERMISSION_PROGRAM_ID,
} from "../../constants.js";
import {
  delegateBufferPdaFromDelegatedAccountAndOwnerProgram,
  delegationMetadataPdaFromDelegatedAccount,
  delegationRecordPdaFromDelegatedAccount,
  permissionPdaFromAccount,
} from "../../pda.js";

const TRANSFER_QUEUE_SEED = Buffer.from("queue");
const TOKEN_PROGRAM_ID = new PublicKey(
  "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
);
const INITIALIZE_TRANSFER_QUEUE_DISCRIMINATOR = 12;
const DEPOSIT_AND_QUEUE_TRANSFER_DISCRIMINATOR = 16;
const ENSURE_TRANSFER_QUEUE_CRANK_DISCRIMINATOR = 17;
const DELEGATE_TRANSFER_QUEUE_DISCRIMINATOR = 19;
const ALLOCATE_TRANSFER_QUEUE_DISCRIMINATOR = 27;

export interface StructuredInstruction {
  accounts: AccountMeta[];
  data: Uint8Array;
  programAddress: PublicKey;
}

export function toTransactionInstruction(
  instruction: StructuredInstruction,
): TransactionInstruction {
  return new TransactionInstruction({
    programId: instruction.programAddress,
    keys: instruction.accounts,
    data: Buffer.from(instruction.data),
  });
}

/**
 * Derive the transfer queue PDA for a mint/validator pair.
 * @param mint - The mint account
 * @param validator - The validator account
 * @returns The transfer queue PDA and bump
 */
export function deriveTransferQueue(
  mint: PublicKey,
  validator: PublicKey,
): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [TRANSFER_QUEUE_SEED, mint.toBuffer(), validator.toBuffer()],
    EPHEMERAL_SPL_TOKEN_PROGRAM_ID,
  );
}

/**
 * Initialize the per-validator transfer queue for a mint.
 * @param payer - The payer account
 * @param queue - The transfer queue PDA
 * @param mint - The mint account
 * @param validator - The validator account
 * @param requestedItems - Optional queue item count. Omit to use the program default.
 * @returns The initialize transfer queue instruction
 */
export function initTransferQueueIx(
  payer: PublicKey,
  queue: PublicKey,
  mint: PublicKey,
  validator: PublicKey,
  requestedItems?: number,
): StructuredInstruction {
  return {
    accounts: [
      { pubkey: payer, isSigner: true, isWritable: true },
      { pubkey: queue, isSigner: false, isWritable: true },
      {
        pubkey: permissionPdaFromAccount(queue),
        isSigner: false,
        isWritable: true,
      },
      { pubkey: mint, isSigner: false, isWritable: false },
      { pubkey: validator, isSigner: false, isWritable: false },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
      { pubkey: PERMISSION_PROGRAM_ID, isSigner: false, isWritable: false },
    ],
    data:
      requestedItems === undefined
        ? new Uint8Array([INITIALIZE_TRANSFER_QUEUE_DISCRIMINATOR])
        : new Uint8Array([
            INITIALIZE_TRANSFER_QUEUE_DISCRIMINATOR,
            ...u32le(requestedItems),
          ]),
    programAddress: EPHEMERAL_SPL_TOKEN_PROGRAM_ID,
  };
}

/**
 * Allocate additional space for a prefunded transfer queue.
 * @param queue - The transfer queue PDA
 * @returns The allocate transfer queue instruction
 */
export function allocateTransferQueueIx(
  queue: PublicKey,
): StructuredInstruction {
  return {
    accounts: [
      { pubkey: queue, isSigner: false, isWritable: true },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
    ],
    data: new Uint8Array([ALLOCATE_TRANSFER_QUEUE_DISCRIMINATOR]),
    programAddress: EPHEMERAL_SPL_TOKEN_PROGRAM_ID,
  };
}

/**
 * Deposit SPL tokens into the vault and queue one or more delayed transfers.
 * @param queue - The transfer queue PDA
 * @param vault - The mint vault PDA
 * @param mint - The mint account
 * @param source - The sender token account
 * @param vaultAta - The vault token account
 * @param destination - The queued destination token account
 * @param owner - The sender authority
 * @param amount - The total amount to queue
 * @param minDelayMs - The minimum delay in milliseconds
 * @param maxDelayMs - The maximum delay in milliseconds
 * @param split - The number of queue entries to create
 * @returns The deposit-and-queue-transfer instruction
 */
export function depositAndQueueTransferIx(
  queue: PublicKey,
  vault: PublicKey,
  mint: PublicKey,
  source: PublicKey,
  vaultAta: PublicKey,
  destination: PublicKey,
  owner: PublicKey,
  amount: bigint,
  minDelayMs: bigint = 0n,
  maxDelayMs: bigint = minDelayMs,
  split: number = 1,
): StructuredInstruction {
  if (!Number.isInteger(split) || split <= 0 || split > 0xffff_ffff) {
    throw new Error("split must fit in u32");
  }
  if (amount < 0n || minDelayMs < 0n || maxDelayMs < 0n) {
    throw new Error("amount and delays must be non-negative");
  }
  if (maxDelayMs < minDelayMs) {
    throw new Error("maxDelayMs must be greater than or equal to minDelayMs");
  }

  return {
    accounts: [
      { pubkey: queue, isSigner: false, isWritable: true },
      { pubkey: vault, isSigner: false, isWritable: false },
      { pubkey: mint, isSigner: false, isWritable: false },
      { pubkey: source, isSigner: false, isWritable: true },
      { pubkey: vaultAta, isSigner: false, isWritable: true },
      { pubkey: destination, isSigner: false, isWritable: false },
      { pubkey: owner, isSigner: true, isWritable: false },
      { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
    ],
    data: new Uint8Array([
      DEPOSIT_AND_QUEUE_TRANSFER_DISCRIMINATOR,
      ...u64le(amount),
      ...u64le(minDelayMs),
      ...u64le(maxDelayMs),
      ...u32le(split),
    ]),
    programAddress: EPHEMERAL_SPL_TOKEN_PROGRAM_ID,
  };
}

/**
 * Ensure the recurring transfer queue crank is scheduled.
 * @param payer - The payer account
 * @param queue - The transfer queue PDA
 * @param magicFeeVault - The validator magic fee vault PDA from the delegation program
 * @param magicContext - The Magic context account
 * @param magicProgram - The Magic program account
 * @returns The ensure transfer queue crank instruction
 */
export function ensureTransferQueueCrankIx(
  payer: PublicKey,
  queue: PublicKey,
  magicFeeVault: PublicKey,
  magicContext: PublicKey = MAGIC_CONTEXT_ID,
  magicProgram: PublicKey = MAGIC_PROGRAM_ID,
): StructuredInstruction {
  return {
    accounts: [
      { pubkey: payer, isSigner: true, isWritable: true },
      { pubkey: queue, isSigner: false, isWritable: true },
      { pubkey: magicFeeVault, isSigner: false, isWritable: true },
      { pubkey: magicContext, isSigner: false, isWritable: true },
      { pubkey: magicProgram, isSigner: false, isWritable: false },
    ],
    data: new Uint8Array([ENSURE_TRANSFER_QUEUE_CRANK_DISCRIMINATOR]),
    programAddress: EPHEMERAL_SPL_TOKEN_PROGRAM_ID,
  };
}

/**
 * Delegate the per-mint transfer queue PDA.
 * @param queue - The transfer queue PDA
 * @param payer - The payer account
 * @param mint - The mint account
 * @returns The delegate transfer queue instruction
 */
export function delegateTransferQueueIx(
  queue: PublicKey,
  payer: PublicKey,
  mint: PublicKey,
): StructuredInstruction {
  return {
    accounts: [
      { pubkey: payer, isSigner: true, isWritable: true },
      { pubkey: queue, isSigner: false, isWritable: true },
      { pubkey: mint, isSigner: false, isWritable: false },
      {
        pubkey: EPHEMERAL_SPL_TOKEN_PROGRAM_ID,
        isSigner: false,
        isWritable: false,
      },
      {
        pubkey: delegateBufferPdaFromDelegatedAccountAndOwnerProgram(
          queue,
          EPHEMERAL_SPL_TOKEN_PROGRAM_ID,
        ),
        isSigner: false,
        isWritable: true,
      },
      {
        pubkey: delegationRecordPdaFromDelegatedAccount(queue),
        isSigner: false,
        isWritable: true,
      },
      {
        pubkey: delegationMetadataPdaFromDelegatedAccount(queue),
        isSigner: false,
        isWritable: true,
      },
      { pubkey: DELEGATION_PROGRAM_ID, isSigner: false, isWritable: false },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
    ],
    data: new Uint8Array([DELEGATE_TRANSFER_QUEUE_DISCRIMINATOR]),
    programAddress: EPHEMERAL_SPL_TOKEN_PROGRAM_ID,
  };
}

function u32le(n: number): number[] {
  if (!Number.isInteger(n) || n < 0 || n > 0xffff_ffff) {
    throw new Error("value out of range for u32");
  }

  return [n & 0xff, (n >>> 8) & 0xff, (n >>> 16) & 0xff, (n >>> 24) & 0xff];
}

function u64le(n: bigint): number[] {
  if (n < 0n || n > 0xffff_ffff_ffff_ffffn) {
    throw new Error("value out of range for u64");
  }

  const out = Buffer.alloc(8);
  out.writeBigUInt64LE(n);
  return Array.from(out);
}
