use core::ffi::{c_int, c_void};

use super::super::MpCgameImport;
use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::{codemp::game::q_shared_h::qhandle_t, ffi::types::qboolean};

/// Arguments for `CG_G2_SETSKIN`.
///
/// Raven wrapper: `return syscall(CG_G2_SETSKIN, ghoul2, modelIndex, customSkin, renderSkin);`
/// Raven transport: `return G2API_SetSkin(&g2[modelIndex], args[3], args[4]);`
///
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:815-817`
/// Args source: `oracle/oracle/codemp/cgame/cg_local.h:2527`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1331-1337`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgG2SetskinArgs {
    ghoul2: *mut c_void,
    model_index: c_int,
    custom_skin: qhandle_t,
    render_skin: qhandle_t,
}

impl CgG2SetskinArgs {
    pub const fn new(
        ghoul2: *mut c_void,
        model_index: c_int,
        custom_skin: qhandle_t,
        render_skin: qhandle_t,
    ) -> Self {
        Self {
            ghoul2,
            model_index,
            custom_skin,
            render_skin,
        }
    }
}

/// `CG_G2_SETSKIN` MP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:264`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:815-817`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:1331-1337`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1331-1337`
pub struct CgG2Setskin;

impl OutboundSysCall for CgG2Setskin {
    type Import = MpCgameImport;
    type Args = CgG2SetskinArgs;
    type Output = qboolean;

    const IMPORT: MpCgameImport = MpCgameImport::CG_G2_SETSKIN;
}

impl EncodeSysCall for CgG2Setskin {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(args.ghoul2),
            args.model_index as isize,
            args.custom_skin as isize,
            args.render_skin as isize,
        ])
    }
}

impl DecodeSysCallReturn for CgG2Setskin {
    fn decode_return(word: isize) -> Self::Output {
        word as qboolean
    }
}
