import { PublicKey } from "@solana/web3.js";

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
  publicKey: PublicKey,
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
 * Force update permissions for a given public key
 * @param rpcUrl - The URL of the RPC server
 * @param publicKey - The public key of the user
 * @returns True if the force update was successful, false otherwise
 */
async function forcePermissionUpdate(
  rpcUrl: string,
  publicKey: PublicKey,
): Promise<boolean> {
  // Build the route from the provided RPC URL
  // Handle the provided token
  const [baseUrl, token] = rpcUrl.replace("/?", "?").split("?");
  let url;
  if (token) {
    url = `${baseUrl}/permission/force-update?${token}&pubkey=${publicKey.toString()}`;
  } else {
    url = `${baseUrl}/permission/force-update?pubkey=${publicKey.toString()}`;
  }

  try {
    const forceUpdateResponse = await fetch(url);
    if (!forceUpdateResponse.ok) {
      throw new Error(
        `Force permission update request failed: ${forceUpdateResponse.statusText}`,
      );
    }
    return true;
  } catch (error) {
    console.error(
      `Failed to force permission update: ${error instanceof Error ? error.message : String(error)}`,
    );
    return false;
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
  publicKey: PublicKey,
  timeout?: number,
): Promise<boolean> {
  const timeoutMs = timeout ?? 5000;

  // First, try getPermissionStatus with retries for the initial timeout
  let startTime = Date.now();
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

  // If timeout reached, try force permission update as fallback
  const forceUpdateSuccess = await forcePermissionUpdate(rpcUrl, publicKey);
  if (forceUpdateSuccess) {
    // Retry permission status for another 5 seconds after force update
    startTime = Date.now();
    while (Date.now() - startTime < timeoutMs) {
      try {
        const { authorizedUsers } = await getPermissionStatus(
          rpcUrl,
          publicKey,
        );
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
  }

  return false;
}
