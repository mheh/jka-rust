//! The winit-to-`keycodes.h` map (DEC-56.4).
//!
//! Raven translated a Windows message into a key number with `MapKey`: the
//! console scan code answers `A_CONSOLE` outright, a `virtualKeyConvert` row
//! answers the named keys, and everything left falls to `MapVirtualKey(vk, 2)`,
//! which is the key's unshifted ASCII. `fakeAscii_t` is laid out so that ASCII
//! value IS the key number, so the punctuation rows below read as their own
//! characters. A key with no row is dropped, Raven's `if (code)` guard.
//!
//! winit reports a physical key, which already carries the extended-key bit
//! Raven read from bit 24 of `lParam`. So `ArrowLeft` is `A_CURSOR_LEFT` and
//! `Numpad4` is `A_KP_4`, the two halves of one `virtualKeyConvert` row.
//!
//! Source: `oracle/codemp/win32/win_wndproc.cpp:103-299`

use core::ffi::c_int;

use mp_engine_client::keycodes::fakeAscii_t;
use winit::event::MouseButton;
use winit::keyboard::KeyCode;

/// The key number a physical key reports, or `None` for a key Raven had no row
/// for.
pub fn map_key(code: KeyCode) -> Option<c_int> {
    let key = match code {
        // Letters, the `VK_A`..`VK_Z` rows: both columns are the capital.
        KeyCode::KeyA => fakeAscii_t::A_CAP_A,
        KeyCode::KeyB => fakeAscii_t::A_CAP_B,
        KeyCode::KeyC => fakeAscii_t::A_CAP_C,
        KeyCode::KeyD => fakeAscii_t::A_CAP_D,
        KeyCode::KeyE => fakeAscii_t::A_CAP_E,
        KeyCode::KeyF => fakeAscii_t::A_CAP_F,
        KeyCode::KeyG => fakeAscii_t::A_CAP_G,
        KeyCode::KeyH => fakeAscii_t::A_CAP_H,
        KeyCode::KeyI => fakeAscii_t::A_CAP_I,
        KeyCode::KeyJ => fakeAscii_t::A_CAP_J,
        KeyCode::KeyK => fakeAscii_t::A_CAP_K,
        KeyCode::KeyL => fakeAscii_t::A_CAP_L,
        KeyCode::KeyM => fakeAscii_t::A_CAP_M,
        KeyCode::KeyN => fakeAscii_t::A_CAP_N,
        KeyCode::KeyO => fakeAscii_t::A_CAP_O,
        KeyCode::KeyP => fakeAscii_t::A_CAP_P,
        KeyCode::KeyQ => fakeAscii_t::A_CAP_Q,
        KeyCode::KeyR => fakeAscii_t::A_CAP_R,
        KeyCode::KeyS => fakeAscii_t::A_CAP_S,
        KeyCode::KeyT => fakeAscii_t::A_CAP_T,
        KeyCode::KeyU => fakeAscii_t::A_CAP_U,
        KeyCode::KeyV => fakeAscii_t::A_CAP_V,
        KeyCode::KeyW => fakeAscii_t::A_CAP_W,
        KeyCode::KeyX => fakeAscii_t::A_CAP_X,
        KeyCode::KeyY => fakeAscii_t::A_CAP_Y,
        KeyCode::KeyZ => fakeAscii_t::A_CAP_Z,

        // The number row, the `0x30`..`0x39` rows.
        KeyCode::Digit0 => fakeAscii_t::A_0,
        KeyCode::Digit1 => fakeAscii_t::A_1,
        KeyCode::Digit2 => fakeAscii_t::A_2,
        KeyCode::Digit3 => fakeAscii_t::A_3,
        KeyCode::Digit4 => fakeAscii_t::A_4,
        KeyCode::Digit5 => fakeAscii_t::A_5,
        KeyCode::Digit6 => fakeAscii_t::A_6,
        KeyCode::Digit7 => fakeAscii_t::A_7,
        KeyCode::Digit8 => fakeAscii_t::A_8,
        KeyCode::Digit9 => fakeAscii_t::A_9,

        // Punctuation: `MapVirtualKey`'s unshifted ASCII, which is the key
        // number itself.
        KeyCode::Minus => fakeAscii_t::A_MINUS,
        KeyCode::Equal => fakeAscii_t::A_EQUALS,
        KeyCode::BracketLeft => fakeAscii_t::A_OPEN_SQUARE,
        KeyCode::BracketRight => fakeAscii_t::A_CLOSE_SQUARE,
        KeyCode::Backslash => fakeAscii_t::A_BACKSLASH,
        KeyCode::Semicolon => fakeAscii_t::A_SEMICOLON,
        KeyCode::Quote => fakeAscii_t::A_SINGLE_QUOTE,
        KeyCode::Comma => fakeAscii_t::A_COMMA,
        KeyCode::Period => fakeAscii_t::A_PERIOD,
        KeyCode::Slash => fakeAscii_t::A_FORWARD_SLASH,

        // The console key. Raven checks its scan code before the table and
        // never lets it produce a character.
        KeyCode::Backquote => fakeAscii_t::A_CONSOLE,

        // Editing and navigation, the extended column of their rows.
        KeyCode::Escape => fakeAscii_t::A_ESCAPE,
        KeyCode::Enter => fakeAscii_t::A_ENTER,
        KeyCode::Tab => fakeAscii_t::A_TAB,
        KeyCode::Backspace => fakeAscii_t::A_BACKSPACE,
        KeyCode::Space => fakeAscii_t::A_SPACE,
        KeyCode::Insert => fakeAscii_t::A_INSERT,
        KeyCode::Delete => fakeAscii_t::A_DELETE,
        KeyCode::Home => fakeAscii_t::A_HOME,
        KeyCode::End => fakeAscii_t::A_END,
        KeyCode::PageUp => fakeAscii_t::A_PAGE_UP,
        KeyCode::PageDown => fakeAscii_t::A_PAGE_DOWN,
        KeyCode::ArrowUp => fakeAscii_t::A_CURSOR_UP,
        KeyCode::ArrowDown => fakeAscii_t::A_CURSOR_DOWN,
        KeyCode::ArrowLeft => fakeAscii_t::A_CURSOR_LEFT,
        KeyCode::ArrowRight => fakeAscii_t::A_CURSOR_RIGHT,

        // Modifiers. Raven has one row per modifier, so both sides agree.
        KeyCode::ShiftLeft | KeyCode::ShiftRight => fakeAscii_t::A_SHIFT,
        KeyCode::ControlLeft | KeyCode::ControlRight => fakeAscii_t::A_CTRL,
        KeyCode::AltLeft | KeyCode::AltRight => fakeAscii_t::A_ALT,
        KeyCode::CapsLock => fakeAscii_t::A_CAPSLOCK,
        KeyCode::NumLock => fakeAscii_t::A_NUMLOCK,
        KeyCode::ScrollLock => fakeAscii_t::A_SCROLLLOCK,
        KeyCode::Pause => fakeAscii_t::A_PAUSE,
        KeyCode::PrintScreen => fakeAscii_t::A_PRINTSCREEN,

        //TODO: Port MapKey NumLock
        // Source: oracle/codemp/win32/win_wndproc.cpp:283-290. Raven rewrote a
        // numpad digit to `A_0`..`A_9` while NumLock was on, which is how a
        // retail numpad bind reaches the same key number as the top row. winit
        // 0.30 does not report NumLock state, so the rows below stay on the
        // NumLock-off arm until the pump can read it.
        // The keypad, the `VK_NUMPAD*` rows.
        KeyCode::Numpad0 => fakeAscii_t::A_KP_0,
        KeyCode::Numpad1 => fakeAscii_t::A_KP_1,
        KeyCode::Numpad2 => fakeAscii_t::A_KP_2,
        KeyCode::Numpad3 => fakeAscii_t::A_KP_3,
        KeyCode::Numpad4 => fakeAscii_t::A_KP_4,
        KeyCode::Numpad5 => fakeAscii_t::A_KP_5,
        KeyCode::Numpad6 => fakeAscii_t::A_KP_6,
        KeyCode::Numpad7 => fakeAscii_t::A_KP_7,
        KeyCode::Numpad8 => fakeAscii_t::A_KP_8,
        KeyCode::Numpad9 => fakeAscii_t::A_KP_9,
        KeyCode::NumpadAdd => fakeAscii_t::A_KP_PLUS,
        KeyCode::NumpadSubtract => fakeAscii_t::A_KP_MINUS,
        KeyCode::NumpadEnter => fakeAscii_t::A_KP_ENTER,
        KeyCode::NumpadDecimal => fakeAscii_t::A_KP_PERIOD,
        KeyCode::NumpadMultiply => fakeAscii_t::A_MULTIPLY,
        KeyCode::NumpadDivide => fakeAscii_t::A_DIVIDE,

        // Function keys. Raven stops at F12; F13 and up are zero rows.
        KeyCode::F1 => fakeAscii_t::A_F1,
        KeyCode::F2 => fakeAscii_t::A_F2,
        KeyCode::F3 => fakeAscii_t::A_F3,
        KeyCode::F4 => fakeAscii_t::A_F4,
        KeyCode::F5 => fakeAscii_t::A_F5,
        KeyCode::F6 => fakeAscii_t::A_F6,
        KeyCode::F7 => fakeAscii_t::A_F7,
        KeyCode::F8 => fakeAscii_t::A_F8,
        KeyCode::F9 => fakeAscii_t::A_F9,
        KeyCode::F10 => fakeAscii_t::A_F10,
        KeyCode::F11 => fakeAscii_t::A_F11,
        KeyCode::F12 => fakeAscii_t::A_F12,

        _ => return None,
    };
    Some(key as c_int)
}

