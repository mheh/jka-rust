use core::ffi::{c_int, c_void};

use super::super::MpUiImport;
use mp_qshared::common::mp::qcommon::sharedRagDollUpdateParams_t;

use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `UI_G2_ANIMATEG2MODELS` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct UiG2Animateg2ModelsArgs {
    /// Ghoul2 model instance handle (opaque void*).
    ghoul2: *mut c_void,
    /// Current time in milliseconds.
    time: c_int,
    /// Ragdoll update parameters written by caller.
    params: *mut sharedRagDollUpdateParams_t,
}

impl UiG2Animateg2ModelsArgs {
    pub fn new(ghoul2: *mut c_void, time: c_int, params: *mut sharedRagDollUpdateParams_t) -> Self {
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

    pub fn params(&self) -> *mut sharedRagDollUpdateParams_t {
        self.params
    }
}

/// `UI_G2_ANIMATEG2MODELS` MP UI imports syscall ABI token.
///
/// Raven: rww - RAGDOLL_END
/// Raven: additional ragdoll options -rww
/// Source: `oracle/oracle/codemp/ui/ui_public.h:545`
pub struct UiG2Animateg2Models;

impl OutboundSysCall for UiG2Animateg2Models {
    type Import = MpUiImport;
    type Args = UiG2Animateg2ModelsArgs;
    type Output = ();

    const IMPORT: MpUiImport = MpUiImport::UI_G2_ANIMATEG2MODELS;
}

impl EncodeSysCall for UiG2Animateg2Models {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(a.ghoul2),
            a.time as isize,
            ptr_to_word(a.params),
        ])
    }
}

impl DecodeSysCallReturn for UiG2Animateg2Models {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
