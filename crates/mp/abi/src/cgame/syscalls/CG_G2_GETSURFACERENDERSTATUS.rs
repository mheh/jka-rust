use core::ffi::{c_char, c_int, c_void};

use super::super::MpCgameImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `CG_G2_GETSURFACERENDERSTATUS`.
///
/// Raven wrapper: `return syscall(CG_G2_GETSURFACERENDERSTATUS, ghoul2, modelIndex, surfaceName);`
/// Raven transport: `return G2API_GetSurfaceRenderStatus(&g2[args[2]], (const char *)VMA(3));`
///
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:976-978`
/// Args source: `oracle/oracle/codemp/cgame/cg_local.h:2563`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1513-1518`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgG2GetsurfacerenderstatusArgs {
    ghoul2: *mut c_void,
    model_index: c_int,
    surface_name: *const c_char,
}

impl CgG2GetsurfacerenderstatusArgs {
    pub const fn new(ghoul2: *mut c_void, model_index: c_int, surface_name: *const c_char) -> Self {
        Self {
            ghoul2,
            model_index,
            surface_name,
        }
    }
}

/// `CG_G2_GETSURFACERENDERSTATUS` MP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:290`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:976-978`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:1513-1518`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1513-1518`
pub struct CgG2Getsurfacerenderstatus;

impl OutboundSysCall for CgG2Getsurfacerenderstatus {
    type Import = MpCgameImport;
    type Args = CgG2GetsurfacerenderstatusArgs;
    type Output = c_int;

    const IMPORT: MpCgameImport = MpCgameImport::CG_G2_GETSURFACERENDERSTATUS;
}

impl EncodeSysCall for CgG2Getsurfacerenderstatus {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(args.ghoul2),
            args.model_index as isize,
            ptr_to_word(args.surface_name),
        ])
    }
}

impl DecodeSysCallReturn for CgG2Getsurfacerenderstatus {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
