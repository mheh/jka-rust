use core::ffi::{c_int, c_void};
use std::ffi::CString;

use crate::codemp::game::q_shared_h::{qhandle_t, vec3_t};
use crate::ffi::types::qboolean;
use crate::ffi::GameImport;

use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `G_G2_ANGLEOVERRIDE` outbound game-to-engine syscall.
///
/// Mirrors `trap_G2API_SetBoneAngles` from `g_syscalls.c`:
/// ```c
/// qboolean trap_G2API_SetBoneAngles(
///     void *ghoul2, int modelIndex, const char *boneName,
///     const vec3_t angles, const int flags,
///     const int up, const int right, const int forward,
///     qhandle_t *modelList, int blendTime, int currentTime)
/// ```
#[derive(Debug)]
pub struct GG2AngleoverrideArgs {
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
}

impl GG2AngleoverrideArgs {
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

    pub fn ghoul2(&self) -> *mut c_void {
        self.ghoul2
    }
    pub fn model_index(&self) -> c_int {
        self.model_index
    }
    pub fn bone_name(&self) -> &CString {
        &self.bone_name
    }
    pub fn angles(&self) -> *const vec3_t {
        self.angles
    }
    pub fn flags(&self) -> c_int {
        self.flags
    }
    pub fn up(&self) -> c_int {
        self.up
    }
    pub fn right(&self) -> c_int {
        self.right
    }
    pub fn forward(&self) -> c_int {
        self.forward
    }
    pub fn model_list(&self) -> *mut qhandle_t {
        self.model_list
    }
    pub fn blend_time(&self) -> c_int {
        self.blend_time
    }
    pub fn current_time(&self) -> c_int {
        self.current_time
    }
}

/// `G_G2_ANGLEOVERRIDE` MP game imports syscall ABI token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:519`
pub struct GG2Angleoverride;

impl OutboundSysCall for GG2Angleoverride {
    type Import = GameImport;
    type Args = GG2AngleoverrideArgs;
    type Output = qboolean;

    const IMPORT: GameImport = GameImport::G_G2_ANGLEOVERRIDE;
}

impl EncodeSysCall for GG2Angleoverride {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(a.ghoul2()),
            a.model_index() as isize,
            ptr_to_word(a.bone_name().as_ptr()),
            ptr_to_word(a.angles()),
            a.flags() as isize,
            a.up() as isize,
            a.right() as isize,
            a.forward() as isize,
            ptr_to_word(a.model_list()),
            a.blend_time() as isize,
            a.current_time() as isize,
        ])
    }
}

impl DecodeSysCallReturn for GG2Angleoverride {
    fn decode_return(word: isize) -> Self::Output {
        word as qboolean
    }
}
