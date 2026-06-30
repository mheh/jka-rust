use core::ffi::c_char;

use super::super::MpCgameImport;
use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::shared::qhandle_t;

/// Arguments for `CG_R_REGISTERFONT`.
///
/// Raven wrapper: `return syscall( CG_R_REGISTERFONT, fontName);`
/// Raven transport: `return re.RegisterFont( (const char *)VMA(1) );`
///
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:282-284`
/// Args source: `oracle/oracle/codemp/cgame/cg_local.h:2253`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:871-872`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgRRegisterfontArgs {
    font_name: *const c_char,
}

impl CgRRegisterfontArgs {
    pub const fn new(font_name: *const c_char) -> Self {
        Self { font_name }
    }
}

/// `CG_R_REGISTERFONT` MP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:121`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:282-284`
/// Output source: `oracle/oracle/codemp/cgame/cg_local.h:2253`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:871-872`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:871-872`
pub struct CgRRegisterfont;

impl OutboundSysCall for CgRRegisterfont {
    type Import = MpCgameImport;
    type Args = CgRRegisterfontArgs;
    type Output = qhandle_t;

    const IMPORT: MpCgameImport = MpCgameImport::CG_R_REGISTERFONT;
}

impl EncodeSysCall for CgRRegisterfont {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.font_name)])
    }
}

impl DecodeSysCallReturn for CgRRegisterfont {
    fn decode_return(word: isize) -> Self::Output {
        word as qhandle_t
    }
}
