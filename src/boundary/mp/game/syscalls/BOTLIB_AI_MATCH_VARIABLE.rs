use core::ffi::c_int;

use crate::ffi::GameImport;

use crate::boundary::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `BOTLIB_AI_MATCH_VARIABLE` outbound game-to-engine syscall.
///
/// C signature:
/// `void trap_BotMatchVariable(void *match, int variable, char *buf, int size)`
#[derive(Debug)]
pub struct BotlibAiMatchVariableArgs {
    /// Pointer to a `bot_match_s` struct (opaque to the game VM).
    match_ptr: *mut core::ffi::c_void,
    /// Variable index to retrieve.
    variable: c_int,
    /// Output buffer the engine writes the variable string into.
    buf: *mut u8,
    /// Size of the output buffer.
    size: c_int,
}

impl BotlibAiMatchVariableArgs {
    pub fn new(
        match_ptr: *mut core::ffi::c_void,
        variable: c_int,
        buf: *mut u8,
        size: c_int,
    ) -> Self {
        Self {
            match_ptr,
            variable,
            buf,
            size,
        }
    }

    pub fn match_ptr(&self) -> *mut core::ffi::c_void {
        self.match_ptr
    }
    pub fn variable(&self) -> c_int {
        self.variable
    }
    pub fn buf(&self) -> *mut u8 {
        self.buf
    }
    pub fn size(&self) -> c_int {
        self.size
    }
}

/// `BOTLIB_AI_MATCH_VARIABLE` MP game imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:433`
pub struct BotlibAiMatchVariable;

impl OutboundSysCall for BotlibAiMatchVariable {
    type Import = GameImport;
    type Args = BotlibAiMatchVariableArgs;
    type Output = ();

    const IMPORT: GameImport = GameImport::BOTLIB_AI_MATCH_VARIABLE;
}

impl EncodeSysCall for BotlibAiMatchVariable {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(a.match_ptr as *const u8),
            a.variable as isize,
            ptr_to_word(a.buf),
            a.size as isize,
        ])
    }
}

impl DecodeSysCallReturn for BotlibAiMatchVariable {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
