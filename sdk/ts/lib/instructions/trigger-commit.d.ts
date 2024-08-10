import * as beet from "@metaplex-foundation/beet";
import * as web3 from "@solana/web3.js";
export declare const commitStruct: beet.FixableBeetArgsStruct<{
    instructionDiscriminator: number[];
}>;
export interface CommitInstructionAccounts {
    payer: web3.PublicKey;
    delegatedAccount: web3.PublicKey;
}
export declare const commitInstructionDiscriminator: number[];
export declare function createCommitInstruction(accounts: CommitInstructionAccounts, programId?: web3.PublicKey): web3.TransactionInstruction;
//# sourceMappingURL=trigger-commit.d.ts.map