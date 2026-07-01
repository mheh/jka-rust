use core::ffi::{c_int, c_void};

use super::super::MpCgameImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use mp_qshared::shared::mdxaBone_t;
use mp_qshared::shared::qboolean;
use mp_qshared::shared::qhandle_t;
use mp_qshared::shared::vec3_t;

/// Arguments for `CG_G2_GETBOLT`.
///
/// Raven wrapper: `return syscall(CG_G2_GETBOLT, ghoul2, modelIndex, boltIndex, matrix, angles, position, frameNum, modelList, scale);`
/// Raven transport: `return G2API_GetBoltMatrix(*((CGhoul2Info_v *)args[1]), args[2], args[3], (mdxaBone_t *)VMA(4), (const float *)VMA(5), (const float *)VMA(6), args[7], (qhandle_t *)VMA(8), (float *)VMA(9));`
///
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:791-794`
/// Args source: `oracle/oracle/codemp/cgame/cg_local.h:2519-2520`
/// Output source: `oracle/oracle/codemp/cgame/cg_syscalls.c:791,794`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:1311-1312`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1311-1312`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgG2GetboltArgs {
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

impl CgG2GetboltArgs {
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

/// `CG_G2_GETBOLT` MP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:260`
pub struct CgG2Getbolt;

impl OutboundSysCall for CgG2Getbolt {
    type Import = MpCgameImport;
    type Args = CgG2GetboltArgs;
    type Output = qboolean;

    const IMPORT: MpCgameImport = MpCgameImport::CG_G2_GETBOLT;
}

impl EncodeSysCall for CgG2Getbolt {
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

impl DecodeSysCallReturn for CgG2Getbolt {
    fn decode_return(word: isize) -> Self::Output {
        word as qboolean
    }
}
