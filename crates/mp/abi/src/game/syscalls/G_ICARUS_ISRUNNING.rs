use core::ffi::c_int;

use super::super::MpGameImport;
use mp_qshared::shared::qboolean;

use abi_transport::generic::{
    DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

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

/// `G_ICARUS_ISRUNNING` MP game imports syscall ABI token.
///
/// Source: `oracle/codemp/game/g_public.h:259`
pub struct GIcarusIsrunning;

impl OutboundSysCall for GIcarusIsrunning {
    type Import = MpGameImport;
    type Args = GIcarusIsrunningArgs;
    type Output = qboolean;

    const IMPORT: MpGameImport = MpGameImport::G_ICARUS_ISRUNNING;
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
