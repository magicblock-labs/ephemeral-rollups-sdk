// Ensure @solana/web3.js is mocked before any test files are loaded
const mockPublicKey = (address) => ({
  toBase58: () => address,
  toString: () => address,
});

jest.mock('@solana/web3.js', () => {
  // We cannot call requireActual here because we want to isolate from the real implementation
  return {
    Connection: jest.fn().mockImplementation(() => ({
      rpcEndpoint: 'http://localhost',
      sendRawTransaction: jest.fn().mockResolvedValue('mock-signature'),
    })),
    Transaction: jest.fn().mockImplementation(() => ({
      feePayer: mockPublicKey('mock-fee-payer'),
      signature: [],
      instructions: [
        {
          keys: [
            { pubkey: mockPublicKey('key1'), isSigner: true, isWritable: true },
            { pubkey: mockPublicKey('key2'), isSigner: false, isWritable: false },
          ],
        },
      ],
      serialize: jest.fn(() => Buffer.from('mock')),
      sign: jest.fn(),
    })),
    Keypair: jest.fn().mockImplementation(() => ({
      publicKey: mockPublicKey('mock-public-key'),
      sign: jest.fn(),
    })),
    PublicKey: jest.fn().mockImplementation((address) => mockPublicKey(address)),
  };
});

// Provide a default fetch mock used across tests
if (typeof global.fetch === 'undefined') {
  // @ts-ignore
  global.fetch = jest.fn(async () =>
    Promise.resolve({
      json: async () => Promise.resolve({ result: { blockhash: 'mock-blockhash' } }),
    }),
  );
}
