//! # Summary
//! Static data structures with const-generics and zero-heap guarantees.
//!
//! # Description
//! Provides `FixedString`, `RingBuffer`, and packed layout types
//! for BSS allocation without dynamic memory.

use core::mem::MaybeUninit;
use core::str::{Utf8Error, from_utf8};

/// # Summary
/// Fixed-capacity UTF-8 compatible byte buffer.
///
/// # Description
/// A compile-time bounded string replacement for `String` in `no_std` contexts.
/// Uses const generics to allocate exact BSS size. Tracks logical length separately.
/// Implements Copy and Clone for zero-heap duplication.
///
/// # Examples
/// ```rust
/// use cli_chat_core::types::FixedString;
/// let mut s = FixedString::<32>::new();
/// s.push(b'A');
/// assert_eq!(s.len(), 1);
/// ```
///
/// # Panics
/// None. Capacity violations are handled via boolean returns.
///
/// # Errors
/// None. Struct methods return Result or saturate safely.
///
/// # Safety
/// Safe. Backed by a fixed array with explicit length tracking.
///
/// # See Also
/// [`RingBuffer`], [`crate::state::AppState`]
#[repr(C)]
#[derive(Copy, Clone)]
pub struct FixedString<const N: usize> {
    buffer: [u8; N],
    len: usize,
}

impl<const N: usize> Default for FixedString<N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize> FixedString<N> {
    /// # Summary
    /// Creates an empty `FixedString` with zeroed backing memory.
    ///
    /// # Description
    /// Initializes the internal buffer with zeros and sets the length to zero.
    ///
    /// # Examples
    /// ```rust
    /// use cli_chat_core::types::FixedString;
    /// let s = FixedString::<32>::new();
    /// assert!(s.is_empty());
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
    /// [`FixedString::clear`]
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self {
            buffer: [0; N],
            len: 0,
        }
    }

    /// # Summary
    /// Appends a byte to the buffer if capacity remains.
    ///
    /// # Description
    /// Writes the byte to the current length index and increments the length.
    /// Returns true if successful, false if the buffer is full.
    ///
    /// # Examples
    /// ```rust
    /// use cli_chat_core::types::FixedString;
    /// let mut s = FixedString::<1>::new();
    /// assert!(s.push(b'A'));
    /// assert!(!s.push(b'B'));
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
    /// [`FixedString::is_full`]
    #[inline]
    pub fn push(&mut self, byte: u8) -> bool {
        if self.len < N {
            self.buffer[self.len] = byte;
            self.len += 1;
            true
        } else {
            false
        }
    }

    /// # Summary
    /// Returns the current logical length of the buffer.
    ///
    /// # Description
    /// Provides the number of bytes currently stored, which is always less than or equal to N.
    ///
    /// # Examples
    /// ```rust
    /// use cli_chat_core::types::FixedString;
    /// let s = FixedString::<32>::new();
    /// assert_eq!(s.len(), 0);
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
    /// [`FixedString::is_empty`]
    #[inline]
    #[must_use]
    pub const fn len(&self) -> usize {
        self.len
    }

    /// # Summary
    /// Returns true if the buffer contains no bytes.
    ///
    /// # Description
    /// Checks if the logical length is zero.
    ///
    /// # Examples
    /// ```rust
    /// use cli_chat_core::types::FixedString;
    /// let s = FixedString::<32>::new();
    /// assert!(s.is_empty());
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
    /// [`FixedString::len`]
    #[inline]
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// # Summary
    /// Returns true if the buffer is at maximum capacity.
    ///
    /// # Description
    /// Checks if the logical length equals the maximum capacity N.
    ///
    /// # Examples
    /// ```rust
    /// use cli_chat_core::types::FixedString;
    /// let mut s = FixedString::<1>::new();
    /// s.push(b'A');
    /// assert!(s.is_full());
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
    /// [`FixedString::push`]
    #[inline]
    #[must_use]
    pub const fn is_full(&self) -> bool {
        self.len == N
    }

    /// # Summary
    /// Clears the buffer by resetting logical length to zero.
    ///
    /// # Description
    /// Does not zero the backing memory, only resets the length tracker.
    ///
    /// # Examples
    /// ```rust
    /// use cli_chat_core::types::FixedString;
    /// let mut s = FixedString::<32>::new();
    /// s.push(b'A');
    /// s.clear();
    /// assert!(s.is_empty());
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
    /// [`FixedString::new`]
    #[inline]
    pub fn clear(&mut self) {
        self.len = 0;
    }

    /// # Summary
    /// Returns a zero-copy slice of the valid bytes.
    ///
    /// # Description
    /// Slices the backing array up to the current logical length.
    ///
    /// # Examples
    /// ```rust
    /// use cli_chat_core::types::FixedString;
    /// let mut s = FixedString::<32>::new();
    /// s.push(b'A');
    /// assert_eq!(s.as_bytes(), &[b'A']);
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
    /// [`FixedString::as_str`]
    #[inline]
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.buffer[..self.len]
    }

    /// # Summary
    /// Attempts to interpret the buffer as a UTF-8 string slice.
    ///
    /// # Description
    /// Validates the current bytes as UTF-8 and returns a string slice.
    ///
    /// # Examples
    /// ```rust
    /// use cli_chat_core::types::FixedString;
    /// let mut s = FixedString::<32>::new();
    /// s.push(b'A');
    /// assert_eq!(s.as_str().unwrap(), "A");
    /// ```
    ///
    /// # Panics
    /// Never.
    ///
    /// # Errors
    /// Returns `Utf8Error` if the bytes are not valid UTF-8.
    ///
    /// # Safety
    /// Safe.
    ///
    /// # See Also
    /// [`FixedString::as_bytes`]
    #[inline]
    pub fn as_str(&self) -> Result<&str, Utf8Error> {
        from_utf8(self.as_bytes())
    }

    /// # Summary
    /// Removes the last byte from the buffer and returns it.
    ///
    /// # Description
    /// Decrements the logical length if the buffer is not empty.
    ///
    /// # Examples
    /// ```rust
    /// use cli_chat_core::types::FixedString;
    /// let mut s = FixedString::<32>::new();
    /// s.push(b'A');
    /// assert_eq!(s.pop(), Some(b'A'));
    /// assert!(s.is_empty());
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
    /// [`FixedString::push`]
    #[inline]
    pub fn pop(&mut self) -> Option<u8> {
        if self.len > 0 {
            self.len -= 1;
            Some(self.buffer[self.len])
        } else {
            None
        }
    }
}

