use core::ffi::c_char;

use super::super::SpCgameImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `CG_R_LOADWORLDMAP`.
///
/// Raven wrapper: `syscall( CG_R_LOADWORLDMAP, mapname );`
/// Raven transport: `re.LoadWorld( (const char *) VMA(1) );`
///
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:299-300`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:652-654`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgRLoadworldmapArgs {
    mapname: *const c_char,
}

impl CgRLoadworldmapArgs {
    pub const fn new(mapname: *const c_char) -> Self {
        Self { mapname }
    }
}

/// `CG_R_LOADWORLDMAP` SP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/code/cgame/cg_public.h:117`
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:299-300`
/// Output source: `oracle/oracle/code/client/cl_cgame.cpp:652-654`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:652-654`
pub struct CgRLoadworldmap;

impl OutboundSysCall for CgRLoadworldmap {
    type Import = SpCgameImport;
    type Args = CgRLoadworldmapArgs;
    type Output = ();

    const IMPORT: SpCgameImport = SpCgameImport::CG_R_LOADWORLDMAP;
}

impl EncodeSysCall for CgRLoadworldmap {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.mapname)])
    }
}

impl DecodeSysCallReturn for CgRLoadworldmap {
    fn decode_return(_word: isize) -> Self::Output {}
}
