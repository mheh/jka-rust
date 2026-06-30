use core::ffi::c_int;

use super::super::MpGameImport;

use crate::abi::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `BOTLIB_EA_GESTURE` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct BotlibEaGestureArgs {
    client: c_int,
}

impl BotlibEaGestureArgs {
    pub fn new(client: c_int) -> Self {
        Self { client }
    }

    pub fn client(&self) -> c_int {
        self.client
    }
}

/// `BOTLIB_EA_GESTURE` MP game imports syscall ABI token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:387`
pub struct BotlibEaGesture;

impl OutboundSysCall for BotlibEaGesture {
    type Import = MpGameImport;
    type Args = BotlibEaGestureArgs;
    type Output = ();

    const IMPORT: MpGameImport = MpGameImport::BOTLIB_EA_GESTURE;
}

impl EncodeSysCall for BotlibEaGesture {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([a.client as isize])
    }
}

impl DecodeSysCallReturn for BotlibEaGesture {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
