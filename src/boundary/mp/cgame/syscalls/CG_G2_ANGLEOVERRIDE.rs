use core::ffi::{c_int, c_void};
use std::ffi::CString;

use super::super::MpCgameImport;
use crate::boundary::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::codemp::game::q_shared_h::{qhandle_t, vec3_t};
use crate::ffi::types::qboolean;

/// Arguments for `CG_G2_ANGLEOVERRIDE`.
///
/// Raven wrapper: `qboolean trap_G2API_SetBoneAngles(...)`.
/// Raven transport: `return G2API_SetBoneAngles(*((CGhoul2Info_v *)args[1]), args[2], (const char *)VMA(3), (float *)VMA(4), args[5],
///     (const Eorientations)args[6], (const Eorientations)args[7], (const Eorientations)args[8],
///     (qhandle_t *)VMA(9), args[10], args[11]);`.
///
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:268`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:861-865`
/// Args source: `oracle/oracle/codemp/cgame/cg_local.h:2549-2551`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:1369-1372`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1369-1372`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CgG2AngleoverrideArgs {
    /// Ghoul2 handle, decoded by Raven as `*((CGhoul2Info_v *)args[1])`.
    ghoul2: *mut c_void,
    /// Model index.
    model_index: c_int,
    /// Bone name transported as a NUL-terminated string pointer.
    bone_name: CString,
    /// Bone angles, decoded by Raven from `VMA(4)`.
    angles: *const vec3_t,
    /// Flags.
    flags: c_int,
    /// Orientation enum word, cast by Raven to `Eorientations`.
    up: c_int,
    /// Orientation enum word, cast by Raven to `Eorientations`.
    right: c_int,
    /// Orientation enum word, cast by Raven to `Eorientations`.
    forward: c_int,
    /// Model list pointer, decoded by Raven from `VMA(9)`.
    model_list: *mut qhandle_t,
    /// Blend time.
    blend_time: c_int,
    /// Current time.
    current_time: c_int,
}

impl CgG2AngleoverrideArgs {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        ghoul2: *mut c_void,
        model_index: c_int,
        bone_name: CString,
        angles: *const vec3_t,
        flags: c_int,
        up: c_int,
        right: c_int,
        forward: c_int,
        model_list: *mut qhandle_t,
        blend_time: c_int,
        current_time: c_int,
    ) -> Self {
        Self {
            ghoul2,
            model_index,
            bone_name,
            angles,
            flags,
            up,
            right,
            forward,
            model_list,
            blend_time,
            current_time,
        }
    }
}

/// `CG_G2_ANGLEOVERRIDE` MP cgame imports syscall boundary token.
///
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:268`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:861-865`
/// Args source: `oracle/oracle/codemp/cgame/cg_local.h:2549-2551`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:1369-1372`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1369-1372`
pub struct CgG2Angleoverride;

impl OutboundSysCall for CgG2Angleoverride {
    type Import = MpCgameImport;
    type Args = CgG2AngleoverrideArgs;
    type Output = qboolean;

    const IMPORT: MpCgameImport = MpCgameImport::CG_G2_ANGLEOVERRIDE;
}

impl EncodeSysCall for CgG2Angleoverride {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(args.ghoul2),
            args.model_index as isize,
            ptr_to_word(args.bone_name.as_ptr()),
            ptr_to_word(args.angles),
            args.flags as isize,
            args.up as isize,
            args.right as isize,
            args.forward as isize,
            ptr_to_word(args.model_list),
            args.blend_time as isize,
            args.current_time as isize,
        ])
    }
}

impl DecodeSysCallReturn for CgG2Angleoverride {
    fn decode_return(word: isize) -> Self::Output {
        word as qboolean
    }
}
