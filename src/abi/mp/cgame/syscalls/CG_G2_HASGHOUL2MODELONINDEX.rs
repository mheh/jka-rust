use core::ffi::{c_int, c_void};

use super::super::MpCgameImport;
use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::shared::qboolean;

/// Arguments for `CG_G2_HASGHOUL2MODELONINDEX`.
///
/// Raven wrapper: `qboolean trap_G2API_HasGhoul2ModelOnIndex(void *ghlInfo, int modelIndex)`.
/// Raven transport: `return G2API_HasGhoul2ModelOnIndex((CGhoul2Info_v **)VMA(1), args[2]);`
///
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:276`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:905-907`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:1433-1435`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1433-1435`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgG2Hasghoul2modelonindexArgs {
    /// Raw Ghoul2 handle word, decoded by Raven as `(CGhoul2Info_v **)VMA(1)`.
    ghl_info: *mut c_void,
    /// Model index, read by Raven as raw `args[2]`.
    model_index: c_int,
}

impl CgG2Hasghoul2modelonindexArgs {
    pub const fn new(ghl_info: *mut c_void, model_index: c_int) -> Self {
        Self {
            ghl_info,
            model_index,
        }
    }
}

/// `CG_G2_HASGHOUL2MODELONINDEX` MP cgame imports syscall ABI token.
///
/// Raven transport: `ghlInfo` is passed as a raw `args[1]` pointer word and
/// `modelIndex` as raw `args[2]`; the engine switch returns `qboolean` as an
/// int-compatible word.
///
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:276`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:905-907`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:1433-1435`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1433-1435`
pub struct CgG2Hasghoul2modelonindex;

impl OutboundSysCall for CgG2Hasghoul2modelonindex {
    type Import = MpCgameImport;
    type Args = CgG2Hasghoul2modelonindexArgs;
    type Output = qboolean;

    const IMPORT: MpCgameImport = MpCgameImport::CG_G2_HASGHOUL2MODELONINDEX;
}

impl EncodeSysCall for CgG2Hasghoul2modelonindex {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.ghl_info), args.model_index as isize])
    }
}

impl DecodeSysCallReturn for CgG2Hasghoul2modelonindex {
    fn decode_return(word: isize) -> Self::Output {
        word as qboolean
    }
}
