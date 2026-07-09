use core::ffi::c_void;

use super::super::MpGameImport;
use mp_qshared::shared::qboolean;

use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `G_G2_RAGFORCESOLVE` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct GG2RagforcesolvArgs {
    ghoul2: *mut c_void,
    force: qboolean,
}

impl GG2RagforcesolvArgs {
    pub fn new(ghoul2: *mut c_void, force: qboolean) -> Self {
        Self { ghoul2, force }
    }

    pub fn ghoul2(&self) -> *mut c_void {
        self.ghoul2
    }

    pub fn force(&self) -> qboolean {
        self.force
    }
}

/// `G_G2_RAGFORCESOLVE` MP game imports syscall ABI token.
///
/// Raven: rww - ik move method, allows you to specify a bone and move it to a world point (within joint constraints)
/// Raven: by using the majority of gil's existing bone angling stuff from the ragdoll code.
/// Source: `oracle/codemp/game/g_public.h:555`
pub struct GG2Ragforcesolve;

impl OutboundSysCall for GG2Ragforcesolve {
    type Import = MpGameImport;
    type Args = GG2RagforcesolvArgs;
    type Output = qboolean;

    const IMPORT: MpGameImport = MpGameImport::G_G2_RAGFORCESOLVE;
}

impl EncodeSysCall for GG2Ragforcesolve {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(a.ghoul2), a.force as isize])
    }
}

impl DecodeSysCallReturn for GG2Ragforcesolve {
    fn decode_return(word: isize) -> Self::Output {
        word as qboolean
    }
}
