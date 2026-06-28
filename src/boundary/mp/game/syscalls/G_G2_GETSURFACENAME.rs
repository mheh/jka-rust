use core::ffi::{c_int, c_void, c_char};

use crate::ffi::GameImport;

use crate::boundary::generic::{ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `G_G2_GETSURFACENAME` outbound game-to-engine syscall.
///
/// Mirrors `trap_G2API_GetSurfaceName(void *ghoul2, int surfNumber, int modelIndex, char *fillBuf)`.
/// `fillBuf` is an out-param written by the engine; it stays as a raw pointer in Args.
#[derive(Debug)]
pub struct GG2GetsurfacenameArgs {
    ghoul2: *mut c_void,
    surf_number: c_int,
    model_index: c_int,
    fill_buf: *mut c_char,
}

impl GG2GetsurfacenameArgs {
    pub fn new(ghoul2: *mut c_void, surf_number: c_int, model_index: c_int, fill_buf: *mut c_char) -> Self {
        Self { ghoul2, surf_number, model_index, fill_buf }
    }

    pub fn ghoul2(&self) -> *mut c_void { self.ghoul2 }
    pub fn surf_number(&self) -> c_int { self.surf_number }
    pub fn model_index(&self) -> c_int { self.model_index }
    pub fn fill_buf(&self) -> *mut c_char { self.fill_buf }
}

pub struct GG2Getsurfacename;

impl OutboundSysCall for GG2Getsurfacename {
    type Import = GameImport;
    type Args = GG2GetsurfacenameArgs;
    type Output = ();

    const IMPORT: GameImport = GameImport::G_G2_GETSURFACENAME;
}

impl EncodeSysCall for GG2Getsurfacename {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(a.ghoul2 as *const _),
            a.surf_number as isize,
            a.model_index as isize,
            ptr_to_word(a.fill_buf as *const _),
        ])
    }
}

impl DecodeSysCallReturn for GG2Getsurfacename {
    fn decode_return(_word: isize) -> Self::Output { () }
}
