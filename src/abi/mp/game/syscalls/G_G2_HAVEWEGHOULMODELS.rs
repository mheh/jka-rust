use core::ffi::c_void;

use crate::ffi::GameImport;
use crate::shared::qboolean;

use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `G_G2_HAVEWEGHOULMODELS`.
///
/// `ghoul2` is an opaque engine-owned ghoul2 instance handle (`void *` in the C
/// ABI); the engine only reads it to test for a live, non-empty instance, so it
/// is held as a raw `*mut c_void` and forwarded by address.
#[derive(Debug)]
pub struct GG2HaveweghoulmodelsArgs {
    ghoul2: *mut c_void,
}

impl GG2HaveweghoulmodelsArgs {
    pub const fn new(ghoul2: *mut c_void) -> Self {
        Self { ghoul2 }
    }

    pub const fn ghoul2(&self) -> *mut c_void {
        self.ghoul2
    }
}

/// `G_G2_HAVEWEGHOULMODELS` MP game imports syscall ABI token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:509`
pub struct GG2Haveweghoulmodels;

impl OutboundSysCall for GG2Haveweghoulmodels {
    type Import = GameImport;
    type Args = GG2HaveweghoulmodelsArgs;
    type Output = qboolean;

    const IMPORT: GameImport = GameImport::G_G2_HAVEWEGHOULMODELS;
}

impl EncodeSysCall for GG2Haveweghoulmodels {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.ghoul2())])
    }
}

impl DecodeSysCallReturn for GG2Haveweghoulmodels {
    // `trap_G2_HaveWeGhoul2Models` returns `qboolean`; the engine's return word
    // carries the flag.
    fn decode_return(word: isize) -> Self::Output {
        word as qboolean
    }
}
