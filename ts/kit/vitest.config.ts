// vitest.config.ts
import { defineConfig } from "vitest/config";

export default defineConfig({
  test: {
    globals: true,            // use global describe/it/expect
    environment: "node",      // node environment for SDK
    include: ["src/__test__/**/*.test.ts"], // your test files
    coverage: {
      reporter: ["text", "lcov"], // optional
    },
  },
  resolve: {
    alias: {
      "@": "/src",
    },
  },
});