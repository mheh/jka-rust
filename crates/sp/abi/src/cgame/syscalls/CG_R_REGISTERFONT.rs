use core::ffi::c_char;

use super::super::SpCgameImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use sp_qshared::shared::qhandle_t;

/// Arguments for `CG_R_REGISTERFONT`.
///
/// Raven wrapper: `return syscall( CG_R_REGISTERFONT, fontName );`
/// Raven transport: `return re.RegisterFont( (const char *) VMA(1) );`
///
/// Args source: `oracle/code/cgame/cg_syscalls.cpp:321-322`
/// Transport/switch source: `oracle/code/client/cl_cgame.cpp:663-664`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgRRegisterfontArgs {
    font_name: *const c_char,
}

impl CgRRegisterfontArgs {
    pub const fn new(font_name: *const c_char) -> Self {
        Self { font_name }
    }
}

/// `CG_R_REGISTERFONT` SP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/code/cgame/cg_public.h:122`
/// Args source: `oracle/code/cgame/cg_syscalls.cpp:321-322`
/// Output source: `oracle/code/client/cl_cgame.cpp:663-664`
/// Transport/switch source: `oracle/code/client/cl_cgame.cpp:663-664`
pub struct CgRRegisterfont;

impl OutboundSysCall for CgRRegisterfont {
    type Import = SpCgameImport;
    type Args = CgRRegisterfontArgs;
    type Output = qhandle_t;

    const IMPORT: SpCgameImport = SpCgameImport::CG_R_REGISTERFONT;
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
