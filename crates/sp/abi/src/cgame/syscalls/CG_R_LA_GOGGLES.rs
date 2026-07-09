use super::super::SpCgameImport;
use abi_transport::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// Arguments for `CG_R_LA_GOGGLES`.
///
/// Raven wrapper: `syscall( CG_R_LA_GOGGLES );`
/// Raven transport: `re.LAGoggles();`
///
/// Args source: `oracle/code/cgame/cg_syscalls.cpp:432-434`
/// Transport/switch source: `oracle/code/client/cl_cgame.cpp:743-745`
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct CgRLaGogglesArgs;

impl CgRLaGogglesArgs {
    pub const fn new() -> Self {
        Self
    }
}

/// `CG_R_LA_GOGGLES` SP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/code/cgame/cg_public.h:148`
/// Args source: `oracle/code/cgame/cg_syscalls.cpp:432-434`
/// Output source: `oracle/code/client/cl_cgame.cpp:743-745`
/// Transport/switch source: `oracle/code/client/cl_cgame.cpp:743-745`
pub struct CgRLaGoggles;

impl OutboundSysCall for CgRLaGoggles {
    type Import = SpCgameImport;
    type Args = CgRLaGogglesArgs;
    type Output = ();

    const IMPORT: SpCgameImport = SpCgameImport::CG_R_LA_GOGGLES;
}

impl EncodeSysCall for CgRLaGoggles {
    fn encode_syscall(_args: &Self::Args) -> SysCallTransport {
        SysCallTransport::empty()
    }
}

impl DecodeSysCallReturn for CgRLaGoggles {
    fn decode_return(_word: isize) -> Self::Output {}
}
