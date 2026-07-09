use core::ffi::{c_int, c_void};

use super::super::MpCgameImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `CG_G2_GETNUMGOREMARKS`.
///
/// Raven wrapper: `int trap_G2API_GetNumGoreMarks(void *ghlInfo, int modelIndex)`.
/// Raven transport: `return G2API_GetNumGoreMarks(&g2[args[2]]);` under `_G2_GORE`,
/// otherwise the switch returns `0`.
///
/// Enum value source: `oracle/codemp/cgame/cg_public.h:279`
/// Args source: `oracle/codemp/cgame/cg_syscalls.c:920-922`
/// Output source: `oracle/codemp/client/cl_cgame.cpp:1450-1457`
/// Transport/switch source: `oracle/codemp/client/cl_cgame.cpp:1450-1457`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgG2GetnumgoremarksArgs {
    /// Raw Ghoul2 handle word, decoded by Raven as `args[1]`.
    ghl_info: *mut c_void,
    /// Model index, read by Raven as raw `args[2]`.
    model_index: c_int,
}

impl CgG2GetnumgoremarksArgs {
    pub const fn new(ghl_info: *mut c_void, model_index: c_int) -> Self {
        Self {
            ghl_info,
            model_index,
        }
    }
}

/// `CG_G2_GETNUMGOREMARKS` MP cgame imports syscall ABI token.
///
/// Raven transport: `ghlInfo` is passed as a raw `args[1]` pointer word and
/// `modelIndex` as raw `args[2]`; the `_G2_GORE` switch arm returns the
/// int-compatible count, otherwise `0`.
///
/// Enum value source: `oracle/codemp/cgame/cg_public.h:279`
/// Args source: `oracle/codemp/cgame/cg_syscalls.c:920-922`
/// Output source: `oracle/codemp/client/cl_cgame.cpp:1450-1457`
/// Transport/switch source: `oracle/codemp/client/cl_cgame.cpp:1450-1457`
pub struct CgG2Getnumgoremarks;

impl OutboundSysCall for CgG2Getnumgoremarks {
    type Import = MpCgameImport;
    type Args = CgG2GetnumgoremarksArgs;
    type Output = c_int;

    const IMPORT: MpCgameImport = MpCgameImport::CG_G2_GETNUMGOREMARKS;
}

impl EncodeSysCall for CgG2Getnumgoremarks {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.ghl_info), args.model_index as isize])
    }
}

impl DecodeSysCallReturn for CgG2Getnumgoremarks {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
