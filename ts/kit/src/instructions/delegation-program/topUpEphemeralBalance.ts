import { Address, Instruction, AccountMeta, AccountRole } from "@solana/kit";
import { SYSTEM_PROGRAM_ADDRESS } from "@solana-program/system";
import { DELEGATION_PROGRAM_ID } from "../../constants";

/**
 * TopUpEphemeralBalance instruction arguments
 */
export interface TopUpEphemeralBalanceInstructionArgs {
  amount: bigint;
  index?: number; // defaults to 255
}

/**
 * Instruction: TopUpEphemeralBalance
 * Discriminator: [9,0,0,0,0,0,0,0]
 */
export function createTopUpEscrowInstruction(
  escrow: Address,
  escrowAuthority: Address,
  payer: Address,
  amount: number,
  index?: number,
): Instruction {
  const accounts: AccountMeta[] = [
    { address: payer, role: AccountRole.WRITABLE_SIGNER },
    { address: escrowAuthority, role: AccountRole.READONLY },
    { address: escrow, role: AccountRole.WRITABLE },
    { address: SYSTEM_PROGRAM_ADDRESS, role: AccountRole.READONLY },
  ];

  const [instructionData] = serializeTopUpEphemeralBalanceInstructionData({
    amount: BigInt(amount),
    index: index ?? 255,
  });

  return {
    accounts,
    data: instructionData,
    programAddress: DELEGATION_PROGRAM_ID,
  };
}

export function serializeTopUpEphemeralBalanceInstructionData(
  args: TopUpEphemeralBalanceInstructionArgs,
): [Uint8Array] {
  const discriminator = [9, 0, 0, 0, 0, 0, 0, 0];
  const buffer = new ArrayBuffer(17);
  const view = new DataView(buffer);
  let offset = 0;

  // Write discriminator
  for (let i = 0; i < 8; i++) {
    view.setUint8(offset++, discriminator[i]);
  }

  // Write amount as u64 little-endian
  view.setBigUint64(offset, args.amount, true);
  offset += 8;

  // Write index as u8
  view.setUint8(offset, args.index ?? 255);

  return [new Uint8Array(buffer, 0, 17)];
}
