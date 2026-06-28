use core::ffi::c_int;

use crate::ffi::{types::qboolean, GameImport};

use crate::boundary::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `G_ICARUS_ISRUNNING` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct GIcarusIsrunningArgs {
    ent_id: c_int,
}

impl GIcarusIsrunningArgs {
    pub fn new(ent_id: c_int) -> Self {
        Self { ent_id }
    }

    pub fn ent_id(&self) -> c_int {
        self.ent_id
    }
}

pub struct GIcarusIsrunning;

impl OutboundSysCall for GIcarusIsrunning {
    type Import = GameImport;
    type Args = GIcarusIsrunningArgs;
    type Output = qboolean;

    const IMPORT: GameImport = GameImport::G_ICARUS_ISRUNNING;
}

impl EncodeSysCall for GIcarusIsrunning {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([a.ent_id as isize])
    }
}

impl DecodeSysCallReturn for GIcarusIsrunning {
    fn decode_return(word: isize) -> Self::Output {
        word as qboolean
    }
}
