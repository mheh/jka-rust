use core::ffi::c_int;
use std::ffi::CString;

use crate::ffi::GameImport;
use crate::boundary::generic::{ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `G_DROP_CLIENT` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct GDropClientArgs {
    client_num: c_int,
    reason: CString,
}

impl GDropClientArgs {
    pub fn new(client_num: c_int, reason: CString) -> Self {
        Self { client_num, reason }
    }

    pub fn client_num(&self) -> c_int {
        self.client_num
    }

    pub fn reason(&self) -> &CString {
        &self.reason
    }
}

pub struct GDropClient;

impl OutboundSysCall for GDropClient {
    type Import = GameImport;
    type Args = GDropClientArgs;
    type Output = ();

    const IMPORT: GameImport = GameImport::G_DROP_CLIENT;
}

impl EncodeSysCall for GDropClient {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            a.client_num as isize,
            ptr_to_word(a.reason.as_ptr()),
        ])
    }
}

impl DecodeSysCallReturn for GDropClient {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
