use core::ffi::{c_char, c_int};

use super::super::MpCgameExport;
use crate::boundary::generic::{
    word_to_c_int, word_to_const_ptr, DecodeVmMain, EncodeVmMainReturn, InboundVmCall,
    VmMainTransport,
};

/// Arguments for `CG_ROFF_NOTETRACK_CALLBACK`.
///
/// Args source: `oracle/oracle/codemp/cgame/cg_main.c:299-301`
/// Function source: `oracle/oracle/codemp/cgame/cg_ents.c:3555`
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CgRoffNotetrackCallbackArgs {
    ent_num: c_int,
    notetrack: *const c_char,
}

impl CgRoffNotetrackCallbackArgs {
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

/// `CG_ROFF_NOTETRACK_CALLBACK` MP cgame exports vmMain boundary token.
///
/// Raven: int entnum, char *notetrack
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:424`
/// Args source: `oracle/oracle/codemp/cgame/cg_main.c:299-301`
/// Output source: `oracle/oracle/codemp/cgame/cg_main.c:299-301`
/// Transport/call-site source: no engine call-site found in initial search; module vmMain switch proves arg slots.
pub struct CgRoffNotetrackCallback;

impl InboundVmCall for CgRoffNotetrackCallback {
    type Command = MpCgameExport;
    type Args = CgRoffNotetrackCallbackArgs;
    type Output = ();

    const COMMAND: MpCgameExport = MpCgameExport::CG_ROFF_NOTETRACK_CALLBACK;
}

impl DecodeVmMain for CgRoffNotetrackCallback {
    fn decode_vm_main(transport: VmMainTransport) -> Self::Args {
        CgRoffNotetrackCallbackArgs::new(
            word_to_c_int(transport.arg(0)),
            word_to_const_ptr(transport.arg(1)),
        )
    }
}

impl EncodeVmMainReturn for CgRoffNotetrackCallback {
    fn encode_return(_output: Self::Output) -> isize {
        0
    }
}
