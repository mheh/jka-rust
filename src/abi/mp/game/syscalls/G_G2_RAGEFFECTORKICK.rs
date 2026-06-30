use core::ffi::{c_char, c_void};

use super::super::MpGameImport;
use crate::shared::qboolean;
use crate::shared::vec3_t;

use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `G_G2_RAGEFFECTORKICK` outbound game-to-engine syscall.
/// C: `qboolean trap_G2API_RagEffectorKick(void *ghoul2, const char *boneName, vec3_t velocity)`
#[derive(Debug)]
pub struct GG2RageffectorkickArgs {
    /// Ghoul2 model instance handle (opaque void*).
    ghoul2: *mut c_void,
    /// Name of the effector bone.
    bone_name: *const c_char,
    /// Velocity vector to kick the bone with.
    velocity: *mut vec3_t,
}

impl GG2RageffectorkickArgs {
    pub fn new(ghoul2: *mut c_void, bone_name: *const c_char, velocity: *mut vec3_t) -> Self {
        Self {
            ghoul2,
            bone_name,
            velocity,
        }
    }

    pub fn ghoul2(&self) -> *mut c_void {
        self.ghoul2
    }

    pub fn bone_name(&self) -> *const c_char {
        self.bone_name
    }

    pub fn velocity(&self) -> *mut vec3_t {
        self.velocity
    }
}

/// `G_G2_RAGEFFECTORKICK` MP game imports syscall ABI token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:554`
pub struct GG2Rageffectorkick;

impl OutboundSysCall for GG2Rageffectorkick {
    type Import = MpGameImport;
    type Args = GG2RageffectorkickArgs;
    type Output = qboolean;

    const IMPORT: MpGameImport = MpGameImport::G_G2_RAGEFFECTORKICK;
}

impl EncodeSysCall for GG2Rageffectorkick {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(a.ghoul2),
            ptr_to_word(a.bone_name),
            ptr_to_word(a.velocity as *mut f32),
        ])
    }
}

impl DecodeSysCallReturn for GG2Rageffectorkick {
    fn decode_return(word: isize) -> Self::Output {
        word as qboolean
    }
}
