use core::ffi::c_int;

use crate::ffi::GameImport;
use crate::ffi::syscalls::pass_float;

use super::super::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

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

pub struct BotlibStartFrame;

impl OutboundSysCall for BotlibStartFrame {
    type Args = BotlibStartFrameArgs;
    type Output = c_int;

    const IMPORT: GameImport = GameImport::BOTLIB_START_FRAME;
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
