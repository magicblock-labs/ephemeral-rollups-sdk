import { getCollateral, verify, Quote } from "@phala/dcap-qvl";
import * as nacl from "tweetnacl";

interface QuoteResponse {
  quote: string;
  error?: string;
}

interface FastQuoteResponse {
  quote: string;
  hclVarDataSha256: string;
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

  const rawQuote = Uint8Array.from(Buffer.from(responseBody.quote, "base64"));
  return !!(await verifyQuote(rawQuote));
}

/**
 * Verify the integrity of the RPC
 * @param rpcUrl - The URL of the RPC server
 * @param validatorIdentity - The expected identity of the validator
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

  const rawQuote = Uint8Array.from(Buffer.from(responseBody.quote, "base64"));
  const quote = await verifyQuote(rawQuote);
  if (!quote) {
    throw new Error("Invalid quote");
  }

  await verifyChallenge(responseBody, quote, challengeBytes);
  return true;
}

async function verifyQuote(rawQuote: Uint8Array): Promise<Quote | null> {
  const pccsUrl = "https://pccs.phala.network/tdx/certification/v4";
  const quoteCollateral = await getCollateral(pccsUrl, rawQuote);
  const now = Math.floor(Date.now() / 1000);

  try {
    verify(rawQuote, quoteCollateral, now);
  } catch (error) {
    // Ignore the error if the SEPT_VE_DISABLE is not enabled
    // The bug has been reported to Azure.
    if (
      error instanceof Error &&
      !error.message.includes("SEPT_VE_DISABLE is not enabled")
    ) {
      throw new Error(error.message);
    }
  }

  return Quote.parse(rawQuote);
}

async function verifyChallenge(
  response: FastQuoteResponse,
  parsedQuote: Quote,
  challengeBytes: Uint8Array,
): Promise<boolean> {
  const msgBytes = Buffer.from(response.challenge, "base64");
  if (!msgBytes.equals(Buffer.from(challengeBytes))) {
    throw new Error("Invalid challenge");
  }

  const pk = Buffer.from(response.pubkey, "base64");
  if (pk.length !== 32) {
    throw new Error(`Invalid pubkey length: ${pk.length}`);
  }

  const sig = Buffer.from(response.signature, "base64");
  const okSig = nacl.sign.detached.verify(challengeBytes, sig, pk);
  if (!okSig) {
    throw new Error("Invalid signature");
  }

  const td = parsedQuote.report.asTd10();
  if (!td) {
    throw new Error("Not a TD10 quote");
  }

  const reportData = Buffer.from(td.reportData);
  if (reportData.length !== 64) {
    throw new Error(`Invalid reportData length: ${reportData.length}`);
  }

  const hclVarDataSha256 = Buffer.from(response.hclVarDataSha256, "base64");
  if (!reportData.subarray(0, 32).equals(hclVarDataSha256)) {
    throw new Error(
      `Quote reportData mismatch: ${reportData.subarray(0, 32).toString("hex")} !== ${hclVarDataSha256.toString("hex")}`,
    );
  }

  return true;
}
