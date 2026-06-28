use core::ffi::c_int;

use crate::ffi::GameImport;

use super::super::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

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

pub struct BotlibEaGesture;

impl OutboundSysCall for BotlibEaGesture {
    type Args = BotlibEaGestureArgs;
    type Output = ();

    const IMPORT: GameImport = GameImport::BOTLIB_EA_GESTURE;
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
