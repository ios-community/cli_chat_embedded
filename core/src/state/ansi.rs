//! # Summary
//! ANSI escape sequence constants for terminal rendering.
//!
//! # Description
//! Contains static string slices for DECSTBM, SGR colors, and cursor manipulation.

/// # Summary
/// Warning message emitted when CRC mismatch triggers a factory reset.
///
/// # Description
/// Uses ANSI yellow text to alert the user that persistent state was lost.
///
/// # Examples
/// ```rust
/// use cli_chat_core::state::ansi::CRC_MISMATCH_WARNING;
/// assert!(CRC_MISMATCH_WARNING.contains("CRC mismatch"));
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
pub const CRC_MISMATCH_WARNING: &str =
    "\x1b[33m[WARNING] CRC mismatch: factory reset applied\x1b[0m\r\n";

/// # Summary
/// ANSI escape sequence to clear the entire screen.
///
/// # Description
/// Uses the `CSI 2 J` sequence to clear the terminal display.
///
/// # Examples
/// ```rust
/// use cli_chat_core::state::ansi::CLEAR_SCREEN;
/// assert_eq!(CLEAR_SCREEN, "\x1b[2J");
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
/// [`CURSOR_HOME`]
pub const CLEAR_SCREEN: &str = "\x1b[2J";

/// # Summary
/// ANSI escape sequence to move the cursor to the top-left corner.
///
/// # Description
/// Uses the `CSI H` sequence to reset the cursor position to row 1, column 1.
///
/// # Examples
/// ```rust
/// use cli_chat_core::state::ansi::CURSOR_HOME;
/// assert_eq!(CURSOR_HOME, "\x1b[H");
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
/// [`CLEAR_SCREEN`]
pub const CURSOR_HOME: &str = "\x1b[H";

/// # Summary
/// ANSI escape sequence to set the scrolling region (DECSTBM).
///
/// # Description
/// Restricts the scrolling area to lines 1 through 22, reserving the bottom for input.
///
/// # Examples
/// ```rust
/// use cli_chat_core::state::ansi::SCROLL_REGION_22;
/// assert_eq!(SCROLL_REGION_22, "\x1b[1;22r");
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
/// [`CURSOR_INPUT_LINE`]
pub const SCROLL_REGION_22: &str = "\x1b[1;22r";

/// # Summary
/// ANSI escape sequence to move the cursor to the input line.
///
/// # Description
/// Moves the cursor to row 24, column 1 for the line editor prompt.
///
/// # Examples
/// ```rust
/// use cli_chat_core::state::ansi::CURSOR_INPUT_LINE;
/// assert_eq!(CURSOR_INPUT_LINE, "\x1b[24;1H");
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
/// [`CURSOR_DIVIDER_LINE`]
pub const CURSOR_INPUT_LINE: &str = "\x1b[24;1H";

/// # Summary
/// ANSI escape sequence to move the cursor to the divider line.
///
/// # Description
/// Moves the cursor to row 23, column 1 to draw the UI separator.
///
/// # Examples
/// ```rust
/// use cli_chat_core::state::ansi::CURSOR_DIVIDER_LINE;
/// assert_eq!(CURSOR_DIVIDER_LINE, "\x1b[23;1H");
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
/// [`CURSOR_INPUT_LINE`]
pub const CURSOR_DIVIDER_LINE: &str = "\x1b[23;1H";

/// # Summary
/// ANSI escape sequence to erase the current line.
///
/// # Description
/// Uses the `CSI 2 K` sequence to clear all characters on the active line.
///
/// # Examples
/// ```rust
/// use cli_chat_core::state::ansi::ERASE_LINE;
/// assert_eq!(ERASE_LINE, "\x1b[2K");
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
/// [`CLEAR_SCREEN`]
pub const ERASE_LINE: &str = "\x1b[2K";

/// # Summary
/// ANSI escape sequence to reset text formatting.
///
/// # Description
/// Uses the `CSI 0 m` sequence to clear all SGR color and style attributes.
///
/// # Examples
/// ```rust
/// use cli_chat_core::state::ansi::COLOR_RESET;
/// assert_eq!(COLOR_RESET, "\x1b[0m");
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
/// [`COLOR_PROMPT`]
pub const COLOR_RESET: &str = "\x1b[0m";

/// # Summary
/// ANSI escape sequence for the prompt color (Green).
///
/// # Description
/// Uses the `CSI 32 m` sequence to set the foreground text color to green.
///
/// # Examples
/// ```rust
/// use cli_chat_core::state::ansi::COLOR_PROMPT;
/// assert_eq!(COLOR_PROMPT, "\x1b[32m");
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
/// [`COLOR_RESET`]
pub const COLOR_PROMPT: &str = "\x1b[32m";

/// # Summary
/// ANSI escape sequence for the divider color (Blue).
///
/// # Description
/// Uses the `CSI 34 m` sequence to set the foreground text color to blue.
///
/// # Examples
/// ```rust
/// use cli_chat_core::state::ansi::COLOR_DIVIDER;
/// assert_eq!(COLOR_DIVIDER, "\x1b[34m");
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
/// [`COLOR_RESET`]
pub const COLOR_DIVIDER: &str = "\x1b[34m";

/// # Summary
/// ANSI escape sequence for system messages (Yellow).
///
/// # Description
/// Uses the `CSI 33 m` sequence to set the foreground text color to yellow.
///
/// # Examples
/// ```rust
/// use cli_chat_core::state::ansi::COLOR_SYSTEM;
/// assert_eq!(COLOR_SYSTEM, "\x1b[33m");
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
/// [`COLOR_RESET`]
pub const COLOR_SYSTEM: &str = "\x1b[33m";

/// # Summary
/// ANSI escape sequence for info messages (Cyan).
///
/// # Description
/// Uses the `CSI 36 m` sequence to set the foreground text color to cyan.
///
/// # Examples
/// ```rust
/// use cli_chat_core::state::ansi::COLOR_INFO;
/// assert_eq!(COLOR_INFO, "\x1b[36m");
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
/// [`COLOR_RESET`]
pub const COLOR_INFO: &str = "\x1b[36m";

/// # Summary
/// ANSI escape sequence to save the current cursor position.
///
/// # Description
/// Uses the `ESC 7` sequence to memorize where the cursor currently is.
///
/// # Examples
/// ```rust
/// use cli_chat_core::state::ansi::SAVE_CURSOR;
/// assert_eq!(SAVE_CURSOR, "\x1b7");
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
/// [`RESTORE_CURSOR`]
pub const SAVE_CURSOR: &str = "\x1b7";

/// # Summary
/// ANSI escape sequence to restore the saved cursor position.
///
/// # Description
/// Uses the `ESC 8` sequence to move the cursor back to the saved location.
///
/// # Examples
/// ```rust
/// use cli_chat_core::state::ansi::RESTORE_CURSOR;
/// assert_eq!(RESTORE_CURSOR, "\x1b8");
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
/// [`SAVE_CURSOR`]
pub const RESTORE_CURSOR: &str = "\x1b8";

/// # Summary
/// ANSI escape sequence to move the cursor to the bottom of the scrolling region.
///
/// # Description
/// Moves the cursor to row 22, column 1.
///
/// # Examples
/// ```rust
/// use cli_chat_core::state::ansi::CURSOR_SCROLL_BOTTOM;
/// assert_eq!(CURSOR_SCROLL_BOTTOM, "\x1b[22;1H");
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
/// [`SCROLL_REGION_22`]
pub const CURSOR_SCROLL_BOTTOM: &str = "\x1b[22;1H";
