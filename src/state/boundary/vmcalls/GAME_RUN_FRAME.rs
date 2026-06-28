use core::ffi::c_int;

use crate::ffi::GameExport;

use super::super::generic::{
    word_to_c_int, DecodeVmMain, EncodeVmMainReturn, InboundVmCall, VmMainTransport,
};

// Flow:
//
//   executable --vmMain(GAME_RUN_FRAME, levelTime, ...)--> jampgame
//   jampgame   --G_RunFrame(levelTime)-------------------> advance level state
//   jampgame   --return 0-------------------------------> executable
//
// `GAME_RUN_FRAME` is an inbound executable-to-game call raised once per server
// frame to advance game simulation state.

/// Arguments for `GAME_RUN_FRAME`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GameRunFrameArgs {
    level_time: c_int,
}

impl GameRunFrameArgs {
    pub const fn new(level_time: c_int) -> Self {
        Self { level_time }
    }

    pub const fn level_time(self) -> c_int {
        self.level_time
    }
}

/// `GAME_RUN_FRAME` advances the level simulation.
pub struct GameRunFrame;

impl InboundVmCall for GameRunFrame {
    type Args = GameRunFrameArgs;
    type Output = ();

    const COMMAND: GameExport = GameExport::GAME_RUN_FRAME;
}

impl DecodeVmMain for GameRunFrame {
    fn decode_vm_main(transport: VmMainTransport) -> Self::Args {
        GameRunFrameArgs::new(word_to_c_int(transport.arg(0)))
    }
}

impl EncodeVmMainReturn for GameRunFrame {
    // `vmMain` returns 0 for `GAME_RUN_FRAME` (the C arm yields no value).
    fn encode_return(_output: Self::Output) -> isize {
        0
    }
}
