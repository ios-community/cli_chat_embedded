# Task Breakdown & Traceability: cli_chat_embedded

**Role**: Architect Oversight → Senior Engineer Execution  
**Methodology**: Spec-Driven Development (SDD) Phased Execution  

## Phase Traceability Matrix

| ID | Task | Phase | Acceptance Criteria | Links | Status | Owner |
|----|------|-------|-------------------|-------|--------|-------|
| T-01 | Project Init & Config | Setup | Workspace `core`/`runner_std`, MSRV 1.94+, Edition 2024, `#![no_std]` core, lint config active, feature gating ready | NFR-04 | ✅ | Senior Eng |
| T-02 | Core Primitives & Static State | Core | `FixedString<const N>`, `RingBuffer<T, const N>`, `AppState` in `.bss`, `Copy+Clone` derives, zero-heap proof, const generics for all bounds | FR-01, NFR-02 | ✅ | Senior Eng |
| T-03 | Raw Fixed-Record Storage | Core | Header `[Magic: b"EMBU"][Ver: u8][Epoch: u32]`, `UserBlock`/`HistoryBlock` with `#[repr(C)]`, CRC32 (table-less), soft-delete via `User.status`, factory fallback on CRC fail | FR-02, NFR-02 | ✅ | Senior Eng |
| T-04 | Serial & ANSI Viewport | Core | `SerialPort` trait, DECSTBM init, SGR colors, cursor lock, byte-by-byte render, ANSI warning on factory reset | FR-04, NFR-01 | ✅ | Senior Eng |
| T-05 | Command Routing & Tick Logic | Core | Minimal line editor in core, guard-clause dispatch, `relative_tick` increment on submit, `should_exit` flag, boot_epoch sync via handshake | FR-01, FR-03, FR-05 | ✅ | Senior Eng |
| T-06 | Testing & Validation | Validation | Roundtrip binary tests, `proptest` fuzz, coverage ≥90%, 100% doctest pass | NFR-02, NFR-03 | ✅ | Senior Eng |
| T-07 | Benchmarking & CI | Performance | `criterion` baselines, `--noplot` CI, regression ≤5%, feature-gated `std` for benches | NFR-01 | ✅ | Senior Eng |
| T-08 | Release & Cross-Compile | Release | `runner_std` demo, `cargo build --target riscv32imac...`, docs clean, tag `v0.1.0`, publish checklist complete | NFR-04 | ✅ | Senior Eng |

## Validation Sequence (Per Phase)

```bash
# Type & ownership validation
cargo check --workspace

# Unit/integration/doctest
cargo test --all-features

# Performance regression check
cargo bench -- --noplot

# Documentation strictness
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps

# Lint compliance
cargo clippy --all-features -- -D warnings

# Formatting
cargo fmt --check
```

## Branching & Merge Policy

### Branch Naming
- `feat/<scope>`: New feature implementation
- `fix/<issue>`: Bug fix
- `perf/<benchmark>`: Performance optimization
- `chore/<maintenance>`: Tooling/docs maintenance

### PR Requirements
- [x] Pass full validation sequence
- [x] Attach benchmark delta vs baseline
- [x] Documentation updated with 7-part rustdoc
- [x] Doctest coverage 100% for new public APIs

### Merge Strategy
- Squash merge after Architect review & CI green
- Commit messages follow [Conventional Commits](https://www.conventionalcommits.org/)

## Performance & Regression Guardrails

### Baseline Management
```bash
# Pin baseline per release
cargo bench -- --save-baseline v0.1.0
```

### Regression Response
- Regression >5% → revert, profile with `cargo flamegraph` / `perf` / `valgrind`
- Memory/CPU contention (`std. dev.` >15%) → adjust chunk/alignment/lock params

### Dependency Hygiene
- Monthly: `cargo update` + `cargo audit`
- Block any dependency that introduces heap allocation in `core/`

## Rollout & Publish Checklist

### Pre-Publish Validation
- [x] Metadata valid (`Cargo.toml`: name, version, description, license, repository, keywords, categories)
- [x] `README.md` matches public API & includes embedded usage examples
- [x] `CHANGELOG.md` follows [Keep a Changelog](https://keepachangelog.com/)
- [x] License file matches `Cargo.toml` (`MIT`, year 2026, Dzulkifli Anwar)
- [x] Docs built clean (`rustdoc` 0 warnings, 100% doctest pass)
- [x] Tests pass (`proptest` + `criterion` stable)
- [x] Bench stable (vs baselines, CI `--noplot` output attached)
- [x] Clippy clean (`-D warnings`, pedantic policy respected)

### Publish Steps
- [x] Tag & push → GitHub/Codeberg (`git tag -a v0.1.0 -m "Release v0.1.0"`)
- [x] `cargo publish --dry-run` → validate
- [x] `cargo publish` → crates.io
- [x] Update Codeberg release notes with changelog

## Architect Review Notes (Active)

### `MaybeUninit` Guard Pattern
```rust
static mut STATE: MaybeUninit<AppState> = MaybeUninit::uninit();
static BOOT_FLAG: AtomicU8 = AtomicU8::new(0); // 0=uninit, 1=ready

// In main():
if BOOT_FLAG.load(Ordering::Acquire) == 1 {
    return Err("Already initialized");
}
// ... perform init ...
BOOT_FLAG.store(1, Ordering::Release);
```
→ Never call `assume_init_mut()` before `init()` completes successfully.

### Binary Layout Alignment
Ensure `#[repr(C)]` with explicit padding in `User` and `Message`:
```rust
#[repr(C)]
pub struct User {
    pub id: u16,      // 2 bytes
    pub status: u8,   // 1 byte
    pub padding: u8,  // 1 byte padding → 4-byte aligned
    pub name: FixedString<32>, // 32 bytes
} // Total: 36 bytes, consistent across RISC-V/ARM
```

### Relative Tick Increment Strategy
- Increment `relative_tick` on `\r`/`\n` submission (command/message complete).
- `boot_epoch` updated only on successful `Storage::flush()` after full state write.

### CRC32 Implementation
- Polynomial: `0xEDB88320` (standard IEEE 802.3).
- Table-less bitwise shift to keep footprint minimal.
- Validate CRC at `AppState::init()`. On mismatch:
  1. Zero-initialize state (factory reset).
  2. Emit ANSI warning via `SerialPort`: `"\x1b[33m[WARNING] CRC mismatch: factory reset applied\x1b[0m\r\n"`.
  3. Proceed with clean state.

## Status Legend
- ✅ Completed & validated
- 🔲 In progress / pending execution
- ❌ Blocked / requires architect decision
```
