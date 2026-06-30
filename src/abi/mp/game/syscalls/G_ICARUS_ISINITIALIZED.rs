use core::ffi::c_int;

use crate::ffi::GameImport;
use crate::shared::qboolean;

use crate::abi::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `G_ICARUS_ISINITIALIZED` outbound game-to-engine syscall.
///
/// Mirrors `syscall!(G_ICARUS_ISINITIALIZED, ent_id)` → `qboolean`.
#[derive(Debug)]
pub struct GIcarusIsinitializedArgs {
    ent_id: c_int,
}

impl GIcarusIsinitializedArgs {
    pub fn new(ent_id: c_int) -> Self {
        Self { ent_id }
    }

    pub fn ent_id(&self) -> c_int {
        self.ent_id
    }
}

/// `G_ICARUS_ISINITIALIZED` MP game imports syscall ABI token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:257`
pub struct GIcarusIsinitialized;

impl OutboundSysCall for GIcarusIsinitialized {
    type Import = GameImport;
    type Args = GIcarusIsinitializedArgs;
    type Output = qboolean;

    const IMPORT: GameImport = GameImport::G_ICARUS_ISINITIALIZED;
}

impl EncodeSysCall for GIcarusIsinitialized {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([a.ent_id as isize])
    }
}

impl DecodeSysCallReturn for GIcarusIsinitialized {
    fn decode_return(word: isize) -> Self::Output {
        word as qboolean
    }
}
