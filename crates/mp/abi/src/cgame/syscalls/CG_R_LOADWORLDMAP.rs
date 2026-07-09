use core::ffi::c_char;

use super::super::MpCgameImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `CG_R_LOADWORLDMAP`.
///
/// Raven wrapper: `syscall( CG_R_LOADWORLDMAP, mapname );`
/// Raven transport: `re.LoadWorld( (const char *)VMA(1) ); return 0;`
///
/// Args source: `oracle/codemp/cgame/cg_syscalls.c:262-263`
/// Args source: `oracle/codemp/cgame/cg_local.h:2245`
/// Transport/switch source: `oracle/codemp/client/cl_cgame.cpp:860-862`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgRLoadworldmapArgs {
    mapname: *const c_char,
}

impl CgRLoadworldmapArgs {
    pub const fn new(mapname: *const c_char) -> Self {
        Self { mapname }
    }
}

/// `CG_R_LOADWORLDMAP` MP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/codemp/cgame/cg_public.h:116`
/// Args source: `oracle/codemp/cgame/cg_syscalls.c:262-263`
/// Output source: `oracle/codemp/client/cl_cgame.cpp:860-862`
/// Transport/switch source: `oracle/codemp/client/cl_cgame.cpp:860-862`
pub struct CgRLoadworldmap;

impl OutboundSysCall for CgRLoadworldmap {
    type Import = MpCgameImport;
    type Args = CgRLoadworldmapArgs;
    type Output = ();

    const IMPORT: MpCgameImport = MpCgameImport::CG_R_LOADWORLDMAP;
}

impl EncodeSysCall for CgRLoadworldmap {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.mapname)])
    }
}

impl DecodeSysCallReturn for CgRLoadworldmap {
    fn decode_return(_word: isize) -> Self::Output {}
}
