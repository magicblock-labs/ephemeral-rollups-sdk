import { sha512 } from "@noble/hashes/sha2";
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
 * Verify the integrity of the RPC.
 * Slower than {@link verifyTeeIntegrity} but more secure as it requests
 * a specific attestation from the secure hardware.
 * @param rpcUrl - The URL of the RPC server
 * @throws If the attestation is invalid
 */
export async function verifyTeeRpcIntegrity(rpcUrl: string): Promise<void> {
  const challengeBytes = Buffer.from(
    Uint8Array.from(
      Array(64)
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
  const quote = await verifyQuote(rawQuote);
  if (!quote) {
    throw new Error("Invalid quote");
  }

  const td10 = quote.report.asTd10();
  const td15 = td10 ? null : quote.report.asTd15();
  const reportData = td10
    ? Buffer.from(td10.reportData)
    : td15
      ? Buffer.from(td15.base.reportData)
      : null;
  if (!reportData) {
    throw new Error("Unsupported quote report format");
  }
  if (!reportData.equals(challengeBytes)) {
    throw new Error("Quote reportData does not match challenge");
  }
}

/**
 * Verify the integrity of the RPC.
 * Faster than {@link verifyTeeRpcIntegrity} by reusing a cached attestation.
 * @param rpcUrl - The URL of the RPC server
 * @throws If the attestation is invalid
 */
export async function verifyTeeIntegrity(rpcUrl: string): Promise<void> {
  const challengeBytes = Buffer.from(
    Uint8Array.from(
      Array(64)
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
}

async function verifyQuote(rawQuote: Uint8Array): Promise<Quote | null> {
  const pccsUrl = "https://pccs.phala.network/tdx/certification/v4";
  const quoteCollateral = await getCollateral(pccsUrl, rawQuote);
  const now = Math.floor(Date.now() / 1000);

  verify(rawQuote, quoteCollateral, now);

  return Quote.parse(rawQuote);
}

async function verifyChallenge(
  response: FastQuoteResponse,
  parsedQuote: Quote,
  challengeBytes: Uint8Array,
): Promise<void> {
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

  let pubkeyHash = sha512(Uint8Array.from(pk));
  if (!reportData.subarray(0, 64).equals(Buffer.from(pubkeyHash))) {
    throw new Error(
      `Quote reportData mismatch: ${reportData.subarray(0, 64).toString("hex")} !== ${Buffer.from(pubkeyHash).toString("hex")}`,
    );
  }
}
