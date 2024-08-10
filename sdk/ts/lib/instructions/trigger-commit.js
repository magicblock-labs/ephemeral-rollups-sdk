"use strict";
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    Object.defineProperty(o, k2, { enumerable: true, get: function() { return m[k]; } });
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || function (mod) {
    if (mod && mod.__esModule) return mod;
    var result = {};
    if (mod != null) for (var k in mod) if (k !== "default" && Object.prototype.hasOwnProperty.call(mod, k)) __createBinding(result, mod, k);
    __setModuleDefault(result, mod);
    return result;
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.createCommitInstruction = exports.commitInstructionDiscriminator = exports.commitStruct = void 0;
var beet = __importStar(require("@metaplex-foundation/beet"));
var web3 = __importStar(require("@solana/web3.js"));
var web3_js_1 = require("@solana/web3.js");
var constants_1 = require("../constants");
exports.commitStruct = new beet.FixableBeetArgsStruct([["instructionDiscriminator", beet.uniformFixedSizeArray(beet.u8, 4)]], "CommitInstructionArgs");
exports.commitInstructionDiscriminator = [1, 0, 0, 0];
function createCommitInstruction(accounts, programId) {
    if (programId === void 0) { programId = new web3_js_1.PublicKey(constants_1.MAGIC_PROGRAM_ID); }
    var data = exports.commitStruct.serialize({
        instructionDiscriminator: exports.commitInstructionDiscriminator,
    })[0];
    var keys = [
        {
            pubkey: accounts.payer,
            isWritable: false,
            isSigner: true,
        },
        {
            pubkey: accounts.delegatedAccount,
            isWritable: true,
            isSigner: false,
        },
    ];
    return new web3.TransactionInstruction({
        programId: programId,
        keys: keys,
        data: data,
    });
}
exports.createCommitInstruction = createCommitInstruction;
//# sourceMappingURL=trigger-commit.js.map