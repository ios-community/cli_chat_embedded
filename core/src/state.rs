//! # Summary
//! Core state machine, command routing, and tick logic.
//!
//! # Description
//! Manages deterministic input parsing, user context switching,
//! and relative timestamp progression without dynamic allocation.

pub mod ansi;
pub mod render;

use crate::serial::SerialPort;
use crate::state::render::{
    init_viewport, render_about, render_active_users, render_empty_line, render_message,
    render_prompt, render_status, render_system_message, render_time_info, render_welcome,
};
use crate::storage::Storage;
use crate::types::{FixedString, MAX_HISTORY, MAX_USERS, Message, RingBuffer, User};
use crate::utils::{Crc32, parse_u16};
use core::mem::size_of_val;
use core::ptr::{addr_of, addr_of_mut};
use core::slice::{from_raw_parts, from_raw_parts_mut};

/// # Summary
/// Central application state managing users, history, and I/O dispatch.
///
/// # Description
/// Holds the complete static memory layout for the CLI chat session.
/// Intended for BSS allocation via `MaybeUninit`.
/// Mutated exclusively through `process_byte` in a single-threaded polling loop.
///
/// # Examples
/// ```rust
/// use cli_chat_core::state::AppState;
/// let state = AppState::uninit();
/// assert!(!state.initialized);
/// ```
///
/// # Panics
/// None. All fallible operations return Result.
///
/// # Errors
/// Returns errors on invalid commands or buffer overflow.
///
/// # Safety
/// Safe. Internal state uses const-generics for deterministic layout.
///
/// # See Also
/// [`crate::serial::SerialPort`]
pub struct AppState {
    /// Static user registry.
    pub users: [User; MAX_USERS],
    /// Circular message history.
    pub history: RingBuffer<Message, MAX_HISTORY>,
    /// Currently active sender ID. None if no user selected.
    pub current_user_id: Option<u16>,
    /// Active line editor input buffer.
    pub input_buffer: FixedString<256>,
    /// Incremental tick counter since boot.
    pub relative_tick: u32,
    /// Host clock epoch at initialization.
    pub boot_epoch: u32,
    /// Initialization guard flag.
    pub initialized: bool,
    /// Flag indicating the host runner should exit.
    pub should_exit: bool,
}

impl AppState {
    /// # Summary
    /// Creates an uninitialized `AppState` ready for BSS placement.
    ///
    /// # Description
    /// Returns a zeroed state container with initialization guard cleared.
    /// Must be placed in static memory before calling initialization routines.
    ///
    /// # Examples
    /// ```rust
    /// use cli_chat_core::state::AppState;
    /// let state = AppState::uninit();
    /// assert!(!state.initialized);
    /// ```
    ///
    /// # Panics
    /// None.
    ///
    /// # Errors
    /// None.
    ///
    /// # Safety
    /// Safe. All fields are zero-initialized to valid inert states.
    ///
    /// # See Also
    /// [`crate::state::AppState`]
    #[inline]
    #[must_use]
    pub const fn uninit() -> Self {
        let empty_user = User {
            id: 0,
            status: 1,
            padding: 0,
            name: FixedString::new(),
        };

        Self {
            users: [empty_user; MAX_USERS],
            history: RingBuffer::new(),
            current_user_id: None,
            input_buffer: FixedString::new(),
            relative_tick: 0,
            boot_epoch: 0,
            initialized: false,
            should_exit: false,
        }
    }

    /// # Summary
    /// Initializes the state from persistent storage.
    ///
    /// # Description
    /// Loads fixed-record binary, validates CRC32, and sets initialization guard.
    /// On CRC mismatch or read failure, triggers factory reset and emits ANSI warning.
    ///
    /// # Examples
    /// ```text
    /// // state.init(&mut storage, &mut port).unwrap();
    /// ```
    ///
    /// # Panics
    /// None.
    ///
    /// # Errors
    /// Returns an error if storage read fails critically after factory reset attempt.
    ///
    /// # Safety
    /// Contains isolated unsafe code to cast byte slices to C-representation structs.
    ///
    /// # See Also
    /// [`AppState::flush`]
    pub fn init(
        &mut self,
        storage: &mut impl Storage,
        port: &mut impl SerialPort,
    ) -> Result<(), &'static str> {
        let mut crc = Crc32::new();
        let mut offset = 0;

