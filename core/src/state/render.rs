//! # Summary
//! Viewport rendering and UI drawing logic.
//!
//! # Description
//! Handles the split-screen ANSI rendering, drawing the divider,
//! and updating the locked input prompt line.

use crate::serial::SerialPort;
use crate::state::AppState;
use crate::state::ansi::{
    CLEAR_SCREEN, COLOR_DIVIDER, COLOR_INFO, COLOR_PROMPT, COLOR_RESET, COLOR_SYSTEM,
    CURSOR_DIVIDER_LINE, CURSOR_HOME, CURSOR_INPUT_LINE, CURSOR_SCROLL_BOTTOM, ERASE_LINE,
    RESTORE_CURSOR, SAVE_CURSOR, SCROLL_REGION_22,
};
use crate::types::Message;
use crate::utils::write_u32;

/// # Summary
/// Writes a string slice to the serial port byte-by-byte.
///
/// # Description
/// Iterates over the UTF-8 bytes of the string and transmits them synchronously.
///
/// # Examples
/// ```rust,ignore
/// // crate::state::render::write_str(&mut port, "Hello");
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
/// [`crate::serial::SerialPort::write_byte`]
pub fn write_str(port: &mut impl SerialPort, string: &str) {
    for &byte in string.as_bytes() {
        port.write_byte(byte);
    }
}

/// # Summary
/// Initializes the ANSI split-screen viewport.
///
/// # Description
/// Clears the screen, sets the DECSTBM scrolling region to lines 1-22,
/// draws a divider on line 23, and moves the cursor to the input line 24.
///
/// # Examples
/// ```rust,ignore
/// // crate::state::render::init_viewport(&mut port);
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
/// [`render_prompt`]
pub fn init_viewport(port: &mut impl SerialPort) {
    write_str(port, CLEAR_SCREEN);
    write_str(port, CURSOR_HOME);
    write_str(port, SCROLL_REGION_22);

    write_str(port, CURSOR_DIVIDER_LINE);
    write_str(port, COLOR_DIVIDER);

    for _ in 0..80 {
        port.write_byte(b'-');
    }

    write_str(port, COLOR_RESET);
    write_str(port, CURSOR_INPUT_LINE);
}

/// # Summary
/// Renders the interactive input prompt and current buffer.
///
/// # Description
/// Locks the cursor to line 24, erases the line, prints the active user's name
/// (or Guest), and outputs the current contents of the line editor buffer.
///
/// # Examples
/// ```rust,ignore
/// // crate::state::render::render_prompt(&state, &mut port);
/// ```
///
/// # Panics
/// Never.
///
/// # Errors
/// Never.
///
/// # Safety
/// Safe.
///
/// # See Also
/// [`init_viewport`]
pub fn render_prompt(state: &AppState, port: &mut impl SerialPort) {
    write_str(port, CURSOR_INPUT_LINE);
    write_str(port, ERASE_LINE);
    write_str(port, COLOR_PROMPT);

    if let Some(user_id) = state.current_user_id {
        let mut found = false;
        for user in &state.users {
            if user.id != user_id || user.status != 0 {
                continue;
            }
            let Ok(name) = user.name.as_str() else {
                continue;
            };
            if name.is_empty() {
                continue;
            }
            write_str(port, name);
            found = true;
            break;
        }

        if !found {
            write_str(port, "User");
            write_u32(port, u32::from(user_id));
        }
    } else {
        write_str(port, "Guest");
    }

    write_str(port, "> ");
    write_str(port, COLOR_RESET);

    let Ok(input) = state.input_buffer.as_str() else {
        return;
    };
    write_str(port, input);
}

/// # Summary
/// Renders a single message into the scrolling history viewport.
///
/// # Description
/// Saves the cursor, moves to the bottom of the scrolling region (line 22),
/// prints the message with a newline to trigger a scroll, and restores the cursor.
///
/// # Examples
/// ```rust,ignore
/// // crate::state::render::render_message(&state, &msg, &mut port);
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
/// [`render_prompt`]
pub fn render_message(state: &AppState, message: &Message, port: &mut impl SerialPort) {
    write_str(port, SAVE_CURSOR);
    write_str(port, CURSOR_SCROLL_BOTTOM);

    write_str(port, "[");
    write_u32(port, message.tick);
    write_str(port, "] <");

    let mut found = false;
    for user in &state.users {
        if user.id != message.user_id || user.status != 0 {
            continue;
        }
        let Ok(name) = user.name.as_str() else {
            continue;
        };
        if name.is_empty() {
            continue;
        }
        write_str(port, name);
        found = true;
        break;
    }

    if !found {
        write_str(port, "User");
        write_u32(port, u32::from(message.user_id));
    }

    write_str(port, "> ");

    if let Ok(content) = message.content.as_str() {
        write_str(port, content);
    }

    write_str(port, "\r\n");
    write_str(port, RESTORE_CURSOR);
}

/// # Summary
/// Renders a local system message to the viewport.
///
/// # Description
/// Used to provide feedback for commands (e.g., errors, help text).
/// System messages are displayed in yellow and are not saved to persistent history.
///
/// # Examples
/// ```rust,ignore
/// // crate::state::render::render_system_message(&mut port, 1, "Command not found");
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
/// [`render_message`]
pub fn render_system_message(port: &mut impl SerialPort, tick: u32, msg: &str) {
    write_str(port, SAVE_CURSOR);
    write_str(port, CURSOR_SCROLL_BOTTOM);

    write_str(port, COLOR_SYSTEM);
    write_str(port, "[");
    write_u32(port, tick);
    write_str(port, "] <System> ");
    write_str(port, msg);
    write_str(port, COLOR_RESET);
    write_str(port, "\r\n");

    write_str(port, RESTORE_CURSOR);
}

