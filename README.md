# Codex Rust Bindings

This repository provides Rust bindings for the Codex library, enabling seamless integration with Rust projects.

## Usage

Include in your Cargo project:

```toml
[dependencies]
codex-rust-bindings = "0.1"
```

For examples on how to use those bindings, please take a look at the [integration tests](./tests/).

## Development

### Prerequisites

Make sure your system has the [prerequisites](https://github.com/codex-storage/nim-codex) to run a local Codex node, including:

- Rust and Cargo
- Git
- Make (for building libcodex)

### Building

```bash
cargo build --release
# or, for debug
cargo build
```

This will automatically:

1. Initialize git submodules if needed
2. Build libcodex if not already built
3. Compile the Rust bindings

### Other Cargo Commands

```bash
# Run all tests
cargo test

# Run unit tests
cargo test-unit

# Run integration tests
cargo test-integration

# Run doctests
cargo test-doc
```

## Linking Modes

This crate supports two linking modes via Cargo features:

### Dynamic Linking (Default)

```bash
cargo build
# or explicitly
cargo build --features dynamic-linking
```

### Static Linking (Default)

```bash
cargo build --features static-linking
```

### In your Cargo.toml

```toml
[dependencies]
codex-rust-bindings = { version = "0.1", features = ["static-linking"] }
```

## License

[MIT](./LICENSE)
