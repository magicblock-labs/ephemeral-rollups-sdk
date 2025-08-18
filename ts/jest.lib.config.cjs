module.exports = {
  testEnvironment: 'node',
  roots: ['<rootDir>/lib'],
  moduleFileExtensions: ['js', 'json'],
  testMatch: ['**/__tests__/**/*.test.js', '**/?(*.)+(spec|test).js'],
  transform: {},
  setupFilesAfterEnv: ['<rootDir>/jest.setup.lib.cjs'],
};