/// # Summary
/// Renders an empty line in the scrolling viewport.
///
/// # Description
/// Used to provide visual separation between system messages and user messages.
///
/// # Examples
/// ```rust,ignore
/// // crate::state::render::render_empty_line(&mut port);
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
/// [`render_system_message`]
pub fn render_empty_line(port: &mut impl SerialPort) {
    write_str(port, SAVE_CURSOR);
    write_str(port, CURSOR_SCROLL_BOTTOM);
    write_str(port, "\r\n");
    write_str(port, RESTORE_CURSOR);
}

/// # Summary
/// Renders the welcome banner on startup.
///
/// # Description
/// Prints a formatted ASCII banner introducing the application and its basic usage.
///
/// # Examples
/// ```rust,ignore
/// // crate::state::render::render_welcome(&mut port);
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
/// [`init_viewport`]
pub fn render_welcome(port: &mut impl SerialPort) {
    render_system_message(
        port,
        0,
        "==================================================",
    );
    render_system_message(port, 0, " CLI Chat Embedded");
    render_system_message(port, 0, " Zero-heap, bare-metal compatible chat core.");
    render_system_message(port, 0, " Type /help to see available commands.");
    render_system_message(
        port,
        0,
        "==================================================",
    );
}

/// # Summary
/// Renders the list of active users.
///
/// # Description
/// Iterates through the user registry and prints the ID and name of each active user.
///
/// # Examples
/// ```rust,ignore
/// // crate::state::render::render_active_users(&state, &mut port);
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
/// [`render_system_message`]
pub fn render_active_users(state: &AppState, port: &mut impl SerialPort) {
    render_system_message(port, state.relative_tick, "--- Active Users ---");
    let mut count = 0;
    for user in &state.users {
        if user.status != 0 {
            continue;
        }
        count += 1;
        write_str(port, SAVE_CURSOR);
        write_str(port, CURSOR_SCROLL_BOTTOM);

        write_str(port, COLOR_INFO);
        write_str(port, "[");
        write_u32(port, state.relative_tick);
        write_str(port, "] <Info> ID: ");
        write_u32(port, u32::from(user.id));
        write_str(port, " | Name: ");

        let Ok(name) = user.name.as_str() else {
            write_str(port, "Unnamed");
            write_str(port, COLOR_RESET);
            write_str(port, "\r\n");
            write_str(port, RESTORE_CURSOR);
            continue;
        };

        if name.is_empty() {
            write_str(port, "Unnamed");
        } else {
            write_str(port, name);
        }

        write_str(port, COLOR_RESET);
        write_str(port, "\r\n");
        write_str(port, RESTORE_CURSOR);
    }

    if count == 0 {
        render_system_message(port, state.relative_tick, "No users registered.");
    }
}

/// # Summary
/// Renders the system status information.
///
/// # Description
/// Prints current uptime ticks and memory usage of the history buffer.
///
/// # Examples
/// ```rust,ignore
/// // crate::state::render::render_status(&state, &mut port);
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
/// [`render_system_message`]
#[allow(clippy::cast_possible_truncation)]
pub fn render_status(state: &AppState, port: &mut impl SerialPort) {
    render_system_message(port, state.relative_tick, "--- System Status ---");

    write_str(port, SAVE_CURSOR);
    write_str(port, CURSOR_SCROLL_BOTTOM);

    write_str(port, COLOR_INFO);
    write_str(port, "[");
    write_u32(port, state.relative_tick);
    write_str(port, "] <Info> Uptime Ticks: ");
    write_u32(port, state.relative_tick);
    write_str(port, COLOR_RESET);
    write_str(port, "\r\n");
    write_str(port, RESTORE_CURSOR);

    write_str(port, SAVE_CURSOR);
    write_str(port, CURSOR_SCROLL_BOTTOM);

    write_str(port, COLOR_INFO);
    write_str(port, "[");
    write_u32(port, state.relative_tick);
    write_str(port, "] <Info> Messages in History: ");
    write_u32(port, state.history.len() as u32);
    write_str(port, " / ");
    write_u32(port, crate::types::MAX_HISTORY as u32);
    write_str(port, COLOR_RESET);
    write_str(port, "\r\n");
    write_str(port, RESTORE_CURSOR);
}

/// # Summary
/// Renders the time and epoch information.
///
/// # Description
/// Prints the current relative tick and the host boot epoch.
///
/// # Examples
/// ```rust,ignore
/// // crate::state::render::render_time_info(&state, &mut port);
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
/// [`render_system_message`]
pub fn render_time_info(state: &AppState, port: &mut impl SerialPort) {
    render_system_message(port, state.relative_tick, "--- Time Info ---");

    write_str(port, SAVE_CURSOR);
    write_str(port, CURSOR_SCROLL_BOTTOM);

    write_str(port, COLOR_INFO);
    write_str(port, "[");
    write_u32(port, state.relative_tick);
    write_str(port, "] <Info> Boot Epoch: ");
    write_u32(port, state.boot_epoch);
    write_str(port, COLOR_RESET);
    write_str(port, "\r\n");
    write_str(port, RESTORE_CURSOR);
}

/// # Summary
/// Renders the application about information.
///
/// # Description
/// Prints the author and license information.
///
/// # Examples
/// ```rust,ignore
/// // crate::state::render::render_about(&state, &mut port);
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
/// [`render_system_message`]
pub fn render_about(state: &AppState, port: &mut impl SerialPort) {
    render_system_message(port, state.relative_tick, "CLI Chat Embedded");
    render_system_message(port, state.relative_tick, "Author: Dzulkifli Anwar");
    render_system_message(port, state.relative_tick, "License: MIT");
}
