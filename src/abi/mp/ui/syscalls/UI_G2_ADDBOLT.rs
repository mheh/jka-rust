use core::ffi::{c_int, c_void};
use std::ffi::CString;

use super::super::MpUiImport;
use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `UI_G2_ADDBOLT` outbound game-to-engine syscall.
///
/// Registers a bolt (attach point) on `model_index` of the ghoul2 instance, identified by
/// `bone_name` (a bone or tag surface name). Returns the bolt index, or `-1` on failure.
#[derive(Debug)]
pub struct UiG2AddboltArgs {
    ghoul2: *mut c_void,
    model_index: c_int,
    bone_name: CString,
}

impl UiG2AddboltArgs {
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

/// `UI_G2_ADDBOLT` MP UI imports syscall ABI token.
///
/// Source: `oracle/oracle/codemp/ui/ui_public.h:517`
pub struct UiG2Addbolt;

impl OutboundSysCall for UiG2Addbolt {
    type Import = MpUiImport;
    type Args = UiG2AddboltArgs;
    type Output = c_int;

    const IMPORT: MpUiImport = MpUiImport::UI_G2_ADDBOLT;
}

impl EncodeSysCall for UiG2Addbolt {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(a.ghoul2),
            a.model_index as isize,
            ptr_to_word(a.bone_name.as_ptr()),
        ])
    }
}

impl DecodeSysCallReturn for UiG2Addbolt {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
