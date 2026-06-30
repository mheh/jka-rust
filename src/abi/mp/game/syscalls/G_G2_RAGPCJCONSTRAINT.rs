use core::ffi::{c_char, c_void};

use crate::ffi::GameImport;
use crate::shared::qboolean;
use crate::shared::vec3_t;

use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `G_G2_RAGPCJCONSTRAINT` outbound game-to-engine syscall.
///
/// C signature: `qboolean trap_G2API_RagPCJConstraint(void *ghoul2, const char *boneName, vec3_t min, vec3_t max)`
#[derive(Debug)]
pub struct GG2RagpcjconstraintArgs {
    ghoul2: *mut c_void,
    bone_name: *const c_char,
    min: *mut vec3_t,
    max: *mut vec3_t,
}

impl GG2RagpcjconstraintArgs {
    pub fn new(
        ghoul2: *mut c_void,
        bone_name: *const c_char,
        min: *mut vec3_t,
        max: *mut vec3_t,
    ) -> Self {
        Self {
            ghoul2,
            bone_name,
            min,
            max,
        }
    }

    pub fn ghoul2(&self) -> *mut c_void {
        self.ghoul2
    }

    pub fn bone_name(&self) -> *const c_char {
        self.bone_name
    }

    pub fn min(&self) -> *mut vec3_t {
        self.min
    }

    pub fn max(&self) -> *mut vec3_t {
        self.max
    }
}

/// `G_G2_RAGPCJCONSTRAINT` MP game imports syscall ABI token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:550`
pub struct GG2Ragpcjconstraint;

impl OutboundSysCall for GG2Ragpcjconstraint {
    type Import = GameImport;
    type Args = GG2RagpcjconstraintArgs;
    type Output = qboolean;

    const IMPORT: GameImport = GameImport::G_G2_RAGPCJCONSTRAINT;
}

impl EncodeSysCall for GG2Ragpcjconstraint {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(a.ghoul2 as *const _),
            ptr_to_word(a.bone_name as *const _),
            ptr_to_word(a.min as *const _),
            ptr_to_word(a.max as *const _),
        ])
    }
}

impl DecodeSysCallReturn for GG2Ragpcjconstraint {
    fn decode_return(word: isize) -> Self::Output {
        word as qboolean
    }
}
