use core::ffi::{c_int, c_void};

use super::super::MpUiImport;
use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::shared::qboolean;
use crate::shared::sharedIKMoveParams_t;

/// `UI_G2_IKMOVE` outbound game-to-engine syscall.
///
/// C signature: `qboolean trap_G2API_IKMove(void *ghoul2, int time, sharedIKMoveParams_t *params)`
#[derive(Debug)]
pub struct UiG2IkmoveArgs {
    ghoul2: *mut c_void,
    time: c_int,
    params: *mut sharedIKMoveParams_t,
}

impl UiG2IkmoveArgs {
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

/// `UI_G2_IKMOVE` MP UI imports syscall ABI token.
///
/// Source: `oracle/oracle/codemp/ui/ui_public.h:560`
pub struct UiG2Ikmove;

impl OutboundSysCall for UiG2Ikmove {
    type Import = MpUiImport;
    type Args = UiG2IkmoveArgs;
    type Output = qboolean;

    const IMPORT: MpUiImport = MpUiImport::UI_G2_IKMOVE;
}

impl EncodeSysCall for UiG2Ikmove {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(a.ghoul2),
            a.time as isize,
            ptr_to_word(a.params),
        ])
    }
}

impl DecodeSysCallReturn for UiG2Ikmove {
    fn decode_return(word: isize) -> Self::Output {
        word as qboolean
    }
}