        let mut header = [0u8; 9];
        let Ok(len) = storage.read(offset, &mut header) else {
            return self.factory_reset(port);
        };
        if len != 9 || &header[0..4] != b"EMBU" || header[4] != 1 {
            return self.factory_reset(port);
        }
        crc.update(&header);
        offset += 9;

        self.boot_epoch = u32::from_le_bytes([header[5], header[6], header[7], header[8]]);

        let users_bytes = unsafe {
            from_raw_parts_mut(
                self.users.as_mut_ptr().cast::<u8>(),
                size_of_val(&self.users),
            )
        };
        let Ok(len) = storage.read(offset, users_bytes) else {
            return self.factory_reset(port);
        };
        if len != users_bytes.len() {
            return self.factory_reset(port);
        }
        crc.update(users_bytes);
        offset += users_bytes.len();

        let history_bytes = unsafe {
            from_raw_parts_mut(
                addr_of_mut!(self.history).cast::<u8>(),
                size_of_val(&self.history),
            )
        };
        let Ok(len) = storage.read(offset, history_bytes) else {
            return self.factory_reset(port);
        };
        if len != history_bytes.len() {
            return self.factory_reset(port);
        }
        crc.update(history_bytes);
        offset += history_bytes.len();

        let mut stored_crc_bytes = [0u8; 4];
        let Ok(len) = storage.read(offset, &mut stored_crc_bytes) else {
            return self.factory_reset(port);
        };
        if len != 4 {
            return self.factory_reset(port);
        }

        let stored_crc = u32::from_le_bytes(stored_crc_bytes);
        if crc.finalize() != stored_crc {
            return self.factory_reset(port);
        }

        self.initialized = true;

        init_viewport(port);
        render_welcome(port);
        render_empty_line(port);

        for i in 0..self.history.len() {
            let Some(msg) = self.history.get(i) else {
                continue;
            };
            render_message(self, msg, port);
        }

