#![allow(non_camel_case_types)]

/// Opaque stand-in for C stdio `FILE`.
///
/// Only ever appears behind a raw pointer (`*mut FILE`); it is never
/// instantiated on the Rust side.
#[repr(C)]
pub struct FILE {
    _opaque: [u8; 0],
}
