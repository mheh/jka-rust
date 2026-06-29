use core::ffi::{c_int, c_void};
use std::ffi::CString;

use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::ffi::GameImport;

/// `G_G2_ADDBOLT` outbound game-to-engine syscall.
///
/// Registers a bolt (attach point) on `model_index` of the ghoul2 instance, identified by
/// `bone_name` (a bone or tag surface name). Returns the bolt index, or `-1` on failure.
#[derive(Debug)]
pub struct GG2AddboltArgs {
    ghoul2: *mut c_void,
    model_index: c_int,
    bone_name: CString,
}

impl GG2AddboltArgs {
    pub fn new(ghoul2: *mut c_void, model_index: c_int, bone_name: CString) -> Self {
        Self {
            ghoul2,
            model_index,
            bone_name,
        }
    }

    pub fn ghoul2(&self) -> *mut c_void {
        self.ghoul2
    }
    pub fn model_index(&self) -> c_int {
        self.model_index
    }
    pub fn bone_name(&self) -> &CString {
        &self.bone_name
    }
}

/// `G_G2_ADDBOLT` MP game imports syscall ABI token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:517`
pub struct GG2Addbolt;

impl OutboundSysCall for GG2Addbolt {
    type Import = GameImport;
    type Args = GG2AddboltArgs;
    type Output = c_int;

    const IMPORT: GameImport = GameImport::G_G2_ADDBOLT;
}

impl EncodeSysCall for GG2Addbolt {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(a.ghoul2),
            a.model_index as isize,
            ptr_to_word(a.bone_name.as_ptr()),
        ])
    }
}

impl DecodeSysCallReturn for GG2Addbolt {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
