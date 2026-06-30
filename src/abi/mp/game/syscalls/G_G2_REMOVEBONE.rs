use core::ffi::c_int;
use std::ffi::CString;

use super::super::MpGameImport;
use crate::shared::qboolean;

use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `G_G2_REMOVEBONE` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct GG2RemoveboneArgs {
    pub ghoul2: *mut core::ffi::c_void,
    pub bone_name: CString,
    pub model_index: c_int,
}

impl GG2RemoveboneArgs {
    pub fn new(ghoul2: *mut core::ffi::c_void, bone_name: CString, model_index: c_int) -> Self {
        Self {
            ghoul2,
            bone_name,
            model_index,
        }
    }

    pub fn ghoul2(&self) -> *mut core::ffi::c_void {
        self.ghoul2
    }
    pub fn bone_name(&self) -> &CString {
        &self.bone_name
    }
    pub fn model_index(&self) -> c_int {
        self.model_index
    }
}

/// `G_G2_REMOVEBONE` MP game imports syscall ABI token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:562`
pub struct GG2Removebone;

impl OutboundSysCall for GG2Removebone {
    type Import = MpGameImport;
    type Args = GG2RemoveboneArgs;
    type Output = qboolean;

    const IMPORT: MpGameImport = MpGameImport::G_G2_REMOVEBONE;
}

impl EncodeSysCall for GG2Removebone {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(a.ghoul2),
            ptr_to_word(a.bone_name.as_ptr()),
            a.model_index as isize,
        ])
    }
}

impl DecodeSysCallReturn for GG2Removebone {
    fn decode_return(word: isize) -> Self::Output {
        word as qboolean
    }
}
