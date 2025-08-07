import axios from "axios";

interface QuoteResponse {
  quote: string;
  error?: string;
}

/**
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

  const response = await axios.get<QuoteResponse>(url);

  if (response.status !== 200) {
    throw new Error(response.data.error);
  }

  // Initialize the WASM module
  await init();

  const rawQuote = Uint8Array.from(Buffer.from(response.data.quote, "base64"));

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
