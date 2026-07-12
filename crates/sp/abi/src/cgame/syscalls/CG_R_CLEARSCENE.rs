use super::super::SpCgameImport;
use abi_transport::generic::{
    DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `CG_R_CLEARSCENE` SP cgame imports syscall ABI token.
///
/// Raven wrapper: `cgi_R_ClearScene` calls `syscall( CG_R_CLEARSCENE );`
/// Raven comment: `Nothing is drawn until R_RenderScene is called.`
/// Enum value source: `oracle/code/cgame/cg_public.h:131`
/// Args source: `oracle/code/cgame/cg_syscalls.cpp:362-363`
/// Output source: `oracle/code/client/cl_cgame.cpp:686-688`
/// Transport/switch source: `oracle/code/client/cl_cgame.cpp:686-688`
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct CgRClearsceneArgs;

impl CgRClearsceneArgs {
    pub const fn new() -> Self {
        Self
    }
}

pub struct CgRClearscene;

impl OutboundSysCall for CgRClearscene {
    type Import = SpCgameImport;
    type Args = CgRClearsceneArgs;
    type Output = ();

    const IMPORT: SpCgameImport = SpCgameImport::CG_R_CLEARSCENE;
}

impl EncodeSysCall for CgRClearscene {
    fn encode_syscall(_args: &Self::Args) -> SysCallTransport {
        SysCallTransport::empty()
    }
}

impl DecodeSysCallReturn for CgRClearscene {
    fn decode_return(_word: isize) -> Self::Output {}
}
