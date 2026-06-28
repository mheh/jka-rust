use core::ffi::c_int;

use crate::ffi::GameImport;
use crate::ffi::syscalls::pass_float;

use crate::boundary::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `BOTLIB_EA_END_REGULAR` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct BotlibEaEndRegularArgs {
    client: c_int,
    thinktime: f32,
}

impl BotlibEaEndRegularArgs {
    pub fn new(client: c_int, thinktime: f32) -> Self {
        Self { client, thinktime }
    }

    pub fn client(&self) -> c_int {
        self.client
    }

    pub fn thinktime(&self) -> f32 {
        self.thinktime
    }
}

pub struct BotlibEaEndRegular;

impl OutboundSysCall for BotlibEaEndRegular {
    type Import = GameImport;
    type Args = BotlibEaEndRegularArgs;
    type Output = ();

    const IMPORT: GameImport = GameImport::BOTLIB_EA_END_REGULAR;
}

impl EncodeSysCall for BotlibEaEndRegular {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            a.client as isize,
            pass_float(a.thinktime),
        ])
    }
}

impl DecodeSysCallReturn for BotlibEaEndRegular {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
