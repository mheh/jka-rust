use super::super::MpCgameImport;
use abi_transport::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};
use mp_qshared::shared::qboolean;

/// Arguments for `CG_R_INITWIREFRAMEAUTO`.
///
/// Raven: initialize automap -rww.
/// Raven wrapper: `return syscall( CG_R_INITWIREFRAMEAUTO );`
/// Raven transport: `return R_InitializeWireframeAutomap();`
///
/// Args source: `oracle/codemp/cgame/cg_syscalls.c:445-448`
/// Args source: `oracle/codemp/cgame/cg_local.h:2305`
/// Transport/switch source: `oracle/codemp/client/cl_cgame.cpp:1079-1080`
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct CgRInitwireframeautoArgs;

impl CgRInitwireframeautoArgs {
    pub const fn new() -> Self {
        Self
    }
}

/// `CG_R_INITWIREFRAMEAUTO` MP cgame imports syscall ABI token.
///
/// Raven: initialize automap -rww
/// Enum value source: `oracle/codemp/cgame/cg_public.h:175`
/// Args source: `oracle/codemp/cgame/cg_syscalls.c:445-448`
/// Output source: `oracle/codemp/client/cl_cgame.cpp:1079-1080`
/// Transport/switch source: `oracle/codemp/client/cl_cgame.cpp:1079-1080`
pub struct CgRInitwireframeauto;

impl OutboundSysCall for CgRInitwireframeauto {
    type Import = MpCgameImport;
    type Args = CgRInitwireframeautoArgs;
    type Output = qboolean;

    const IMPORT: MpCgameImport = MpCgameImport::CG_R_INITWIREFRAMEAUTO;
}

impl EncodeSysCall for CgRInitwireframeauto {
    fn encode_syscall(_args: &Self::Args) -> SysCallTransport {
        SysCallTransport::empty()
    }
}

impl DecodeSysCallReturn for CgRInitwireframeauto {
    fn decode_return(word: isize) -> Self::Output {
        word as qboolean
    }
}
