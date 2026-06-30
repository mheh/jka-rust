use core::ffi::c_int;

use super::super::MpGameImport;
use crate::abi::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

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

/// `G_ARGC` MP game imports syscall ABI token.
///
/// Raven: ( void );
/// Raven: ClientCommand and ServerCommand parameter access
/// Source: `oracle/oracle/codemp/game/g_public.h:128`
pub struct GArgc;

impl OutboundSysCall for GArgc {
    type Import = MpGameImport;
    type Args = GArgcArgs;
    type Output = c_int;

    const IMPORT: MpGameImport = MpGameImport::G_ARGC;
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
