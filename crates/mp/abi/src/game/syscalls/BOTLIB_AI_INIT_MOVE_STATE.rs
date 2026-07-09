use core::ffi::{c_int, c_void};

use super::super::MpGameImport;

use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `BOTLIB_AI_INIT_MOVE_STATE` outbound game-to-engine syscall.
///
/// C ABI: `void trap_BotInitMoveState(int handle, void *initmove)`
#[derive(Debug)]
pub struct BotlibAiInitMoveStateArgs {
    handle: c_int,
    initmove: *const c_void,
}

impl BotlibAiInitMoveStateArgs {
    pub fn new(handle: c_int, initmove: *const c_void) -> Self {
        Self { handle, initmove }
    }

    pub fn handle(&self) -> c_int {
        self.handle
    }

    pub fn initmove(&self) -> *const c_void {
        self.initmove
    }
}

/// `BOTLIB_AI_INIT_MOVE_STATE` MP game imports syscall ABI token.
///
/// Source: `oracle/codemp/game/g_public.h:473`
pub struct BotlibAiInitMoveState;

impl OutboundSysCall for BotlibAiInitMoveState {
    type Import = MpGameImport;
    type Args = BotlibAiInitMoveStateArgs;
    type Output = ();

    const IMPORT: MpGameImport = MpGameImport::BOTLIB_AI_INIT_MOVE_STATE;
}

impl EncodeSysCall for BotlibAiInitMoveState {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([a.handle as isize, ptr_to_word(a.initmove)])
    }
}

impl DecodeSysCallReturn for BotlibAiInitMoveState {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
