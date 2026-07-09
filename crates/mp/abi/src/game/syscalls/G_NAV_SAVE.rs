use core::ffi::c_int;
use std::ffi::CString;

use super::super::MpGameImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use mp_qshared::shared::qboolean;

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

/// `G_NAV_SAVE` MP game imports syscall ABI token.
///
/// Source: `oracle/codemp/game/g_public.h:301`
pub struct GNavSave;

impl OutboundSysCall for GNavSave {
    type Import = MpGameImport;
    type Args = GNavSaveArgs;
    type Output = qboolean;

    const IMPORT: MpGameImport = MpGameImport::G_NAV_SAVE;
}

impl EncodeSysCall for GNavSave {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(a.filename.as_ptr()), a.checksum as isize])
    }
}

impl DecodeSysCallReturn for GNavSave {
    fn decode_return(word: isize) -> Self::Output {
        word as qboolean
    }
}
