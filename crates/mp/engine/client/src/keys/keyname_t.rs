#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::{c_char, c_int};

use mp_ui::keycodes::fake_ascii_t::fakeAscii_t;
use mp_ui::keycodes::fake_ascii_t::fakeAscii_t::*;

use crate::keys::key_globals_s::MAX_KEYS;

/// Raven `keyname_t` — a key name/binding table entry.
///
/// Type definition source: `oracle/codemp/client/keys.h:36-43`
#[repr(C)]
pub struct keyname_t {
    pub upper: u16,
    pub lower: u16,
    pub name: *mut c_char,
    pub keynum: c_int,
    pub menukey: bool,
}

#[cfg(target_pointer_width = "64")]
const _: () = {
    assert!(core::mem::size_of::<keyname_t>() == 24);
    assert!(core::mem::offset_of!(keyname_t, upper) == 0);
    assert!(core::mem::offset_of!(keyname_t, lower) == 2);
    assert!(core::mem::offset_of!(keyname_t, name) == 8);
    assert!(core::mem::offset_of!(keyname_t, keynum) == 16);
    assert!(core::mem::offset_of!(keyname_t, menukey) == 20);
};
// The `name` pointer is null-valid and every other field is a scalar, so the
// all-zero image is a valid inhabitant.
unsafe impl native_platform::ZeroValid for keyname_t {}

// ILP32 twin: clang i386 ground truth (msvc and linux-gnu agree).
#[cfg(target_pointer_width = "32")]
const _: () = {
    assert!(core::mem::size_of::<keyname_t>() == 16);
    assert!(core::mem::offset_of!(keyname_t, upper) == 0);
    assert!(core::mem::offset_of!(keyname_t, lower) == 2);
    assert!(core::mem::offset_of!(keyname_t, name) == 4);
    assert!(core::mem::offset_of!(keyname_t, keynum) == 8);
    assert!(core::mem::offset_of!(keyname_t, menukey) == 12);
};

/// A `KEYNAMES` row with no console name and no menu key.
///
/// Source: `oracle/codemp/client/cl_keys.cpp:22-353`
const fn plain(upper: u16, lower: u16, keynum: fakeAscii_t) -> keyname_t {
    keyname_t {
        upper,
        lower,
        name: core::ptr::null_mut(),
        keynum: keynum as c_int,
        menukey: false,
    }
}

/// A `KEYNAMES` row with a console name and no menu key.
/// `name` must end with a NUL byte, because `Key_KeyToName` hands it out as a C string.
///
/// Source: `oracle/codemp/client/cl_keys.cpp:22-353`
const fn named(upper: u16, lower: u16, name: &'static [u8], keynum: fakeAscii_t) -> keyname_t {
    keyname_t {
        upper,
        lower,
        name: name.as_ptr() as *mut c_char,
        keynum: keynum as c_int,
        menukey: false,
    }
}

/// A `KEYNAMES` row with a console name that also sets `menukey`.
/// Only the twelve function keys take this row shape.
///
/// Source: `oracle/codemp/client/cl_keys.cpp:22-353`
const fn menu(upper: u16, lower: u16, name: &'static [u8], keynum: fakeAscii_t) -> keyname_t {
    keyname_t {
        upper,
        lower,
        name: name.as_ptr() as *mut c_char,
        keynum: keynum as c_int,
        menukey: true,
    }
}

