use core::ffi::c_int;
use std::ffi::CString;

use super::super::MpUiImport;

use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `UI_TESTPRINTINT` outbound game-to-engine syscall.
///
/// C signature: `void testPrintInt( char *string, int i )`
#[derive(Debug)]
pub struct UiTestprintintArgs {
    string: CString,
    i: c_int,
}

impl UiTestprintintArgs {
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

/// `UI_TESTPRINTINT` MP UI imports syscall ABI token.
///
/// Source: `oracle/codemp/ui/ui_public.h:289`
pub struct UiTestprintint;

impl OutboundSysCall for UiTestprintint {
    type Import = MpUiImport;
    type Args = UiTestprintintArgs;
    type Output = ();

    const IMPORT: MpUiImport = MpUiImport::UI_TESTPRINTINT;
}

impl EncodeSysCall for UiTestprintint {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(a.string.as_ptr()), a.i as isize])
    }
}

impl DecodeSysCallReturn for UiTestprintint {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
