# Architecture & Design Specification: cli_chat_embedded

**Role**: Architect Directive → Senior Engineer Blueprint  
**Revision**: 0.1.0  
**Toolchain**: Rust 1.94+ (Edition 2024)  

## Architect Directives

### Decoupling Rule
Core logic strictly interacts with hardware via `SerialPort` & `Storage` traits. No direct `std::io` or UART register access in `core/`.

### Memory Ownership Constraint
- All state is static `.bss` allocated via `MaybeUninit<AppState>`.
- Zero-copy deserialization via `#[repr(C)]` raw byte mapping.
- No `Clone` or `Drop` for persistence structs (`User`, `Message`).
- `Copy + Clone` derived only for types with `Copy` fields (`FixedString`, `User`, `Message`).

### Thread Model Requirement
- Single-threaded deterministic execution.
- `Send`/`Sync` irrelevant for core, but traits must remain object-safe for runner abstraction.
- Ownership: `&mut AppState` passed explicitly per loop iteration. No `Arc`/`Mutex`.

### Feature Gating
- `core/`: Compiles unconditionally as `#![no_std]`, zero external dependencies.
- `runner_std/`: Feature-gated via `feature = "std"`; uses `crossterm`/`termios` exclusively.
- Testing/Benchmarking: `proptest` & `criterion` in `[dev-dependencies]`, gated by `feature = "std"`.

## System Architecture
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

- **Pipeline**:  
`Input Byte` → `Minimal Line Editor` → `Guard-Checked Parser` → `State Mutation (Static)` → `ANSI Stream Output` → `Trait Write`

## Module Structure & Responsibilities

| Module | Path | Responsibility | Architect Constraint |
|--------|------|---------------|---------------------|
| `lib` | `core/src/lib.rs` | `#![no_std]`, re-exports, trait definitions | `#![allow(unsafe_code)]` |
| `types` | `core/src/types.rs` | `FixedString<const N>`, `RingBuffer<T, const N>`, `User`, `Message` | Const generics, `Copy+Clone`, zero-heap |
| `state` | `core/src/state.rs` | Command routing, guard clauses, `relative_tick` logic, minimal line editor | Early-return `&'static str` errors |
| `state::ansi` | `core/src/state/ansi.rs` | ANSI escape sequence constants | Static string slices only |
| `state::render` | `core/src/state/render.rs` | Viewport rendering and UI drawing logic | No allocation, direct trait writes |
| `storage` | `core/src/storage.rs` | Raw fixed-record serializer, header/CRC validation, factory fallback | `#[repr(C)]` layout, explicit padding |
| `serial` | `core/src/serial.rs` | Byte-stream I/O trait definition | Byte-stream only, no buffering |
| `utils` | `core/src/utils.rs` | Table-less CRC32, integer parsing/formatting | Zero-allocation utilities |
| `runner_std` | `runner_std/src/main.rs` | `std::io` bridge, non-blocking stdin, `crossterm` raw mode, file simulation | Feature-gated, CI-only |

## Concurrency & Memory Model

### Read Path
- Direct pointer cast from `.bss` slice to `&AppState`.
- Zero-copy field access via `FixedString::as_bytes()` / `as_str()`.

### Write Path
- In-place mutation of static `AppState`.
- `Storage::write()` dumps contiguous `#[repr(C)]` layout to trait sink.

### Reclamation
- No `Drop` needed for persistence structs.
- `MaybeUninit::assume_init_mut()` called exactly once during boot, guarded by `AtomicU8` boot flag.
- Static lifetime `'static` guaranteed by `.bss` placement.

### Ownership
- `&mut AppState` passed explicitly per loop iteration.
- Traits accept `&mut impl Trait` for polymorphic runners.
- No `Arc`/`Mutex`/`Rc` in core.

## API Surface Contract

