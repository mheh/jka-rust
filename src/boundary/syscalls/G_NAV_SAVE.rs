use core::ffi::c_int;
use std::ffi::CString;

use crate::ffi::GameImport;
use crate::ffi::types::qboolean;
use super::super::generic::{ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `G_NAV_SAVE` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct GNavSaveArgs {
    filename: CString,
    checksum: c_int,
}

impl GNavSaveArgs {
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

pub struct GNavSave;

impl OutboundSysCall for GNavSave {
    type Args = GNavSaveArgs;
    type Output = qboolean;

    const IMPORT: GameImport = GameImport::G_NAV_SAVE;
}

impl EncodeSysCall for GNavSave {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(a.filename.as_ptr()),
            a.checksum as isize,
        ])
    }
}

impl DecodeSysCallReturn for GNavSave {
    fn decode_return(word: isize) -> Self::Output {
        word as qboolean
    }
}