/// # Summary
/// Ring buffer for static circular message history.
///
/// # Description
/// A zero-heap, fixed-size circular queue for messages.
/// Tracks head, tail, and count. Overwrites oldest entry on overflow.
///
/// # Examples
/// ```rust
/// use cli_chat_core::types::RingBuffer;
/// let mut rb = RingBuffer::<u32, 64>::new();
/// rb.push(42);
/// assert_eq!(rb.len(), 1);
/// ```
///
/// # Panics
/// None. Index arithmetic wraps safely via modulo logic.
///
/// # Errors
/// None.
///
/// # Safety
/// Contains isolated unsafe code for `MaybeUninit` array initialization and access.
///
/// # See Also
/// [`FixedString`], [`crate::state::AppState`]
#[repr(C)]
pub struct RingBuffer<T, const N: usize> {
    buf: [MaybeUninit<T>; N],
    head: usize,
    tail: usize,
    count: usize,
}

impl<T, const N: usize> Default for RingBuffer<T, N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T, const N: usize> RingBuffer<T, N> {
    /// # Summary
    /// Creates an empty `RingBuffer`.
    ///
    /// # Description
    /// Initializes the backing array with uninitialized memory.
    ///
    /// # Examples
    /// ```rust
    /// use cli_chat_core::types::RingBuffer;
    /// let rb = RingBuffer::<u32, 64>::new();
    /// assert!(rb.is_empty());
    /// ```
    ///
    /// # Panics
    /// Never.
    ///
    /// # Errors
    /// None.
    ///
    /// # Safety
    /// Safe to call. Internally uses unsafe to assume initialization of `MaybeUninit` array,
    /// which is valid because `MaybeUninit` does not require initialization.
    ///
    /// # See Also
    /// [`RingBuffer::push`]
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self {
            buf: unsafe { MaybeUninit::uninit().assume_init() },
            head: 0,
            tail: 0,
            count: 0,
        }
    }

    /// # Summary
    /// Returns the number of elements currently stored.
    ///
    /// # Description
    /// Provides the count of valid initialized elements in the buffer.
    ///
    /// # Examples
    /// ```rust
    /// use cli_chat_core::types::RingBuffer;
    /// let rb = RingBuffer::<u32, 64>::new();
    /// assert_eq!(rb.len(), 0);
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
    /// [`RingBuffer::is_empty`]
    #[inline]
    #[must_use]
    pub const fn len(&self) -> usize {
        self.count
    }

    /// # Summary
    /// Returns true if the buffer contains no elements.
    ///
    /// # Description
    /// Checks if the internal count is zero.
    ///
    /// # Examples
    /// ```rust
    /// use cli_chat_core::types::RingBuffer;
    /// let rb = RingBuffer::<u32, 64>::new();
    /// assert!(rb.is_empty());
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
    /// [`RingBuffer::len`]
    #[inline]
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// # Summary
    /// Returns true if the buffer has reached maximum capacity.
    ///
    /// # Description
    /// Checks if the internal count equals the maximum capacity N.
    ///
    /// # Examples
    /// ```rust
    /// use cli_chat_core::types::RingBuffer;
    /// let mut rb = RingBuffer::<u32, 1>::new();
    /// rb.push(42);
    /// assert!(rb.is_full());
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
    /// [`RingBuffer::push`]
    #[inline]
    #[must_use]
    pub const fn is_full(&self) -> bool {
        self.count == N
    }

    /// # Summary
    /// Pushes a value into the buffer.
    ///
    /// # Description
    /// Writes the value to the tail index. If the buffer is full, it overwrites the oldest element
    /// by advancing the head index.
    ///
    /// # Examples
    /// ```rust
    /// use cli_chat_core::types::RingBuffer;
    /// let mut rb = RingBuffer::<u32, 1>::new();
    /// rb.push(1);
    /// rb.push(2);
    /// assert_eq!(*rb.get(0).unwrap(), 2);
    /// ```
    ///
    /// # Panics
    /// Never.
    ///
    /// # Errors
    /// None.
    ///
    /// # Safety
    /// Safe to call. Internally uses unsafe to write to `MaybeUninit`, which is valid.
    ///
    /// # See Also
    /// [`RingBuffer::get`]
    #[inline]
    pub fn push(&mut self, value: T) {
        if self.count == N {
            self.head = (self.head + 1) % N;
            self.count -= 1;
        }
        unsafe {
            self.buf[self.tail].as_mut_ptr().write(value);
        }
        self.tail = (self.tail + 1) % N;
        self.count += 1;
    }

    /// # Summary
    /// Retrieves a reference to the element at the given logical index.
    ///
    /// # Description
    /// Maps the logical index to the physical index based on the head position.
    /// Returns None if the index is out of bounds.
    ///
    /// # Examples
    /// ```rust
    /// use cli_chat_core::types::RingBuffer;
    /// let mut rb = RingBuffer::<u32, 64>::new();
    /// rb.push(42);
    /// assert_eq!(*rb.get(0).unwrap(), 42);
    /// ```
    ///
    /// # Panics
    /// Never.
    ///
    /// # Errors
    /// Returns None if the index is greater than or equal to the current count.
    ///
    /// # Safety
    /// Safe to call. Internally uses unsafe to assume initialization of the accessed element,
    /// which is guaranteed by the count tracking.
    ///
    /// # See Also
    /// [`RingBuffer::push`]
    #[inline]
    #[must_use]
    pub fn get(&self, index: usize) -> Option<&T> {
        if index >= self.count {
            return None;
        }
        let actual = (self.head + index) % N;
        Some(unsafe { self.buf[actual].assume_init_ref() })
    }
}

