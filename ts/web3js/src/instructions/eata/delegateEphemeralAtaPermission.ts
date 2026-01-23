import {
  PublicKey,
  TransactionInstruction,
  SystemProgram,
  AccountMeta,
} from "@solana/web3.js";
import {
  EATA_PROGRAM_ID,
  DELEGATION_PROGRAM_ID,
  PERMISSION_PROGRAM_ID,
} from "../../constants";
import {
  ephemeralAtaPdaWithBumpFromOwnerAndMint,
  permissionPdaFromAccount,
  delegateBufferPdaFromDelegatedAccountAndOwnerProgram,
  delegationRecordPdaFromDelegatedAccount,
  delegationMetadataPdaFromDelegatedAccount,
} from "../../pda";

export interface DelegateEphemeralAtaPermissionAccounts {
  payer: PublicKey;
  user: PublicKey;
  mint: PublicKey;
  validator: PublicKey;
}

export function createDelegateEphemeralAtaPermissionInstruction(
  accounts: DelegateEphemeralAtaPermissionAccounts,
): TransactionInstruction {
  const [ephemeralAta, bump] = ephemeralAtaPdaWithBumpFromOwnerAndMint(
    accounts.user,
    accounts.mint,
  );
  const permission = permissionPdaFromAccount(ephemeralAta);
  const buffer = delegateBufferPdaFromDelegatedAccountAndOwnerProgram(
    permission,
    PERMISSION_PROGRAM_ID,
  );
  const record = delegationRecordPdaFromDelegatedAccount(permission);
  const metadata = delegationMetadataPdaFromDelegatedAccount(permission);

  const keys: AccountMeta[] = [
    { pubkey: accounts.payer, isWritable: true, isSigner: true },
    { pubkey: ephemeralAta, isWritable: true, isSigner: false },
    { pubkey: PERMISSION_PROGRAM_ID, isWritable: false, isSigner: false },
    { pubkey: permission, isWritable: true, isSigner: false },
    { pubkey: SystemProgram.programId, isWritable: false, isSigner: false },
    { pubkey: buffer, isWritable: true, isSigner: false },
    { pubkey: record, isWritable: true, isSigner: false },
    { pubkey: metadata, isWritable: true, isSigner: false },
    { pubkey: DELEGATION_PROGRAM_ID, isWritable: false, isSigner: false },
    { pubkey: accounts.validator, isWritable: false, isSigner: false },
  ];

  const dataBuffer = Buffer.alloc(2);
  dataBuffer[0] = 7; // discriminator
  dataBuffer[1] = bump;

  return new TransactionInstruction({
    programId: EATA_PROGRAM_ID,
    keys,
    data: dataBuffer,
  });
}
