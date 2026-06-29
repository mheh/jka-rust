use super::super::MpCgameImport;
use crate::abi::generic::{
    DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::ffi::types::qboolean;

/// Arguments for `CG_LANGUAGE_ISASIAN`.
///
/// Raven wrapper: `return syscall( CG_LANGUAGE_ISASIAN );`
/// Raven transport: `return re.Language_IsAsian();`
///
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:307-309`
/// Args source: `oracle/oracle/codemp/cgame/cg_local.h:2258`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:882-883`
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct CgLanguageIsasianArgs;

impl CgLanguageIsasianArgs {
    pub const fn new() -> Self {
        Self
    }
}

/// `CG_LANGUAGE_ISASIAN` MP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:126`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:307-309`
/// Output source: `oracle/oracle/codemp/cgame/cg_local.h:2258`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:882-883`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:882-883`
pub struct CgLanguageIsasian;

impl OutboundSysCall for CgLanguageIsasian {
    type Import = MpCgameImport;
    type Args = CgLanguageIsasianArgs;
    type Output = qboolean;

    const IMPORT: MpCgameImport = MpCgameImport::CG_LANGUAGE_ISASIAN;
}

impl EncodeSysCall for CgLanguageIsasian {
    fn encode_syscall(_args: &Self::Args) -> SysCallTransport {
        SysCallTransport::empty()
    }
}

impl DecodeSysCallReturn for CgLanguageIsasian {
    fn decode_return(word: isize) -> Self::Output {
        word as qboolean
    }
}
