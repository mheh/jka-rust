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
