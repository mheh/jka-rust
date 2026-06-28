use core::ffi::{c_int, c_void};

use crate::ffi::GameImport;

use super::super::generic::{ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `G_G2_SIZE` outbound game-to-engine syscall.
///
/// Returns the number of ghoul2 models in the instance (`trap_G2API_Ghoul2Size`).
#[derive(Debug)]
pub struct GG2SizeArgs {
    ghl_info: *mut c_void,
}

impl GG2SizeArgs {
    pub fn new(ghl_info: *mut c_void) -> Self {
        Self { ghl_info }
    }

    pub fn ghl_info(&self) -> *mut c_void {
        self.ghl_info
    }
}

pub struct GG2Size;

impl OutboundSysCall for GG2Size {
    type Args = GG2SizeArgs;
    type Output = c_int;

    const IMPORT: GameImport = GameImport::G_G2_SIZE;
}

impl EncodeSysCall for GG2Size {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(a.ghl_info)])
    }
}

impl DecodeSysCallReturn for GG2Size {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
