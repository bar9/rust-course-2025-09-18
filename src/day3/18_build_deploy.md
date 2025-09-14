# Chapter 18: Build, Package & Deploy

## Learning Objectives
- Master Cargo workspaces for multi-crate projects
- Use features for conditional compilation
- Set up cross-compilation for different targets
- Create and publish crates to crates.io
- Implement CI/CD pipelines
- Optimize binary size and build times

## Cargo Workspaces

Workspaces allow you to manage multiple related packages together.

### Creating a Workspace

```toml
# Cargo.toml (workspace root)
[workspace]
members = [
    "core",
    "cli",
    "server",
]

# Shared dependencies
[workspace.dependencies]
serde = { version = "1.0", features = ["derive"] }
tokio = { version = "1.35", features = ["full"] }

# Share compilation profile
[profile.release]
lto = true
opt-level = 3
```

### Member Crates

```toml
# core/Cargo.toml
[package]
name = "myproject-core"
version = "0.1.0"

[dependencies]
serde = { workspace = true }

# cli/Cargo.toml
[package]
name = "myproject-cli"
version = "0.1.0"

[dependencies]
myproject-core = { path = "../core" }
tokio = { workspace = true }
```

### Workspace Commands

```bash
# Build all workspace members
cargo build --workspace

# Test specific package
cargo test -p myproject-core

# Run specific binary
cargo run -p myproject-cli

# Check all members
cargo check --workspace --all-targets
```

## Features and Conditional Compilation

Features allow optional functionality and dependencies.

### Defining Features

```toml
# Cargo.toml
[features]
default = ["json"]
json = ["serde", "serde_json"]
async = ["tokio", "async-trait"]
full = ["json", "async", "metrics"]

# Optional dependencies
[dependencies]
serde = { version = "1.0", optional = true }
serde_json = { version = "1.0", optional = true }
tokio = { version = "1.35", optional = true }
async-trait = { version = "0.1", optional = true }
```

### Using Features in Code

```rust
// Conditional compilation
#[cfg(feature = "json")]
use serde::{Serialize, Deserialize};

#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub struct Config {
    pub name: String,
    pub port: u16,
}

// Feature-gated modules
#[cfg(feature = "async")]
pub mod async_client {
    use tokio::net::TcpStream;
    
    pub async fn connect(addr: &str) -> Result<TcpStream, std::io::Error> {
        TcpStream::connect(addr).await
    }
}

// Platform-specific code
#[cfg(target_os = "windows")]
fn platform_specific() {
    println!("Running on Windows");
}

#[cfg(target_os = "linux")]
fn platform_specific() {
    println!("Running on Linux");
}
```

## Cross-Compilation

Build for different platforms from a single machine.

### Setting Up Targets

```bash
# Install target toolchains
rustup target add x86_64-pc-windows-gnu
rustup target add aarch64-unknown-linux-gnu
rustup target add wasm32-unknown-unknown

# List installed targets
rustup target list --installed
```

### Cross-Compiling

```bash
# Build for Windows from Linux/Mac
cargo build --target x86_64-pc-windows-gnu

# Build for ARM Linux
cargo build --target aarch64-unknown-linux-gnu

# Build for WebAssembly
cargo build --target wasm32-unknown-unknown
```

### Cross Configuration

```toml
# .cargo/config.toml
[target.x86_64-pc-windows-gnu]
linker = "x86_64-w64-mingw32-gcc"

[target.aarch64-unknown-linux-gnu]
linker = "aarch64-linux-gnu-gcc"

# Set default target
[build]
target = "x86_64-unknown-linux-musl"
```

## Publishing Crates

### Preparing for Publication

```toml
# Cargo.toml
[package]
name = "awesome-crate"
version = "0.1.0"
authors = ["Your Name <you@example.com>"]
edition = "2021"
description = "A brief description of your crate"
documentation = "https://docs.rs/awesome-crate"
homepage = "https://github.com/yourusername/awesome-crate"
repository = "https://github.com/yourusername/awesome-crate"
license = "MIT OR Apache-2.0"
keywords = ["networking", "async", "protocol"]
categories = ["network-programming", "asynchronous"]

[badges]
maintenance = { status = "actively-developed" }
```

### Publishing Process

```bash
# Login to crates.io
cargo login YOUR_API_TOKEN

# Verify package
cargo package --list

# Dry run
cargo publish --dry-run

# Publish
cargo publish

# Yank a version (emergency)
cargo yank --vers 0.1.0
```

## Binary Size Optimization

### Release Profile Optimization

```toml
# Cargo.toml
[profile.release]
opt-level = "z"     # Optimize for size
lto = true          # Link-time optimization
codegen-units = 1   # Single codegen unit
strip = true        # Strip symbols
panic = "abort"     # Smaller panic handler

[profile.release-small]
inherits = "release"
opt-level = "s"
```

