# cli_chat_embedded

[![CI](https://github.com/ios-community/cli_chat_embedded/actions/workflows/ci.yml/badge.svg)](https://github.com/ios-community/cli_chat_embedded/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/cli_chat_core.svg)](https://crates.io/crates/cli_chat_core)
[![docs.rs](https://img.shields.io/docsrs/cli_chat_core)](https://docs.rs/cli_chat_core)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

A pure `#![no_std]`, zero-heap, single-threaded CLI chat state machine designed for resource-constrained embedded environments (bare-metal).

## Features

- **Zero-Heap Allocation** \
Strictly `#![no_std]` with no `alloc` crate required. Uses const-generics for bounded memory.
- **Raw Fixed-Record Storage** \
Direct binary serialization with table-less CRC32 validation.
- **ANSI Split-Screen Viewport** \
Built-in terminal UI rendering using DECSTBM and SGR colors.
- **Host Simulator** \
Includes `runner_std`, a host simulator using `crossterm` and `std::fs` to test the core logic on your PC.
- **Mirrored Repositories** \
Maintained on both [GitHub](https://github.com/ios-community/cli_chat_embedded) and [Codeberg](https://codeberg.org/ios-community/cli_chat_embedded).

## Installation

Add to `Cargo.toml`:

```toml
[dependencies]
cli_chat_core = "0.1.0"
```

## Quick Start

```rust
use cli_chat_core::{AppState, SerialPort, Storage};
use core::mem::MaybeUninit;

// 1. Allocate state in .bss
static mut STATE: MaybeUninit<AppState> = MaybeUninit::uninit();

// 2. Initialize and run the polling loop
unsafe {
    let state = STATE.write(AppState::uninit());
    
    // Assuming `storage` and `port` are your hardware-specific implementations
    state.init(&mut storage, &mut port).unwrap();
    
    // In your hardware interrupt or polling loop:
    // if let Some(byte) = port.read_byte() {
    //     state.process_byte(byte, &mut port, &mut storage).unwrap();
    // }
}
```

### Available Commands
Once running, use the following commands in the prompt:
- `/help` : Show available commands.
- `/user <id>` or `/switch <id>` : Switch to a specific user ID.
- `/name <str>` : Set the display name for the current user.
- `/users` : List all active registered users.
- `/status` : Show system uptime and history buffer usage.
- `/time` : Show boot epoch and relative tick information.
- `/ping` : Check system responsiveness.
- `/about` : Show application and license information.
- `/clear` : Clear the screen and redraw the viewport.
- `/exit` : Gracefully exit the application (in host simulator).

## Architecture

```text
[ runner_std (bin) ]          [ core (lib) ]              [ target hardware ]
   │                             │                             │
   ├─ impl SerialPort ◄──────────┼── trait SerialPort ─────────┼─ UART / Console
   ├─ impl Storage ◄─────────────┼── trait Storage ────────────┼─ Flash / EEPROM
   ├─ ANSI Split-Screen Init     │                             │
   └─ Non-blocking stdin poll ──►├─ state.process_byte()       │
                                 ├─ minimal line editor        │
                                 ├─ cmd dispatch (guard)       │
                                 ├─ render history & prompt    │
                                 └─ fixed-record load/save     │
```

- **Pipeline**: `Input Byte` → `Minimal Line Editor` → `Guard-Checked Parser` → `State Mutation (Static)` → `ANSI Stream Output` → `Trait Write`
- **Memory**: Static `.bss` allocation via `MaybeUninit<AppState>`.
- **Storage**: Raw fixed-record binary layout with CRC32 validation.

## Performance (v0.1.0 Baseline)

| Operation | Time |
|-----------|------|
| `parse_latency` | **~76.5 µs** |
| `storage_serialize` | **~76.0 µs** |
| `ansi_render` | **~270.2 ns** |

*Tested on Microsoft Windows 11 IoT Enterprise (Build 26200), LENOVO 2347B16 (Intel Core i5-3320M @ 2.60GHz), Rust 1.95.0 (59807616e 2026-04-14) (Rev5, Built by MSYS2 project). Meets strict NFR-01 performance bounds.*

## Safety & MSRV

- **MSRV** \
`1.94.0` | Edition: `2024`
- **Public API** \
100% safe. Fallible operations return `Result` with static string slices. No panics under normal operation.
- **Concurrency** \
Single-threaded deterministic execution. `&mut self` enforces exclusive access.
- **Memory** \
Zero-heap allocation. `unsafe` confined to `MaybeUninit` array initialization and pointer math for storage serialization.

## License

Distributed under the MIT License. See [LICENSE](LICENSE) for details.

## Coding Standards

- **Safety** \
`unsafe` blocks must include `// SAFETY:` comments explaining invariants.
- **Documentation** \
All `pub` items follow `Summary → Description → Examples → Panics → Errors → Safety → See Also`.
- **Testing** \
Unit tests for logic, `proptest` for fuzzing byte parser, `criterion` for performance regression.

## Out of Scope

- Dynamic heap allocation (`Box`, `Vec`, `String`, `Arc`, `Mutex`).
- TOML/JSON/XML parsing in `core/`.
- Multi-session networking or WebSocket.
- Async runtimes (`tokio`, `async-std`).
