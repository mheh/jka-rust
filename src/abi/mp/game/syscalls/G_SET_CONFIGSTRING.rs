use core::ffi::c_int;
use std::ffi::CString;

use super::super::MpGameImport;
use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `G_SET_CONFIGSTRING` outbound game-to-engine syscall.
///
/// Mirrors the C ABI: `syscall(G_SET_CONFIGSTRING, num, string)` → void.
#[derive(Debug)]
pub struct GSetConfigstringArgs {
    /// Configstring index.
    num: c_int,
    /// Null-terminated value string.
    string: CString,
}

impl GSetConfigstringArgs {
    pub fn new(num: c_int, string: CString) -> Self {
        Self { num, string }
    }

    pub fn num(&self) -> c_int {
        self.num
    }

    pub fn string(&self) -> &CString {
        &self.string
    }
}

/// `G_SET_CONFIGSTRING` MP game imports syscall ABI token.
///
/// Raven: ( int num, const char *string );
/// Raven: config strings hold all the index strings, and various other information
/// Raven: that is reliably communicated to all clients
/// Raven: All of the current configstrings are sent to clients when
/// Raven: they connect, and changes are sent to all connected clients.
/// Raven: All confgstrings are cleared at each level start.
/// Source: `oracle/oracle/codemp/game/g_public.h:157`
pub struct GSetConfigstring;

impl OutboundSysCall for GSetConfigstring {
    type Import = MpGameImport;
    type Args = GSetConfigstringArgs;
    type Output = ();

    const IMPORT: MpGameImport = MpGameImport::G_SET_CONFIGSTRING;
}

impl EncodeSysCall for GSetConfigstring {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([a.num as isize, ptr_to_word(a.string.as_ptr())])
    }
}

impl DecodeSysCallReturn for GSetConfigstring {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
