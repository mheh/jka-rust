use core::ffi::c_void;

use crate::ffi::GameImport;

use crate::boundary::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `G_G2_LISTBONES` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct GG2ListbonesArgs {
    ghl_info: *mut c_void,
    frame: i32,
}

impl GG2ListbonesArgs {
    pub fn new(ghl_info: *mut c_void, frame: i32) -> Self {
        Self { ghl_info, frame }
    }

    pub fn ghl_info(&self) -> *mut c_void {
        self.ghl_info
    }

    pub fn frame(&self) -> i32 {
        self.frame
    }
}

/// `G_G2_LISTBONES` MP game imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:507`
pub struct GG2Listbones;

impl OutboundSysCall for GG2Listbones {
    type Import = GameImport;
    type Args = GG2ListbonesArgs;
    type Output = ();

    const IMPORT: GameImport = GameImport::G_G2_LISTBONES;
}

impl EncodeSysCall for GG2Listbones {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(a.ghl_info), a.frame as isize])
    }
}

impl DecodeSysCallReturn for GG2Listbones {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
