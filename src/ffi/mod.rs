//! Compatibility names for generated MP game ABI calls.
//!
//! New ABI code should prefer `crate::abi::mp::game::*`; this module
//! keeps the generated call files compiling while they are migrated.

pub use crate::abi::mp::game::{MpGameExport as GameExport, MpGameImport as GameImport};

pub mod types {
    #![allow(non_camel_case_types)]

    use core::ffi::{c_char, c_float, c_int};

    pub use crate::shared::{qboolean, QFALSE, QTRUE};

    pub type fileHandle_t = c_int;
    pub type cvarHandle_t = c_int;

    pub const MAX_CVAR_VALUE_STRING: usize = 256;

    #[repr(C)]
    #[derive(Clone, Copy)]
    pub struct vmCvar_t {
        pub handle: cvarHandle_t,
        pub modificationCount: c_int,
        pub value: c_float,
        pub integer: c_int,
        pub string: [c_char; MAX_CVAR_VALUE_STRING],
    }

    impl vmCvar_t {
        pub const fn zeroed() -> Self {
            vmCvar_t {
                handle: 0,
                modificationCount: 0,
                value: 0.0,
                integer: 0,
                string: [0; MAX_CVAR_VALUE_STRING],
            }
        }
    }

    impl Default for vmCvar_t {
        fn default() -> Self {
            Self::zeroed()
        }
    }
}
