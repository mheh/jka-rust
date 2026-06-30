//! Compatibility names for generated MP game ABI calls.
//!
//! New ABI code should prefer `crate::abi::mp::game::*`; this module
//! keeps the generated call files compiling while they are migrated.

pub use crate::abi::mp::game::{MpGameExport as GameExport, MpGameImport as GameImport};

pub mod types {
    #![allow(non_camel_case_types)]

    pub use crate::shared::{qboolean, QFALSE, QTRUE};
}
