use core::ffi::{c_int, c_void};
use std::ffi::CString;

use super::super::MpGameImport;
use crate::shared::qboolean;

use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `G_G2_GETBONEANIM` outbound game-to-engine syscall.
///
/// Mirrors `trap_G2API_GetBoneAnim` in `oracle/codemp/game/g_syscalls.c`.
#[derive(Debug)]
pub struct GG2GetboneanimArgs {
    /// Ghoul2 instance handle (opaque void*).
    pub ghoul2: *mut c_void,
    /// Name of the bone (null-terminated).
    pub bone_name: CString,
    /// Current time in milliseconds.
    pub current_time: c_int,
    /// Out-param: current frame written by engine.
    pub current_frame: *mut f32,
    /// Out-param: start frame written by engine.
    pub start_frame: *mut c_int,
    /// Out-param: end frame written by engine.
    pub end_frame: *mut c_int,
    /// Out-param: flags written by engine.
    pub flags: *mut c_int,
    /// Out-param: anim speed written by engine.
    pub anim_speed: *mut f32,
    /// Out-param: model list written by engine.
    pub model_list: *mut c_int,
    /// Index of the model within the Ghoul2 instance.
    pub model_index: c_int,
}

impl GG2GetboneanimArgs {
    pub fn new(
        ghoul2: *mut c_void,
        bone_name: CString,
        current_time: c_int,
        current_frame: *mut f32,
        start_frame: *mut c_int,
        end_frame: *mut c_int,
        flags: *mut c_int,
        anim_speed: *mut f32,
        model_list: *mut c_int,
        model_index: c_int,
    ) -> Self {
        Self {
            ghoul2,
            bone_name,
            current_time,
            current_frame,
            start_frame,
            end_frame,
            flags,
            anim_speed,
            model_list,
            model_index,
        }
    }

    pub fn ghoul2(&self) -> *mut c_void {
        self.ghoul2
    }
    pub fn bone_name(&self) -> &CString {
        &self.bone_name
    }
    pub fn current_time(&self) -> c_int {
        self.current_time
    }
    pub fn current_frame(&self) -> *mut f32 {
        self.current_frame
    }
    pub fn start_frame(&self) -> *mut c_int {
        self.start_frame
    }
    pub fn end_frame(&self) -> *mut c_int {
        self.end_frame
    }
    pub fn flags(&self) -> *mut c_int {
        self.flags
    }
    pub fn anim_speed(&self) -> *mut f32 {
        self.anim_speed
    }
    pub fn model_list(&self) -> *mut c_int {
        self.model_list
    }
    pub fn model_index(&self) -> c_int {
        self.model_index
    }
}

/// `G_G2_GETBONEANIM` MP game imports syscall ABI token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:521`
pub struct GG2Getboneanim;

impl OutboundSysCall for GG2Getboneanim {
    type Import = MpGameImport;
    type Args = GG2GetboneanimArgs;
    type Output = qboolean;

    const IMPORT: MpGameImport = MpGameImport::G_G2_GETBONEANIM;
}

impl EncodeSysCall for GG2Getboneanim {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(a.ghoul2 as *const _),
            ptr_to_word(a.bone_name.as_ptr() as *const _),
            a.current_time as isize,
            ptr_to_word(a.current_frame as *const _),
            ptr_to_word(a.start_frame as *const _),
            ptr_to_word(a.end_frame as *const _),
            ptr_to_word(a.flags as *const _),
            ptr_to_word(a.anim_speed as *const _),
            ptr_to_word(a.model_list as *const _),
            a.model_index as isize,
        ])
    }
}

impl DecodeSysCallReturn for GG2Getboneanim {
    fn decode_return(word: isize) -> Self::Output {
        word as qboolean
    }
}
