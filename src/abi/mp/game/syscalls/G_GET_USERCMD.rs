use core::ffi::c_int;

use super::super::MpGameImport;
use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::common::mp::qcommon::usercmd_t;

/// `G_GET_USERCMD` outbound game-to-engine syscall.
///
/// C ABI: `void trap_GetUsercmd( int clientNum, usercmd_t *cmd )`
/// The engine fills `*cmd` in-place; `cmd` is an out-param kept as a raw pointer.
#[derive(Debug)]
pub struct GGetUsercmdArgs {
    /// Index of the client whose usercmd the engine should copy out.
    client_num: c_int,
    /// Caller-owned storage that the engine writes into.
    cmd: *mut usercmd_t,
}

impl GGetUsercmdArgs {
    pub fn new(client_num: c_int, cmd: *mut usercmd_t) -> Self {
        Self { client_num, cmd }
    }

    pub fn client_num(&self) -> c_int {
        self.client_num
    }

    pub fn cmd(&self) -> *mut usercmd_t {
        self.cmd
    }
}

/// `G_GET_USERCMD` MP game imports syscall ABI token.
///
/// Raven: ( int clientNum, usercmd_t *cmd )
/// Source: `oracle/oracle/codemp/game/g_public.h:219`
pub struct GGetUsercmd;

impl OutboundSysCall for GGetUsercmd {
    type Import = MpGameImport;
    type Args = GGetUsercmdArgs;
    type Output = ();

    const IMPORT: MpGameImport = MpGameImport::G_GET_USERCMD;
}

impl EncodeSysCall for GGetUsercmd {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([a.client_num as isize, ptr_to_word(a.cmd)])
    }
}

impl DecodeSysCallReturn for GGetUsercmd {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
