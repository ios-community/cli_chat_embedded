# Requirements Specification: cli_chat_embedded

**Role**: Architect Engineer → Senior Software Engineer  
**Status**: Frozen  
**Target Registry**: GitHub / Codeberg / crates.io  
**MSRV**: Rust 1.94+ | **Edition**: 2024  

## Architect Directives

### Primary Objective
Deliver a pure `#![no_std]`, zero-heap, single-session CLI chat core with deterministic memory layout, abstracted I/O, and native host simulation capability.

### Crate Structure
- `core/`: `lib` crate, `#![no_std]`, zero external dependencies
- `runner_std/`: `bin` crate, `std`-based host simulator with `crossterm`/`std::fs`

### Concurrency Model
- Single-threaded, explicit non-blocking byte polling
- No async runtime, no background threads, no `Arc`/`Mutex`

### Memory Safety Constraints
- Zero heap allocation (`Vec`, `String`, `Box`, `Arc` forbidden in core)
- Static `.bss` allocation via `core::mem::MaybeUninit`
- Strict `#![no_std]` in `core/`
- `unsafe` is globally allowed (`#![allow(unsafe_code)]`) to facilitate zero-copy serialization and uninitialized memory management.

### Portability Targets
| Target Triple | Device Example |
|--------------|----------------|
| `riscv32imac-unknown-none-elf` | Milk-V Duo |
| `armv6-unknown-none-eabihf` | Raspberry Pi Zero |
| `armv7-unknown-none-eabihf` | Onion Omega2+ |

### Documentation Contract
- `rustdoc -D warnings`, 100% public API doc coverage
- Strict 7-part rustdoc structure for all `pub` items
- `clippy::pedantic` enforced

## Functional Requirements (FR)

| ID | Requirement | Owner | Description |
|----|-------------|-------|-------------|
| FR-01 | Static State Allocation | Core | `AppState` dialokasikan di `.bss` via `MaybeUninit`. Zero stack pressure. Init guard eksplisit via `AtomicU8` boot flag. |
| FR-02 | Raw Fixed-Record Storage | Core/Runner | Serialisasi biner langsung tanpa parser TLV. Layout: `[Magic: b"EMBU"][Ver: u8][Epoch: u32][UserBlock][HistoryBlock][CRC32]`. Soft-delete via `User.status: u8` (`0`=active, `1`=deleted). |
| FR-03 | Relative Tick Timestamping | Core | Timestamp berbasis `u32 relative_tick` inkremental per boot. Header menyimpan `u32 boot_epoch` untuk persistence lintas reboot. Increment on `\r`/`\n` submit. |
| FR-04 | ANSI Split-Screen Viewport | Core | Render UI via `SerialPort` trait menggunakan DECSTBM `\x1b[r`, SGR colors, dan cursor lock di baris input. |
| FR-05 | Minimal Line Editor (no_std) | Core | Implementasi line editor internal di `core/`: handle normal char (0x20-0x7E), backspace (`\x08`/`\x7F` → send `\x08 \x08`), submit (`\r`/`\n`). Buffer: `FixedString<256>`. |

## Non-Functional Requirements (NFR)

| ID | Category | Constraint | Validation Method |
|----|----------|------------|------------------|
| NFR-01 | Performance | Parse latency ≤ 100μs/byte, storage write ≤ 100μs/full-state, ANSI render ≤ 500ns | `criterion` benchmarks: `parse_latency`, `storage_serialize`, `ansi_render` |
| NFR-02 | Safety | Zero heap alloc, stack usage < 4KB, `.bss` ≤ 28 KB | `cargo test`, `stack-sizes`, `cargo bloat`, `objdump` |
| NFR-03 | Documentation | 0 warnings, 100% public API doc coverage, strict 7-part rustdoc, runnable doctest | `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps`, `cargo test --doc` |
| NFR-04 | Compatibility | MSRV 1.94+, Edition 2024, `no_std` core isolation, feature-gated `std` runner | `cargo check --all-features`, workspace dependency audit |

## Performance Targets (Validated v0.1.0)

| Component | Metric | Bound | Actual (Win11, i5-3320M) |
|-----------|--------|-------|--------------------------|
| Byte Parser | Latency per input byte | ≤ 100μs | ~76.5 μs |
| Storage Serialize | Fixed-record dump (full state) | ≤ 100μs | ~76.0 μs |
| ANSI Render | Viewport update overhead | ≤ 500ns | ~270.2 ns |

---

## Static Bounds (Compile-Time Constants)

| Constant | Value | Purpose |
|----------|-------|---------|
| `MAX_USERS` | `256` | Size of static user registry array |
| `MAX_HISTORY` | `64` | Capacity of circular message buffer |
| `USER_STRING_CAPACITY` | `32` | Max bytes for `User.name` (`FixedString<32>`) |
| `MESSAGE_STRING_CAPACITY` | `256` | Max bytes for `Message.content` (`FixedString<256>`) |
| `INPUT_BUFFER_CAPACITY` | `256` | Max bytes for active line editor buffer |

## Binary Storage Layout Specification
```text
[Header]
├─ Magic: [u8; 4] = b"EMBU"
├─ Version: u8
├─ boot_epoch: u32 (host clock at init)

[UserBlock]
├─ [User; MAX_USERS] with #[repr(C)] layout:
   ├─ id: u16
   ├─ status: u8 (0=active, 1=soft-deleted)
   ├─ padding: u8 (4-byte alignment)
   └─ name: FixedString<32>

[HistoryBlock]
├─ RingBuffer<Message, MAX_HISTORY> serialized as contiguous #[repr(C)] array:
   ├─ user_id: u16
   ├─ padding: u16 (4-byte alignment)
   ├─ tick: u32 (relative_tick at creation)
   └─ content: FixedString<256>

[Footer]
└─ CRC32: u32 (polynomial 0xEDB88320, table-less bitwise implementation)
```

- **CRC Failure Handling**: If CRC32 invalid at boot → factory reset (zero-initialized state) + emit ANSI warning via `SerialPort`.

## Out of Scope

- ❌ Dynamic heap allocation (`Box`, `Vec`, `String`, `Arc`, `Mutex`)
- ❌ TOML/JSON/XML parsing in `core/`
- ❌ Multi-session networking or WebSocket
- ❌ Async runtimes (`tokio`, `async-std`)
- ❌ OS-level signal handling or background threads in `core/`
- ❌ `Drop` semantics for persistence structs (`User`, `Message`)

## Architect Sign-Off

- [x] Constraints finalized (Static .bss, Raw Fixed-Record, Relative Tick, Const Generics)
- [x] Performance bounds defined and validated
- [x] Safety & compliance gates locked (pedantic clippy, zero-heap)
- [x] Documentation standard locked (7-part rustdoc, 100% coverage)
