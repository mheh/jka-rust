use crate::boundary::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::ffi::types::qboolean;
use crate::ffi::GameImport;
use core::ffi::c_int;
use std::ffi::CString;

/// `G_G2_SETROOTSURFACE` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct GG2SetrootsurfaceArgs {
    pub ghoul2: *mut core::ffi::c_void,
    pub model_index: c_int,
    pub surface_name: CString,
}

impl GG2SetrootsurfaceArgs {
    pub fn new(ghoul2: *mut core::ffi::c_void, model_index: c_int, surface_name: CString) -> Self {
        Self {
            ghoul2,
            model_index,
            surface_name,
        }
    }

    pub fn ghoul2(&self) -> *mut core::ffi::c_void {
        self.ghoul2
    }
    pub fn model_index(&self) -> c_int {
        self.model_index
    }
    pub fn surface_name(&self) -> &CString {
        &self.surface_name
    }
}

/// `G_G2_SETROOTSURFACE` MP game imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:533`
pub struct GG2Setrootsurface;

impl OutboundSysCall for GG2Setrootsurface {
    type Import = GameImport;
    type Args = GG2SetrootsurfaceArgs;
    type Output = qboolean;

    const IMPORT: GameImport = GameImport::G_G2_SETROOTSURFACE;
}

impl EncodeSysCall for GG2Setrootsurface {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(a.ghoul2),
            a.model_index as isize,
            ptr_to_word(a.surface_name.as_ptr()),
        ])
    }
}

impl DecodeSysCallReturn for GG2Setrootsurface {
    fn decode_return(word: isize) -> Self::Output {
        word as qboolean
    }
}
