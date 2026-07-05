//! Shared platform compatibility aliases used across Raven-derived code.
//!
//! Source: `oracle/oracle/codemp/qcommon/platform.h:13-20`

#![allow(non_camel_case_types)]

use core::ffi::{c_char, c_uchar, c_uint, c_ulong, c_void};

// #if defined (__linux__)
pub type LPCTSTR = *const c_char;
pub type LPCSTR = *const c_char;
pub type DWORD = c_ulong;
pub type UINT = c_uint;
pub type HANDLE = *mut c_void;
pub type COLORREF = DWORD;
pub type BYTE = c_uchar;
// #endif

/// Fatal print+exit primitive `mp_engine_core::sys_error` delegates to
/// (LIFE-D3: `core` already depends downhill on `native/platform`). The Win32
/// message-box/console-show surface of Raven's `Sys_Error` is deferred to the
/// client-shell slice (headless dedicated needs only print + exit; DEC-01).
///
/// Source: `oracle/oracle/codemp/win32/win_main.cpp:350` (print + exit tail);
/// dedicated `oracle/oracle/codemp/null/win_main.cpp:324`.
pub fn sys_fatal_print_exit(msg: &str) -> ! {
    eprintln!("Sys_Error: {msg}");
    std::process::exit(1)
}
