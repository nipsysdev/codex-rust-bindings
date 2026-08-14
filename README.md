# Storage Rust Bindings

This repository provides Rust bindings for the Storage library, enabling seamless integration with Rust projects.

## Usage

Include in your Cargo project:

```toml
[dependencies]
storage-bindings = "0.4.4"
```

To learn how to use those bindings, take a look at the [integration tests](./tests/) directory.

## Building

Building will automatically:

1. Fetch the pinned static library release of libstorage for your platform from the [logos-storage-nim](https://github.com/logos-storage/logos-storage-nim/releases) repository releases
2. Generate Rust bindings and compile the crate

**Note**: The first build will download the prebuilt libraries (~135MB on Linux, ~12MB on macOS). Subsequent builds will use the cached version.

## Caching

Prebuilt binaries are automatically cached to improve build performance and reduce network usage.

### Cache Location

Prebuilt binaries are cached in a platform-specific location:

- **Linux**: `~/.cache/storage-bindings/`
- **macOS**: `~/Library/Caches/storage-bindings/`
- **Windows**: `%LOCALAPPDATA%\storage-bindings\cache\`

The cache is organized by version and platform:

```
~/.cache/storage-bindings/
├── v0.4.4/
│   ├── linux-amd64/
│   │   ├── libstorage.a
│   │   ├── libnatpmp.a
│   │   ├── libminiupnpc.a
│   │   ├── libbacktrace.a
│   │   ├── libplum.a
│   │   └── libstorage.h
│   └── darwin-arm64/
│       └── ...
└── v0.4.3/
    └── ...
```

### Managing the Cache

#### Force Re-download

To force a fresh download without clearing the cache:

```bash
STORAGE_BINDINGS_FORCE_DOWNLOAD=1 cargo build
```

#### Clean Entire Cache

To remove all cached binaries:

```bash
# Linux/macOS
rm -rf ~/.cache/storage-bindings/

# Windows
rmdir /s /q %LOCALAPPDATA%\storage-bindings\cache

# Or using the build script
STORAGE_BINDINGS_CLEAN_CACHE=1 cargo build
```

### Supported Platforms

- Linux x86_64 (x86_64-unknown-linux-gnu)
- Linux ARM64 (aarch64-unknown-linux-gnu)
- macOS Apple Silicon (aarch64-apple-darwin)

### Libstorage Version Pinning

This crate is pinned to a stable release by default. You can override the version if needed. [See available versions here](https://github.com/logos-storage/logos-storage-nim/releases).

**Option 1: Cargo.toml metadata**

Add to your `Cargo.toml`:

```toml
[package.metadata.prebuilt]
libstorage = "v0.4.4"
```

**Option 2: Environment variable (for local overrides)**

```bash
export LOGOS_STORAGE_VERSION=v0.4.4
cargo build
```

#### Using Latest Stable Release

To use the latest stable release:

```toml
[package.metadata.prebuilt]
# Leave empty or remove the section to use latest
```

Or via environment variable:

```bash
export LOGOS_STORAGE_VERSION=latest
cargo build
```

### Building from source

```bash
cargo build --release
# or, for debug
cargo build
```

### Building using local libraries

To use locally built libraries instead of downloading from GitHub, set the `STORAGE_BINDINGS_LOCAL_LIBS` environment variable to the path of the dist folder:

```bash
export STORAGE_BINDINGS_LOCAL_LIBS=/path/to/libstorage-linux-amd64-v0.4.4-static
cargo build
```

### Testing

The library includes comprehensive integration tests that demonstrate all major functionality.

#### Running All Tests

```bash
# Run all tests (unit tests + integration tests)
cargo test
```

#### Running Specific Tests

```bash
# Run only unit tests
cargo test --lib

# Run only integration tests
cargo test --test $test_name
```

#### Available Integration Tests

- **basic_usage**: Demonstrates basic upload/download functionality
- **chunk_operations**: Shows chunk-based upload and download operations
- **debug_operations**: Demonstrates debug operations and logging
- **p2p_networking**: Shows P2P networking operations
- **storage_management**: Demonstrates storage management operations
- **two_node_network**: Shows two-node network setup and data transfer
- **thread_safe_tests**: Tests thread-safe node lifecycle and concurrent operations
- **private_queries**: Tests toggling Mix private queries

## License

[MIT](./LICENSE)
