use core::ffi::{c_int, c_void};

use super::super::MpCgameImport;
use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::shared::qboolean;

/// Arguments for `CG_G2_SKINLESSMODEL`.
///
/// Raven wrapper: `qboolean trap_G2API_SkinlessModel(void *ghlInfo, int modelIndex)`.
/// Raven transport: `CGhoul2Info_v &g2 = *((CGhoul2Info_v *)args[1]); return G2API_SkinlessModel(&g2[args[2]]);`
///
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:278`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:915-917`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:1444-1448`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1444-1448`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgG2SkinlessmodelArgs {
    /// Raw Ghoul2 handle word, decoded by Raven as `args[1]`.
    ghl_info: *mut c_void,
    /// Model index, read by Raven as raw `args[2]`.
    model_index: c_int,
}

impl CgG2SkinlessmodelArgs {
    pub const fn new(ghl_info: *mut c_void, model_index: c_int) -> Self {
        Self {
            ghl_info,
            model_index,
        }
    }
}

/// `CG_G2_SKINLESSMODEL` MP cgame imports syscall ABI token.
///
/// Raven transport: `ghlInfo` is passed as a raw `args[1]` pointer word and
/// `modelIndex` as raw `args[2]`; the engine returns the int-compatible
/// `qboolean` result from `G2API_SkinlessModel`.
///
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:278`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:915-917`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:1444-1448`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1444-1448`
pub struct CgG2Skinlessmodel;

impl OutboundSysCall for CgG2Skinlessmodel {
    type Import = MpCgameImport;
    type Args = CgG2SkinlessmodelArgs;
    type Output = qboolean;

    const IMPORT: MpCgameImport = MpCgameImport::CG_G2_SKINLESSMODEL;
}

impl EncodeSysCall for CgG2Skinlessmodel {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.ghl_info), args.model_index as isize])
    }
}

impl DecodeSysCallReturn for CgG2Skinlessmodel {
    fn decode_return(word: isize) -> Self::Output {
        word as qboolean
    }
}
