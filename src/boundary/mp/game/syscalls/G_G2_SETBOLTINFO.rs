use core::ffi::{c_int, c_void};

use crate::ffi::GameImport;

use crate::boundary::generic::{ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `G_G2_SETBOLTINFO` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct GG2SetboltinfoArgs {
    ghoul2: *mut c_void,
    model_index: c_int,
    bolt_info: c_int,
}

impl GG2SetboltinfoArgs {
    pub fn new(ghoul2: *mut c_void, model_index: c_int, bolt_info: c_int) -> Self {
        Self { ghoul2, model_index, bolt_info }
    }

    pub fn ghoul2(&self) -> *mut c_void { self.ghoul2 }
    pub fn model_index(&self) -> c_int { self.model_index }
    pub fn bolt_info(&self) -> c_int { self.bolt_info }
}

pub struct GG2Setboltinfo;

impl OutboundSysCall for GG2Setboltinfo {
    type Import = GameImport;
    type Args = GG2SetboltinfoArgs;
    type Output = ();

    const IMPORT: GameImport = GameImport::G_G2_SETBOLTINFO;
}

impl EncodeSysCall for GG2Setboltinfo {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(a.ghoul2()),
            a.model_index() as isize,
            a.bolt_info() as isize,
        ])
    }
}

impl DecodeSysCallReturn for GG2Setboltinfo {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
