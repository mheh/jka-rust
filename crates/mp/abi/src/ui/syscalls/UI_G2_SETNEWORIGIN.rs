use core::ffi::c_void;

use super::super::MpUiImport;
use mp_qshared::shared::qboolean;

use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `UI_G2_SETNEWORIGIN` outbound game-to-engine syscall.
///
/// Re-origins a Ghoul2 instance to the bolt at `bolt_index`.
/// Mirrors `trap_G2API_SetNewOrigin(ghoul2, bolt_index)`.
#[derive(Debug)]
pub struct UiG2SetneworiginArgs {
    /// Ghoul2 instance handle.
    ghoul2: *mut c_void,
    /// Index of the bolt to re-origin to.
    bolt_index: i32,
}

impl UiG2SetneworiginArgs {
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

/// `UI_G2_SETNEWORIGIN` MP UI imports syscall ABI token.
///
/// Source: `oracle/oracle/codemp/ui/ui_public.h:535`
pub struct UiG2Setneworigin;

impl OutboundSysCall for UiG2Setneworigin {
    type Import = MpUiImport;
    type Args = UiG2SetneworiginArgs;
    type Output = qboolean;

    const IMPORT: MpUiImport = MpUiImport::UI_G2_SETNEWORIGIN;
}

impl EncodeSysCall for UiG2Setneworigin {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(a.ghoul2), a.bolt_index as isize])
    }
}

impl DecodeSysCallReturn for UiG2Setneworigin {
    fn decode_return(word: isize) -> Self::Output {
        word as qboolean
    }
}
