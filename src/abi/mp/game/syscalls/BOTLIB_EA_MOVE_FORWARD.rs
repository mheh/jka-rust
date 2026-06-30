use core::ffi::c_int;

use super::super::MpGameImport;

use crate::abi::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// Arguments for the `BOTLIB_EA_MOVE_FORWARD` game→engine syscall.
///
/// Mirrors `syscall!(BOTLIB_EA_MOVE_FORWARD, client)`.
#[derive(Debug)]
pub struct BotlibEaMoveForwardArgs {
    /// Bot client number.
    client: c_int,
}

impl BotlibEaMoveForwardArgs {
    pub fn new(client: c_int) -> Self {
        Self { client }
    }

    pub fn client(&self) -> c_int {
        self.client
    }
}

/// `BOTLIB_EA_MOVE_FORWARD` MP game imports syscall ABI token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:397`
pub struct BotlibEaMoveForward;

impl OutboundSysCall for BotlibEaMoveForward {
    type Import = MpGameImport;
    type Args = BotlibEaMoveForwardArgs;
    type Output = ();

    const IMPORT: MpGameImport = MpGameImport::BOTLIB_EA_MOVE_FORWARD;
}

impl EncodeSysCall for BotlibEaMoveForward {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([a.client as isize])
    }
}

impl DecodeSysCallReturn for BotlibEaMoveForward {
    fn decode_return(_word: isize) -> Self::Output {}
}
