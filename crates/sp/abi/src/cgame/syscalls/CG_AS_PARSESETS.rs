use super::super::SpCgameImport;
use abi_transport::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// Arguments for `CG_AS_PARSESETS`.
///
/// Raven wrapper: `syscall( CG_AS_PARSESETS );`
/// Raven transport: `AS_ParseSets();`
///
/// Args source: `oracle/code/cgame/cg_syscalls.cpp:189-190`
/// Transport/switch source: `oracle/code/client/cl_cgame.cpp:572-574`
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct CgAsParsesetsArgs;

impl CgAsParsesetsArgs {
    pub const fn new() -> Self {
        Self
    }
}

/// `CG_AS_PARSESETS` SP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/code/cgame/cg_public.h:164`
/// Args source: `oracle/code/cgame/cg_syscalls.cpp:189-190`
/// Output source: `oracle/code/client/cl_cgame.cpp:572-574`
/// Transport/switch source: `oracle/code/client/cl_cgame.cpp:572-574`
pub struct CgAsParsesets;

impl OutboundSysCall for CgAsParsesets {
    type Import = SpCgameImport;
    type Args = CgAsParsesetsArgs;
    type Output = ();

    const IMPORT: SpCgameImport = SpCgameImport::CG_AS_PARSESETS;
}

impl EncodeSysCall for CgAsParsesets {
    fn encode_syscall(_args: &Self::Args) -> SysCallTransport {
        SysCallTransport::empty()
    }
}

impl DecodeSysCallReturn for CgAsParsesets {
    fn decode_return(_word: isize) -> Self::Output {}
}
