//! # Summary
//! ANSI terminal control and serial port abstraction.
//!
//! # Description
//! Provides byte-stream I/O traits and deterministic ANSI escape generators
//! for split-screen viewport rendering.

/// # Summary
/// Hardware abstraction trait for byte-stream serial communication.
///
/// # Description
/// Defines the minimal interface for reading and writing single bytes to a terminal or UART.
/// Implementors must guarantee non-blocking reads and synchronous byte writes.
///
/// # Examples
/// ```text
/// // Example implementation stub
/// ```
///
/// # Panics
/// Never. Implementors must handle hardware errors internally or return None.
///
/// # Errors
/// Byte I/O is modeled as fallible via Option or internal state.
///
/// # Safety
/// Safe. No raw pointer exposure or memory unsafety allowed in implementations.
///
/// # See Also
/// [`crate::state::AppState`]
pub trait SerialPort {
    /// # Summary
    /// Writes a single byte to the output stream.
    ///
    /// # Description
    /// Synchronously transmits one byte to the underlying hardware.
    ///
    /// # Examples
    /// ```text
    /// // port.write_byte(b'A');
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
    /// [`SerialPort::read_byte`]
    fn write_byte(&mut self, byte: u8);

    /// # Summary
    /// Reads a single byte from the input stream.
    ///
    /// # Description
    /// Non-blocking read. Returns None if the hardware buffer is empty.
    ///
    /// # Examples
    /// ```text
    /// // let b = port.read_byte();
    /// ```
    ///
    /// # Panics
    /// Never.
    ///
    /// # Errors
    /// Returns None if no byte is available.
    ///
    /// # Safety
    /// Safe.
    ///
    /// # See Also
    /// [`SerialPort::write_byte`]
    fn read_byte(&mut self) -> Option<u8>;
}
