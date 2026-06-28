use core::ffi::c_int;

use crate::ffi::GameImport;

use super::super::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `G_BOT_ALLOCATE_CLIENT` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct GBotAllocateClientArgs;

impl GBotAllocateClientArgs {
    pub fn new() -> Self {
        Self
    }
}

pub struct GBotAllocateClient;

impl OutboundSysCall for GBotAllocateClient {
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
