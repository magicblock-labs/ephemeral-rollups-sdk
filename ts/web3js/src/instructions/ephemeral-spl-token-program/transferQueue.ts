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
  TOKEN_PROGRAM_ID,
} from "../../constants.js";
import {
  delegateBufferPdaFromDelegatedAccountAndOwnerProgram,
  delegationMetadataPdaFromDelegatedAccount,
  delegationRecordPdaFromDelegatedAccount,
  permissionPdaFromAccount,
} from "../../pda.js";
import { instructionBytes, instructionU8Array } from "./index.js";

const TRANSFER_QUEUE_SEED = Buffer.from("queue");
const QUEUE_REFILL_STATE_SEED = Buffer.from("queue-refill");
const RENT_PDA_SEED = Buffer.from("rent");
const LAMPORTS_PDA_SEED = Buffer.from("lamports");
const INITIALIZE_TRANSFER_QUEUE_DISCRIMINATOR = 12;
const DEPOSIT_AND_QUEUE_TRANSFER_DISCRIMINATOR = 16;
const ENSURE_TRANSFER_QUEUE_CRANK_DISCRIMINATOR = 17;
const DELEGATE_TRANSFER_QUEUE_DISCRIMINATOR = 19;
const ALLOCATE_TRANSFER_QUEUE_DISCRIMINATOR = 27;
const PROCESS_PENDING_TRANSFER_QUEUE_REFILL_DISCRIMINATOR = 28;

export interface StructuredInstruction {
  accounts: AccountMeta[];
  data: Uint8Array;
  programAddress: PublicKey;
}

export function toTransactionInstruction(
  instruction: StructuredInstruction | TransactionInstruction,
): TransactionInstruction {
  if ("keys" in instruction) {
    return instruction;
  }

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
): TransactionInstruction {
  return toTransactionInstruction({
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
        ? new Uint8Array([
            ...instructionBytes(INITIALIZE_TRANSFER_QUEUE_DISCRIMINATOR),
            0 /* None tag */,
          ])
        : new Uint8Array([
            ...instructionBytes(INITIALIZE_TRANSFER_QUEUE_DISCRIMINATOR),
            1, // Some tag
            ...u32le(requestedItems),
          ]),
    programAddress: EPHEMERAL_SPL_TOKEN_PROGRAM_ID,
  });
}

/**
 * Allocate additional space for a prefunded transfer queue.
 * @param queue - The transfer queue PDA
 * @returns The allocate transfer queue instruction
 */
export function allocateTransferQueueIx(
  queue: PublicKey,
): TransactionInstruction {
  return toTransactionInstruction({
    accounts: [
      { pubkey: queue, isSigner: false, isWritable: true },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
    ],
    data: instructionU8Array(ALLOCATE_TRANSFER_QUEUE_DISCRIMINATOR),
    programAddress: EPHEMERAL_SPL_TOKEN_PROGRAM_ID,
  });
}

