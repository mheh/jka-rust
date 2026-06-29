use core::ffi::c_int;
use std::ffi::CString;

use crate::boundary::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::ffi::types::qboolean;
use crate::ffi::GameImport;

/// `G_NAV_LOAD` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct GNavLoadArgs {
    filename: CString,
    checksum: c_int,
}

impl GNavLoadArgs {
    pub fn new(filename: CString, checksum: c_int) -> Self {
        Self { filename, checksum }
    }

    pub fn filename(&self) -> &CString {
        &self.filename
    }

    pub fn checksum(&self) -> c_int {
        self.checksum
    }
}

/// `G_NAV_LOAD` MP game imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:300`
pub struct GNavLoad;

impl OutboundSysCall for GNavLoad {
    type Import = GameImport;
    type Args = GNavLoadArgs;
    type Output = qboolean;

    const IMPORT: GameImport = GameImport::G_NAV_LOAD;
}

impl EncodeSysCall for GNavLoad {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(a.filename.as_ptr()), a.checksum as isize])
    }
}

impl DecodeSysCallReturn for GNavLoad {
    fn decode_return(word: isize) -> Self::Output {
        word as qboolean
    }
}
