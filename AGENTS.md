# AGENTS.md

Guidelines for agentic coding agents working in the riskie codebase.

## Project Overview

riskie is a Rust-based disk automounting daemon for Linux. It provides:
- Automatic mounting of removable devices via udisks2 D-Bus
- System tray interface using ksni (StatusNotifierItem)
- Desktop notifications via notify-rust
- Mount/unmount/eject functionality for devices

## Build Commands

```bash
# Check compilation without building (FAST - use this during development)
cargo check

# Run linter (must pass before committing)
cargo clippy --all-targets --all-features -- -D warnings

# Format code
cargo fmt

# Build binary for testing (requires D-Bus and system tray support)
cargo build --release

# Run release binary directly
./target/release/riskie
```

**CRITICAL - BUILD POLICY**:
- **NEVER** run `cargo build` or `cargo build --release` during development
- **ALWAYS** use `cargo check` and `cargo clippy` for correctness verification
- Building is slow and unnecessary - `cargo check` verifies compilation without producing binaries
- Only build when the user explicitly asks to test the actual binary on their system

## Test Commands

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run tests with verbose output
cargo test -- --nocapture
```

Note: This project has limited unit tests. Most functionality requires a running D-Bus session and udisks2 daemon.

## Code Style Guidelines

### Imports

Import ordering follows `rustfmt.toml`:
1. Standard library (`std::`)
2. External crates (alphabetically)
3. Module imports (`mod`, `use crate::`)

```rust
use std::sync::{Arc, RwLock};

use anyhow::Result;
use futures::StreamExt;
use tracing::{error, info};
use zbus::Connection;

mod notify;
mod tray;
mod udisks2;
```

### Formatting

- Use `cargo fmt` before committing
- Max line length: default (100 chars)
- Use 4-space indentation
- Match arms align with `=>`

### Types and Naming

- Use `Result<T, anyhow::Error>` for fallible operations in binary code
- Use `thiserror` for library-style error types (if added)
- Async functions use `async fn` with `-> Result<T>`
- Helper types use descriptive names: `TrayCommand`, `TrayHandle`, `Device`
- Constants use `SCREAMING_SNAKE_CASE`

### Error Handling

- Never use `.unwrap()` or `.expect()` in production code
- Use proper `match` or `let Ok(...) = ... else { return }` pattern
- Log errors with context using `tracing::error!`
- Use `anyhow::Result` for main binary error handling

```rust
// Good: proper error handling
let guard = match devices.read() {
    Ok(g) => g,
    Err(e) => {
        error!("Failed to acquire read lock: {}", e);
        return;
    }
};

// Bad: panics on error
let guard = devices.read().unwrap();
```

### Async Patterns

- Use `tokio::task::spawn_blocking` for blocking operations (e.g., notify-rust)
- Release locks before `.await` points to avoid blocking the runtime
- Use `std::sync::RwLock` for sync code, release before async calls

```rust
// Good: release lock before await
let data = {
    let guard = devices.read()?;
    guard.clone()
};
some_async_operation(data).await;
```

### D-Bus / zbus Patterns

- Use `zbus::proxy` attribute for interface definitions
- Byte arrays (`ay`) require special handling with `zvariant::Array`
- Object paths use `zbus::zvariant::ObjectPath`
- Use `OwnedValue` for extracted properties

### Logging

- Use `tracing` crate with macros: `info!`, `error!`, `debug!`, `warn!`
- Initialize with `tracing_subscriber::fmt::init()` in main
- Include context in log messages: `"Failed to mount {}: {}"`

### Comments

- No comments explaining *what* code does (code should be self-documenting)
- Use `///` doc comments for public items
- Use `// TODO(#issue):` for TODOs requiring follow-up

## Project Structure

```
src/
├── main.rs      - Entry point, event loop, device tracking
├── tray.rs      - System tray implementation (ksni)
├── udisks2.rs   - D-Bus udisks2 client
└── notify.rs    - Desktop notifications (notify-rust)
```

## Key Dependencies

| Crate | Purpose |
|-------|---------|
| tokio | Async runtime |
| zbus | D-Bus bindings for udisks2 |
| ksni | StatusNotifierItem system tray |
| notify-rust | Desktop notifications |
| tracing | Logging |
| anyhow | Error handling |

## Development Workflow

1. Make changes
2. Run `cargo fmt`
3. Run `cargo clippy --all-targets --all-features -- -D warnings`
4. Commit with conventional commit message format

## Commit Message Format

Use conventional commits:
- `feat:` for new features
- `fix:` for bug fixes
- `refactor:` for code refactoring
- `docs:` for documentation
- `chore:` for build/tooling changes