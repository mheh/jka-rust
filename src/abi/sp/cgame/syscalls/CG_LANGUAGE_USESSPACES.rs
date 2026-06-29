use super::super::SpCgameImport;
use crate::abi::generic::{
    DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::ffi::types::qboolean;

/// `CG_LANGUAGE_USESSPACES` SP cgame imports syscall ABI token.
///
/// Arguments for `CG_LANGUAGE_USESSPACES`.
///
/// Raven wrapper: `return syscall( CG_LANGUAGE_USESSPACES );`
/// Raven transport: `return re.Language_UsesSpaces();`
///
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:342-344`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:676-677`
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct CgLanguageUsesspacesArgs;

impl CgLanguageUsesspacesArgs {
    pub const fn new() -> Self {
        Self
    }
}

/// `CG_LANGUAGE_USESSPACES` SP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/code/cgame/cg_public.h:128`
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:342-344`
/// Output source: `oracle/oracle/code/client/cl_cgame.cpp:676-677`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:676-677`
pub struct CgLanguageUsesspaces;

impl OutboundSysCall for CgLanguageUsesspaces {
    type Import = SpCgameImport;
    type Args = CgLanguageUsesspacesArgs;
    type Output = qboolean;

    const IMPORT: SpCgameImport = SpCgameImport::CG_LANGUAGE_USESSPACES;
}

impl EncodeSysCall for CgLanguageUsesspaces {
    fn encode_syscall(_args: &Self::Args) -> SysCallTransport {
        SysCallTransport::empty()
    }
}

impl DecodeSysCallReturn for CgLanguageUsesspaces {
    fn decode_return(word: isize) -> Self::Output {
        word as qboolean
    }
}
