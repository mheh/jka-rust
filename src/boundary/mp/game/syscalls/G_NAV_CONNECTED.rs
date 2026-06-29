use core::ffi::c_int;

use crate::ffi::types::qboolean;
use crate::ffi::GameImport;

use crate::boundary::generic::{
    DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for the `G_NAV_CONNECTED` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct GNavConnectedArgs {
    start_id: c_int,
    end_id: c_int,
}

impl GNavConnectedArgs {
    pub fn new(start_id: c_int, end_id: c_int) -> Self {
        Self { start_id, end_id }
    }

    pub fn start_id(&self) -> c_int {
        self.start_id
    }

    pub fn end_id(&self) -> c_int {
        self.end_id
    }
}

/// `G_NAV_CONNECTED` MP game imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:314`
pub struct GNavConnected;

impl OutboundSysCall for GNavConnected {
    type Import = GameImport;
    type Args = GNavConnectedArgs;
    type Output = qboolean;

    const IMPORT: GameImport = GameImport::G_NAV_CONNECTED;
}

impl EncodeSysCall for GNavConnected {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([a.start_id as isize, a.end_id as isize])
    }
}

impl DecodeSysCallReturn for GNavConnected {
    fn decode_return(word: isize) -> Self::Output {
        word as qboolean
    }
}
