module.exports = {
  testEnvironment: "node",
  roots: ["<rootDir>/lib"],
  moduleFileExtensions: ["js", "ts", "json"],
  testMatch: ["**/__tests__/**/*.test.js", "**/__tests__/**/*.test.ts"],
  transform: { "^.+\\.ts$": "ts-jest" },
  setupFilesAfterEnv: ["<rootDir>/jest.setup.lib.cjs"],
};