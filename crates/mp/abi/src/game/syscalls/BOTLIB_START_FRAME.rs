use core::ffi::c_int;

use super::super::MpGameImport;
use abi_transport::pass_float;

use abi_transport::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `BOTLIB_START_FRAME` outbound game-to-engine syscall.
///
/// Advances the bot library's clock to `time` (seconds, as a float passed
/// through the C ABI via bit-cast integer word).
#[derive(Debug)]
pub struct BotlibStartFrameArgs {
    /// Simulation time in seconds.
    time: f32,
}

impl BotlibStartFrameArgs {
    pub fn new(time: f32) -> Self {
        Self { time }
    }

    pub fn time(&self) -> f32 {
        self.time
    }
}

/// `BOTLIB_START_FRAME` MP game imports syscall ABI token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:347`
pub struct BotlibStartFrame;

impl OutboundSysCall for BotlibStartFrame {
    type Import = MpGameImport;
    type Args = BotlibStartFrameArgs;
    type Output = c_int;

    const IMPORT: MpGameImport = MpGameImport::BOTLIB_START_FRAME;
}

impl EncodeSysCall for BotlibStartFrame {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([pass_float(a.time)])
    }
}

impl DecodeSysCallReturn for BotlibStartFrame {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
