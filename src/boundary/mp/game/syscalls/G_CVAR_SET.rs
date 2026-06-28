use std::ffi::CString;

use crate::ffi::GameImport;

use crate::boundary::generic::{ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `G_CVAR_SET` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct GCvarSetArgs {
    var_name: CString,
    value: CString,
}

impl GCvarSetArgs {
    pub fn new(var_name: CString, value: CString) -> Self {
        Self { var_name, value }
    }

    pub fn var_name(&self) -> &CString {
        &self.var_name
    }

    pub fn value(&self) -> &CString {
        &self.value
    }
}

pub struct GCvarSet;

impl OutboundSysCall for GCvarSet {
    type Import = GameImport;
    type Args = GCvarSetArgs;
    type Output = ();

    const IMPORT: GameImport = GameImport::G_CVAR_SET;
}

impl EncodeSysCall for GCvarSet {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(a.var_name.as_ptr()),
            ptr_to_word(a.value.as_ptr()),
        ])
    }
}

impl DecodeSysCallReturn for GCvarSet {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