### Reducing Dependencies

```rust
// Use no_std when possible
#![no_std]

// Avoid large dependencies
// Instead of:
// use regex::Regex;

// Consider:
use simple_pattern_match;

// Feature-gate heavy dependencies
#[cfg(feature = "full")]
use heavy_dependency;
```

## CI/CD with GitHub Actions

### Basic Rust CI Pipeline

```yaml
# .github/workflows/ci.yml
name: CI

on:
  push:
    branches: [ main ]
  pull_request:
    branches: [ main ]

env:
  CARGO_TERM_COLOR: always

jobs:
  test:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        rust: [stable, beta, nightly]
    steps:
    - uses: actions/checkout@v3
    
    - name: Install Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: ${{ matrix.rust }}
        override: true
        components: rustfmt, clippy
    
    - name: Cache cargo
      uses: actions/cache@v3
      with:
        path: |
          ~/.cargo/registry
          ~/.cargo/git
          target
        key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
    
    - name: Check formatting
      run: cargo fmt -- --check
    
    - name: Clippy
      run: cargo clippy -- -D warnings
    
    - name: Test
      run: cargo test --verbose
    
    - name: Build
      run: cargo build --release

  cross-compile:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        target: [x86_64-pc-windows-gnu, aarch64-unknown-linux-gnu]
    steps:
    - uses: actions/checkout@v3
    - uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
        target: ${{ matrix.target }}
    - run: cargo build --target ${{ matrix.target }}
```

### Release Pipeline

```yaml
# .github/workflows/release.yml
name: Release

on:
  release:
    types: [created]

jobs:
  publish:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v3
    - uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
    
    - name: Publish to crates.io
      run: cargo publish --token ${{ secrets.CRATES_TOKEN }}
    
    - name: Build binaries
      run: |
        cargo build --release --target x86_64-unknown-linux-musl
        cargo build --release --target x86_64-pc-windows-gnu
        cargo build --release --target x86_64-apple-darwin
    
    - name: Upload artifacts
      uses: actions/upload-release-asset@v1
      with:
        upload_url: ${{ github.event.release.upload_url }}
        asset_path: ./target/release/myapp
        asset_name: myapp-linux
        asset_content_type: application/octet-stream
```

## Docker Containerization

### Multi-stage Dockerfile

```dockerfile
# Build stage
FROM rust:1.75 as builder

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src

# Build in release mode
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/myapp /usr/local/bin/myapp

EXPOSE 8080
CMD ["myapp"]
```

### Minimal Alpine Image

```dockerfile
# Build with musl for static linking
FROM rust:1.75-alpine as builder

RUN apk add --no-cache musl-dev

WORKDIR /app
COPY . .

RUN cargo build --release --target x86_64-unknown-linux-musl

# Minimal runtime
FROM scratch

COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/myapp /myapp

EXPOSE 8080
ENTRYPOINT ["/myapp"]
```

## Comparison with C++/.NET

| Aspect | Rust (Cargo) | C++ | .NET |
|--------|--------------|-----|------|
| Package Manager | Built-in (Cargo) | External (vcpkg, conan) | Built-in (NuGet) |
| Build System | Cargo | CMake, Make, Bazel | MSBuild |
| Cross-compilation | Native support | Complex setup | Limited |
| Docker size | Can be <10MB | Usually >100MB | >100MB |
| CI/CD | Simple YAML | Complex scripts | Azure DevOps |

## Exercises

### Exercise 17.1: Create a Workspace
Create a workspace with three crates:
- `common`: Shared types and utilities
- `server`: HTTP server using the common crate
- `client`: CLI client using the common crate

### Exercise 17.2: Feature Flags
Add these features to your crate:
- `metrics`: Enable performance metrics
- `tls`: Enable TLS support
- `compression`: Enable response compression

### Exercise 17.3: CI Pipeline
Create a GitHub Actions workflow that:
1. Runs tests on Linux, Windows, and macOS
2. Checks code formatting
3. Runs clippy
4. Builds for multiple targets
5. Caches dependencies

## Key Takeaways

✅ **Workspaces simplify multi-crate projects** - Share dependencies and compilation

✅ **Features enable conditional compilation** - Ship different configurations

✅ **Cross-compilation is straightforward** - Build for any target from any host

✅ **Publishing to crates.io is simple** - One command to share with the world

✅ **CI/CD with GitHub Actions is powerful** - Automate testing and releases

✅ **Docker images can be tiny** - Static binaries in scratch containers

---

Next: [Chapter 19: Capstone Project](./19_capstone.md)