        render_prompt(self, port);
        Ok(())
    }

    /// # Summary
    /// Performs a factory reset on the application state.
    ///
    /// # Description
    /// Zero-initializes the state and emits an ANSI warning via the serial port.
    ///
    /// # Examples
    /// ```text
    /// // state.factory_reset(&mut port).unwrap();
    /// ```
    ///
    /// # Panics
    /// None.
    ///
    /// # Errors
    /// None.
    ///
    /// # Safety
    /// Safe.
    ///
    /// # See Also
    /// [`AppState::init`]
    pub fn factory_reset(&mut self, port: &mut impl SerialPort) -> Result<(), &'static str> {
        *self = AppState::uninit();
        self.initialized = true;

        init_viewport(port);
        render_welcome(port);
        render_empty_line(port);
        render_system_message(port, 0, "CRC mismatch: factory reset applied");
        render_empty_line(port);
        render_prompt(self, port);

        Ok(())
    }

    /// # Summary
    /// Flushes the current state to persistent storage.
    ///
    /// # Description
    /// Serializes the header, users, and history blocks, computes the CRC32,
    /// and writes everything to the provided storage medium.
    ///
    /// # Examples
    /// ```text
    /// // state.flush(&mut storage).unwrap();
    /// ```
    ///
    /// # Panics
    /// None.
    ///
    /// # Errors
    /// Returns an error if the storage write or flush operations fail.
    ///
    /// # Safety
    /// Contains isolated unsafe code to cast C-representation structs to byte slices.
    ///
    /// # See Also
    /// [`AppState::init`]
    pub fn flush(&self, storage: &mut impl Storage) -> Result<(), &'static str> {
        let mut crc = Crc32::new();
        let mut offset = 0;

        let mut header = [0u8; 9];
        header[0..4].copy_from_slice(b"EMBU");
        header[4] = 1;
        header[5..9].copy_from_slice(&self.boot_epoch.to_le_bytes());
        storage.write(offset, &header)?;
        crc.update(&header);
        offset += 9;

        let users_bytes =
            unsafe { from_raw_parts(self.users.as_ptr().cast::<u8>(), size_of_val(&self.users)) };

        storage.write(offset, users_bytes)?;
        crc.update(users_bytes);
        offset += users_bytes.len();

        let history_bytes = unsafe {
            from_raw_parts(
                addr_of!(self.history).cast::<u8>(),
                size_of_val(&self.history),
            )
        };

        storage.write(offset, history_bytes)?;
        crc.update(history_bytes);
        offset += history_bytes.len();

        let crc_val = crc.finalize();
        storage.write(offset, &crc_val.to_le_bytes())?;

        storage.flush()?;
        Ok(())
    }

    /// # Summary
    /// Processes a single input byte, mutates state, and writes to serial.
    ///
    /// # Description
    /// Core polling loop entry point for byte-by-byte terminal I/O.
    ///
    /// # Examples
    /// ```text
    /// // state.process_byte(b'h', &mut port, &mut storage).unwrap();
    /// ```
    ///
    /// # Panics
    /// None.
    ///
    /// # Errors
    /// Returns error if state uninitialized.
    ///
    /// # Safety
    /// Safe.
    ///
    /// # See Also
    /// [`AppState::init`]
    #[allow(clippy::collapsible_match)]
    pub fn process_byte(
        &mut self,
        byte: u8,
        port: &mut impl SerialPort,
        storage: &mut impl Storage,
    ) -> Result<(), &'static str> {
        if !self.initialized {
            return Err("AppState not initialized. Call init() first.");
        }

        match byte {
            // Backspace (ASCII 0x08 or DEL 0x7F)
            0x08 | 0x7F => {
                if self.input_buffer.pop().is_some() {
                    port.write_byte(0x08);
                    port.write_byte(b' ');
                    port.write_byte(0x08);
                }
            }

            // Enter / Submit
            b'\r' | b'\n' => {
                if self.input_buffer.is_empty() {
                    return Ok(());
                }

                self.relative_tick = self.relative_tick.wrapping_add(1);
                let input_bytes = self.input_buffer.as_bytes();

                if input_bytes.starts_with(b"/") {
                    self.dispatch_command(port);
                } else {
                    self.dispatch_message(port);
                }

                self.input_buffer.clear();
                self.flush(storage)?;

                render_prompt(self, port);
            }

            // Printable ASCII
            0x20..=0x7E => {
                if self.input_buffer.push(byte) {
                    port.write_byte(byte);
                }
            }

            _ => {}
        }

        Ok(())
    }

    /// # Summary
    /// Dispatches a command parsed from the input buffer.
    ///
    /// # Description
    /// Handles internal commands like `/user`, `/switch`, `/name`, `/users`, `/status`, `/time`, `/ping`, `/about`, `/clear`, `/exit`, and `/help`.
    /// Provides visual feedback via system messages framed by empty lines.
    ///
    /// # Examples
    /// ```text
    /// // state.dispatch_command(&mut port).unwrap();
    /// ```
    ///
    /// # Panics
    /// None.
    ///
    /// # Errors
    /// None.
    ///
    /// # Safety
    /// Safe.
    ///
    /// # See Also
    /// [`AppState::process_byte`]
    fn dispatch_command(&mut self, port: &mut impl SerialPort) {
        let bytes = self.input_buffer.as_bytes();

        if bytes == b"/exit" {
            self.should_exit = true;
            return;
        }

        if bytes == b"/clear" {
            init_viewport(port);
            render_welcome(port);
            render_empty_line(port);
            return;
        }

        render_empty_line(port);

        if bytes.starts_with(b"/user ") || bytes.starts_with(b"/switch ") {
            let prefix_len = if bytes.starts_with(b"/user ") { 6 } else { 8 };
            let id_bytes = &bytes[prefix_len..];

            let Some(id) = parse_u16(id_bytes) else {
                render_system_message(port, self.relative_tick, "Error: Invalid user ID.");
                render_empty_line(port);
                return;
            };

            self.current_user_id = Some(id);

            let mut exists = false;
            let mut empty_slot = None;
            for (i, user) in self.users.iter().enumerate() {
                if user.id == id && user.status == 0 {
                    exists = true;
                    break;
                }
                if user.status == 1 && empty_slot.is_none() {
                    empty_slot = Some(i);
                }
            }

            if !exists && let Some(index) = empty_slot {
                self.users[index].id = id;
                self.users[index].status = 0;
                self.users[index].name.clear();
            }

            render_system_message(port, self.relative_tick, "User switched successfully.");
            render_empty_line(port);
            return;
        }

        if bytes.starts_with(b"/name ") {
            let Some(user_id) = self.current_user_id else {
                render_system_message(
                    port,
                    self.relative_tick,
                    "Error: Please switch to a user first.",
                );
                render_empty_line(port);
                return;
            };

            let name_bytes = &bytes[6..];
            for user in &mut self.users {
                if user.id != user_id || user.status != 0 {
                    continue;
                }
                user.name.clear();
                for &byte in name_bytes {
                    if !user.name.push(byte) {
                        break;
                    }
                }
                break;
            }
            render_system_message(port, self.relative_tick, "Display name updated.");
            render_empty_line(port);
            return;
        }

        match bytes {
            b"/help" => render_system_message(
                port,
                self.relative_tick,
                "Commands: /user <id>, /switch <id>, /name <str>, /users, /status, /time, /ping, /about, /clear, /exit, /help",
            ),
            b"/users" => render_active_users(self, port),
            b"/status" => render_status(self, port),
            b"/ping" => {
                render_system_message(port, self.relative_tick, "Pong! System is responsive.");
            }
            b"/time" => render_time_info(self, port),
            b"/about" => render_about(self, port),
            _ => render_system_message(
                port,
                self.relative_tick,
                "Unknown command. Type /help for a list of commands.",
            ),
        }

        render_empty_line(port);
    }

    /// # Summary
    /// Dispatches a chat message from the input buffer.
    ///
    /// # Description
    /// Creates a new Message struct, pushes it to the history ring buffer,
    /// and renders it to the scrolling viewport.
    ///
    /// # Examples
    /// ```text
    /// // state.dispatch_message(&mut port).unwrap();
    /// ```
    ///
    /// # Panics
    /// None.
    ///
    /// # Errors
    /// None.
    ///
    /// # Safety
    /// Safe.
    ///
    /// # See Also
    /// [`AppState::process_byte`]
    fn dispatch_message(&mut self, port: &mut impl SerialPort) {
        let user_id = self.current_user_id.unwrap_or(0);
        let mut message = Message {
            user_id,
            padding: 0,
            tick: self.relative_tick,
            content: FixedString::new(),
        };

        for &byte in self.input_buffer.as_bytes() {
            message.content.push(byte);
        }

        self.history.push(message);
        render_message(self, &message, port);
    }

    /// # Summary
    /// Sets the boot epoch from the host clock.
    ///
    /// # Description
    /// Allows the host environment to inject the current UNIX timestamp
    /// after a factory reset to maintain accurate relative time tracking.
    ///
    /// # Examples
    /// ```text
    /// // state.set_boot_epoch(1779417956);
    /// ```
    ///
    /// # Panics
    /// None.
    ///
    /// # Errors
    /// None.
    ///
    /// # Safety
    /// Safe.
    ///
    /// # See Also
    /// [`AppState::init`]
    pub fn set_boot_epoch(&mut self, epoch: u32) {
        self.boot_epoch = epoch;
    }
}
