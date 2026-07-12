use core::ffi::c_int;
use std::ffi::CString;

use super::super::MpGameImport;

use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `G_TESTPRINTINT` outbound game-to-engine syscall.
///
/// C signature: `void testPrintInt( char *string, int i )`
#[derive(Debug)]
pub struct GTestprintintArgs {
    string: CString,
    i: c_int,
}

impl GTestprintintArgs {
    pub fn new(string: CString, i: c_int) -> Self {
        Self { string, i }
    }

    pub fn string(&self) -> &CString {
        &self.string
    }

    pub fn i(&self) -> c_int {
        self.i
    }
}

/// `G_TESTPRINTINT` MP game imports syscall ABI token.
///
/// Source: `oracle/codemp/game/g_public.h:289`
pub struct GTestprintint;

impl OutboundSysCall for GTestprintint {
    type Import = MpGameImport;
    type Args = GTestprintintArgs;
    type Output = ();

    const IMPORT: MpGameImport = MpGameImport::G_TESTPRINTINT;
}

impl EncodeSysCall for GTestprintint {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(a.string.as_ptr()), a.i as isize])
    }
}

impl DecodeSysCallReturn for GTestprintint {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
