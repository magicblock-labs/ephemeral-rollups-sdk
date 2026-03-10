import { PublicKey } from "@solana/web3.js";

interface QuoteResponse {
  quote: string;
  error?: string;
}

interface FastQuoteResponse {
  quote: string;
  pubkey: string;
  challenge: string;
  signature: string;
  error?: string;
}

interface ErrorResponse {
  error: string;
}

/**
 * @deprecated Use {@link verifyTeeIntegrity} instead.
 * Verify the integrity of the RPC
 * @param rpcUrl - The URL of the RPC server
 * @returns True if the quote is valid, false otherwise
 */
export async function verifyTeeRpcIntegrity(rpcUrl: string): Promise<boolean> {
  // Import the WASM module
  const {
    default: init,
    js_get_collateral: jsGetCollateral,
    js_verify: jsVerify,
  } = await import("@phala/dcap-qvl-web");

  const challengeBytes = Buffer.from(
    Uint8Array.from(
      Array(32)
        .fill(0)
        .map(() => Math.floor(Math.random() * 256)),
    ),
  );
  const challenge = challengeBytes.toString("base64");
  const url = `${rpcUrl}/quote?challenge=${encodeURIComponent(challenge)}`;

  const response = await fetch(url);
  const responseBody: QuoteResponse | ErrorResponse = await response.json();

  if (response.status !== 200 || !("quote" in responseBody)) {
    throw new Error(responseBody.error ?? "Failed to get quote");
  }

  return verifyQuote(responseBody.quote);
}

/**
 * Verify the integrity of the RPC
 * @param rpcUrl - The URL of the RPC server
 * @returns True if the quote is valid, false otherwise
 */
export async function verifyTeeIntegrity(rpcUrl: string): Promise<boolean> {
  const challengeBytes = Buffer.from(
    Uint8Array.from(
      Array(32)
        .fill(0)
        .map(() => Math.floor(Math.random() * 256)),
    ),
  );
  const challenge = challengeBytes.toString("base64");
  const url = `${rpcUrl}/fast-quote?challenge=${encodeURIComponent(challenge)}`;

  const response = await fetch(url);
  const responseBody: FastQuoteResponse | ErrorResponse = await response.json();

  if (response.status !== 200 || !("quote" in responseBody)) {
    throw new Error(responseBody.error ?? "Failed to get quote");
  }

  if (!await verifySolanaSignature({
    message: base64ToBytes(responseBody.challenge),
    signature: responseBody.signature,
    publicKey: responseBody.pubkey,
  })) {
    throw new Error("Invalid signature");
  }

  return verifyQuote(responseBody.quote);
}

async function verifyQuote(quote: string): Promise<boolean> {
  const {
    default: init,
    js_get_collateral: jsGetCollateral,
    js_verify: jsVerify,
  } = await import("@phala/dcap-qvl-web");

  await init();

  const rawQuote = Uint8Array.from(Buffer.from(quote, "base64"));

  const pccsUrl = "https://pccs.phala.network/tdx/certification/v4";
  const quoteCollateral = await jsGetCollateral(pccsUrl, rawQuote);

  const now = BigInt(Math.floor(Date.now() / 1000));

  try {
    jsVerify(rawQuote, quoteCollateral, now);
    return true;
  } catch (error) {
    return false;
  }
}

async function verifySolanaSignature({
  message,
  signature,
  publicKey,
}: {
  message: string | Uint8Array;
  signature: string | Uint8Array;
  publicKey: string | Uint8Array;
}): Promise<boolean> {
  const bs58 = (await import("bs58")).default;
  const nacl = (await import("tweetnacl")).default;

  const msgBytes =
    typeof message === "string" ? new TextEncoder().encode(message) : message;

  const sigBytes =
    typeof signature === "string" ? bs58.decode(signature) : signature;

  const pubKeyBytes =
    typeof publicKey === "string" ? new PublicKey(publicKey).toBytes() : publicKey;

  return nacl.sign.detached.verify(msgBytes, sigBytes, pubKeyBytes);
}

function base64ToBytes(base64: string): Uint8Array {
  const bin = atob(base64);
  return Uint8Array.from(bin, (c) => c.charCodeAt(0));
}