import { Address, Instruction, AccountMeta, AccountRole } from "@solana/kit";
import { SYSTEM_PROGRAM_ADDRESS } from "@solana-program/system";
import { DELEGATION_PROGRAM_ID } from "../../constants";

/**
 * CloseEphemeralBalance instruction arguments
 */
export interface CloseEphemeralBalanceInstructionArgs {
  index?: number; // defaults to 255
}

/**
 * Instruction: CloseEphemeralBalance
 * Discriminator: [11,0,0,0,0,0,0,0]
 */
export function createCloseEscrowInstruction(
  escrow: Address,
  escrowAuthority: Address,
  index?: number,
): Instruction {
  const [instructionData] = serializeCloseEphemeralBalanceInstructionData({
    index: index ?? 255,
  });

  const accounts_: AccountMeta[] = [
    {
      address: escrowAuthority,
      role: AccountRole.READONLY_SIGNER,
    },
    {
      address: escrow,
      role: AccountRole.WRITABLE,
    },
    {
      address: SYSTEM_PROGRAM_ADDRESS,
      role: AccountRole.READONLY,
    },
  ];

  return {
    accounts: accounts_,
    data: instructionData,
    programAddress: DELEGATION_PROGRAM_ID,
  };
}

export function serializeCloseEphemeralBalanceInstructionData(
  args?: CloseEphemeralBalanceInstructionArgs,
): [Uint8Array] {
  const discriminator = [11, 0, 0, 0, 0, 0, 0, 0];
  const buffer = new ArrayBuffer(9);
  const view = new DataView(buffer);
  let offset = 0;

  // Write discriminator
  for (let i = 0; i < 8; i++) {
    view.setUint8(offset++, discriminator[i]);
  }

  // Write index as u8
  view.setUint8(offset, args?.index ?? 255);

  return [new Uint8Array(buffer, 0, 9)];
}