```rust
// core/src/serial.rs
pub trait SerialPort {
    fn write_byte(&mut self, byte: u8);
    fn read_byte(&mut self) -> Option<u8>;
}

// core/src/storage.rs
pub trait Storage {
    fn read(&mut self, offset: usize, buffer: &mut [u8]) -> Result<usize, &'static str>;
    fn write(&mut self, offset: usize, buffer: &[u8]) -> Result<(), &'static str>;
    fn flush(&mut self) -> Result<(), &'static str>;
}

// core/src/state.rs
pub struct AppState { /* private fields including should_exit */ }

impl AppState {
    pub const fn uninit() -> Self;
    pub fn init(&mut self, storage: &mut impl Storage, port: &mut impl SerialPort) -> Result<(), &'static str>;
    pub fn process_byte(&mut self, byte: u8, port: &mut impl SerialPort, storage: &mut impl Storage) -> Result<(), &'static str>;
}
```

### Thread Safety
Not applicable. Single-threaded explicit poll. `&mut self` enforces exclusive access.

### Error Policy
- All fallible functions return `Result<(), &'static str>`.
- No panics in core.

## Documentation & Testing Strategy

### Strict rustdoc (7-Part Contract)
For ALL `pub` items:
1. **Summary**: One-sentence description.
2. **Description**: Detailed context, algorithm, behavior.
3. **Examples**: Runnable doctest.
4. **Panics**: Conditions that cause panic (if any).
5. **Errors**: When `Result::Err` is returned.
6. **Safety**: Invariants for `unsafe` blocks (if any).
7. **See Also**: Links to related items.

### CI Enforcement
```bash
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps
cargo test --doc
```

### Testing
- Unit tests: `#[cfg(test)]` in `core/`, auto-uses `std` on host.
- `proptest`: Byte-stream fuzzing for parser & state machine.
- Coverage target: ≥90% branch coverage via `cargo-llvm-cov`.

### Benchmarking
- `criterion` 0.8.2 in `core/benches/`, gated by `feature = "std"`.
- Benchmark groups: `parse_latency`, `storage_serialize`, `ansi_render`.
- Baseline: `cargo bench -- --save-baseline v0.1.0`.
- CI: `--noplot` flag for headless runs.

## Engineering Implementation Notes

### Edition Rules
- Leverage `let` chains in guard clauses (Edition 2024).
- `impl Trait` for polymorphic runners.
- `const fn` for `FixedString::new()`, `RingBuffer::new()`, `AppState::uninit()`.

### Lint Policy
```toml
[workspace.lints.rust]
missing_docs = "warn"

[workspace.lints.clippy]
pedantic = { level = "warn", priority = -1 }
cargo = { level = "warn", priority = -1 }
module_name_repetitions = "allow"
missing_panics_doc = "warn"
```

### Feature Gating
- `#[cfg(feature = "std")]` only in `runner_std/` and `core/` dev-deps.
- Core compiles identically across all targets.

### FFI/Unsafe Contracts
- `MaybeUninit` init guarded by `AtomicU8` boot flag.
- `unsafe` is globally allowed (`#![allow(unsafe_code)]`) but strictly confined to `MaybeUninit` array initialization and raw pointer casting for storage serialization.
- CRC32: Table-less bitwise polynomial (`0xEDB88320`) to keep `no_std` footprint minimal.

### Minimal Line Editor (no_std)
- Active buffer: `FixedString<256>` in `AppState`.
- Byte handling:
  - `0x20..=0x7E`: Append if not full, echo back.
  - `0x08`/`0x7F` (Backspace): Decrement len if >0, send `\x08 \x08` for visual erase.
  - `\r`/`\n`: Submit buffer as command/message, clear buffer, increment `relative_tick`.

## Senior Engineer Sign-Off

- [x] Architecture mapped to modules
- [x] Concurrency/memory model formally specified (Static .bss, Raw Fixed-Record, Relative Tick, Const Generics)
- [x] API contracts & safety boundaries defined
- [x] Validation pipeline aligned with NFRs
- [x] Documentation standard locked (7-part rustdoc, 100% coverage)
- [x] Minimal line editor spec integrated into core
