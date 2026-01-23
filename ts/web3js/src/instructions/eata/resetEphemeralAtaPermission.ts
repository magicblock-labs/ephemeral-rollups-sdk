import {
  PublicKey,
  TransactionInstruction,
  AccountMeta,
} from "@solana/web3.js";
import { EATA_PROGRAM_ID, PERMISSION_PROGRAM_ID } from "../../constants";
import {
  ephemeralAtaPdaWithBumpFromOwnerAndMint,
  permissionPdaFromAccount,
} from "../../pda";

export interface ResetEphemeralAtaPermissionAccounts {
  owner: PublicKey;
  mint: PublicKey;
}

export interface ResetEphemeralAtaPermissionArgs {
  flags: number;
}

export function createResetEphemeralAtaPermissionInstruction(
  accounts: ResetEphemeralAtaPermissionAccounts,
  args: ResetEphemeralAtaPermissionArgs = { flags: 0 },
): TransactionInstruction {
  const [ephemeralAta, bump] = ephemeralAtaPdaWithBumpFromOwnerAndMint(
    accounts.owner,
    accounts.mint,
  );
  const permission = permissionPdaFromAccount(ephemeralAta);

  const keys: AccountMeta[] = [
    { pubkey: ephemeralAta, isWritable: true, isSigner: false },
    { pubkey: permission, isWritable: true, isSigner: false },
    { pubkey: accounts.owner, isWritable: false, isSigner: true },
    { pubkey: PERMISSION_PROGRAM_ID, isWritable: false, isSigner: false },
  ];

  const buffer = Buffer.alloc(3);
  buffer[0] = 9; // discriminator
  buffer[1] = bump;
  buffer[2] = args.flags;

  return new TransactionInstruction({
    programId: EATA_PROGRAM_ID,
    keys,
    data: buffer,
  });
}
