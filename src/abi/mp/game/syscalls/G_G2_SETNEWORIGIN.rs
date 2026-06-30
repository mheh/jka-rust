use core::ffi::c_void;

use crate::ffi::GameImport;
use crate::shared::qboolean;

use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `G_G2_SETNEWORIGIN` outbound game-to-engine syscall.
///
/// Re-origins a Ghoul2 instance to the bolt at `bolt_index`.
/// Mirrors `trap_G2API_SetNewOrigin(ghoul2, bolt_index)`.
#[derive(Debug)]
pub struct GG2SetneworiginArgs {
    /// Ghoul2 instance handle.
    ghoul2: *mut c_void,
    /// Index of the bolt to re-origin to.
    bolt_index: i32,
}

impl GG2SetneworiginArgs {
    pub fn new(ghoul2: *mut c_void, bolt_index: i32) -> Self {
        Self { ghoul2, bolt_index }
    }

    pub fn ghoul2(&self) -> *mut c_void {
        self.ghoul2
    }

    pub fn bolt_index(&self) -> i32 {
        self.bolt_index
    }
}

/// `G_G2_SETNEWORIGIN` MP game imports syscall ABI token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:535`
pub struct GG2Setneworigin;

impl OutboundSysCall for GG2Setneworigin {
    type Import = GameImport;
    type Args = GG2SetneworiginArgs;
    type Output = qboolean;

    const IMPORT: GameImport = GameImport::G_G2_SETNEWORIGIN;
}

impl EncodeSysCall for GG2Setneworigin {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(a.ghoul2), a.bolt_index as isize])
    }
}

impl DecodeSysCallReturn for GG2Setneworigin {
    fn decode_return(word: isize) -> Self::Output {
        word as qboolean
    }
}
