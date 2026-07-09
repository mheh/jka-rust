use core::ffi::c_int;

use super::super::MpGameImport;

use abi_transport::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `BOTLIB_AI_ALLOC_MOVE_STATE` outbound game-to-engine syscall.
///
/// C ABI: `int trap_BotAllocMoveState(void)` — no arguments, returns a move
/// state handle as an `int`.
#[derive(Debug)]
pub struct BotlibAiAllocMoveStateArgs;

impl BotlibAiAllocMoveStateArgs {
    pub fn new() -> Self {
        Self
    }
}

/// `BOTLIB_AI_ALLOC_MOVE_STATE` MP game imports syscall ABI token.
///
/// Source: `oracle/codemp/game/g_public.h:471`
pub struct BotlibAiAllocMoveState;

impl OutboundSysCall for BotlibAiAllocMoveState {
    type Import = MpGameImport;
    type Args = BotlibAiAllocMoveStateArgs;
    type Output = c_int;

    const IMPORT: MpGameImport = MpGameImport::BOTLIB_AI_ALLOC_MOVE_STATE;
}

impl EncodeSysCall for BotlibAiAllocMoveState {
    fn encode_syscall(_a: &Self::Args) -> SysCallTransport {
        SysCallTransport::empty()
    }
}

impl DecodeSysCallReturn for BotlibAiAllocMoveState {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
