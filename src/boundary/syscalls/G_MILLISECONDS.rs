use core::ffi::c_int;

use crate::ffi::GameImport;

use super::super::generic::{
    DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `G_MILLISECONDS`.
///
/// `trap_Milliseconds` takes no arguments; this is a unit-like carrier kept for
/// symmetry with the other typed boundary definitions.
#[derive(Debug, Default)]
pub struct GMillisecondsArgs;

impl GMillisecondsArgs {
    pub const fn new() -> Self {
        Self
    }
}

/// `G_MILLISECONDS` outbound game-to-engine syscall.
pub struct GMilliseconds;

impl OutboundSysCall for GMilliseconds {
    type Args = GMillisecondsArgs;
    type Output = c_int;

    const IMPORT: GameImport = GameImport::G_MILLISECONDS;
}

impl EncodeSysCall for GMilliseconds {
    fn encode_syscall(_args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([] as [isize; 0])
    }
}

impl DecodeSysCallReturn for GMilliseconds {
    // `trap_Milliseconds` returns `int`; the engine's return word is that value.
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