/// # Summary
/// Maximum number of registered users supported by the static layout.
///
/// # Description
/// Defines the capacity of the user registry array in the application state.
///
/// # Examples
/// ```rust
/// use cli_chat_core::types::MAX_USERS;
/// assert_eq!(MAX_USERS, 256);
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
/// [`User`]
pub const MAX_USERS: usize = 256;

/// # Summary
/// Maximum number of messages retained in the circular history buffer.
///
/// # Description
/// Defines the capacity of the ring buffer used for message history.
///
/// # Examples
/// ```rust
/// use cli_chat_core::types::MAX_HISTORY;
/// assert_eq!(MAX_HISTORY, 64);
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
/// [`Message`]
pub const MAX_HISTORY: usize = 64;

/// # Summary
/// Represents a registered chat participant in static memory.
///
/// # Description
/// Fixed-layout user record with soft-delete flag and bounded name buffer.
/// Designed for direct C-representation serialization to non-volatile storage.
/// Status byte controls visibility where 0 is active and 1 is soft-deleted.
///
/// # Examples
/// ```rust
/// use cli_chat_core::types::{User, FixedString};
/// let u = User {
///     id: 1,
///     status: 0,
///     padding: 0,
///     name: FixedString::<32>::new(),
/// };
/// assert_eq!(u.id, 1);
/// ```
///
/// # Panics
/// None.
///
/// # Errors
/// None.
///
/// # Safety
/// Safe. Zero drop semantics required for persistence roundtrips.
///
/// # See Also
/// [`Message`], [`MAX_USERS`]
#[repr(C)]
#[derive(Copy, Clone)]
pub struct User {
    /// Unique numeric identifier.
    pub id: u16,
    /// Soft-delete flag where 0 is active and 1 is deleted.
    pub status: u8,
    /// Explicit padding for 4-byte alignment across targets.
    pub padding: u8,
    /// Display name bounded to 32 bytes.
    pub name: FixedString<32>,
}

/// # Summary
/// Represents a single broadcast message with relative timestamp.
///
/// # Description
/// Fixed-layout message record containing sender ID, tick, and content.
/// Content is bounded to 256 bytes to prevent buffer overflow in constrained RAM.
/// Layout matches C-representation requirements for raw binary persistence.
///
/// # Examples
/// ```rust
/// use cli_chat_core::types::{Message, FixedString};
/// let m = Message {
///     user_id: 1,
///     padding: 0,
///     tick: 42,
///     content: FixedString::<256>::new(),
/// };
/// assert_eq!(m.tick, 42);
/// ```
///
/// # Panics
/// None.
///
/// # Errors
/// None.
///
/// # Safety
/// Safe. Zero drop semantics required for persistence roundtrips.
///
/// # See Also
/// [`User`], [`MAX_HISTORY`]
#[repr(C)]
#[derive(Copy, Clone)]
pub struct Message {
    /// ID of the sender.
    pub user_id: u16,
    /// Padding for 4-byte alignment.
    pub padding: u16,
    /// Relative tick at time of creation.
    pub tick: u32,
    /// Message content bounded to 256 bytes.
    pub content: FixedString<256>,
}
