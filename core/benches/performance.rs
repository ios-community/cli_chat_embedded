//! # Summary
//! Performance benchmarking suite using criterion.
//!
//! # Description
//! Measures parse latency, storage serialization speed, and ANSI rendering overhead.
//! Ensures the core state machine meets the strict NFR-01 performance bounds.

use cli_chat_core::types::{FixedString, Message};
use cli_chat_core::{AppState, SerialPort, Storage};
use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;

/// # Summary
/// Mock serial port for benchmarking.
///
/// # Description
/// Discards all written bytes and always returns None on read to measure pure CPU overhead.
///
/// # Examples
/// ```rust,ignore
/// // Used internally by criterion benches.
/// ```
///
/// # Panics
/// Never.
///
/// # Errors
/// None.
///
/// # Safety
/// Safe.
///
/// # See Also
/// [`cli_chat_core::SerialPort`]
struct BenchSerial;

impl SerialPort for BenchSerial {
    #[inline]
    fn write_byte(&mut self, byte: u8) {}

    #[inline]
    fn read_byte(&mut self) -> Option<u8> {
        None
    }
}

/// # Summary
/// Mock storage for benchmarking.
///
/// # Description
/// Writes data to a pre-allocated static-sized array in memory to simulate fast storage I/O.
///
/// # Examples
/// ```rust,ignore
/// // Used internally by criterion benches.
/// ```
///
/// # Panics
/// Never.
///
/// # Errors
/// Returns an error if the write exceeds the buffer capacity.
///
/// # Safety
/// Safe.
///
/// # See Also
/// [`cli_chat_core::Storage`]
struct BenchStorage {
    buf: [u8; 32768],
}

impl BenchStorage {
    /// # Summary
    /// Creates a new BenchStorage instance.
    ///
    /// # Description
    /// Initializes the internal 32KB buffer with zeros.
    ///
    /// # Examples
    /// ```rust,ignore
    /// // let storage = BenchStorage::new();
    /// ```
    ///
    /// # Panics
    /// Never.
    ///
    /// # Errors
    /// None.
    ///
    /// # Safety
    /// Safe.
    ///
    /// # See Also
    /// [`BenchStorage`]
    fn new() -> Self {
        Self { buf: [0; 32768] }
    }
}

impl Storage for BenchStorage {
    #[inline]
    fn read(&mut self, _offset: usize, _buffer: &mut [u8]) -> Result<usize, &'static str> {
        Ok(0)
    }

    #[inline]
    fn write(&mut self, offset: usize, buffer: &[u8]) -> Result<(), &'static str> {
        let end = offset + buffer.len();
        if end <= self.buf.len() {
            self.buf[offset..end].copy_from_slice(buffer);
            Ok(())
        } else {
            Err("Out of memory in BenchStorage")
        }
    }

    #[inline]
    fn flush(&mut self) -> Result<(), &'static str> {
        Ok(())
    }
}

/// # Summary
/// Benchmarks the byte parsing latency.
///
/// # Description
/// Feeds a standard message string into the state machine byte-by-byte and measures the time taken.
///
/// # Examples
/// ```rust,ignore
/// // criterion_group!(benches, bench_parse_latency);
/// ```
///
/// # Panics
/// Panics if state initialization fails.
///
/// # Errors
/// None.
///
/// # Safety
/// Safe.
///
/// # See Also
/// [`cli_chat_core::AppState::process_byte`]
fn bench_parse_latency(criterion: &mut Criterion) {
    let mut state = AppState::uninit();
    let mut port = BenchSerial;
    let mut storage = BenchStorage::new();
    state.init(&mut storage, &mut port).unwrap();

    let input = b"Hello, this is a benchmark message!\r";

    criterion.bench_function("parse_latency", |bench| {
        bench.iter(|| {
            for &byte in input {
                state
                    .process_byte(black_box(byte), &mut port, &mut storage)
                    .unwrap();
            }
        });
    });
}

/// # Summary
/// Benchmarks the storage serialization speed.
///
/// # Description
/// Measures the time required to serialize the entire AppState and write it to the mock storage.
///
/// # Examples
/// ```rust,ignore
/// // criterion_group!(benches, bench_storage_serialize);
/// ```
///
/// # Panics
/// Panics if state initialization fails.
///
/// # Errors
/// None.
///
/// # Safety
/// Safe.
///
/// # See Also
/// [`cli_chat_core::AppState::flush`]
fn bench_storage_serialize(criterion: &mut Criterion) {
    let mut state = AppState::uninit();
    let mut port = BenchSerial;
    let mut storage = BenchStorage::new();
    state.init(&mut storage, &mut port).unwrap();

    for &byte in b"/user 1\r/name BenchUser\r" {
        state.process_byte(byte, &mut port, &mut storage).unwrap();
    }

    criterion.bench_function("storage_serialize", |bench| {
        bench.iter(|| {
            state.flush(black_box(&mut storage)).unwrap();
        });
    });
}

/// # Summary
/// Benchmarks the ANSI rendering overhead.
///
/// # Description
/// Measures the time taken to generate and write ANSI escape sequences for the prompt and a message.
///
/// # Examples
/// ```rust,ignore
/// // criterion_group!(benches, bench_ansi_render);
/// ```
///
/// # Panics
/// Panics if state initialization fails.
///
/// # Errors
/// None.
///
/// # Safety
/// Safe.
///
/// # See Also
/// [`cli_chat_core::state::render`]
fn bench_ansi_render(criterion: &mut Criterion) {
    let mut state = AppState::uninit();
    let mut port = BenchSerial;
    let mut storage = BenchStorage::new();
    state.init(&mut storage, &mut port).unwrap();

    let msg = Message {
        user_id: 1,
        padding: 0,
        tick: 100,
        content: FixedString::new(),
    };

    criterion.bench_function("ansi_render", |bench| {
        bench.iter(|| {
            cli_chat_core::state::render::render_prompt(black_box(&state), &mut port);
            cli_chat_core::state::render::render_message(
                black_box(&state),
                black_box(&msg),
                &mut port,
            );
        });
    });
}

criterion_group!(
    benches,
    bench_parse_latency,
    bench_storage_serialize,
    bench_ansi_render
);
criterion_main!(benches);
