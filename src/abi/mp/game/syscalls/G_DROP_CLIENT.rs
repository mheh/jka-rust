use core::ffi::c_int;
use std::ffi::CString;

use super::super::MpGameImport;
use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

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

/// `G_DROP_CLIENT` MP game imports syscall ABI token.
///
/// Raven: ( int clientNum, const char *reason );
/// Raven: kick a client off the server with a message
/// Source: `oracle/oracle/codemp/game/g_public.h:150`
pub struct GDropClient;

impl OutboundSysCall for GDropClient {
    type Import = MpGameImport;
    type Args = GDropClientArgs;
    type Output = ();

    const IMPORT: MpGameImport = MpGameImport::G_DROP_CLIENT;
}

impl EncodeSysCall for GDropClient {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([a.client_num as isize, ptr_to_word(a.reason.as_ptr())])
    }
}

impl DecodeSysCallReturn for GDropClient {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