/// The key number a mouse button reports, Raven's `mouseConvert` table.
///
/// Source: `oracle/codemp/win32/win_input.cpp:565-572`
pub fn map_mouse_button(button: MouseButton) -> Option<c_int> {
    let key = match button {
        MouseButton::Left => fakeAscii_t::A_MOUSE1,
        MouseButton::Right => fakeAscii_t::A_MOUSE2,
        MouseButton::Middle => fakeAscii_t::A_MOUSE3,
        MouseButton::Back => fakeAscii_t::A_MOUSE4,
        MouseButton::Forward => fakeAscii_t::A_MOUSE5,
        MouseButton::Other(_) => return None,
    };
    Some(key as c_int)
}

/// The two wheel key numbers. Raven queues a down and an up for one notch.
///
/// Source: `oracle/codemp/win32/win_wndproc.cpp:345-355`
pub fn wheel_key(up: bool) -> c_int {
    if up {
        fakeAscii_t::A_MWHEELUP as c_int
    } else {
        fakeAscii_t::A_MWHEELDOWN as c_int
    }
}

/// The character a typed key contributes, or `None` where Raven's `WM_CHAR`
/// would carry nothing. Raven passes the raw Windows character through, and
/// `fakeAscii_t` is Latin-1 above 127, so anything wider is dropped.
///
/// Source: `oracle/codemp/win32/win_wndproc.cpp:522-526`
pub fn map_char(character: char) -> Option<c_int> {
    let code = character as u32;
    if code <= 0xFF {
        Some(code as c_int)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_fake_ascii_rows_are_their_own_characters() {
        assert_eq!(map_key(KeyCode::Space), Some(b' ' as c_int));
        assert_eq!(map_key(KeyCode::KeyA), Some(b'A' as c_int));
        assert_eq!(map_key(KeyCode::Digit7), Some(b'7' as c_int));
        assert_eq!(map_key(KeyCode::Semicolon), Some(b';' as c_int));
    }

    #[test]
    fn arrows_and_keypad_split_the_same_row() {
        assert_eq!(map_key(KeyCode::ArrowLeft), Some(172));
        assert_eq!(map_key(KeyCode::Numpad4), Some(20));
    }

    #[test]
    fn an_unmapped_key_is_dropped() {
        assert_eq!(map_key(KeyCode::F13), None);
        assert_eq!(map_key(KeyCode::SuperLeft), None);
    }
}
