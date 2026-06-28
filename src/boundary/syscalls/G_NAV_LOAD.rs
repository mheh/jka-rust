use core::ffi::c_int;
use std::ffi::CString;

use crate::ffi::types::qboolean;
use crate::ffi::GameImport;
use super::super::generic::{ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

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

pub struct GNavLoad;

impl OutboundSysCall for GNavLoad {
    type Args = GNavLoadArgs;
    type Output = qboolean;

    const IMPORT: GameImport = GameImport::G_NAV_LOAD;
}

impl EncodeSysCall for GNavLoad {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(a.filename.as_ptr()),
            a.checksum as isize,
        ])
    }
}

impl DecodeSysCallReturn for GNavLoad {
    fn decode_return(word: isize) -> Self::Output {
        word as qboolean
    }
}
