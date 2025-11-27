# MagicBlock Permission Client (Rust)

A Rust client SDK for interacting with the MagicBlock Permission Program on Solana. This crate provides type-safe bindings for permission management, including creating and managing permission groups and individual permissions.

- **Crate**: `magicblock-permission-client`
- **Docs**: <https://docs.magicblock.gg/pages/private-ephemeral-rollups-pers/how-to-guide/quickstart>
- **Repository**: <https://github.com/magicblock-labs/magicblock-permission-program>

## Features

- Type-safe account structures for `Group` and `Permission`
- Instruction builders for:
  - `create_group` - Create a new permission group
  - `create_permission` - Create a new permission for an account
  - `update_permission` - Update an existing permission
- Optional features:
  - `anchor` - Anchor framework compatibility
  - `serde` - Serialization/deserialization support

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
magicblock-permission-client = { path = "../access-control" }
```

Or from crates.io (when published):

```toml
[dependencies]
magicblock-permission-client = "0.6.0"
```

## Usage

### Basic Example

```rust
use magicblock_permission_client::{
    ID,
    accounts::{Group, Permission},
    instructions::CreatePermission,
};

// Create a permission instruction
let create_permission = CreatePermission {
    permission: permission_pubkey,
    delegated_account: account_pubkey,
    group: group_pubkey,
    payer: payer_pubkey,
    system_program: solana_program::system_program::id(),
};

let instruction = create_permission.instruction();
```

### Program ID

```rust
use magicblock_permission_client::ID;

// Use the program ID constant
let program_id = ID;
```

### Account Constants

```rust
use magicblock_permission_client::accounts::Group;

// Get the account size
let group_size = Group::LEN;

// Get the discriminator
let discriminator = Group::DISCRIMINATOR;
```

## Generated Code

This crate contains auto-generated code from the MagicBlock Permission Program IDL using [kinobi](https://github.com/metaplex-foundation/kinobi). The generated code includes:

- Account structures (`Group`, `Permission`)
- Instruction builders (`CreateGroup`, `CreatePermission`, `UpdatePermission`)
- Error types
- Program ID constant

**Note**: Do not manually edit files in the `generated/` directory. Regenerate using kinobi if the program IDL changes.

## License

See the LICENSE file in the repository root.

## Related Projects

For a high-level overview and examples, see the repository README at the project root.
