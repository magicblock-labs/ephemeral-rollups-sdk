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
 * 
 * Verify the integrity of the RPC
 * @param rpcUrl - The URL of the RPC server
 * @returns True if the quote is valid, false otherwise
 */
export async function verifyTeeRpcIntegrity(rpcUrl: string): Promise<boolean> {
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
 * @param validatorIdentity - The expected identity of the validator
 * @returns True if the quote is valid, false otherwise
 */
export async function verifyTeeIntegrity(rpcUrl: string, validatorIdentity: PublicKey): Promise<boolean> {
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

  if (!await verifyChallenge(responseBody, validatorIdentity)) {
    throw new Error("Invalid signature");
  }

  return verifyQuote(responseBody.quote);
}

/**
 * Verify the integrity of a quote
 * @param quote - The quote to verify (base64 encoded)
 * @returns True if the quote is valid, false otherwise
 */
async function verifyQuote(quote: string): Promise<boolean> {
  // Import the WASM module
  const {
    default: init,
    js_get_collateral: jsGetCollateral,
    js_verify: jsVerify,
  } = await import("@phala/dcap-qvl-web");

  // Initialize the WASM module
  await init();

  const rawQuote = Uint8Array.from(Buffer.from(quote, "base64"));

  // Get the quote collateral
  const pccsUrl = "https://pccs.phala.network/tdx/certification/v4";
  const quoteCollateral = await jsGetCollateral(pccsUrl, rawQuote);

  // Current timestamp
  const now = BigInt(Math.floor(Date.now() / 1000));

  // Call the js_verify function
  try {
    jsVerify(rawQuote, quoteCollateral, now);
    return true;
  } catch (error) {
    return false;
  }
}

async function verifyChallenge(response: FastQuoteResponse, validatorIdentity: PublicKey): Promise<boolean> {
  // Import this way because they are ECMAScript modules
  const bs58 = (await import("bs58")).default;
  const nacl = (await import("tweetnacl")).default;

  const msgBytes = Buffer.from(response.challenge, "base64");
  const sigBytes = bs58.decode(response.signature);
  const pk = new PublicKey(response.pubkey);

  if (!pk.equals(validatorIdentity)) {
    throw new Error("Invalid validator identity");
  }

  return nacl.sign.detached.verify(msgBytes, sigBytes, validatorIdentity.toBytes());
}