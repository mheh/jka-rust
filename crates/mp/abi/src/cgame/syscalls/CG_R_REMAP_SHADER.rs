use std::ffi::CString;

use super::super::MpCgameImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `CG_R_REMAP_SHADER`.
#[derive(Debug)]
pub struct CgRRemapShaderArgs {
    old_shader: CString,
    new_shader: CString,
    time_offset: CString,
}

impl CgRRemapShaderArgs {
    pub fn new(old_shader: CString, new_shader: CString, time_offset: CString) -> Self {
        Self {
            old_shader,
            new_shader,
            time_offset,
        }
    }

    pub fn old_shader(&self) -> &CString {
        &self.old_shader
    }

    pub fn new_shader(&self) -> &CString {
        &self.new_shader
    }

    pub fn time_offset(&self) -> &CString {
        &self.time_offset
    }
}

/// `CG_R_REMAP_SHADER` MP cgame imports syscall ABI token.
///
/// Source: `oracle/codemp/cgame/cg_public.h:167`
pub struct CgRRemapShader;

impl OutboundSysCall for CgRRemapShader {
    type Import = MpCgameImport;
    type Args = CgRRemapShaderArgs;
    type Output = ();

    const IMPORT: MpCgameImport = MpCgameImport::CG_R_REMAP_SHADER;
}

impl EncodeSysCall for CgRRemapShader {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(args.old_shader().as_ptr()),
            ptr_to_word(args.new_shader().as_ptr()),
            ptr_to_word(args.time_offset().as_ptr()),
        ])
    }
}

impl DecodeSysCallReturn for CgRRemapShader {
    fn decode_return(_word: isize) -> Self::Output {}
}
