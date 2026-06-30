use core::ffi::{c_int, c_void};

use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::codemp::game::q_shared_h::sharedIKMoveParams_t;
use crate::ffi::GameImport;
use crate::shared::qboolean;

/// `G_G2_IKMOVE` outbound game-to-engine syscall.
///
/// C signature: `qboolean trap_G2API_IKMove(void *ghoul2, int time, sharedIKMoveParams_t *params)`
#[derive(Debug)]
pub struct GG2IkmoveArgs {
    ghoul2: *mut c_void,
    time: c_int,
    params: *mut sharedIKMoveParams_t,
}

impl GG2IkmoveArgs {
    pub fn new(ghoul2: *mut c_void, time: c_int, params: *mut sharedIKMoveParams_t) -> Self {
        Self {
            ghoul2,
            time,
            params,
        }
    }

    pub fn ghoul2(&self) -> *mut c_void {
        self.ghoul2
    }

    pub fn time(&self) -> c_int {
        self.time
    }

    pub fn params(&self) -> *mut sharedIKMoveParams_t {
        self.params
    }
}

/// `G_G2_IKMOVE` MP game imports syscall ABI token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:560`
pub struct GG2Ikmove;

impl OutboundSysCall for GG2Ikmove {
    type Import = GameImport;
    type Args = GG2IkmoveArgs;
    type Output = qboolean;

    const IMPORT: GameImport = GameImport::G_G2_IKMOVE;
}

impl EncodeSysCall for GG2Ikmove {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(a.ghoul2),
            a.time as isize,
            ptr_to_word(a.params),
        ])
    }
}

impl DecodeSysCallReturn for GG2Ikmove {
    fn decode_return(word: isize) -> Self::Output {
        word as qboolean
    }
}
