import { PublicKey } from "@solana/web3.js";

interface PermissionStatusResponse {
  authorizedUsers?: string[];
}

/**
 * Get the auth token for a given public key
 * @param rpcUrl - The URL of the RPC server
 * @param publicKey - The public key of the user
 * @returns The permission status response
 */
export async function getPermissionStatus(
  rpcUrl: string,
  publicKey: PublicKey
): Promise<PermissionStatusResponse> {
  // Build the route from the provided RPC URL
  // Handle the provided token
  let [baseUrl, token] = rpcUrl.replace("/?", "?").split("?");
  let url;
  if (token) {
    url = `${baseUrl}/permission?${token}&pubkey=${publicKey.toString()}`;
  } else {
    url = `${baseUrl}/permission?pubkey=${publicKey.toString()}`;
  }

  try {
    const permissionStatusResponse = await fetch(url);
    if (!permissionStatusResponse.ok) {
      throw new Error(
        `Permission status request failed: ${permissionStatusResponse.statusText}`
      );
    }
    const response: PermissionStatusResponse =
      await permissionStatusResponse.json();
    return response;
  } catch (error) {
    throw new Error(
      `Failed to get permission status: ${error instanceof Error ? error.message : String(error)}`
    );
  }
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
  publicKey: PublicKey,
  timeout?: number
): Promise<boolean> {
  const startTime = Date.now();
  while (Date.now() - startTime < (timeout || 30000)) {
    try {
      const { authorizedUsers } = await getPermissionStatus(rpcUrl, publicKey);
      if (!!authorizedUsers) {
        return true;
      }
    } catch (error) {
      return false;
    }
    await new Promise((resolve) => setTimeout(resolve, 400));
  }
  return false;
}
