use crate::ffi::types::qboolean;
use crate::ffi::GameImport;

use super::super::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `G_ROFF_PLAY` outbound game-to-engine syscall.
///
/// Mirrors `trap_ROFF_Play(ent_id, roff_id, do_translation)`.
#[derive(Debug)]
pub struct GRoffPlayArgs {
    /// Entity to attach the ROFF playback to.
    ent_id: i32,
    /// Handle returned by `trap_ROFF_Cache`.
    roff_id: i32,
    /// Whether to apply origin translation (`qtrue`/`qfalse`).
    do_translation: qboolean,
}

impl GRoffPlayArgs {
    pub fn new(ent_id: i32, roff_id: i32, do_translation: qboolean) -> Self {
        Self { ent_id, roff_id, do_translation }
    }

    pub fn ent_id(&self) -> i32 {
        self.ent_id
    }

    pub fn roff_id(&self) -> i32 {
        self.roff_id
    }

    pub fn do_translation(&self) -> qboolean {
        self.do_translation
    }
}

pub struct GRoffPlay;

impl OutboundSysCall for GRoffPlay {
    type Args = GRoffPlayArgs;
    type Output = qboolean;

    const IMPORT: GameImport = GameImport::G_ROFF_PLAY;
}

impl EncodeSysCall for GRoffPlay {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            a.ent_id as isize,
            a.roff_id as isize,
            a.do_translation as isize,
        ])
    }
}

impl DecodeSysCallReturn for GRoffPlay {
    fn decode_return(word: isize) -> Self::Output {
        word as qboolean
    }
}
