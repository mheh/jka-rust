use core::ffi::c_void;

use super::super::MpGameImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use mp_qshared::shared::qboolean;

/// `G_G2_REMOVEGHOUL2MODELS` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct GG2Removeghoul2ModelsArgs {
    /// Pointer to the ghoul2 instance to free (callers pass `&ent.ghoul2` cast to `*mut c_void`).
    ghl_info: *mut c_void,
}

impl GG2Removeghoul2ModelsArgs {
    pub fn new(ghl_info: *mut c_void) -> Self {
        Self { ghl_info }
    }

    pub fn ghl_info(&self) -> *mut c_void {
        self.ghl_info
    }
}

/// `G_G2_REMOVEGHOUL2MODELS` MP game imports syscall ABI token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:528`
pub struct GG2Removeghoul2Models;

impl OutboundSysCall for GG2Removeghoul2Models {
    type Import = MpGameImport;
    type Args = GG2Removeghoul2ModelsArgs;
    type Output = qboolean;

    const IMPORT: MpGameImport = MpGameImport::G_G2_REMOVEGHOUL2MODELS;
}

impl EncodeSysCall for GG2Removeghoul2Models {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(a.ghl_info)])
    }
}

impl DecodeSysCallReturn for GG2Removeghoul2Models {
    fn decode_return(word: isize) -> Self::Output {
        word as qboolean
    }
}
