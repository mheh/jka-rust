use core::ffi::{c_int, c_void};

use super::super::MpCgameImport;
use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::codemp::game::q_shared_h::{mdxaBone_t, qhandle_t};
use crate::shared::qboolean;
use crate::shared::vec3_t;

/// Arguments for `CG_G2_GETBOLT_NOREC_NOROT`.
///
/// Raven wrapper: `//Same as above but force it to not reconstruct the skeleton before getting the bolt position`
/// Raven transport: `//gG2_GBMNoReconstruct = qtrue;`
/// Raven transport: `//Yeah, this was probably BAD.`
/// Raven transport: `gG2_GBMUseSPMethod = qtrue; return G2API_GetBoltMatrix(*((CGhoul2Info_v *)args[1]), args[2], args[3], (mdxaBone_t *)VMA(4), (const float *)VMA(5), (const float *)VMA(6), args[7], (qhandle_t *)VMA(8), (float *)VMA(9));`
///
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:803-806`
/// Args source: `oracle/oracle/codemp/cgame/cg_local.h:2523-2524`
/// Output source: `oracle/oracle/codemp/cgame/cg_syscalls.c:803,806`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:1318-1322`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1318-1322`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgG2GetboltNorecNorotArgs {
    ghoul2: *mut c_void,
    model_index: c_int,
    bolt_index: c_int,
    matrix: *mut mdxaBone_t,
    angles: *const vec3_t,
    position: *const vec3_t,
    frame_num: c_int,
    model_list: *mut qhandle_t,
    scale: *mut vec3_t,
}

impl CgG2GetboltNorecNorotArgs {
    pub const fn new(
        ghoul2: *mut c_void,
        model_index: c_int,
        bolt_index: c_int,
        matrix: *mut mdxaBone_t,
        angles: *const vec3_t,
        position: *const vec3_t,
        frame_num: c_int,
        model_list: *mut qhandle_t,
        scale: *mut vec3_t,
    ) -> Self {
        Self {
            ghoul2,
            model_index,
            bolt_index,
            matrix,
            angles,
            position,
            frame_num,
            model_list,
            scale,
        }
    }
}

/// `CG_G2_GETBOLT_NOREC_NOROT` MP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:262`
pub struct CgG2GetboltNorecNorot;

impl OutboundSysCall for CgG2GetboltNorecNorot {
    type Import = MpCgameImport;
    type Args = CgG2GetboltNorecNorotArgs;
    type Output = qboolean;

    const IMPORT: MpCgameImport = MpCgameImport::CG_G2_GETBOLT_NOREC_NOROT;
}

impl EncodeSysCall for CgG2GetboltNorecNorot {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(args.ghoul2 as *const _),
            args.model_index as isize,
            args.bolt_index as isize,
            ptr_to_word(args.matrix as *const _),
            ptr_to_word(args.angles),
            ptr_to_word(args.position),
            args.frame_num as isize,
            ptr_to_word(args.model_list as *const _),
            ptr_to_word(args.scale as *const _),
        ])
    }
}

impl DecodeSysCallReturn for CgG2GetboltNorecNorot {
    fn decode_return(word: isize) -> Self::Output {
        word as qboolean
    }
}
