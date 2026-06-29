use crate::boundary::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::ffi::types::qboolean;
use crate::ffi::GameImport;
use core::ffi::c_int;
use std::ffi::CString;

/// `G_G2_DOESBONEEXIST` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct GG2DoesboneexistArgs {
    pub ghoul2: *mut core::ffi::c_void,
    pub model_index: c_int,
    pub bone_name: CString,
}

impl GG2DoesboneexistArgs {
    pub fn new(ghoul2: *mut core::ffi::c_void, model_index: c_int, bone_name: CString) -> Self {
        Self {
            ghoul2,
            model_index,
            bone_name,
        }
    }

    pub fn ghoul2(&self) -> *mut core::ffi::c_void {
        self.ghoul2
    }
    pub fn model_index(&self) -> c_int {
        self.model_index
    }
    pub fn bone_name(&self) -> &CString {
        &self.bone_name
    }
}

/// `G_G2_DOESBONEEXIST` MP game imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:536`
pub struct GG2Doesboneexist;

impl OutboundSysCall for GG2Doesboneexist {
    type Import = GameImport;
    type Args = GG2DoesboneexistArgs;
    type Output = qboolean;

    const IMPORT: GameImport = GameImport::G_G2_DOESBONEEXIST;
}

impl EncodeSysCall for GG2Doesboneexist {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(a.ghoul2),
            a.model_index as isize,
            ptr_to_word(a.bone_name.as_ptr()),
        ])
    }
}

impl DecodeSysCallReturn for GG2Doesboneexist {
    fn decode_return(word: isize) -> Self::Output {
        word as qboolean
    }
}
