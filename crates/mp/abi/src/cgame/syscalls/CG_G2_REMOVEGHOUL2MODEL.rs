use core::ffi::{c_int, c_void};

use super::super::MpCgameImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use mp_qshared::shared::qboolean;

/// Arguments for `CG_G2_REMOVEGHOUL2MODEL`.
///
/// Raven wrapper: `qboolean trap_G2API_RemoveGhoul2Model(void *ghlInfo, int modelIndex)`.
/// Raven transport: `return G2API_RemoveGhoul2Model((CGhoul2Info_v **)VMA(1), args[2]);`
///
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:277`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:910-912`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:1437-1442`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1437-1442`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgG2Removeghoul2modelArgs {
    /// Raw Ghoul2 handle word, decoded by Raven as `(CGhoul2Info_v **)VMA(1)`.
    ghl_info: *mut c_void,
    /// Model index, read by Raven as raw `args[2]`.
    model_index: c_int,
}

impl CgG2Removeghoul2modelArgs {
    pub const fn new(ghl_info: *mut c_void, model_index: c_int) -> Self {
        Self {
            ghl_info,
            model_index,
        }
    }
}

/// `CG_G2_REMOVEGHOUL2MODEL` MP cgame imports syscall ABI token.
///
/// Raven transport: `ghlInfo` is passed as a raw `args[1]` pointer word and
/// `modelIndex` as raw `args[2]`; the switch path also carries the
/// `_FULL_G2_LEAK_CHECKING` side effect before returning the int-compatible
/// `qboolean`.
///
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:277`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:910-912`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:1437-1442`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1437-1442`
pub struct CgG2Removeghoul2model;

impl OutboundSysCall for CgG2Removeghoul2model {
    type Import = MpCgameImport;
    type Args = CgG2Removeghoul2modelArgs;
    type Output = qboolean;

    const IMPORT: MpCgameImport = MpCgameImport::CG_G2_REMOVEGHOUL2MODEL;
}

impl EncodeSysCall for CgG2Removeghoul2model {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.ghl_info), args.model_index as isize])
    }
}

impl DecodeSysCallReturn for CgG2Removeghoul2model {
    fn decode_return(word: isize) -> Self::Output {
        word as qboolean
    }
}
