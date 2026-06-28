use core::ffi::c_void;

use crate::ffi::GameImport;
use super::super::generic::{ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `G_G2_LISTSURFACES` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct GG2ListsurfacesArgs {
    /// Ghoul2 instance pointer whose surface list is dumped to the console.
    ghl_info: *mut c_void,
}

impl GG2ListsurfacesArgs {
    pub fn new(ghl_info: *mut c_void) -> Self {
        Self { ghl_info }
    }

    pub fn ghl_info(&self) -> *mut c_void {
        self.ghl_info
    }
}

pub struct GG2Listsurfaces;

impl OutboundSysCall for GG2Listsurfaces {
    type Args = GG2ListsurfacesArgs;
    type Output = ();

    const IMPORT: GameImport = GameImport::G_G2_LISTSURFACES;
}

impl EncodeSysCall for GG2Listsurfaces {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(a.ghl_info)])
    }
}

impl DecodeSysCallReturn for GG2Listsurfaces {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
