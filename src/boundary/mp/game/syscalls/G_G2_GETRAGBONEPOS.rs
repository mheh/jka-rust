use core::ffi::c_void;
use std::ffi::CString;

use crate::codemp::game::q_shared_h::vec3_t;
use crate::ffi::types::qboolean;
use crate::ffi::GameImport;

use crate::boundary::generic::{ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `G_G2_GETRAGBONEPOS` outbound game-to-engine syscall.
///
/// C ABI: `qboolean trap_G2API_GetRagBonePos(void *ghoul2, const char *boneName, vec3_t pos, vec3_t entAngles, vec3_t entPos, vec3_t entScale)`
#[derive(Debug)]
pub struct GG2GetragboneposArgs {
    /// Opaque Ghoul2 model handle.
    ghoul2: *mut c_void,
    /// Bone name as a null-terminated C string.
    bone_name: CString,
    /// Out: world-space position of the bone.
    pos: *mut vec3_t,
    /// Entity angles (used for bone-space → world-space transform).
    ent_angles: *mut vec3_t,
    /// Entity world position.
    ent_pos: *mut vec3_t,
    /// Entity scale.
    ent_scale: *mut vec3_t,
}

impl GG2GetragboneposArgs {
    pub fn new(
        ghoul2: *mut c_void,
        bone_name: CString,
        pos: *mut vec3_t,
        ent_angles: *mut vec3_t,
        ent_pos: *mut vec3_t,
        ent_scale: *mut vec3_t,
    ) -> Self {
        Self { ghoul2, bone_name, pos, ent_angles, ent_pos, ent_scale }
    }

    pub fn ghoul2(&self) -> *mut c_void { self.ghoul2 }
    pub fn bone_name(&self) -> &CString { &self.bone_name }
    pub fn pos(&self) -> *mut vec3_t { self.pos }
    pub fn ent_angles(&self) -> *mut vec3_t { self.ent_angles }
    pub fn ent_pos(&self) -> *mut vec3_t { self.ent_pos }
    pub fn ent_scale(&self) -> *mut vec3_t { self.ent_scale }
}

/// `G_G2_GETRAGBONEPOS` outbound game-to-engine syscall.
pub struct GG2Getragbonepos;

impl OutboundSysCall for GG2Getragbonepos {
    type Import = GameImport;
    type Args = GG2GetragboneposArgs;
    type Output = qboolean;

    const IMPORT: GameImport = GameImport::G_G2_GETRAGBONEPOS;
}

impl EncodeSysCall for GG2Getragbonepos {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(a.ghoul2 as *const _),
            ptr_to_word(a.bone_name.as_ptr()),
            ptr_to_word(a.pos as *const _),
            ptr_to_word(a.ent_angles as *const _),
            ptr_to_word(a.ent_pos as *const _),
            ptr_to_word(a.ent_scale as *const _),
        ])
    }
}

impl DecodeSysCallReturn for GG2Getragbonepos {
    fn decode_return(word: isize) -> Self::Output {
        word as qboolean
    }
}
