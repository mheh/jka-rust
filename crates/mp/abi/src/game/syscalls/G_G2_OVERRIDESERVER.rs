use core::ffi::c_void;

use super::super::MpGameImport;
use mp_qshared::shared::qboolean;

use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `G_G2_OVERRIDESERVER` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct GG2OverrideserverArgs {
    /// Server-side ghoul2 instance to make authoritative.
    server_instance: *mut c_void,
}

impl GG2OverrideserverArgs {
    pub fn new(server_instance: *mut c_void) -> Self {
        Self { server_instance }
    }

    pub fn server_instance(&self) -> *mut c_void {
        self.server_instance
    }
}

/// `G_G2_OVERRIDESERVER` MP game imports syscall ABI token.
///
/// Source: `oracle/codemp/game/g_public.h:567`
pub struct GG2Overrideserver;

impl OutboundSysCall for GG2Overrideserver {
    type Import = MpGameImport;
    type Args = GG2OverrideserverArgs;
    type Output = qboolean;

    const IMPORT: MpGameImport = MpGameImport::G_G2_OVERRIDESERVER;
}

impl EncodeSysCall for GG2Overrideserver {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(a.server_instance)])
    }
}

impl DecodeSysCallReturn for GG2Overrideserver {
    fn decode_return(word: isize) -> Self::Output {
        word as qboolean
    }
}
