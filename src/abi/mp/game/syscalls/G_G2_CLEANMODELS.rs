use core::ffi::c_void;

use crate::ffi::GameImport;

use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `G_G2_CLEANMODELS` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct GG2CleanmodelsArgs {
    /// Pointer to the ghoul2 instance pointer; the engine frees the instance and NULLs the slot.
    ghoul2_ptr: *mut *mut c_void,
}

impl GG2CleanmodelsArgs {
    pub fn new(ghoul2_ptr: *mut *mut c_void) -> Self {
        Self { ghoul2_ptr }
    }

    pub fn ghoul2_ptr(&self) -> *mut *mut c_void {
        self.ghoul2_ptr
    }
}

/// `G_G2_CLEANMODELS` MP game imports syscall ABI token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:529`
pub struct GG2Cleanmodels;

impl OutboundSysCall for GG2Cleanmodels {
    type Import = GameImport;
    type Args = GG2CleanmodelsArgs;
    type Output = ();

    const IMPORT: GameImport = GameImport::G_G2_CLEANMODELS;
}

impl EncodeSysCall for GG2Cleanmodels {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(a.ghoul2_ptr as *const _)])
    }
}

impl DecodeSysCallReturn for GG2Cleanmodels {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
