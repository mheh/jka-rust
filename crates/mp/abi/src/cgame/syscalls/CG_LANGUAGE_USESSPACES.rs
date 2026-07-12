use super::super::MpCgameImport;
use abi_transport::generic::{
    DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use mp_qshared::shared::qboolean;

/// Arguments for `CG_LANGUAGE_USESSPACES`.
///
/// Raven wrapper: `return syscall( CG_LANGUAGE_USESSPACES );`
/// Raven transport: `return re.Language_UsesSpaces();`
///
/// Args source: `oracle/codemp/cgame/cg_syscalls.c:312-314`
/// Args source: `oracle/codemp/cgame/cg_local.h:2259`
/// Transport/switch source: `oracle/codemp/client/cl_cgame.cpp:884-885`
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct CgLanguageUsesspacesArgs;

impl CgLanguageUsesspacesArgs {
    pub const fn new() -> Self {
        Self
    }
}

/// `CG_LANGUAGE_USESSPACES` MP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/codemp/cgame/cg_public.h:127`
/// Args source: `oracle/codemp/cgame/cg_syscalls.c:312-314`
/// Output source: `oracle/codemp/cgame/cg_local.h:2259`
/// Output source: `oracle/codemp/client/cl_cgame.cpp:884-885`
/// Transport/switch source: `oracle/codemp/client/cl_cgame.cpp:884-885`
pub struct CgLanguageUsesspaces;

impl OutboundSysCall for CgLanguageUsesspaces {
    type Import = MpCgameImport;
    type Args = CgLanguageUsesspacesArgs;
    type Output = qboolean;

    const IMPORT: MpCgameImport = MpCgameImport::CG_LANGUAGE_USESSPACES;
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
