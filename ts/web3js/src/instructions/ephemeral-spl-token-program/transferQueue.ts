import {
  PublicKey,
  SystemProgram,
  TransactionInstruction,
} from "@solana/web3.js";

import {
  DELEGATION_PROGRAM_ID,
  EPHEMERAL_SPL_TOKEN_PROGRAM_ID,
  MAGIC_CONTEXT_ID,
  MAGIC_PROGRAM_ID,
} from "../../constants.js";
import {
  delegateBufferPdaFromDelegatedAccountAndOwnerProgram,
  delegationMetadataPdaFromDelegatedAccount,
  delegationRecordPdaFromDelegatedAccount,
} from "../../pda.js";

const TRANSFER_QUEUE_SEED = Buffer.from("queue");
const TOKEN_PROGRAM_ID = new PublicKey(
  "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
);
const INITIALIZE_TRANSFER_QUEUE_DISCRIMINATOR = 12;
const DEPOSIT_AND_QUEUE_TRANSFER_DISCRIMINATOR = 16;
const ENSURE_TRANSFER_QUEUE_CRANK_DISCRIMINATOR = 17;
const DELEGATE_TRANSFER_QUEUE_DISCRIMINATOR = 19;

/**
 * Derive the transfer queue PDA for a mint.
 * @param mint - The mint account
 * @returns The transfer queue PDA and bump
 */
export function deriveTransferQueue(mint: PublicKey): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [TRANSFER_QUEUE_SEED, mint.toBuffer()],
    EPHEMERAL_SPL_TOKEN_PROGRAM_ID,
  );
}

/**
 * Initialize the per-mint transfer queue.
 * @param payer - The payer account
 * @param queue - The transfer queue PDA
 * @param mint - The mint account
 * @param sizeBytes - Optional queue size in bytes. Omit to use the program default.
 * @returns The initialize transfer queue instruction
 */
export function initTransferQueueIx(
  payer: PublicKey,
  queue: PublicKey,
  mint: PublicKey,
  sizeBytes?: number,
): TransactionInstruction {
  return new TransactionInstruction({
    programId: EPHEMERAL_SPL_TOKEN_PROGRAM_ID,
    keys: [
      { pubkey: payer, isSigner: true, isWritable: true },
      { pubkey: queue, isSigner: false, isWritable: true },
      { pubkey: mint, isSigner: false, isWritable: false },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
    ],
    data:
      sizeBytes === undefined
        ? Buffer.from([INITIALIZE_TRANSFER_QUEUE_DISCRIMINATOR])
        : Buffer.from([
            INITIALIZE_TRANSFER_QUEUE_DISCRIMINATOR,
            ...u32le(sizeBytes),
          ]),
  });
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
 * @param shuttleAta - The shuttle ATA that receives fund if queue is full
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
  shuttleAta?: PublicKey,
): TransactionInstruction {
  if (!Number.isInteger(split) || split <= 0 || split > 0xffff_ffff) {
    throw new Error("split must fit in u32");
  }
  if (amount < 0n || minDelayMs < 0n || maxDelayMs < 0n) {
    throw new Error("amount and delays must be non-negative");
  }
  if (maxDelayMs < minDelayMs) {
    throw new Error("maxDelayMs must be greater than or equal to minDelayMs");
  }

  return new TransactionInstruction({
    programId: EPHEMERAL_SPL_TOKEN_PROGRAM_ID,
    keys: [
      { pubkey: queue, isSigner: false, isWritable: true },
      { pubkey: vault, isSigner: false, isWritable: false },
      { pubkey: mint, isSigner: false, isWritable: false },
      { pubkey: source, isSigner: false, isWritable: true },
      { pubkey: vaultAta, isSigner: false, isWritable: true },
      { pubkey: destination, isSigner: false, isWritable: false },
      { pubkey: owner, isSigner: true, isWritable: false },
      { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
      { pubkey: shuttleAta ?? EPHEMERAL_SPL_TOKEN_PROGRAM_ID, isSigner: false, isWritable: true },
    ],
    data: Buffer.from([
      DEPOSIT_AND_QUEUE_TRANSFER_DISCRIMINATOR,
      ...u64le(amount),
      ...u64le(minDelayMs),
      ...u64le(maxDelayMs),
      ...u32le(split),
    ]),
  });
}

/**
 * Ensure the recurring transfer queue crank is scheduled.
 * @param payer - The payer account
 * @param queue - The transfer queue PDA
 * @param magicContext - The Magic context account
 * @param magicProgram - The Magic program account
 * @returns The ensure transfer queue crank instruction
 */
export function ensureTransferQueueCrankIx(
  payer: PublicKey,
  queue: PublicKey,
  magicContext: PublicKey = MAGIC_CONTEXT_ID,
  magicProgram: PublicKey = MAGIC_PROGRAM_ID,
): TransactionInstruction {
  return new TransactionInstruction({
    programId: EPHEMERAL_SPL_TOKEN_PROGRAM_ID,
    keys: [
      { pubkey: payer, isSigner: true, isWritable: true },
      { pubkey: queue, isSigner: false, isWritable: true },
      { pubkey: magicContext, isSigner: false, isWritable: true },
      { pubkey: magicProgram, isSigner: false, isWritable: false },
    ],
    data: Buffer.from([ENSURE_TRANSFER_QUEUE_CRANK_DISCRIMINATOR]),
  });
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
): TransactionInstruction {
  return new TransactionInstruction({
    programId: EPHEMERAL_SPL_TOKEN_PROGRAM_ID,
    keys: [
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
    data: Buffer.from([DELEGATE_TRANSFER_QUEUE_DISCRIMINATOR]),
  });
}

function u32le(n: number): number[] {
  if (!Number.isInteger(n) || n < 0 || n > 0xffff_ffff) {
    throw new Error("sizeBytes out of range for u32");
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
