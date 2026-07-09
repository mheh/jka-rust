use core::ffi::c_int;

use super::super::MpGameImport;
use abi_transport::pass_float;

use abi_transport::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

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

/// `BOTLIB_EA_END_REGULAR` MP game imports syscall ABI token.
///
/// Source: `oracle/codemp/game/g_public.h:408`
pub struct BotlibEaEndRegular;

impl OutboundSysCall for BotlibEaEndRegular {
    type Import = MpGameImport;
    type Args = BotlibEaEndRegularArgs;
    type Output = ();

    const IMPORT: MpGameImport = MpGameImport::BOTLIB_EA_END_REGULAR;
}

impl EncodeSysCall for BotlibEaEndRegular {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([a.client as isize, pass_float(a.thinktime)])
    }
}

impl DecodeSysCallReturn for BotlibEaEndRegular {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
