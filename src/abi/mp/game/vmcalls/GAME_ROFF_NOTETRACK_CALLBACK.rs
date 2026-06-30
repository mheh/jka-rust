use core::ffi::{c_char, c_int};

use super::super::MpGameExport;

use crate::abi::generic::InboundVmCall;

/// `GAME_ROFF_NOTETRACK_CALLBACK` MP game exports vmMain ABI token.
///
/// Raven: int entnum, char *notetrack
/// Source (enum): `oracle/oracle/codemp/game/g_public.h:766`
/// Source (args): `oracle/oracle/codemp/game/g_main.c:547`
/// Source (output): `oracle/oracle/codemp/game/g_main.c:549`
/// Source (call site): `oracle/oracle/codemp/qcommon/RoffSystem.cpp:957`
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GameRoffNotetrackCallbackArgs {
    ent_num: c_int,
    notetrack: *const c_char,
}

impl GameRoffNotetrackCallbackArgs {
    pub const fn new(ent_num: c_int, notetrack: *const c_char) -> Self {
        Self { ent_num, notetrack }
    }

    pub const fn ent_num(self) -> c_int {
        self.ent_num
    }

    pub const fn notetrack(self) -> *const c_char {
        self.notetrack
    }
}

pub struct GameRoffNotetrackCallback;

impl InboundVmCall for GameRoffNotetrackCallback {
    type Command = MpGameExport;
    type Args = GameRoffNotetrackCallbackArgs;
    type Output = ();

    const COMMAND: MpGameExport = MpGameExport::GAME_ROFF_NOTETRACK_CALLBACK;
}
