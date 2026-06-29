use super::super::MpCgameImport;
use crate::boundary::generic::{
    DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `CG_R_CLEARSCENE`.
///
/// Raven wrapper: `syscall( CG_R_CLEARSCENE );`
/// Raven transport: `re.ClearScene(); return 0;`
///
/// Raven comment: `Nothing is drawn until R_RenderScene is called.`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:322-323`
/// Args source: `oracle/oracle/codemp/cgame/cg_local.h:2264-2265`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:888-890`
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct CgRClearsceneArgs;

impl CgRClearsceneArgs {
    pub const fn new() -> Self {
        Self
    }
}

/// `CG_R_CLEARSCENE` MP cgame imports syscall boundary token.
///
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:149`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:322-323`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:888-890`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:888-890`
pub struct CgRClearscene;

impl OutboundSysCall for CgRClearscene {
    type Import = MpCgameImport;
    type Args = CgRClearsceneArgs;
    type Output = ();

    const IMPORT: MpCgameImport = MpCgameImport::CG_R_CLEARSCENE;
}

impl EncodeSysCall for CgRClearscene {
    fn encode_syscall(_args: &Self::Args) -> SysCallTransport {
        SysCallTransport::empty()
    }
}

impl DecodeSysCallReturn for CgRClearscene {
    fn decode_return(_word: isize) -> Self::Output {}
}
