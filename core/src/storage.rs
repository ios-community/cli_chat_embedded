//! # Summary
//! Raw fixed-record serialization and storage abstraction.
//!
//! # Description
//! Manages block-level persistence for user registry and chat history
//! with strict CRC32 validation and zero-copy layout mapping.

/// # Summary
/// Hardware abstraction trait for non-volatile storage.
///
/// # Description
/// Provides synchronous block read, write, and flush operations for persistent state.
/// Implementations must respect packed alignment guarantees.
///
/// # Examples
/// ```rust,ignore
/// // Example implementation stub
/// ```
///
/// # Panics
/// Never. All bounds or media errors must be returned via Result.
///
/// # Errors
/// Returns static string on out-of-bounds access, media failure, or CRC mismatch.
///
/// # Safety
/// Safe. Implementors must avoid unsafe slice manipulation unless strictly validated.
///
/// # See Also
/// [`crate::state::AppState`]
pub trait Storage {
    /// # Summary
    /// Reads a contiguous block into the buffer starting at the offset.
    ///
    /// # Description
    /// Populates the provided buffer with bytes from the storage medium.
    ///
    /// # Examples
    /// ```rust,ignore
    /// // storage.read(0, &mut buf);
    /// ```
    ///
    /// # Panics
    /// Never.
    ///
    /// # Errors
    /// Returns static string on read failure.
    ///
    /// # Safety
    /// Safe.
    ///
    /// # See Also
    /// [`Storage::write`]
    fn read(&mut self, offset: usize, buffer: &mut [u8]) -> Result<usize, &'static str>;

    /// # Summary
    /// Writes the buffer to the specified offset.
    ///
    /// # Description
    /// Commits the provided bytes to the storage medium at the specified offset.
    ///
    /// # Examples
    /// ```rust,ignore
    /// // storage.write(0, &buf);
    /// ```
    ///
    /// # Panics
    /// Never.
    ///
    /// # Errors
    /// Returns static string on write failure.
    ///
    /// # Safety
    /// Safe.
    ///
    /// # See Also
    /// [`Storage::read`]
    fn write(&mut self, offset: usize, buffer: &[u8]) -> Result<(), &'static str>;

    /// # Summary
    /// Flushes pending writes to physical media.
    ///
    /// # Description
    /// Ensures all previously written data is permanently stored.
    ///
    /// # Examples
    /// ```rust,ignore
    /// // storage.flush();
    /// ```
    ///
    /// # Panics
    /// Never.
    ///
    /// # Errors
    /// Returns static string on flush failure.
    ///
    /// # Safety
    /// Safe.
    ///
    /// # See Also
    /// [`Storage::write`]
    fn flush(&mut self) -> Result<(), &'static str>;
}
