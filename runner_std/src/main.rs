//! # Summary
//! Host simulator bridging standard I/O to the core traits.
//!
//! # Description
//! Uses raw terminal mode for non-blocking byte polling and split-screen ANSI rendering.
//! Implements the Architect's `MaybeUninit` and `AtomicU8` boot guard pattern.

use cli_chat_core::{AppState, SerialPort, Storage};
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, poll, read};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use std::fs::{File, OpenOptions};
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::mem::MaybeUninit;
use std::ptr::addr_of_mut;
use std::sync::atomic::{AtomicU8, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Global static state allocated in BSS.
static mut STATE: MaybeUninit<AppState> = MaybeUninit::uninit();
/// Boot flag: 0 = uninit, 1 = ready
static BOOT_FLAG: AtomicU8 = AtomicU8::new(0);

/// # Summary
/// Standard output serial port implementation.
///
/// # Description
/// Writes bytes directly to stdout and flushes immediately.
struct StdSerial {
    stdout: io::Stdout,
}

impl StdSerial {
    fn new() -> Self {
        Self {
            stdout: io::stdout(),
        }
    }
}

impl SerialPort for StdSerial {
    fn write_byte(&mut self, byte: u8) {
        let _ = self.stdout.write_all(&[byte]);
        let _ = self.stdout.flush();
    }

    fn read_byte(&mut self) -> Option<u8> {
        // Reading is handled by crossterm event polling in the main loop
        None
    }
}

/// # Summary
/// File-based storage implementation.
///
/// # Description
/// Uses `std::fs::File` to persist the raw fixed-record binary.
struct FileStorage {
    file: File,
}

impl FileStorage {
    fn new(path: &str) -> io::Result<Self> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)?;
        Ok(Self { file })
    }
}

impl Storage for FileStorage {
    fn read(&mut self, offset: usize, buffer: &mut [u8]) -> Result<usize, &'static str> {
        self.file
            .seek(SeekFrom::Start(offset as u64))
            .map_err(|_| "Seek failed")?;
        let bytes_read = self.file.read(buffer).map_err(|_| "Read failed")?;
        Ok(bytes_read)
    }

    fn write(&mut self, offset: usize, buffer: &[u8]) -> Result<(), &'static str> {
        self.file
            .seek(SeekFrom::Start(offset as u64))
            .map_err(|_| "Seek failed")?;
        self.file.write_all(buffer).map_err(|_| "Write failed")?;
        Ok(())
    }

    fn flush(&mut self) -> Result<(), &'static str> {
        self.file.sync_all().map_err(|_| "Sync failed")?;
        Ok(())
    }
}

/// # Summary
/// Entry point for the standard host runner.
///
/// # Description
/// Initializes the terminal into raw mode, sets up the mock storage and serial port,
/// and enters the main polling loop to feed bytes into the core state machine.
fn main() -> io::Result<()> {
    // Initialize Hardware Abstractions
    let mut port = StdSerial::new();
    let mut storage = FileStorage::new("chat_state.bin")?;

    // Guard Pattern for Initialization
    if BOOT_FLAG.load(Ordering::Acquire) == 1 {
        eprintln!("Already initialized");
        return Ok(());
    }

    let state = unsafe {
        let state_ptr = addr_of_mut!(STATE);
        (*state_ptr).write(AppState::uninit());
        (*state_ptr).assume_init_mut()
    };

    // Boot State Machine
    if state.init(&mut storage, &mut port).is_err() {
        // If factory reset occured, inject current host time
        if let Ok(duration) = SystemTime::now().duration_since(UNIX_EPOCH) {
            state.set_boot_epoch(u32::try_from(duration.as_secs()).unwrap_or(0));
        }
    }

    BOOT_FLAG.store(1, Ordering::Release);

    // Enter Raw Mode for UI
    enable_raw_mode()?;

    // Main Polling Loop
    loop {
        if state.should_exit {
            break;
        }

        if poll(Duration::from_millis(10))?
            && let Event::Key(KeyEvent {
                code,
                modifiers,
                kind,
                ..
            }) = read()?
        {
            if kind != KeyEventKind::Press {
                continue;
            }

            // Handle graceful Exit (Ctrl + C)
            if modifiers.contains(KeyModifiers::CONTROL) && code == KeyCode::Char('c') {
                break;
            }

            // Map crossterm keys to raw bytes for the core parser
            let byte = match code {
                KeyCode::Char(c) if c.is_ascii() => Some(c as u8),
                KeyCode::Enter => Some(b'\r'),
                KeyCode::Backspace => Some(0x08),
                _ => None,
            };

            if let Some(b) = byte {
                let _ = state.process_byte(b, &mut port, &mut storage);
            }
        }
    }

    // Cleanup
    disable_raw_mode()?;
    println!("\r\nExiting cli_chat_embedded runner...");
    Ok(())
}
