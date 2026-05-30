//! # Summary
//! Unit tests, proptest fuzzing, and validation suite.
//!
//! # Description
//! Contains roundtrip binary tests, state machine validation, and fuzzing
//! for the byte parser to ensure zero panics and memory safety.

#![cfg(test)]

extern crate std;

use crate::serial::SerialPort;
use crate::state::AppState;
use crate::storage::Storage;
use crate::types::{FixedString, RingBuffer};
use proptest::collection::vec;
use proptest::prelude::*;
use std::cmp::min;
use std::vec::Vec;

/// Mock implementation of SerialPort for testing.
struct MockSerial {
    pub output: Vec<u8>,
}

impl MockSerial {
    fn new() -> Self {
        Self { output: Vec::new() }
    }
}

impl SerialPort for MockSerial {
    fn write_byte(&mut self, byte: u8) {
        self.output.push(byte);
    }

    fn read_byte(&mut self) -> Option<u8> {
        None
    }
}

/// Mock implementation of Storage for testing.
struct MockStorage {
    pub data: Vec<u8>,
}

impl MockStorage {
    fn new() -> Self {
        Self { data: Vec::new() }
    }
}

impl Storage for MockStorage {
    fn read(&mut self, offset: usize, buffer: &mut [u8]) -> Result<usize, &'static str> {
        if offset >= self.data.len() {
            return Ok(0);
        }

        let end = min(offset + buffer.len(), self.data.len());
        let len = end - offset;
        buffer[..len].copy_from_slice(&self.data[offset..end]);
        Ok(len)
    }

    fn write(&mut self, offset: usize, buffer: &[u8]) -> Result<(), &'static str> {
        let end = offset + buffer.len();
        if end > self.data.len() {
            self.data.resize(end, 0);
        }
        self.data[offset..end].copy_from_slice(buffer);
        Ok(())
    }

    fn flush(&mut self) -> Result<(), &'static str> {
        Ok(())
    }
}

#[test]
fn test_fixed_string() {
    let mut string = FixedString::<5>::new();
    assert!(string.is_empty());
    assert!(string.push(b'H'));
    assert!(string.push(b'e'));
    assert_eq!(string.len(), 2);
    assert_eq!(string.as_str().unwrap(), "He");
    assert_eq!(string.pop(), Some(b'e'));
    assert_eq!(string.len(), 1);
    assert_eq!(string.as_str().unwrap(), "H");
}

#[test]
fn test_ring_buffer() {
    let mut ring_buffer = RingBuffer::<u32, 3>::new();
    assert!(ring_buffer.is_empty());
    ring_buffer.push(1);
    ring_buffer.push(2);
    ring_buffer.push(3);
    assert!(ring_buffer.is_full());
    assert_eq!(ring_buffer.len(), 3);

    // Overwrite oldest
    ring_buffer.push(4);
    assert_eq!(ring_buffer.len(), 3);
    assert_eq!(*ring_buffer.get(0).unwrap(), 2);
    assert_eq!(*ring_buffer.get(1).unwrap(), 3);
    assert_eq!(*ring_buffer.get(2).unwrap(), 4);
    assert_eq!(ring_buffer.get(3), None);
}

#[test]
fn test_storage_roundtrip() {
    let mut storage = MockStorage::new();
    let mut port = MockSerial::new();

    // First boot: Storage is empty, should trigger factory reset
    let mut state1 = AppState::uninit();
    state1.init(&mut storage, &mut port).unwrap();
    assert!(state1.initialized);

    // Simulate user registration
    for &byte in b"/user 1\r" {
        state1.process_byte(byte, &mut port, &mut storage).unwrap();
    }
    for &b in b"/name Alice\r" {
        state1.process_byte(b, &mut port, &mut storage).unwrap();
    }

    // Simulate sending a message
    for &byte in b"Hello World\r" {
        state1.process_byte(byte, &mut port, &mut storage).unwrap();
    }

    // Second boot: Load from the populated storage
    let mut state2 = AppState::uninit();
    let mut port2 = MockSerial::new();
    state2.init(&mut storage, &mut port2).unwrap();

    assert!(state2.initialized);

    // Verify user persistence
    let user = &state2.users[0];
    assert_eq!(user.id, 1);
    assert_eq!(user.status, 0);
    assert_eq!(user.name.as_str().unwrap(), "Alice");

    // Verify history persistence
    assert_eq!(state2.history.len(), 1);
    let msg = state2.history.get(0).unwrap();
    assert_eq!(msg.user_id, 1);
    assert_eq!(msg.content.as_str().unwrap(), "Hello World");
}

proptest! {
    #[test]
    fn fuzzy_process_byte(bytes in vec(any::<u8>(), 0..512)) {
        let mut storage = MockStorage::new();
        let mut port = MockSerial::new();
        let mut state = AppState::uninit();

        // Initialize state to ensure it is ready to process bytes
        let _ = state.init(&mut storage, &mut port);

        // Feed random bytes into the state machine
        for byte in bytes {
            let _ = state.process_byte(byte, &mut port, &mut storage);
        }
    }
}
