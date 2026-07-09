use core::ffi::c_void;

use super::super::MpCgameImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `CG_R_SETCOLOR`.
///
/// Raven wrapper: `void trap_R_SetColor( const float *rgba )`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgRSetcolorArgs {
    rgba: *const f32,
}

impl CgRSetcolorArgs {
    /// # Safety
    /// `rgba` must point to at least four readable floats for the duration of the syscall.
    pub const unsafe fn new(rgba: *const f32) -> Self {
        Self { rgba }
    }

    pub const fn rgba(&self) -> *const f32 {
        self.rgba
    }
}

/// `CG_R_SETCOLOR` MP cgame imports syscall ABI token.
///
/// Source: `oracle/codemp/cgame/cg_public.h:159`
pub struct CgRSetcolor;

impl OutboundSysCall for CgRSetcolor {
    type Import = MpCgameImport;
    type Args = CgRSetcolorArgs;
    type Output = ();

    const IMPORT: MpCgameImport = MpCgameImport::CG_R_SETCOLOR;
}

impl EncodeSysCall for CgRSetcolor {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.rgba() as *const c_void)])
    }
}

impl DecodeSysCallReturn for CgRSetcolor {
    fn decode_return(_word: isize) -> Self::Output {}
}
