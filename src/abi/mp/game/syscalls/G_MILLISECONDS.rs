use core::ffi::c_int;

use crate::ffi::GameImport;

use crate::abi::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// Arguments for `G_MILLISECONDS`.
///
/// `trap_Milliseconds` takes no arguments; this is a unit-like carrier kept for
/// symmetry with the other typed ABI definitions.
#[derive(Debug, Default)]
pub struct GMillisecondsArgs;

impl GMillisecondsArgs {
    pub const fn new() -> Self {
        Self
    }
}

/// `G_MILLISECONDS` MP game imports syscall ABI token.
///
/// Raven: ( void );
/// Raven: get current time for profiling reasons
/// Raven: this should NOT be used for any game related tasks,
/// Raven: because it is not journaled
/// Raven: Also for profiling.. do not use for game related tasks.
/// Source: `oracle/oracle/codemp/game/g_public.h:111`
pub struct GMilliseconds;

impl OutboundSysCall for GMilliseconds {
    type Import = GameImport;
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
