use core::ffi::{c_char, c_int, c_void};

use super::super::MpCgameImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use mp_qshared::shared::qboolean;

/// Arguments for `CG_G2_SETSURFACEONOFF`.
///
/// Raven wrapper: `return syscall(CG_G2_SETSURFACEONOFF, ghoul2, surfaceName, flags);`
/// Raven transport: `return G2API_SetSurfaceOnOff(*((CGhoul2Info_v *)args[1]), (const char *)VMA(2), args[3]);`
///
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:960-962`
/// Args source: `oracle/oracle/codemp/cgame/cg_local.h:2560`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1501-1502`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgG2SetsurfaceonoffArgs {
    ghoul2: *mut c_void,
    surface_name: *const c_char,
    flags: c_int,
}

impl CgG2SetsurfaceonoffArgs {
    pub const fn new(ghoul2: *mut c_void, surface_name: *const c_char, flags: c_int) -> Self {
        Self {
            ghoul2,
            surface_name,
            flags,
        }
    }
}

/// `CG_G2_SETSURFACEONOFF` MP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:287`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:960-962`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:1501-1502`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1501-1502`
pub struct CgG2Setsurfaceonoff;

impl OutboundSysCall for CgG2Setsurfaceonoff {
    type Import = MpCgameImport;
    type Args = CgG2SetsurfaceonoffArgs;
    type Output = qboolean;

    const IMPORT: MpCgameImport = MpCgameImport::CG_G2_SETSURFACEONOFF;
}

impl EncodeSysCall for CgG2Setsurfaceonoff {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(args.ghoul2),
            ptr_to_word(args.surface_name),
            args.flags as isize,
        ])
    }
}

impl DecodeSysCallReturn for CgG2Setsurfaceonoff {
    fn decode_return(word: isize) -> Self::Output {
        word as qboolean
    }
}
