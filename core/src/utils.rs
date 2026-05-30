//! # Summary
//! Internal utilities including table-less CRC32 calculation.
//!
//! # Description
//! Provides pure `no_std` helper functions that do not allocate memory.

use crate::SerialPort;

/// # Summary
/// Table-less CRC32 calculator using IEEE 802.3 polynomial.
///
/// # Description
/// Computes CRC32 iteratively without a lookup table to minimize binary footprint.
/// Uses the standard 0xEDB88320 polynomial.
///
/// # Examples
/// ```rust
/// use cli_chat_core::utils::Crc32;
/// let mut crc = Crc32::new();
/// crc.update(b"123456789");
/// assert_eq!(crc.finalize(), 0xCBF43926);
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
/// [`crate::state::AppState::init`]
pub struct Crc32 {
    state: u32,
}

impl Default for Crc32 {
    fn default() -> Self {
        Self::new()
    }
}

impl Crc32 {
    /// # Summary
    /// Creates a new CRC32 calculator with initial state.
    ///
    /// # Description
    /// Initializes the internal state to 0xFFFFFFFF.
    ///
    /// # Examples
    /// ```rust
    /// use cli_chat_core::utils::Crc32;
    /// let crc = Crc32::new();
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
    /// [`Crc32::update`]
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self { state: 0xFFFF_FFFF }
    }

    /// # Summary
    /// Updates the CRC32 state with a chunk of data.
    ///
    /// # Description
    /// Processes each byte iteratively using bitwise shifts.
    ///
    /// # Examples
    /// ```rust
    /// use cli_chat_core::utils::Crc32;
    /// let mut crc = Crc32::new();
    /// crc.update(b"hello");
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
    /// [`Crc32::finalize`]
    #[inline]
    pub fn update(&mut self, data: &[u8]) {
        for &byte in data {
            self.state ^= u32::from(byte);
            for _ in 0..8 {
                if self.state & 1 != 0 {
                    self.state = (self.state >> 1) ^ 0xEDB8_8320;
                } else {
                    self.state >>= 1;
                }
            }
        }
    }

    /// # Summary
    /// Finalizes and returns the computed CRC32 value.
    ///
    /// # Description
    /// Inverts the internal state to produce the final checksum.
    ///
    /// # Examples
    /// ```rust
    /// use cli_chat_core::utils::Crc32;
    /// let crc = Crc32::new();
    /// assert_eq!(crc.finalize(), 0);
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
    /// [`Crc32::update`]
    #[inline]
    #[must_use]
    pub const fn finalize(self) -> u32 {
        !self.state
    }
}

/// # Summary
/// Writes a 32-bit unsigned integer to the serial port as ASCII.
///
/// # Description
/// Converts the integer to its base-10 ASCII representation and writes it
/// byte-by-byte to the provided serial port without allocating memory.
///
/// # Examples
/// ```text
/// // crate::utils::write_u32(&mut port, 42);
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
/// [`crate::serial::SerialPort`]
pub fn write_u32(port: &mut impl SerialPort, mut n: u32) {
    if n == 0 {
        port.write_byte(b'0');
        return;
    }

    let mut buffer = [0u8; 10];
    let mut i = 10;

    while n > 0 {
        i -= 1;
        buffer[i] = (n % 10) as u8 + b'0';
        n /= 10;
    }

    for &byte in &buffer[i..] {
        port.write_byte(byte);
    }
}

/// # Summary
/// Parses a 16-bit unsigned integer from a byte slice.
///
/// # Description
/// Iterates over ASCII digits and accumulates the value. Returns None if invalid.
///
/// # Examples
/// ```rust
/// use cli_chat_core::utils::parse_u16;
/// assert_eq!(parse_u16(b"42"), Some(42));
/// assert_eq!(parse_u16(b"abc"), None);
/// ```
///
/// # Panics
/// Never.
///
/// # Errors
/// Returns None if the slice is empty, contains non-digits, or overflows u16.
///
/// # Safety
/// Safe.
///
/// # See Also
/// [`write_u32`]
#[must_use]
pub fn parse_u16(bytes: &[u8]) -> Option<u16> {
    if bytes.is_empty() {
        return None;
    }

    let mut result: u16 = 0;
    for &byte in bytes {
        if byte.is_ascii_digit() {
            let digit = u16::from(byte - b'0');
            result = result.checked_mul(10)?.checked_add(digit)?;
        } else {
            return None;
        }
    }
    Some(result)
}
