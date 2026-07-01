use core::ffi::c_int;
use std::ffi::CString;

use super::super::MpGameImport;

use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `G_G2_INITGHOUL2MODEL` outbound game-to-engine syscall.
///
/// C ABI: `int trap_G2API_InitGhoul2Model(void **ghoul2Ptr, const char *fileName,
///   int modelIndex, qhandle_t customSkin, qhandle_t customShader,
///   int modelFlags, int lodBias)`
#[derive(Debug)]
pub struct GG2Initghoul2ModelArgs {
    /// Out-param: engine writes the ghoul2 handle through this pointer.
    pub ghoul2_ptr: *mut *mut core::ffi::c_void,
    /// File name of the model (owned so the pointer stays live during the call).
    pub file_name: CString,
    pub model_index: c_int,
    pub custom_skin: c_int,
    pub custom_shader: c_int,
    pub model_flags: c_int,
    pub lod_bias: c_int,
}

impl GG2Initghoul2ModelArgs {
    pub fn new(
        ghoul2_ptr: *mut *mut core::ffi::c_void,
        file_name: CString,
        model_index: c_int,
        custom_skin: c_int,
        custom_shader: c_int,
        model_flags: c_int,
        lod_bias: c_int,
    ) -> Self {
        Self {
            ghoul2_ptr,
            file_name,
            model_index,
            custom_skin,
            custom_shader,
            model_flags,
            lod_bias,
        }
    }

    pub fn ghoul2_ptr(&self) -> *mut *mut core::ffi::c_void {
        self.ghoul2_ptr
    }
    pub fn file_name(&self) -> &CString {
        &self.file_name
    }
    pub fn model_index(&self) -> c_int {
        self.model_index
    }
    pub fn custom_skin(&self) -> c_int {
        self.custom_skin
    }
    pub fn custom_shader(&self) -> c_int {
        self.custom_shader
    }
    pub fn model_flags(&self) -> c_int {
        self.model_flags
    }
    pub fn lod_bias(&self) -> c_int {
        self.lod_bias
    }
}

/// `G_G2_INITGHOUL2MODEL` MP game imports syscall ABI token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:514`
pub struct GG2Initghoul2Model;

impl OutboundSysCall for GG2Initghoul2Model {
    type Import = MpGameImport;
    type Args = GG2Initghoul2ModelArgs;
    type Output = c_int;

    const IMPORT: MpGameImport = MpGameImport::G_G2_INITGHOUL2MODEL;
}

impl EncodeSysCall for GG2Initghoul2Model {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(a.ghoul2_ptr as *const _),
            ptr_to_word(a.file_name.as_ptr()),
            a.model_index as isize,
            a.custom_skin as isize,
            a.custom_shader as isize,
            a.model_flags as isize,
            a.lod_bias as isize,
        ])
    }
}

impl DecodeSysCallReturn for GG2Initghoul2Model {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
