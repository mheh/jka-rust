use core::ffi::{c_int, c_void};

use crate::codemp::game::q_shared_h::sharedRagDollUpdateParams_t;
use crate::ffi::GameImport;

use crate::boundary::generic::{ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `G_G2_ANIMATEG2MODELS` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct GG2Animateg2ModelsArgs {
    /// Ghoul2 model instance handle (opaque void*).
    ghoul2: *mut c_void,
    /// Current time in milliseconds.
    time: c_int,
    /// Ragdoll update parameters written by caller.
    params: *mut sharedRagDollUpdateParams_t,
}

impl GG2Animateg2ModelsArgs {
    pub fn new(
        ghoul2: *mut c_void,
        time: c_int,
        params: *mut sharedRagDollUpdateParams_t,
    ) -> Self {
        Self { ghoul2, time, params }
    }

    pub fn ghoul2(&self) -> *mut c_void {
        self.ghoul2
    }

    pub fn time(&self) -> c_int {
        self.time
    }

    pub fn params(&self) -> *mut sharedRagDollUpdateParams_t {
        self.params
    }
}

pub struct GG2Animateg2Models;

impl OutboundSysCall for GG2Animateg2Models {
    type Import = GameImport;
    type Args = GG2Animateg2ModelsArgs;
    type Output = ();

    const IMPORT: GameImport = GameImport::G_G2_ANIMATEG2MODELS;
}

impl EncodeSysCall for GG2Animateg2Models {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(a.ghoul2),
            a.time as isize,
            ptr_to_word(a.params),
        ])
    }
}

impl DecodeSysCallReturn for GG2Animateg2Models {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
