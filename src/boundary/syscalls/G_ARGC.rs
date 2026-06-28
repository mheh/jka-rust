use core::ffi::c_int;

use crate::ffi::GameImport;
use super::super::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `G_ARGC` outbound game-to-engine syscall.
///
/// Returns the number of tokens in the current command string.
/// C signature: `int trap_Argc( void )`
#[derive(Debug)]
pub struct GArgcArgs;

impl GArgcArgs {
    pub fn new() -> Self {
        GArgcArgs
    }
}

pub struct GArgc;

impl OutboundSysCall for GArgc {
    type Args = GArgcArgs;
    type Output = c_int;

    const IMPORT: GameImport = GameImport::G_ARGC;
}

impl EncodeSysCall for GArgc {
    fn encode_syscall(_a: &Self::Args) -> SysCallTransport {
        SysCallTransport::empty()
    }
}

impl DecodeSysCallReturn for GArgc {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
