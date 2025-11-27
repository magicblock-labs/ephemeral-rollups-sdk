import { describe, it, expect, vi, beforeEach } from "vitest";
import { PublicKey } from "@solana/web3.js";
import { getAuthToken } from "../access-control/auth";

describe("Access Control (web3.js)", () => {
  const mockRpcUrl = "http://localhost:8899";
  const mockPublicKey = new PublicKey("11111111111111111111111111111111");

  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe("getAuthToken", () => {
    it("should request a challenge and return token with expiration", async () => {
      const mockChallenge = "test-challenge";
      const mockToken = "test-token";
      const mockSignature = new Uint8Array([1, 2, 3]);

      global.fetch = vi
        .fn()
        .mockResolvedValueOnce({
          json: async () => ({ challenge: mockChallenge }),
        })
        .mockResolvedValueOnce({
          status: 200,
          json: async () => ({ token: mockToken }),
        });

      const signMessage = vi.fn().mockResolvedValue(mockSignature);

      const result = await getAuthToken(mockRpcUrl, mockPublicKey, signMessage);

      expect(result).toHaveProperty("token");
      expect(result).toHaveProperty("expiresAt");
      expect(result.token).toBe(mockToken);
      expect(typeof result.expiresAt).toBe("number");
      expect(result.expiresAt).toBeGreaterThan(Date.now());
    });

    it("should use expiresAt from server response if provided", async () => {
      const mockChallenge = "test-challenge";
      const mockToken = "test-token";
      const mockServerExpiresAt = Date.now() + 7200000; // 2 hours from now
      const mockSignature = new Uint8Array([1, 2, 3]);

      global.fetch = vi
        .fn()
        .mockResolvedValueOnce({
          json: async () => ({ challenge: mockChallenge }),
        })
        .mockResolvedValueOnce({
          status: 200,
          json: async () => ({
            token: mockToken,
            expiresAt: mockServerExpiresAt,
          }),
        });

      const signMessage = vi.fn().mockResolvedValue(mockSignature);

      const result = await getAuthToken(mockRpcUrl, mockPublicKey, signMessage);

      expect(result.expiresAt).toBe(mockServerExpiresAt);
    });

    it("should throw error on authentication failure", async () => {
      const mockChallenge = "test-challenge";
      const mockError = "Invalid signature";

      global.fetch = vi
        .fn()
        .mockResolvedValueOnce({
          json: async () => ({ challenge: mockChallenge }),
        })
        .mockResolvedValueOnce({
          status: 401,
          json: async () => ({ error: mockError }),
        });

      const signMessage = vi.fn().mockResolvedValue(new Uint8Array([1, 2, 3]));

      await expect(
        getAuthToken(mockRpcUrl, mockPublicKey, signMessage),
      ).rejects.toThrow("Failed to authenticate");
    });

    it("should call signMessage with the challenge", async () => {
      const mockChallenge = "test-challenge";
      const mockToken = "test-token";
      const mockSignature = new Uint8Array([1, 2, 3]);

      global.fetch = vi
        .fn()
        .mockResolvedValueOnce({
          json: async () => ({ challenge: mockChallenge }),
        })
        .mockResolvedValueOnce({
          status: 200,
          json: async () => ({ token: mockToken }),
        });

      const signMessage = vi.fn().mockResolvedValue(mockSignature);

      await getAuthToken(mockRpcUrl, mockPublicKey, signMessage);

      expect(signMessage).toHaveBeenCalledWith(
        new Uint8Array(Buffer.from(mockChallenge, "utf-8")),
      );
    });

    it("should send correct request to auth/challenge endpoint", async () => {
      const mockChallenge = "test-challenge";
      const mockToken = "test-token";

      global.fetch = vi
        .fn()
        .mockResolvedValueOnce({
          json: async () => ({ challenge: mockChallenge }),
        })
        .mockResolvedValueOnce({
          status: 200,
          json: async () => ({ token: mockToken }),
        });

      const signMessage = vi.fn().mockResolvedValue(new Uint8Array([1, 2, 3]));

      await getAuthToken(mockRpcUrl, mockPublicKey, signMessage);

      const firstCall = (global.fetch as any).mock.calls[0];
      expect(firstCall[0]).toContain(`${mockRpcUrl}/auth/challenge`);
      expect(firstCall[0]).toContain(`pubkey=${mockPublicKey.toString()}`);
    });

    it("should send POST request to auth/login with signature", async () => {
      const mockChallenge = "test-challenge";
      const mockToken = "test-token";

      global.fetch = vi
        .fn()
        .mockResolvedValueOnce({
          json: async () => ({ challenge: mockChallenge }),
        })
        .mockResolvedValueOnce({
          status: 200,
          json: async () => ({ token: mockToken }),
        });

      const signMessage = vi.fn().mockResolvedValue(new Uint8Array([1, 2, 3]));

      await getAuthToken(mockRpcUrl, mockPublicKey, signMessage);

      const secondCall = (global.fetch as any).mock.calls[1];
      expect(secondCall[0]).toContain(`${mockRpcUrl}/auth/login`);
      expect(secondCall[1].method).toBe("POST");
      expect(secondCall[1].headers["Content-Type"]).toBe("application/json");
    });

    it("should include pubkey, challenge, and signature in login request", async () => {
      const mockChallenge = "test-challenge";
      const mockToken = "test-token";

      global.fetch = vi
        .fn()
        .mockResolvedValueOnce({
          json: async () => ({ challenge: mockChallenge }),
        })
        .mockResolvedValueOnce({
          status: 200,
          json: async () => ({ token: mockToken }),
        });

      const signMessage = vi.fn().mockResolvedValue(new Uint8Array([1, 2, 3]));

      await getAuthToken(mockRpcUrl, mockPublicKey, signMessage);

      const secondCall = (global.fetch as any).mock.calls[1];
      // eslint-disable-next-line @typescript-eslint/no-unsafe-argument
      const body = JSON.parse(secondCall[1].body);

      expect(body).toHaveProperty("pubkey");
      expect(body).toHaveProperty("challenge");
      expect(body).toHaveProperty("signature");
      expect(body.pubkey).toBe(mockPublicKey.toString());
      expect(body.challenge).toBe(mockChallenge);
    });
  });
});
