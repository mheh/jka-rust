use core::ffi::{c_char, c_int};

use super::super::MpGameExport;

use abi_transport::generic::{
    word_to_c_int, word_to_const_ptr, DecodeVmMain, EncodeVmMainReturn, InboundVmCall,
    VmMainTransport,
};

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

impl DecodeVmMain for GameRoffNotetrackCallback {
    fn decode_vm_main(t: VmMainTransport) -> Self::Args {
        // `G_ROFF_NotetrackCallback( &g_entities[arg0], (const char *)arg1 )`
        // — g_main.c:548. arg0 is the entity index; the entity pointer is
        // resolved at the dispatch call site; arg1 is a real `const char *`.
        GameRoffNotetrackCallbackArgs::new(word_to_c_int(t.arg(0)), word_to_const_ptr(t.arg(1)))
    }
}

impl EncodeVmMainReturn for GameRoffNotetrackCallback {
    fn encode_return(_output: Self::Output) -> isize {
        // `G_ROFF_NotetrackCallback(...); return 0;` — g_main.c:548-549.
        0
    }
}
