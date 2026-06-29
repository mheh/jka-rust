use core::ffi::{c_int, c_void};

use crate::boundary::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::codemp::game::q_shared_h::{mdxaBone_t, qhandle_t, vec3_t};
use crate::ffi::types::qboolean;
use crate::ffi::GameImport;

/// `G_G2_GETBOLT_NOREC_NOROT` outbound game-to-engine syscall.
///
/// ABI mirror of `trap_G2API_GetBoltMatrix_NoRecNoRot` in `g_syscalls.c`.
/// No skeleton reconstruction; no rotation applied before bolt sampling.
#[derive(Debug)]
pub struct GG2GetboltNorecNorotArgs {
    /// Opaque Ghoul2 instance handle.
    pub ghoul2: *mut c_void,
    /// Model index within the Ghoul2 instance.
    pub model_index: c_int,
    /// Bolt index on the model.
    pub bolt_index: c_int,
    /// Output matrix filled by the engine.
    pub matrix: *mut mdxaBone_t,
    /// World-space angles used for the query.
    pub angles: *const vec3_t,
    /// World-space position used for the query.
    pub position: *const vec3_t,
    /// Frame number for animation sampling.
    pub frame_num: c_int,
    /// Model list handle array.
    pub model_list: *mut qhandle_t,
    /// Scale vector applied to the model.
    pub scale: *const vec3_t,
}

impl GG2GetboltNorecNorotArgs {
    #[allow(clippy::too_many_arguments)]
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

/// `G_G2_GETBOLT_NOREC_NOROT` MP game imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:513`
pub struct GG2GetboltNorecNorot;

impl OutboundSysCall for GG2GetboltNorecNorot {
    type Import = GameImport;
    type Args = GG2GetboltNorecNorotArgs;
    type Output = qboolean;

    const IMPORT: GameImport = GameImport::G_G2_GETBOLT_NOREC_NOROT;
}

impl EncodeSysCall for GG2GetboltNorecNorot {
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

impl DecodeSysCallReturn for GG2GetboltNorecNorot {
    fn decode_return(word: isize) -> Self::Output {
        word as qboolean
    }
}
