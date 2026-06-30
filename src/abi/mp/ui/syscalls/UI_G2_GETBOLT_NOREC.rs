use core::ffi::{c_int, c_void};

use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::codemp::game::q_shared_h::{mdxaBone_t, qhandle_t};
use crate::ffi::GameImport;
use crate::shared::qboolean;
use crate::shared::vec3_t;

/// `UiG2_GETBOLT_NOREC` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct GG2GetboltNorecArgs {
    pub ghoul2: *mut c_void,
    pub model_index: c_int,
    pub bolt_index: c_int,
    pub matrix: *mut mdxaBone_t,
    pub angles: *const vec3_t,
    pub position: *const vec3_t,
    pub frame_num: c_int,
    pub model_list: *mut qhandle_t,
    pub scale: *mut vec3_t,
}

impl GG2GetboltNorecArgs {
    pub fn new(
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

    pub fn ghoul2(&self) -> *mut c_void {
        self.ghoul2
    }
    pub fn model_index(&self) -> c_int {
        self.model_index
    }
    pub fn bolt_index(&self) -> c_int {
        self.bolt_index
    }
    pub fn matrix(&self) -> *mut mdxaBone_t {
        self.matrix
    }
    pub fn angles(&self) -> *const vec3_t {
        self.angles
    }
    pub fn position(&self) -> *const vec3_t {
        self.position
    }
    pub fn frame_num(&self) -> c_int {
        self.frame_num
    }
    pub fn model_list(&self) -> *mut qhandle_t {
        self.model_list
    }
    pub fn scale(&self) -> *mut vec3_t {
        self.scale
    }
}

/// `UiG2_GETBOLT_NOREC` MP game imports syscall ABI token.
///
/// Source: `oracle/oracle/codemp/ui/ui_public.h:512`
pub struct GG2GetboltNorec;

impl OutboundSysCall for GG2GetboltNorec {
    type Import = GameImport;
    type Args = GG2GetboltNorecArgs;
    type Output = qboolean;

    const IMPORT: GameImport = GameImport::UiG2_GETBOLT_NOREC;
}

impl EncodeSysCall for GG2GetboltNorec {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(a.ghoul2),
            a.model_index as isize,
            a.bolt_index as isize,
            ptr_to_word(a.matrix),
            ptr_to_word(a.angles),
            ptr_to_word(a.position),
            a.frame_num as isize,
            ptr_to_word(a.model_list),
            ptr_to_word(a.scale),
        ])
    }
}

impl DecodeSysCallReturn for GG2GetboltNorec {
    fn decode_return(word: isize) -> Self::Output {
        word as qboolean
    }
}
