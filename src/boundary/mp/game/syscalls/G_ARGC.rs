use core::ffi::c_int;

use crate::boundary::generic::{
    DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::ffi::GameImport;

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

/// `G_ARGC` MP game imports syscall boundary token.
///
/// Raven: ( void );
/// Raven: ClientCommand and ServerCommand parameter access
/// Source: `oracle/oracle/codemp/game/g_public.h:128`
pub struct GArgc;

impl OutboundSysCall for GArgc {
    type Import = GameImport;
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
