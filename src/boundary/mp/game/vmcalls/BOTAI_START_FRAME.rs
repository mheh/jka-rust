use core::ffi::c_int;

use crate::ffi::GameExport;

use crate::boundary::generic::InboundVmCall;

// Flow:
//
//   executable --vmMain(BOTAI_START_FRAME, time, ...)--> jampgame
//   jampgame   --BotAIStartFrame(time)-----------------> advance bot AI state
//   jampgame   --return int----------------------------> executable
//
// `BOTAI_START_FRAME` is an inbound executable-to-game call raised once per bot
// frame to advance bot library and AI state.

/// Arguments for `BOTAI_START_FRAME`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BotAiStartFrameArgs {
    time: c_int,
}

impl BotAiStartFrameArgs {
    pub const fn new(time: c_int) -> Self {
        Self { time }
    }

    pub const fn time(self) -> c_int {
        self.time
    }
}

/// `BOTAI_START_FRAME` MP game exports vmMain boundary token.
///
/// Raven: ( int time );
/// Source: `oracle/oracle/codemp/game/g_public.h:764`
pub struct BotAiStartFrame;

impl InboundVmCall for BotAiStartFrame {
    type Command = GameExport;
    type Args = BotAiStartFrameArgs;
    type Output = c_int;

    const COMMAND: GameExport = GameExport::BOTAI_START_FRAME;
}
