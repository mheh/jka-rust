use core::ffi::{c_char, c_int, c_void};

use crate::boundary::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::ffi::syscalls::pass_float;
use crate::ffi::types::qboolean;
use crate::ffi::GameImport;

/// `G_G2_PLAYANIM` outbound game-to-engine syscall.
///
/// Mirrors `trap_G2API_SetBoneAnim`:
/// ```c
/// qboolean trap_G2API_SetBoneAnim(void *ghoul2, int modelIndex, const char *boneName,
///     int startFrame, int endFrame, int flags, float animSpeed,
///     int currentTime, float setFrame, int blendTime);
/// ```
#[derive(Debug)]
pub struct GG2PlayanimArgs {
    ghoul2: *mut c_void,
    model_index: c_int,
    bone_name: *const c_char,
    start_frame: c_int,
    end_frame: c_int,
    flags: c_int,
    anim_speed: f32,
    current_time: c_int,
    set_frame: f32,
    blend_time: c_int,
}

impl GG2PlayanimArgs {
    pub fn new(
        ghoul2: *mut c_void,
        model_index: c_int,
        bone_name: *const c_char,
        start_frame: c_int,
        end_frame: c_int,
        flags: c_int,
        anim_speed: f32,
        current_time: c_int,
        set_frame: f32,
        blend_time: c_int,
    ) -> Self {
        Self {
            ghoul2,
            model_index,
            bone_name,
            start_frame,
            end_frame,
            flags,
            anim_speed,
            current_time,
            set_frame,
            blend_time,
        }
    }

    pub fn ghoul2(&self) -> *mut c_void {
        self.ghoul2
    }
    pub fn model_index(&self) -> c_int {
        self.model_index
    }
    pub fn bone_name(&self) -> *const c_char {
        self.bone_name
    }
    pub fn start_frame(&self) -> c_int {
        self.start_frame
    }
    pub fn end_frame(&self) -> c_int {
        self.end_frame
    }
    pub fn flags(&self) -> c_int {
        self.flags
    }
    pub fn anim_speed(&self) -> f32 {
        self.anim_speed
    }
    pub fn current_time(&self) -> c_int {
        self.current_time
    }
    pub fn set_frame(&self) -> f32 {
        self.set_frame
    }
    pub fn blend_time(&self) -> c_int {
        self.blend_time
    }
}

/// `G_G2_PLAYANIM` MP game imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:520`
pub struct GG2Playanim;

impl OutboundSysCall for GG2Playanim {
    type Import = GameImport;
    type Args = GG2PlayanimArgs;
    type Output = qboolean;

    const IMPORT: GameImport = GameImport::G_G2_PLAYANIM;
}

impl EncodeSysCall for GG2Playanim {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(a.ghoul2()),
            a.model_index() as isize,
            ptr_to_word(a.bone_name()),
            a.start_frame() as isize,
            a.end_frame() as isize,
            a.flags() as isize,
            pass_float(a.anim_speed()),
            a.current_time() as isize,
            pass_float(a.set_frame()),
            a.blend_time() as isize,
        ])
    }
}

impl DecodeSysCallReturn for GG2Playanim {
    fn decode_return(word: isize) -> Self::Output {
        word as qboolean
    }
}
