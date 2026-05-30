# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-05-30

### Added
- **Core State Machine** \
Pure `#![no_std]`, zero-heap, single-threaded chat core (`cli_chat_core`).
- **Static Data Structures** \
`FixedString<const N>` and `RingBuffer<T, const N>` for `.bss` allocation.
- **Raw Fixed-Record Storage** \
Direct binary serialization with table-less CRC32 validation and factory reset fallback.
- **ANSI Split-Screen Viewport** \
Built-in terminal UI rendering using DECSTBM, SGR colors, and cursor locking.
- **Minimal Line Editor** \
Internal byte-by-byte parser with backspace support and visual erase.
- **Command Routing** \
Built-in support for `/user`, `/name`, and message dispatching.
- **Host Simulator** \
`runner_std` binary using `crossterm` and `std::fs::File` to simulate the embedded environment on a host OS.
- **CI/CD Pipelines** \
Automated validation for formatting, clippy, tests, benchmarks, and cross-compilation (RISC-V) via GitHub Actions and Forgejo.
