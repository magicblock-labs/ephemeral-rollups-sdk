import { PublicKey } from "@solana/web3.js";
import axios from "axios";

interface AuthChallengeResponse {
  challenge: string;
}

interface AuthLoginResponse {
  token: string;
  error?: string;
}

/**
 * Get the auth token for a given public key
 * @param rpcUrl - The URL of the RPC server
 * @param publicKey - The public key of the user
 * @param signMessage - The function to sign a message
 * @returns The auth token
 */
export async function getAuthToken(
  rpcUrl: string,
  publicKey: PublicKey,
  signMessage: (message: Uint8Array) => Promise<Uint8Array>,
) {
  // Import this way because bs58 is an ECMAScript module
  const bs58 = (await import("bs58")).default;

  // Getting the challenge from the RPC
  const challengeResponse = await axios.get(
    `${rpcUrl}/auth/challenge?pubkey=${publicKey.toString()}`,
  );
  const challengeJson: AuthChallengeResponse = challengeResponse.data;

  // Signing the challenge
  const signature = await signMessage(
    new Uint8Array(Buffer.from(challengeJson.challenge, "utf-8")),
  );
  const signatureString = bs58.encode(signature);

  // Get the token from the RPC
  const authResponse = await axios.post(`${rpcUrl}/auth/login`, {
    pubkey: publicKey.toString(),
    challenge: challengeJson.challenge,
    signature: signatureString,
  });
  const authJson: AuthLoginResponse = authResponse.data;

  if (authResponse.status !== 200) {
    throw new Error(`Failed to authenticate: ${authJson.error}`);
  }

  return authJson.token;
}
