use core::ffi::{c_char, c_int};

use super::super::MpCgameImport;
use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::shared::qboolean;

/// Arguments for `CG_SP_GETSTRINGTEXTSTRING`.
///
/// Raven wrapper: `return syscall( CG_SP_GETSTRINGTEXTSTRING, text, buffer, bufferLength );`
/// Raven transport: `SE_GetString( (const char *) VMA(1) )`, then writes either
/// the translated string or a `??` fallback into `(char *)VMA(2)`.
///
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:725-727`
/// Args source: `oracle/oracle/codemp/cgame/cg_local.h:2419`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1654-1678`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgSpGetstringtextstringArgs {
    text: *const c_char,
    buffer: *mut c_char,
    buffer_length: c_int,
}

impl CgSpGetstringtextstringArgs {
    pub const fn new(text: *const c_char, buffer: *mut c_char, buffer_length: c_int) -> Self {
        Self {
            text,
            buffer,
            buffer_length,
        }
    }
}

/// `CG_SP_GETSTRINGTEXTSTRING` MP cgame imports syscall ABI token.
///
/// Raven comment: `// CG_SP_PRINT,`
/// Raven switch comment: `// case CG_SP_GETSTRINGTEXT:`
///
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:240`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:725-727`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:1654-1678`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1654-1678`
pub struct CgSpGetstringtextstring;

impl OutboundSysCall for CgSpGetstringtextstring {
    type Import = MpCgameImport;
    type Args = CgSpGetstringtextstringArgs;
    type Output = qboolean;

    const IMPORT: MpCgameImport = MpCgameImport::CG_SP_GETSTRINGTEXTSTRING;
}

impl EncodeSysCall for CgSpGetstringtextstring {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(args.text),
            ptr_to_word(args.buffer),
            args.buffer_length as isize,
        ])
    }
}

impl DecodeSysCallReturn for CgSpGetstringtextstring {
    fn decode_return(word: isize) -> Self::Output {
        word as qboolean
    }
}
