use super::super::MpGameImport;

use crate::abi::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `G_ICARUS_INIT` outbound game-to-engine syscall.
///
/// Initialises the engine-side ICARUS scripting instance. Called at `GAME_INIT`.
/// C ABI: `void trap_ICARUS_Init(void)` — no arguments, no return value.
#[derive(Debug)]
pub struct GIcarusInitArgs;

impl GIcarusInitArgs {
    pub fn new() -> Self {
        GIcarusInitArgs
    }
}

/// `G_ICARUS_INIT` MP game imports syscall ABI token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:255`
pub struct GIcarusInit;

impl OutboundSysCall for GIcarusInit {
    type Import = MpGameImport;
    type Args = GIcarusInitArgs;
    type Output = ();

    const IMPORT: MpGameImport = MpGameImport::G_ICARUS_INIT;
}

impl EncodeSysCall for GIcarusInit {
    fn encode_syscall(_a: &Self::Args) -> SysCallTransport {
        SysCallTransport::empty()
    }
}

impl DecodeSysCallReturn for GIcarusInit {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
