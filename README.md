# Codex Rust Bindings

This repository provides Rust bindings for the Codex library, enabling seamless integration with Rust projects.

## Usage

Include in your Cargo project:

```toml
[dependencies]
codex-bindings = "0.1.3"
```

To learn how to use those bindings, take a look at the [example project](https://github.com/nipsysdev/example-codex-rust-bindings) or the [integration tests](./tests/).

## Building

### Requirements

This crate automatically builds the required libcodex library during compilation, so you don't need to install nim-codex separately. However, you will need:

- **Rust and Cargo**
- **Git**
- **Make**
- **C compiler**

Building will automatically:

1. Clone the nim-codex repository and it's submodules
2. Build the Nim compiler from source
3. Build libcodex with the Nim compiler
4. Generate Rust bindings and compile the crate

**Note**: The first build may take 10-20 minutes as it needs to build the Nim compiler from source. Subsequent builds will be much faster.

### Building from source

```bash
cargo build --release
# or, for debug
cargo build
```

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

### Static Linking

```bash
cargo build --features static-linking
```

### In your Cargo.toml

```toml
[dependencies]
codex-bindings = { version = "0.1.3", features = ["static-linking"] }
```

## License

[MIT](./LICENSE)
