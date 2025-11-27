import { Address } from "@solana/kit";

const SESSION_DURATION = 1000 * 60 * 60 * 24 * 30; // 30 days

interface AuthChallengeResponse {
  challenge: string;
}

interface AuthLoginResponse {
  token: string;
  expiresAt?: number;
  error?: string;
}

/**
 * Get the auth token for a given public key
 * @param rpcUrl - The URL of the RPC server
 * @param publicKey - The public key of the user
 * @param signMessage - The function to sign a message
 * @returns The auth token and its expiration time
 */
export async function getAuthToken(
  rpcUrl: string,
  publicKey: Address,
  signMessage: (message: Uint8Array) => Promise<Uint8Array>,
): Promise<{ token: string; expiresAt: number }> {
  // Import this way because bs58 is an ECMAScript module
  const bs58 = (await import("bs58")).default;

  // Getting the challenge from the RPC
  const challengeResponse = await fetch(
    `${rpcUrl}/auth/challenge?pubkey=${publicKey.toString()}`,
  );
  const { challenge }: AuthChallengeResponse = await challengeResponse.json();

  // Signing the challenge
  const signature = await signMessage(
    new Uint8Array(Buffer.from(challenge, "utf-8")),
  );
  const signatureString = bs58.encode(signature);

  // Get the token from the RPC
  const authResponse = await fetch(`${rpcUrl}/auth/login`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      pubkey: publicKey.toString(),
      challenge,
      signature: signatureString,
    }),
  });
  const authJson: AuthLoginResponse = await authResponse.json();

  if (authResponse.status !== 200) {
    throw new Error(`Failed to authenticate: ${authJson.error}`);
  }

  const expiresAt = authJson.expiresAt ?? Date.now() + SESSION_DURATION;
  return { token: authJson.token, expiresAt };
}
