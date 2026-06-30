use super::super::SpCgameImport;
use crate::abi::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};
use crate::ffi::types::qboolean;

/// Arguments for `CG_LANGUAGE_ISASIAN`.
///
/// Raven wrapper: `return syscall( CG_LANGUAGE_ISASIAN );`
/// Raven transport: `return re.Language_IsAsian();`
///
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:337-339`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:674-675`
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct CgLanguageIsasianArgs;

impl CgLanguageIsasianArgs {
    pub const fn new() -> Self {
        Self
    }
}

/// `CG_LANGUAGE_ISASIAN` SP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/code/cgame/cg_public.h:127`
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:337-339`
/// Output source: `oracle/oracle/code/client/cl_cgame.cpp:674-675`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:674-675`
pub struct CgLanguageIsasian;

impl OutboundSysCall for CgLanguageIsasian {
    type Import = SpCgameImport;
    type Args = CgLanguageIsasianArgs;
    type Output = qboolean;

    const IMPORT: SpCgameImport = SpCgameImport::CG_LANGUAGE_ISASIAN;
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
