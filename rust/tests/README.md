# Integration Tests

This directory contains integration tests for the ephemeral-rollups-sdk workspace.

## Structure

- `common/` - Shared test utilities and fixtures
- `integration/` - Integration tests for the SDK
- `unit/` - Unit tests for specific modules

## Running Tests

Run all tests:
```bash
cargo test
```

Run specific test:
```bash
cargo test --test <test_name>
```

Run with output:
```bash
cargo test -- --nocapture
```

## Writing Tests

Place new integration tests in the appropriate directory:
- Tests that verify multiple components working together go in `integration/`
- Tests for specific modules go in `unit/`
- Shared test helpers go in `common/`
