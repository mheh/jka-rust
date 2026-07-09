use core::ffi::c_int;

use super::super::MpGameImport;
use mp_qshared::shared::qboolean;

use abi_transport::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `G_ICARUS_MAINTAINTASKMANAGER` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct GIcarusMaintaintaskmanagerArgs {
    ent_id: c_int,
}

impl GIcarusMaintaintaskmanagerArgs {
    pub fn new(ent_id: c_int) -> Self {
        Self { ent_id }
    }

    pub fn ent_id(&self) -> c_int {
        self.ent_id
    }
}

/// `G_ICARUS_MAINTAINTASKMANAGER` MP game imports syscall ABI token.
///
/// Source: `oracle/codemp/game/g_public.h:258`
pub struct GIcarusMaintaintaskmanager;

impl OutboundSysCall for GIcarusMaintaintaskmanager {
    type Import = MpGameImport;
    type Args = GIcarusMaintaintaskmanagerArgs;
    type Output = qboolean;

    const IMPORT: MpGameImport = MpGameImport::G_ICARUS_MAINTAINTASKMANAGER;
}

impl EncodeSysCall for GIcarusMaintaintaskmanager {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([a.ent_id as isize])
    }
}

impl DecodeSysCallReturn for GIcarusMaintaintaskmanager {
    fn decode_return(word: isize) -> Self::Output {
        word as qboolean
    }
}
