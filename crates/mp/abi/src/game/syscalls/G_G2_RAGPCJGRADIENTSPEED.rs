use core::ffi::{c_void, CStr};
use std::ffi::CString;

use super::super::MpGameImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use mp_qshared::shared::qboolean;

/// `G_G2_RAGPCJGRADIENTSPEED` outbound game-to-engine syscall.
/// C: `qboolean trap_G2API_RagPCJGradientSpeed(void *ghoul2, const char *boneName, const float speed)`
#[derive(Debug)]
pub struct GG2RagpcjgradientspeedArgs {
    ghoul2: *mut c_void,
    bone_name: CString,
    speed: f32,
}

impl GG2RagpcjgradientspeedArgs {
    pub fn new(ghoul2: *mut c_void, bone_name: CString, speed: f32) -> Self {
        Self {
            ghoul2,
            bone_name,
            speed,
        }
    }

    pub fn ghoul2(&self) -> *mut c_void {
        self.ghoul2
    }

    pub fn bone_name(&self) -> &CStr {
        self.bone_name.as_c_str()
    }

    pub fn speed(&self) -> f32 {
        self.speed
    }
}

/// `G_G2_RAGPCJGRADIENTSPEED` MP game imports syscall ABI token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:551`
pub struct GG2Ragpcjgradientspeed;

impl OutboundSysCall for GG2Ragpcjgradientspeed {
    type Import = MpGameImport;
    type Args = GG2RagpcjgradientspeedArgs;
    type Output = qboolean;

    const IMPORT: MpGameImport = MpGameImport::G_G2_RAGPCJGRADIENTSPEED;
}

impl EncodeSysCall for GG2Ragpcjgradientspeed {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(a.ghoul2 as *const _),
            ptr_to_word(a.bone_name.as_ptr()),
            abi_transport::pass_float(a.speed),
        ])
    }
}

impl DecodeSysCallReturn for GG2Ragpcjgradientspeed {
    fn decode_return(word: isize) -> Self::Output {
        word as qboolean
    }
}
