use core::ffi::c_void;

use crate::ffi::types::qboolean;
use crate::ffi::GameImport;

use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `G_G2_ABSURDSMOOTHING` outbound game-to-engine syscall.
///
/// Hack for smoothing during ugly situations. forgive me.
/// Mirrors `trap_G2API_AbsurdSmoothing(ghoul2, status)`.
#[derive(Debug)]
pub struct GG2AbsurdsmoothingArgs {
    ghoul2: *mut c_void,
    status: qboolean,
}

impl GG2AbsurdsmoothingArgs {
    pub fn new(ghoul2: *mut c_void, status: qboolean) -> Self {
        Self { ghoul2, status }
    }

    pub fn ghoul2(&self) -> *mut c_void {
        self.ghoul2
    }

    pub fn status(&self) -> qboolean {
        self.status
    }
}

/// `G_G2_ABSURDSMOOTHING` MP game imports syscall ABI token.
///
/// Raven: rww - RAGDOLL_BEGIN
/// Source: `oracle/oracle/codemp/game/g_public.h:539`
pub struct GG2Absurdsmoothing;

impl OutboundSysCall for GG2Absurdsmoothing {
    type Import = GameImport;
    type Args = GG2AbsurdsmoothingArgs;
    type Output = ();

    const IMPORT: GameImport = GameImport::G_G2_ABSURDSMOOTHING;
}

impl EncodeSysCall for GG2Absurdsmoothing {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(a.ghoul2), a.status as isize])
    }
}

impl DecodeSysCallReturn for GG2Absurdsmoothing {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
