// Provide a default fetch mock used across tests
if (typeof global.fetch === 'undefined') {
  // @ts-ignore
  global.fetch = jest.fn(async () =>
    Promise.resolve({
      json: async () => Promise.resolve({ result: { blockhash: 'mock-blockhash' } }),
    }),
  );
}
