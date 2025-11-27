import { Address } from "@solana/kit";

export interface PermissionStatusResponse {
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
  publicKey: Address,
): Promise<PermissionStatusResponse> {
  // Build the route from the provided RPC URL
  // Handle the provided token
  const [baseUrl, token] = rpcUrl.replace("/?", "?").split("?");
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
        `Permission status request failed: ${permissionStatusResponse.statusText}`,
      );
    }
    const response: PermissionStatusResponse =
      await permissionStatusResponse.json();
    return response;
  } catch (error) {
    throw new Error(
      `Failed to get permission status: ${error instanceof Error ? error.message : String(error)}`,
    );
  }
}

/**
 * Wait until the permission is active for a given public key
 * @param rpcUrl - The URL of the RPC server
 * @param publicKey - The public key of the user
 * @param timeout - The timeout in milliseconds
 * @returns True if the permission is active, false otherwise
 */
export async function waitUntilPermissionActive(
  rpcUrl: string,
  publicKey: Address,
  timeout?: number,
): Promise<boolean> {
  const startTime = Date.now();
  const timeoutMs = timeout ?? 30000;
  while (Date.now() - startTime < timeoutMs) {
    try {
      const { authorizedUsers } = await getPermissionStatus(rpcUrl, publicKey);
      if (authorizedUsers && authorizedUsers.length > 0) {
        return true;
      }
    } catch (error) {
      console.error(error);
    }
    await new Promise((resolve) => {
      setTimeout(resolve, 400);
    });
  }
  return false;
}
