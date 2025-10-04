import "@solana/web3.js";

declare module "@solana/web3.js" {
  interface Connection {
    getClosestValidator: () => Promise<{ identity: string; fqdn?: string }>;
    getDelegationStatus: (
      account: string | PublicKey,
    ) => Promise<{ isDelegated: boolean }>;
    getLatestBlockhashForTransaction: (
      transaction: import("@solana/web3.js").Transaction,
      options?: import("@solana/web3.js").ConfirmOptions,
    ) => Promise<import("@solana/web3.js").BlockhashWithExpiryBlockHeight>;
    prepareTransaction: (
      transaction: import("@solana/web3.js").Transaction,
      options?: import("@solana/web3.js").ConfirmOptions,
    ) => Promise<import("@solana/web3.js").Transaction>;
    sendTransaction: (
      transaction: import("@solana/web3.js").Transaction,
      signersOrOptions?:
        | Array<import("@solana/web3.js").Signer>
        | import("@solana/web3.js").SendOptions,
      options?: import("@solana/web3.js").SendOptions,
    ) => Promise<string>;
    sendAndConfirmTransaction: (
      transaction: import("@solana/web3.js").Transaction,
      signers: Array<import("@solana/web3.js").Signer>,
      options?: import("@solana/web3.js").ConfirmOptions & {
        abortSignal?: AbortSignal;
      },
    ) => Promise<string>;
  }
}
