use core::ffi::c_int;
use core::ffi::c_void;

use crate::codemp::game::q_shared_h::{mdxaBone_t, qhandle_t};
use crate::ffi::GameImport;
use crate::shared::qboolean;
use crate::shared::vec3_t;

use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `G_G2_GETBOLT` outbound game-to-engine syscall.
///
/// C ABI: `qboolean trap_G2API_GetBoltMatrix(void *ghoul2, const int modelIndex, const int boltIndex,
///     mdxaBone_t *matrix, const vec3_t angles, const vec3_t position, const int frameNum,
///     qhandle_t *modelList, vec3_t scale)`
#[derive(Debug)]
pub struct GG2GetboltArgs {
    ghoul2: *mut c_void,
    model_index: c_int,
    bolt_index: c_int,
    matrix: *mut mdxaBone_t,
    angles: *const vec3_t,
    position: *const vec3_t,
    frame_num: c_int,
    model_list: *mut qhandle_t,
    scale: *const vec3_t,
}

impl GG2GetboltArgs {
    pub fn new(
        ghoul2: *mut c_void,
        model_index: c_int,
        bolt_index: c_int,
        matrix: *mut mdxaBone_t,
        angles: *const vec3_t,
        position: *const vec3_t,
        frame_num: c_int,
        model_list: *mut qhandle_t,
        scale: *const vec3_t,
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
    pub fn scale(&self) -> *const vec3_t {
        self.scale
    }
}

/// `G_G2_GETBOLT` MP game imports syscall ABI token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:511`
pub struct GG2Getbolt;

impl OutboundSysCall for GG2Getbolt {
    type Import = GameImport;
    type Args = GG2GetboltArgs;
    type Output = qboolean;

    const IMPORT: GameImport = GameImport::G_G2_GETBOLT;
}

impl EncodeSysCall for GG2Getbolt {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(a.ghoul2 as *const u8),
            a.model_index as isize,
            a.bolt_index as isize,
            ptr_to_word(a.matrix as *const u8),
            ptr_to_word(a.angles as *const u8),
            ptr_to_word(a.position as *const u8),
            a.frame_num as isize,
            ptr_to_word(a.model_list as *const u8),
            ptr_to_word(a.scale as *const u8),
        ])
    }
}

impl DecodeSysCallReturn for GG2Getbolt {
    fn decode_return(word: isize) -> Self::Output {
        word as qboolean
    }
}
