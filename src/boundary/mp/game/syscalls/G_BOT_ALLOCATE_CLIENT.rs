use core::ffi::c_int;

use crate::ffi::GameImport;

use crate::boundary::generic::{
    DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `G_BOT_ALLOCATE_CLIENT` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct GBotAllocateClientArgs;

impl GBotAllocateClientArgs {
    pub fn new() -> Self {
        Self
    }
}

/// `G_BOT_ALLOCATE_CLIENT` MP game imports syscall boundary token.
///
/// Raven: ( void );
/// Source: `oracle/oracle/codemp/game/g_public.h:215`
pub struct GBotAllocateClient;

impl OutboundSysCall for GBotAllocateClient {
    type Import = GameImport;
    type Args = GBotAllocateClientArgs;
    type Output = c_int;

    const IMPORT: GameImport = GameImport::G_BOT_ALLOCATE_CLIENT;
}

impl EncodeSysCall for GBotAllocateClient {
    fn encode_syscall(_a: &Self::Args) -> SysCallTransport {
        SysCallTransport::empty()
    }
}

impl DecodeSysCallReturn for GBotAllocateClient {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
