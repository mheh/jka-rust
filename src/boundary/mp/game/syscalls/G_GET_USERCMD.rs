use core::ffi::c_int;

use crate::codemp::game::q_shared_h::usercmd_t;
use crate::ffi::GameImport;
use crate::boundary::generic::{ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

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

pub struct GGetUsercmd;

impl OutboundSysCall for GGetUsercmd {
    type Import = GameImport;
    type Args = GGetUsercmdArgs;
    type Output = ();

    const IMPORT: GameImport = GameImport::G_GET_USERCMD;
}

impl EncodeSysCall for GGetUsercmd {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            a.client_num as isize,
            ptr_to_word(a.cmd),
        ])
    }
}

impl DecodeSysCallReturn for GGetUsercmd {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
