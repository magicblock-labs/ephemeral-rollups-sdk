import { defineConfig } from "vitest/config";

export default defineConfig({
  test: {
    globals: true,
    environment: "node",
    include: ["tests/**/*.test.ts"],
    // The lifecycle spans two validators with commit latency; give it room.
    testTimeout: 120_000,
    hookTimeout: 120_000,
    // web3.js and kit clients keep their own global state; isolate per file.
    fileParallelism: false,
  },
});
