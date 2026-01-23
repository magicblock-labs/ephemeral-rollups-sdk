import {
  PublicKey,
  TransactionInstruction,
  SystemProgram,
  AccountMeta,
} from "@solana/web3.js";
import { EATA_PROGRAM_ID, DELEGATION_PROGRAM_ID } from "../../constants";
import {
  ephemeralAtaPdaWithBumpFromOwnerAndMint,
  delegateBufferPdaFromDelegatedAccountAndOwnerProgram,
  delegationRecordPdaFromDelegatedAccount,
  delegationMetadataPdaFromDelegatedAccount,
} from "../../pda";

export interface DelegateEphemeralAtaAccounts {
  payer: PublicKey;
  user: PublicKey;
  mint: PublicKey;
  validator?: PublicKey | null;
}

export function createDelegateEphemeralAtaInstruction(
  accounts: DelegateEphemeralAtaAccounts,
): TransactionInstruction {
  const [ephemeralAta, bump] = ephemeralAtaPdaWithBumpFromOwnerAndMint(
    accounts.user,
    accounts.mint,
  );
  const buffer = delegateBufferPdaFromDelegatedAccountAndOwnerProgram(
    ephemeralAta,
    EATA_PROGRAM_ID,
  );
  const delegationRecord =
    delegationRecordPdaFromDelegatedAccount(ephemeralAta);
  const delegationMetadata =
    delegationMetadataPdaFromDelegatedAccount(ephemeralAta);

  const keys: AccountMeta[] = [
    { pubkey: accounts.payer, isWritable: true, isSigner: true },
    { pubkey: ephemeralAta, isWritable: true, isSigner: false },
    { pubkey: EATA_PROGRAM_ID, isWritable: false, isSigner: false },
    { pubkey: buffer, isWritable: true, isSigner: false },
    { pubkey: delegationRecord, isWritable: true, isSigner: false },
    { pubkey: delegationMetadata, isWritable: true, isSigner: false },
    { pubkey: DELEGATION_PROGRAM_ID, isWritable: false, isSigner: false },
    { pubkey: SystemProgram.programId, isWritable: false, isSigner: false },
  ];

  const dataBuffer = Buffer.alloc(35);
  let offset = 0;
  dataBuffer[offset++] = 4; // discriminator
  dataBuffer[offset++] = bump;

  if (accounts.validator) {
    dataBuffer[offset++] = 1; // Some
    dataBuffer.set(accounts.validator.toBuffer(), offset);
    offset += 32;
  } else {
    dataBuffer[offset++] = 0; // None
  }

  return new TransactionInstruction({
    programId: EATA_PROGRAM_ID,
    keys,
    data: dataBuffer.subarray(0, offset),
  });
}
