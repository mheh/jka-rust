use core::ffi::{c_char, c_int};
use std::ffi::CString;

use super::super::MpGameImport;

use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `G_CVAR_VARIABLE_STRING_BUFFER` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct GCvarVariableStringBufferArgs {
    var_name: CString,
    buffer: *mut c_char,
    bufsize: c_int,
}

impl GCvarVariableStringBufferArgs {
    pub fn new(var_name: CString, buffer: *mut c_char, bufsize: c_int) -> Self {
        Self {
            var_name,
            buffer,
            bufsize,
        }
    }

    pub fn var_name(&self) -> &CString {
        &self.var_name
    }

    pub fn buffer(&self) -> *mut c_char {
        self.buffer
    }

    pub fn bufsize(&self) -> c_int {
        self.bufsize
    }
}

/// `G_CVAR_VARIABLE_STRING_BUFFER` MP game imports syscall ABI token.
///
/// Raven: ( const char *var_name, char *buffer, int bufsize );
/// Source: `oracle/codemp/game/g_public.h:126`
pub struct GCvarVariableStringBuffer;

impl OutboundSysCall for GCvarVariableStringBuffer {
    type Import = MpGameImport;
    type Args = GCvarVariableStringBufferArgs;
    type Output = ();

    const IMPORT: MpGameImport = MpGameImport::G_CVAR_VARIABLE_STRING_BUFFER;
}

impl EncodeSysCall for GCvarVariableStringBuffer {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(a.var_name.as_ptr()),
            ptr_to_word(a.buffer),
            a.bufsize as isize,
        ])
    }
}

impl DecodeSysCallReturn for GCvarVariableStringBuffer {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
