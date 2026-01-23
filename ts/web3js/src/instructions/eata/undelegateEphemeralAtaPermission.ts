import {
  PublicKey,
  TransactionInstruction,
  AccountMeta,
} from "@solana/web3.js";
import {
  EATA_PROGRAM_ID,
  MAGIC_PROGRAM_ID,
  MAGIC_CONTEXT_ID,
  PERMISSION_PROGRAM_ID,
} from "../../constants";
import {
  ephemeralAtaPdaWithBumpFromOwnerAndMint,
  permissionPdaFromAccount,
} from "../../pda";

export interface UndelegateEphemeralAtaPermissionAccounts {
  payer: PublicKey;
  user: PublicKey;
  mint: PublicKey;
}

export function createUndelegateEphemeralAtaPermissionInstruction(
  accounts: UndelegateEphemeralAtaPermissionAccounts,
): TransactionInstruction {
  const [ephemeralAta] = ephemeralAtaPdaWithBumpFromOwnerAndMint(
    accounts.user,
    accounts.mint,
  );
  const permission = permissionPdaFromAccount(ephemeralAta);

  const keys: AccountMeta[] = [
    { pubkey: accounts.payer, isWritable: false, isSigner: true },
    { pubkey: ephemeralAta, isWritable: true, isSigner: false },
    { pubkey: permission, isWritable: true, isSigner: false },
    { pubkey: PERMISSION_PROGRAM_ID, isWritable: false, isSigner: false },
    { pubkey: MAGIC_PROGRAM_ID, isWritable: false, isSigner: false },
    { pubkey: MAGIC_CONTEXT_ID, isWritable: true, isSigner: false },
  ];

  const buffer = Buffer.alloc(1);
  buffer[0] = 8; // discriminator

  return new TransactionInstruction({
    programId: EATA_PROGRAM_ID,
    keys,
    data: buffer,
  });
}
