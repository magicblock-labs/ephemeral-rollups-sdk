import { PublicKey } from "@solana/web3.js";
export declare function DelegateAccounts(accountToDelegate: PublicKey, ownerProgram: PublicKey): {
    delegationPda: PublicKey;
    delegationMetadata: PublicKey;
    bufferPda: PublicKey;
    commitStateRecordPda: PublicKey;
    commitStatePda: PublicKey;
};
export declare function UndelegateAccounts(accountToDelegate: PublicKey, ownerProgram: PublicKey): {
    delegationPda: PublicKey;
    delegationMetadata: PublicKey;
    bufferPda: PublicKey;
    commitStateRecordPda: PublicKey;
    commitStatePda: PublicKey;
};
//# sourceMappingURL=accounts.d.ts.map