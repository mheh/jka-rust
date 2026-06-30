use super::super::MpUiImport;
use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::common::mp::qcommon::sharedRagDollParams_t;
use core::ffi::c_void;

/// `UiG2_SETRAGDOLL` outbound game-to-engine syscall.
///
/// Maps to `trap_G2API_SetRagDoll(void *ghoul2, sharedRagDollParams_t *params)` — void return.
#[derive(Debug)]
pub struct GG2SetragdollArgs {
    ghoul2: *mut c_void,
    params: *mut sharedRagDollParams_t,
}

impl GG2SetragdollArgs {
    pub fn new(ghoul2: *mut c_void, params: *mut sharedRagDollParams_t) -> Self {
        Self { ghoul2, params }
    }

    pub fn ghoul2(&self) -> *mut c_void {
        self.ghoul2
    }

    pub fn params(&self) -> *mut sharedRagDollParams_t {
        self.params
    }
}

/// `UiG2_SETRAGDOLL` MP game imports syscall ABI token.
///
/// Source: `oracle/oracle/codemp/ui/ui_public.h:544`
pub struct GG2Setragdoll;

impl OutboundSysCall for GG2Setragdoll {
    type Import = MpUiImport;
    type Args = GG2SetragdollArgs;
    type Output = ();

    const IMPORT: MpUiImport = MpUiImport::UiG2_SETRAGDOLL;
}

impl EncodeSysCall for GG2Setragdoll {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(a.ghoul2 as *const _),
            ptr_to_word(a.params as *const _),
        ])
    }
}

impl DecodeSysCallReturn for GG2Setragdoll {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
