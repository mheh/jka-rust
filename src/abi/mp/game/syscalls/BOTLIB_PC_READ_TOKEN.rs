use core::ffi::c_int;

use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::codemp::game::q_shared_h::pc_token_t;
use crate::ffi::GameImport;

/// `BOTLIB_PC_READ_TOKEN` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct BotlibPcReadTokenArgs {
    pub handle: c_int,
    pub pc_token: *mut pc_token_t,
}

impl BotlibPcReadTokenArgs {
    pub fn new(handle: c_int, pc_token: *mut pc_token_t) -> Self {
        Self { handle, pc_token }
    }

    pub fn handle(&self) -> c_int {
        self.handle
    }

    pub fn pc_token(&self) -> *mut pc_token_t {
        self.pc_token
    }
}

/// `BOTLIB_PC_READ_TOKEN` MP game imports syscall ABI token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:500`
pub struct BotlibPcReadToken;

impl OutboundSysCall for BotlibPcReadToken {
    type Import = GameImport;
    type Args = BotlibPcReadTokenArgs;
    type Output = c_int;

    const IMPORT: GameImport = GameImport::BOTLIB_PC_READ_TOKEN;
}

impl EncodeSysCall for BotlibPcReadToken {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([a.handle as isize, ptr_to_word(a.pc_token)])
    }
}

impl DecodeSysCallReturn for BotlibPcReadToken {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
