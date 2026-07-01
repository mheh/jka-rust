use core::ffi::{c_char, c_int, c_void};

use super::super::MpCgameImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use mp_qshared::shared::qboolean;

/// Arguments for `CG_G2_SETROOTSURFACE`.
///
/// Raven wrapper: `return syscall(CG_G2_SETROOTSURFACE, ghoul2, modelIndex, surfaceName);`
/// Raven transport: `return G2API_SetRootSurface(*((CGhoul2Info_v *)args[1]), args[2], (const char *)VMA(3));`
///
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:955-957`
/// Args source: `oracle/oracle/codemp/cgame/cg_local.h:2559`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1498-1499`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgG2SetrootsurfaceArgs {
    ghoul2: *mut c_void,
    model_index: c_int,
    surface_name: *const c_char,
}

impl CgG2SetrootsurfaceArgs {
    pub const fn new(ghoul2: *mut c_void, model_index: c_int, surface_name: *const c_char) -> Self {
        Self {
            ghoul2,
            model_index,
            surface_name,
        }
    }
}

/// `CG_G2_SETROOTSURFACE` MP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:286`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:955-957`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:1498-1499`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1498-1499`
pub struct CgG2Setrootsurface;

impl OutboundSysCall for CgG2Setrootsurface {
    type Import = MpCgameImport;
    type Args = CgG2SetrootsurfaceArgs;
    type Output = qboolean;

    const IMPORT: MpCgameImport = MpCgameImport::CG_G2_SETROOTSURFACE;
}

impl EncodeSysCall for CgG2Setrootsurface {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(args.ghoul2),
            args.model_index as isize,
            ptr_to_word(args.surface_name),
        ])
    }
}

impl DecodeSysCallReturn for CgG2Setrootsurface {
    fn decode_return(word: isize) -> Self::Output {
        word as qboolean
    }
}
