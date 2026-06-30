use core::ffi::c_void;

use super::super::MpUiImport;
use crate::shared::qboolean;

use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `UI_G2_HAVEWEGHOULMODELS`.
///
/// `ghoul2` is an opaque engine-owned ghoul2 instance handle (`void *` in the C
/// ABI); the engine only reads it to test for a live, non-empty instance, so it
/// is held as a raw `*mut c_void` and forwarded by address.
#[derive(Debug)]
pub struct UiG2HaveweghoulmodelsArgs {
    ghoul2: *mut c_void,
}

impl UiG2HaveweghoulmodelsArgs {
    pub const fn new(ghoul2: *mut c_void) -> Self {
        Self { ghoul2 }
    }

    pub const fn ghoul2(&self) -> *mut c_void {
        self.ghoul2
    }
}

/// `UI_G2_HAVEWEGHOULMODELS` MP UI imports syscall ABI token.
///
/// Source: `oracle/oracle/codemp/ui/ui_public.h:509`
pub struct UiG2Haveweghoulmodels;

impl OutboundSysCall for UiG2Haveweghoulmodels {
    type Import = MpUiImport;
    type Args = UiG2HaveweghoulmodelsArgs;
    type Output = qboolean;

    const IMPORT: MpUiImport = MpUiImport::UI_G2_HAVEWEGHOULMODELS;
}

impl EncodeSysCall for UiG2Haveweghoulmodels {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.ghoul2())])
    }
}

impl DecodeSysCallReturn for UiG2Haveweghoulmodels {
    // `trap_G2_HaveWeGhoul2Models` returns `qboolean`; the engine's return word
    // carries the flag.
    fn decode_return(word: isize) -> Self::Output {
        word as qboolean
    }
}
