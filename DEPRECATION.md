# Deprecation Guide

This document outlines deprecated APIs and their migration paths.

## Deprecations

### v0.7.0 Deprecations

#### `MagicInstructionBuilder` → `MagicIntentBundleBuilder`

**Status**: Deprecated since v0.7.0  
**Removal Target**: v1.0.0  
**Migration**: Use `MagicIntentBundleBuilder` instead

**Before** (Deprecated):
```rust
use ephemeral_rollups_sdk::ephem::MagicInstructionBuilder;

let builder = MagicInstructionBuilder {
    payer: payer_account,
    magic_context: context_account,
    magic_program: program_account,
    magic_fee_vault: Some(vault_account),
};
let (accounts, instruction) = builder.build();
```

**After** (Current):
```rust
use ephemeral_rollups_sdk::ephem::MagicIntentBundleBuilder;

let mut builder = MagicIntentBundleBuilder::new(
    payer_account,
    context_account,
    program_account,
);
builder.set_magic_fee_vault(Some(vault_account));
let intent_instructions = builder.build()?;
```

---

#### `MagicAction` → `MagicIntent`

**Status**: Deprecated since v0.7.0  
**Removal Target**: v1.0.0  
**Migration**: Use `MagicIntent` enum instead

**Before** (Deprecated):
```rust
use ephemeral_rollups_sdk::ephem::MagicAction;

let action = MagicAction::Commit(commit_type);
```

**After** (Current):
```rust
use ephemeral_rollups_sdk::ephem::MagicIntent;

let intent = MagicIntent::Commit(commit_type);
```

---

#### `CommitType` → `CommitIntentBuilder`

**Status**: Deprecated since v0.7.0  
**Removal Target**: v1.0.0  
**Migration**: Use `CommitIntentBuilder` instead

**Before** (Deprecated):
```rust
use ephemeral_rollups_sdk::ephem::CommitType;

let commit = CommitType::Standalone(accounts_to_commit);
```

**After** (Current):
```rust
use ephemeral_rollups_sdk::ephem::CommitIntentBuilder;

let mut builder = CommitIntentBuilder::new();
for account in accounts_to_commit {
    builder.add_account(account);
}
```

---

### VRF SDK Deprecations

#### VRF Request Randomness

**Status**: Deprecated  
**Migration**: Use scoped identity randomness

**Before**:
```rust
// Using global randomness
```

**After**:
```rust
// Using scoped identity for deterministic randomness
```

---

## Timeline

- **v0.7.0 - Current**: Deprecated APIs available with warnings
- **v0.8.0 - Next Minor**: Continue support, encourage migration
- **v1.0.0 - Breaking Change**: Remove all deprecated APIs

---

## Support

If you encounter issues migrating from deprecated APIs, please file an issue on GitHub with:
1. Your current code using the deprecated API
2. What you're trying to accomplish
3. Your Rust version and OS
