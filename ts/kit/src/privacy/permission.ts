import { Address } from "@solana/kit";

interface PermissionStatusResponse {
  authorizedUsers?: string[];
}

/**
 * Get the auth token for a given public key
 * @param rpcUrl - The URL of the RPC server
 * @param publicKey - The public key of the user
 * @returns The auth token
 */
export async function getPermissionStatus(
  rpcUrl: string,
  publicKey: Address
): Promise<PermissionStatusResponse> {
  // Getting the challenge from the RPC
  const permissionStatusResponse = await fetch(
    `${rpcUrl}/permission?pubkey=${publicKey.toString()}`
  );
  const response: PermissionStatusResponse =
    await permissionStatusResponse.json();

  return response;
}

/**
 * Wait until the permission is granted for a given public key
 * @param rpcUrl - The URL of the RPC server
 * @param publicKey - The public key of the user
 * @param timeout - The timeout in milliseconds
 * @returns True if the permission is granted, false otherwise
 */
export async function waitUntilPermissionGranted(
  rpcUrl: string,
  publicKey: Address,
  timeout: number = 5000
): Promise<void> {
  const startTime = Date.now();
  while (Date.now() - startTime < timeout) {
    const { authorizedUsers } = await getPermissionStatus(rpcUrl, publicKey);
    if (!!authorizedUsers) {
      return;
    }
    await new Promise((resolve) => setTimeout(resolve, 400));
  }
}