/**
 * Deposit SPL tokens into the vault and queue one or more delayed transfers.
 * @param queue - The transfer queue PDA
 * @param vault - The mint vault PDA
 * @param mint - The mint account
 * @param source - The sender token account
 * @param vaultAta - The vault token account
 * @param destination - The queued destination owner
 * @param owner - The sender authority
 * @param amount - The total amount to queue
 * @param minDelayMs - The minimum delay in milliseconds
 * @param maxDelayMs - The maximum delay in milliseconds
 * @param split - The number of queue entries to create
 * @param reimbursementTokenInfo - Reimbursement token account used by the queue-full fallback path
 * @param clientRefId - Optional client-provided reference ID attached to each queued split
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
  reimbursementTokenInfo: PublicKey = source,
  clientRefId?: bigint,
): TransactionInstruction {
  if (!Number.isInteger(split) || split <= 0 || split > 0xffff_ffff) {
    throw new Error("split must fit in u32");
  }
  if (
    amount < 0n ||
    minDelayMs < 0n ||
    maxDelayMs < 0n ||
    (clientRefId !== undefined && clientRefId < 0n)
  ) {
    throw new Error("amount, delays, and clientRefId must be non-negative");
  }
  if (maxDelayMs < minDelayMs) {
    throw new Error("maxDelayMs must be greater than or equal to minDelayMs");
  }

  const data = [
    ...instructionBytes(DEPOSIT_AND_QUEUE_TRANSFER_DISCRIMINATOR),
    ...u64le(amount),
    ...u64le(minDelayMs),
    ...u64le(maxDelayMs),
    ...u32le(split),
    0, // None; flags: Option<u8>
  ];
  if (clientRefId !== undefined) {
    data.push(1); // Some: clientRefId
    data.push(...u64le(clientRefId));
  } else {
    data.push(0); // None: clientRefId
  }

  console.log(
    "DEPOSIT_AND_QUEUE_TRANSFER_DISCRIMINATOR: ixdata-len: ",
    data.length,
  );

  return toTransactionInstruction({
    accounts: [
      { pubkey: queue, isSigner: false, isWritable: true },
      { pubkey: vault, isSigner: false, isWritable: false },
      { pubkey: mint, isSigner: false, isWritable: false },
      { pubkey: source, isSigner: false, isWritable: true },
      { pubkey: vaultAta, isSigner: false, isWritable: true },
      { pubkey: destination, isSigner: false, isWritable: false },
      { pubkey: owner, isSigner: true, isWritable: false },
      { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
      { pubkey: reimbursementTokenInfo, isSigner: false, isWritable: true },
    ],
    data: new Uint8Array(data),
    programAddress: EPHEMERAL_SPL_TOKEN_PROGRAM_ID,
  });
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
): TransactionInstruction {
  return toTransactionInstruction({
    accounts: [
      { pubkey: payer, isSigner: true, isWritable: true },
      { pubkey: queue, isSigner: false, isWritable: true },
      { pubkey: magicFeeVault, isSigner: false, isWritable: true },
      { pubkey: magicContext, isSigner: false, isWritable: true },
      { pubkey: magicProgram, isSigner: false, isWritable: false },
    ],
    data: instructionU8Array(ENSURE_TRANSFER_QUEUE_CRANK_DISCRIMINATOR),
    programAddress: EPHEMERAL_SPL_TOKEN_PROGRAM_ID,
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
  return toTransactionInstruction({
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
    data: instructionU8Array(DELEGATE_TRANSFER_QUEUE_DISCRIMINATOR),
    programAddress: EPHEMERAL_SPL_TOKEN_PROGRAM_ID,
  });
}

/**
 * Execute a pending transfer-queue refill through the sponsored lamports flow.
 * @param queue - The delegated transfer queue PDA
 * @returns The pending transfer queue refill instruction
 */
export function processPendingTransferQueueRefillIx(
  queue: PublicKey,
): TransactionInstruction {
  const [rentPda] = PublicKey.findProgramAddressSync(
    [RENT_PDA_SEED],
    EPHEMERAL_SPL_TOKEN_PROGRAM_ID,
  );
  const [refillState] = PublicKey.findProgramAddressSync(
    [QUEUE_REFILL_STATE_SEED, queue.toBuffer()],
    EPHEMERAL_SPL_TOKEN_PROGRAM_ID,
  );
  const [lamportsPda] = PublicKey.findProgramAddressSync(
    [LAMPORTS_PDA_SEED, rentPda.toBuffer(), queue.toBuffer(), queue.toBuffer()],
    EPHEMERAL_SPL_TOKEN_PROGRAM_ID,
  );

  return toTransactionInstruction({
    accounts: [
      { pubkey: refillState, isSigner: false, isWritable: true },
      { pubkey: queue, isSigner: false, isWritable: true },
      { pubkey: rentPda, isSigner: false, isWritable: true },
      { pubkey: lamportsPda, isSigner: false, isWritable: true },
      {
        pubkey: EPHEMERAL_SPL_TOKEN_PROGRAM_ID,
        isSigner: false,
        isWritable: false,
      },
      {
        pubkey: delegateBufferPdaFromDelegatedAccountAndOwnerProgram(
          lamportsPda,
          EPHEMERAL_SPL_TOKEN_PROGRAM_ID,
        ),
        isSigner: false,
        isWritable: true,
      },
      {
        pubkey: delegationRecordPdaFromDelegatedAccount(lamportsPda),
        isSigner: false,
        isWritable: true,
      },
      {
        pubkey: delegationMetadataPdaFromDelegatedAccount(lamportsPda),
        isSigner: false,
        isWritable: true,
      },
      { pubkey: DELEGATION_PROGRAM_ID, isSigner: false, isWritable: false },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
      {
        pubkey: delegationRecordPdaFromDelegatedAccount(queue),
        isSigner: false,
        isWritable: false,
      },
    ],
    data: instructionU8Array(
      PROCESS_PENDING_TRANSFER_QUEUE_REFILL_DISCRIMINATOR,
    ),
    programAddress: EPHEMERAL_SPL_TOKEN_PROGRAM_ID,
  });
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
