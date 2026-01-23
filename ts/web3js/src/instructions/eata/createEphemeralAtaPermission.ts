import {
  PublicKey,
  TransactionInstruction,
  SystemProgram,
  AccountMeta,
} from "@solana/web3.js";
import { EATA_PROGRAM_ID, PERMISSION_PROGRAM_ID } from "../../constants";
import {
  ephemeralAtaPdaWithBumpFromOwnerAndMint,
  permissionPdaFromAccount,
} from "../../pda";

export interface CreateEphemeralAtaPermissionAccounts {
  payer: PublicKey;
  user: PublicKey;
  mint: PublicKey;
}

export interface CreateEphemeralAtaPermissionArgs {
  flags: number;
}

export function createCreateEphemeralAtaPermissionInstruction(
  accounts: CreateEphemeralAtaPermissionAccounts,
  args: CreateEphemeralAtaPermissionArgs = { flags: 0 },
): TransactionInstruction {
  const [ephemeralAta, bump] = ephemeralAtaPdaWithBumpFromOwnerAndMint(
    accounts.user,
    accounts.mint,
  );
  const permission = permissionPdaFromAccount(ephemeralAta);

  const keys: AccountMeta[] = [
    { pubkey: ephemeralAta, isWritable: true, isSigner: false },
    { pubkey: permission, isWritable: true, isSigner: false },
    { pubkey: accounts.payer, isWritable: true, isSigner: true },
    { pubkey: SystemProgram.programId, isWritable: false, isSigner: false },
    { pubkey: PERMISSION_PROGRAM_ID, isWritable: false, isSigner: false },
  ];

  const buffer = Buffer.alloc(3);
  buffer[0] = 6; // discriminator
  buffer[1] = bump;
  buffer[2] = args.flags;

  return new TransactionInstruction({
    programId: EATA_PROGRAM_ID,
    keys,
    data: buffer,
  });
}
