"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.UndelegateAccounts = exports.DelegateAccounts = void 0;
var web3_js_1 = require("@solana/web3.js");
var seeds_1 = require("./seeds");
var constants_1 = require("./constants");
function DelegateAccounts(accountToDelegate, ownerProgram) {
    return getAccounts(accountToDelegate, ownerProgram, true);
}
exports.DelegateAccounts = DelegateAccounts;
function UndelegateAccounts(accountToDelegate, ownerProgram) {
    return getAccounts(accountToDelegate, ownerProgram, false);
}
exports.UndelegateAccounts = UndelegateAccounts;
function getAccounts(accountToDelegate, ownerProgram, ownedBuffer) {
    if (ownedBuffer === void 0) { ownedBuffer = true; }
    var pdaBytes = accountToDelegate.toBytes();
    var delegationPda = web3_js_1.PublicKey.findProgramAddressSync([Buffer.from(seeds_1.SEED_DELEGATION), pdaBytes], new web3_js_1.PublicKey(constants_1.DELEGATION_PROGRAM_ID))[0];
    var delegationMetadata = web3_js_1.PublicKey.findProgramAddressSync([Buffer.from(seeds_1.SEED_DELEGATION_METADATA), pdaBytes], new web3_js_1.PublicKey(constants_1.DELEGATION_PROGRAM_ID))[0];
    var bufferPda = web3_js_1.PublicKey.findProgramAddressSync([Buffer.from(seeds_1.SEED_BUFFER_PDA), pdaBytes], ownedBuffer
        ? new web3_js_1.PublicKey(ownerProgram)
        : new web3_js_1.PublicKey(constants_1.DELEGATION_PROGRAM_ID))[0];
    var commitStateRecordPda = web3_js_1.PublicKey.findProgramAddressSync([Buffer.from(seeds_1.SEED_COMMIT_STATE_RECORD_PDA), pdaBytes], new web3_js_1.PublicKey(constants_1.DELEGATION_PROGRAM_ID))[0];
    var commitStatePda = web3_js_1.PublicKey.findProgramAddressSync([Buffer.from(seeds_1.SEED_STATE_DIFF_PDA), pdaBytes], new web3_js_1.PublicKey(constants_1.DELEGATION_PROGRAM_ID))[0];
    return {
        delegationPda: delegationPda,
        delegationMetadata: delegationMetadata,
        bufferPda: bufferPda,
        commitStateRecordPda: commitStateRecordPda,
        commitStatePda: commitStatePda,
    };
}
//# sourceMappingURL=accounts.js.map