/// Raven `keynames[MAX_KEYS]`, the key name and keynum table.
///
/// Raven: do NOT blithely change any of the key names (3rd field) here, since
/// they have to match the key binds in the CFG files, they're also prepended
/// with "KEYNAME_" when looking up StringEd references.
///
/// The table is immutable data, so it is a `const` and not client state. Each
/// row keeps Raven's column order behind `plain`/`named`/`menu`, which carry the
/// `name` and `menukey` columns.
/// Source: `oracle/codemp/client/cl_keys.cpp:22-353`
#[rustfmt::skip]
pub const KEYNAMES: [keyname_t; MAX_KEYS] = [
    plain(0x00, 0x00, A_NULL),
    named(0x01, 0x01, b"SHIFT\0", A_SHIFT),
    named(0x02, 0x02, b"CTRL\0", A_CTRL),
    named(0x03, 0x03, b"ALT\0", A_ALT),
    named(0x04, 0x04, b"CAPSLOCK\0", A_CAPSLOCK),
    named(0x05, 0x05, b"KP_NUMLOCK\0", A_NUMLOCK),
    named(0x06, 0x06, b"SCROLLLOCK\0", A_SCROLLLOCK),
    named(0x07, 0x07, b"PAUSE\0", A_PAUSE),
    named(0x08, 0x08, b"BACKSPACE\0", A_BACKSPACE),
    named(0x09, 0x09, b"TAB\0", A_TAB),
    named(0x0a, 0x0a, b"ENTER\0", A_ENTER),
    named(0x0b, 0x0b, b"KP_PLUS\0", A_KP_PLUS),
    named(0x0c, 0x0c, b"KP_MINUS\0", A_KP_MINUS),
    named(0x0d, 0x0d, b"KP_ENTER\0", A_KP_ENTER),
    named(0x0e, 0x0e, b"KP_DEL\0", A_KP_PERIOD),
    plain(0x0f, 0x0f, A_PRINTSCREEN),
    named(0x10, 0x10, b"KP_INS\0", A_KP_0),
    named(0x11, 0x11, b"KP_END\0", A_KP_1),
    named(0x12, 0x12, b"KP_DOWNARROW\0", A_KP_2),
    named(0x13, 0x13, b"KP_PGDN\0", A_KP_3),
    named(0x14, 0x14, b"KP_LEFTARROW\0", A_KP_4),
    named(0x15, 0x15, b"KP_5\0", A_KP_5),
    named(0x16, 0x16, b"KP_RIGHTARROW\0", A_KP_6),
    named(0x17, 0x17, b"KP_HOME\0", A_KP_7),
    named(0x18, 0x18, b"KP_UPARROW\0", A_KP_8),
    named(0x19, 0x19, b"KP_PGUP\0", A_KP_9),
    named(0x1a, 0x1a, b"CONSOLE\0", A_CONSOLE),
    named(0x1b, 0x1b, b"ESCAPE\0", A_ESCAPE),
    menu(0x1c, 0x1c, b"F1\0", A_F1),
    menu(0x1d, 0x1d, b"F2\0", A_F2),
    menu(0x1e, 0x1e, b"F3\0", A_F3),
    menu(0x1f, 0x1f, b"F4\0", A_F4),

    named(0x20, 0x20, b"SPACE\0", A_SPACE),
    plain(0x21, 0x21, A_PLING),
    plain(0x22, 0x22, A_DOUBLE_QUOTE),
    plain(0x23, 0x23, A_HASH),
    plain(0x24, 0x24, A_STRING),
    plain(0x25, 0x25, A_PERCENT),
    plain(0x26, 0x26, A_AND),
    plain(0x27, 0x27, A_SINGLE_QUOTE),
    plain(0x28, 0x28, A_OPEN_BRACKET),
    plain(0x29, 0x29, A_CLOSE_BRACKET),
    plain(0x2a, 0x2a, A_STAR),
    plain(0x2b, 0x2b, A_PLUS),
    plain(0x2c, 0x2c, A_COMMA),
    plain(0x2d, 0x2d, A_MINUS),
    plain(0x2e, 0x2e, A_PERIOD),
    plain(0x2f, 0x2f, A_FORWARD_SLASH),
    plain(0x30, 0x30, A_0),
    plain(0x31, 0x31, A_1),
    plain(0x32, 0x32, A_2),
    plain(0x33, 0x33, A_3),
    plain(0x34, 0x34, A_4),
    plain(0x35, 0x35, A_5),
    plain(0x36, 0x36, A_6),
    plain(0x37, 0x37, A_7),
    plain(0x38, 0x38, A_8),
    plain(0x39, 0x39, A_9),
    plain(0x3a, 0x3a, A_COLON),
    named(0x3b, 0x3b, b"SEMICOLON\0", A_SEMICOLON),
    plain(0x3c, 0x3c, A_LESSTHAN),
    plain(0x3d, 0x3d, A_EQUALS),
    plain(0x3e, 0x3e, A_GREATERTHAN),
    plain(0x3f, 0x3f, A_QUESTION),

    plain(0x40, 0x40, A_AT),
    plain(0x41, 0x61, A_CAP_A),
    plain(0x42, 0x62, A_CAP_B),
    plain(0x43, 0x63, A_CAP_C),
    plain(0x44, 0x64, A_CAP_D),
    plain(0x45, 0x65, A_CAP_E),
    plain(0x46, 0x66, A_CAP_F),
    plain(0x47, 0x67, A_CAP_G),
    plain(0x48, 0x68, A_CAP_H),
    plain(0x49, 0x69, A_CAP_I),
    plain(0x4a, 0x6a, A_CAP_J),
    plain(0x4b, 0x6b, A_CAP_K),
    plain(0x4c, 0x6c, A_CAP_L),
    plain(0x4d, 0x6d, A_CAP_M),
    plain(0x4e, 0x6e, A_CAP_N),
    plain(0x4f, 0x6f, A_CAP_O),
    plain(0x50, 0x70, A_CAP_P),
    plain(0x51, 0x71, A_CAP_Q),
    plain(0x52, 0x72, A_CAP_R),
    plain(0x53, 0x73, A_CAP_S),
    plain(0x54, 0x74, A_CAP_T),
    plain(0x55, 0x75, A_CAP_U),
    plain(0x56, 0x76, A_CAP_V),
    plain(0x57, 0x77, A_CAP_W),
    plain(0x58, 0x78, A_CAP_X),
    plain(0x59, 0x79, A_CAP_Y),
    plain(0x5a, 0x7a, A_CAP_Z),
    plain(0x5b, 0x5b, A_OPEN_SQUARE),
    plain(0x5c, 0x5c, A_BACKSLASH),
    plain(0x5d, 0x5d, A_CLOSE_SQUARE),
    plain(0x5e, 0x5e, A_CARET),
    plain(0x5f, 0x5f, A_UNDERSCORE),

    plain(0x60, 0x60, A_LEFT_SINGLE_QUOTE),
    plain(0x41, 0x61, A_LOW_A),
    plain(0x42, 0x62, A_LOW_B),
    plain(0x43, 0x63, A_LOW_C),
    plain(0x44, 0x64, A_LOW_D),
    plain(0x45, 0x65, A_LOW_E),
    plain(0x46, 0x66, A_LOW_F),
    plain(0x47, 0x67, A_LOW_G),
    plain(0x48, 0x68, A_LOW_H),
    plain(0x49, 0x69, A_LOW_I),
    plain(0x4a, 0x6a, A_LOW_J),
    plain(0x4b, 0x6b, A_LOW_K),
    plain(0x4c, 0x6c, A_LOW_L),
    plain(0x4d, 0x6d, A_LOW_M),
    plain(0x4e, 0x6e, A_LOW_N),
    plain(0x4f, 0x6f, A_LOW_O),
    plain(0x50, 0x70, A_LOW_P),
    plain(0x51, 0x71, A_LOW_Q),
    plain(0x52, 0x72, A_LOW_R),
    plain(0x53, 0x73, A_LOW_S),
    plain(0x54, 0x74, A_LOW_T),
    plain(0x55, 0x75, A_LOW_U),
    plain(0x56, 0x76, A_LOW_V),
    plain(0x57, 0x77, A_LOW_W),
    plain(0x58, 0x78, A_LOW_X),
    plain(0x59, 0x79, A_LOW_Y),
    plain(0x5a, 0x7a, A_LOW_Z),
    plain(0x7b, 0x7b, A_OPEN_BRACE),
    plain(0x7c, 0x7c, A_BAR),
    plain(0x7d, 0x7d, A_CLOSE_BRACE),
    plain(0x7e, 0x7e, A_TILDE),
    named(0x7f, 0x7f, b"DEL\0", A_DELETE),

    named(0x80, 0x80, b"EURO\0", A_EURO),
    named(0x81, 0x81, b"SHIFT\0", A_SHIFT2),
    named(0x82, 0x82, b"CTRL\0", A_CTRL2),
    named(0x83, 0x83, b"ALT\0", A_ALT2),
    menu(0x84, 0x84, b"F5\0", A_F5),
    menu(0x85, 0x85, b"F6\0", A_F6),
    menu(0x86, 0x86, b"F7\0", A_F7),
    menu(0x87, 0x87, b"F8\0", A_F8),
    named(0x88, 0x88, b"CIRCUMFLEX\0", A_CIRCUMFLEX),
    named(0x89, 0x89, b"MWHEELUP\0", A_MWHEELUP),
    // Raven flags this row: `upper` or `lower` is not the row index.
    plain(0x8a, 0x9a, A_CAP_SCARON),
    named(0x8b, 0x8b, b"MWHEELDOWN\0", A_MWHEELDOWN),
    // Raven flags this row: `upper` or `lower` is not the row index.
    plain(0x8c, 0x9c, A_CAP_OE),
    named(0x8d, 0x8d, b"MOUSE1\0", A_MOUSE1),
    named(0x8e, 0x8e, b"MOUSE2\0", A_MOUSE2),
    named(0x8f, 0x8f, b"INS\0", A_INSERT),
    named(0x90, 0x90, b"HOME\0", A_HOME),
    named(0x91, 0x91, b"PGUP\0", A_PAGE_UP),
    plain(0x92, 0x92, A_RIGHT_SINGLE_QUOTE),
    plain(0x93, 0x93, A_LEFT_DOUBLE_QUOTE),
    plain(0x94, 0x94, A_RIGHT_DOUBLE_QUOTE),
    menu(0x95, 0x95, b"F9\0", A_F9),
    menu(0x96, 0x96, b"F10\0", A_F10),
    menu(0x97, 0x97, b"F11\0", A_F11),
    menu(0x98, 0x98, b"F12\0", A_F12),
    plain(0x99, 0x99, A_TRADEMARK),
    // Raven flags this row: `upper` or `lower` is not the row index.
    plain(0x8a, 0x9a, A_LOW_SCARON),
    named(0x9b, 0x9b, b"SHIFT_ENTER\0", A_ENTER),
    // Raven flags this row: `upper` or `lower` is not the row index.
    plain(0x8c, 0x9c, A_LOW_OE),
    named(0x9d, 0x9d, b"END\0", A_END),
    named(0x9e, 0x9e, b"PGDN\0", A_PAGE_DOWN),
    // Raven flags this row: `upper` or `lower` is not the row index.
    plain(0x9f, 0xff, A_CAP_YDIERESIS),

    named(0xa0, 0x00, b"SHIFT_SPACE\0", A_SPACE),
    // Raven: upside down '!' - undisplayable.
    plain(0xa1, 0xa1, A_EXCLAMDOWN),
    plain(0xa2, 0xa2, A_CENT),
    plain(0xa3, 0xa3, A_POUND),
    named(0xa4, 0x00, b"SHIFT_KP_ENTER\0", A_KP_ENTER),
    plain(0xa5, 0xa5, A_YEN),
    named(0xa6, 0xa6, b"MOUSE3\0", A_MOUSE3),
    named(0xa7, 0xa7, b"MOUSE4\0", A_MOUSE4),
    named(0xa8, 0xa8, b"MOUSE5\0", A_MOUSE5),
    plain(0xa9, 0xa9, A_COPYRIGHT),
    named(0xaa, 0xaa, b"UPARROW\0", A_CURSOR_UP),
    named(0xab, 0xab, b"DOWNARROW\0", A_CURSOR_DOWN),
    named(0xac, 0xac, b"LEFTARROW\0", A_CURSOR_LEFT),
    named(0xad, 0xad, b"RIGHTARROW\0", A_CURSOR_RIGHT),
    plain(0xae, 0xae, A_REGISTERED),
    plain(0xaf, 0x00, A_UNDEFINED_7),
    plain(0xb0, 0x00, A_UNDEFINED_8),
    plain(0xb1, 0x00, A_UNDEFINED_9),
    plain(0xb2, 0x00, A_UNDEFINED_10),
    plain(0xb3, 0x00, A_UNDEFINED_11),
    plain(0xb4, 0x00, A_UNDEFINED_12),
    plain(0xb5, 0x00, A_UNDEFINED_13),
    plain(0xb6, 0x00, A_UNDEFINED_14),
    plain(0xb7, 0x00, A_UNDEFINED_15),
    plain(0xb8, 0x00, A_UNDEFINED_16),
    plain(0xb9, 0x00, A_UNDEFINED_17),
    plain(0xba, 0x00, A_UNDEFINED_18),
    plain(0xbb, 0x00, A_UNDEFINED_19),
    plain(0xbc, 0x00, A_UNDEFINED_20),
    plain(0xbd, 0x00, A_UNDEFINED_21),
    plain(0xbe, 0x00, A_UNDEFINED_22),
    plain(0xbf, 0xbf, A_QUESTION_DOWN),

    plain(0xc0, 0xe0, A_CAP_AGRAVE),
    plain(0xc1, 0xe1, A_CAP_AACUTE),
    plain(0xc2, 0xe2, A_CAP_ACIRCUMFLEX),
    plain(0xc3, 0xe3, A_CAP_ATILDE),
    plain(0xc4, 0xe4, A_CAP_ADIERESIS),
    plain(0xc5, 0xe5, A_CAP_ARING),
    plain(0xc6, 0xe6, A_CAP_AE),
    plain(0xc7, 0xe7, A_CAP_CCEDILLA),
    plain(0xc8, 0xe8, A_CAP_EGRAVE),
    plain(0xc9, 0xe9, A_CAP_EACUTE),
    plain(0xca, 0xea, A_CAP_ECIRCUMFLEX),
    plain(0xcb, 0xeb, A_CAP_EDIERESIS),
    plain(0xcc, 0xec, A_CAP_IGRAVE),
    plain(0xcd, 0xed, A_CAP_IACUTE),
    plain(0xce, 0xee, A_CAP_ICIRCUMFLEX),
    plain(0xcf, 0xef, A_CAP_IDIERESIS),
    plain(0xd0, 0xf0, A_CAP_ETH),
    plain(0xd1, 0xf1, A_CAP_NTILDE),
    plain(0xd2, 0xf2, A_CAP_OGRAVE),
    plain(0xd3, 0xf3, A_CAP_OACUTE),
    plain(0xd4, 0xf4, A_CAP_OCIRCUMFLEX),
    plain(0xd5, 0xf5, A_CAP_OTILDE),
    plain(0xd6, 0xf6, A_CAP_ODIERESIS),
    named(0xd7, 0xd7, b"KP_STAR\0", A_MULTIPLY),
    plain(0xd8, 0xf8, A_CAP_OSLASH),
    plain(0xd9, 0xf9, A_CAP_UGRAVE),
    plain(0xda, 0xfa, A_CAP_UACUTE),
    plain(0xdb, 0xfb, A_CAP_UCIRCUMFLEX),
    plain(0xdc, 0xfc, A_CAP_UDIERESIS),
    plain(0xdd, 0xfd, A_CAP_YACUTE),
    plain(0xde, 0xfe, A_CAP_THORN),
    plain(0xdf, 0xdf, A_GERMANDBLS),

    plain(0xc0, 0xe0, A_LOW_AGRAVE),
    plain(0xc1, 0xe1, A_LOW_AACUTE),
    plain(0xc2, 0xe2, A_LOW_ACIRCUMFLEX),
    plain(0xc3, 0xe3, A_LOW_ATILDE),
    plain(0xc4, 0xe4, A_LOW_ADIERESIS),
    plain(0xc5, 0xe5, A_LOW_ARING),
    plain(0xc6, 0xe6, A_LOW_AE),
    plain(0xc7, 0xe7, A_LOW_CCEDILLA),
    plain(0xc8, 0xe8, A_LOW_EGRAVE),
    plain(0xc9, 0xe9, A_LOW_EACUTE),
    plain(0xca, 0xea, A_LOW_ECIRCUMFLEX),
    plain(0xcb, 0xeb, A_LOW_EDIERESIS),
    plain(0xcc, 0xec, A_LOW_IGRAVE),
    plain(0xcd, 0xed, A_LOW_IACUTE),
    plain(0xce, 0xee, A_LOW_ICIRCUMFLEX),
    plain(0xcf, 0xef, A_LOW_IDIERESIS),
    plain(0xd0, 0xf0, A_LOW_ETH),
    plain(0xd1, 0xf1, A_LOW_NTILDE),
    plain(0xd2, 0xf2, A_LOW_OGRAVE),
    plain(0xd3, 0xf3, A_LOW_OACUTE),
    plain(0xd4, 0xf4, A_LOW_OCIRCUMFLEX),
    plain(0xd5, 0xf5, A_LOW_OTILDE),
    plain(0xd6, 0xf6, A_LOW_ODIERESIS),
    named(0xf7, 0xf7, b"KP_SLASH\0", A_DIVIDE),
    plain(0xd8, 0xf8, A_LOW_OSLASH),
    plain(0xd9, 0xf9, A_LOW_UGRAVE),
    plain(0xda, 0xfa, A_LOW_UACUTE),
    plain(0xdb, 0xfb, A_LOW_UCIRCUMFLEX),
    plain(0xdc, 0xfc, A_LOW_UDIERESIS),
    plain(0xdd, 0xfd, A_LOW_YACUTE),
    plain(0xde, 0xfe, A_LOW_THORN),
    // Raven flags this row: `upper` or `lower` is not the row index.
    plain(0x9f, 0xff, A_LOW_YDIERESIS),

    named(0x100, 0x100, b"JOY0\0", A_JOY0),
    named(0x101, 0x101, b"JOY1\0", A_JOY1),
    named(0x102, 0x102, b"JOY2\0", A_JOY2),
    named(0x103, 0x103, b"JOY3\0", A_JOY3),
    named(0x104, 0x104, b"JOY4\0", A_JOY4),
    named(0x105, 0x105, b"JOY5\0", A_JOY5),
    named(0x106, 0x106, b"JOY6\0", A_JOY6),
    named(0x107, 0x107, b"JOY7\0", A_JOY7),
    named(0x108, 0x108, b"JOY8\0", A_JOY8),
    named(0x109, 0x109, b"JOY9\0", A_JOY9),
    named(0x10a, 0x10a, b"JOY10\0", A_JOY10),
    named(0x10b, 0x10b, b"JOY11\0", A_JOY11),
    named(0x10c, 0x10c, b"JOY12\0", A_JOY12),
    named(0x10d, 0x10d, b"JOY13\0", A_JOY13),
    named(0x10e, 0x10e, b"JOY14\0", A_JOY14),
    named(0x10f, 0x10f, b"JOY15\0", A_JOY15),
    named(0x110, 0x110, b"JOY16\0", A_JOY16),
    named(0x111, 0x111, b"JOY17\0", A_JOY17),
    named(0x112, 0x112, b"JOY18\0", A_JOY18),
    named(0x113, 0x113, b"JOY19\0", A_JOY19),
    named(0x114, 0x114, b"JOY20\0", A_JOY20),
    named(0x115, 0x115, b"JOY21\0", A_JOY21),
    named(0x116, 0x116, b"JOY22\0", A_JOY22),
    named(0x117, 0x117, b"JOY23\0", A_JOY23),
    named(0x118, 0x118, b"JOY24\0", A_JOY24),
    named(0x119, 0x119, b"JOY25\0", A_JOY25),
    named(0x11a, 0x11a, b"JOY26\0", A_JOY26),
    named(0x11b, 0x11b, b"JOY27\0", A_JOY27),
    named(0x11c, 0x11c, b"JOY28\0", A_JOY28),
    named(0x11d, 0x11d, b"JOY29\0", A_JOY29),
    named(0x11e, 0x11e, b"JOY30\0", A_JOY30),
    named(0x11f, 0x11f, b"JOY31\0", A_JOY31),

    named(0x120, 0x120, b"AUX0\0", A_AUX0),
    named(0x121, 0x121, b"AUX1\0", A_AUX1),
    named(0x122, 0x122, b"AUX2\0", A_AUX2),
    named(0x123, 0x123, b"AUX3\0", A_AUX3),
    named(0x124, 0x124, b"AUX4\0", A_AUX4),
    named(0x125, 0x125, b"AUX5\0", A_AUX5),
    named(0x126, 0x126, b"AUX6\0", A_AUX6),
    named(0x127, 0x127, b"AUX7\0", A_AUX7),
    named(0x128, 0x128, b"AUX8\0", A_AUX8),
    named(0x129, 0x129, b"AUX9\0", A_AUX9),
    named(0x12a, 0x12a, b"AUX10\0", A_AUX10),
    named(0x12b, 0x12b, b"AUX11\0", A_AUX11),
    named(0x12c, 0x12c, b"AUX12\0", A_AUX12),
    named(0x12d, 0x12d, b"AUX13\0", A_AUX13),
    named(0x12e, 0x12e, b"AUX14\0", A_AUX14),
    named(0x12f, 0x12f, b"AUX15\0", A_AUX15),
    named(0x130, 0x130, b"AUX16\0", A_AUX16),
    named(0x131, 0x131, b"AUX17\0", A_AUX17),
    named(0x132, 0x132, b"AUX18\0", A_AUX18),
    named(0x133, 0x133, b"AUX19\0", A_AUX19),
    named(0x134, 0x134, b"AUX20\0", A_AUX20),
    named(0x135, 0x135, b"AUX21\0", A_AUX21),
    named(0x136, 0x136, b"AUX22\0", A_AUX22),
    named(0x137, 0x137, b"AUX23\0", A_AUX23),
    named(0x138, 0x138, b"AUX24\0", A_AUX24),
    named(0x139, 0x139, b"AUX25\0", A_AUX25),
    named(0x13a, 0x13a, b"AUX26\0", A_AUX26),
    named(0x13b, 0x13b, b"AUX27\0", A_AUX27),
    named(0x13c, 0x13c, b"AUX28\0", A_AUX28),
    named(0x13d, 0x13d, b"AUX29\0", A_AUX29),
    named(0x13e, 0x13e, b"AUX30\0", A_AUX30),
    named(0x13f, 0x13f, b"AUX31\0", A_AUX31),
];
