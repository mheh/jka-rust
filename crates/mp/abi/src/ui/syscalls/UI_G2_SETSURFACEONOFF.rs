use core::ffi::{c_char, c_void};

use super::super::MpUiImport;
use mp_qshared::shared::qboolean;

use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `UI_G2_SETSURFACEONOFF` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct UiG2SetsurfaceonoffArgs {
    /// Ghoul2 instance handle.
    ghoul2: *mut c_void,
    /// Surface name as a raw C string (caller owns the buffer).
    surface_name: *const c_char,
    /// Surface flags to apply (0 = on, non-zero bits = off/variant).
    flags: i32,
}

impl UiG2SetsurfaceonoffArgs {
    pub fn new(ghoul2: *mut c_void, surface_name: *const c_char, flags: i32) -> Self {
        Self {
            ghoul2,
            surface_name,
            flags,
        }
    }

    pub fn ghoul2(&self) -> *mut c_void {
        self.ghoul2
    }
    pub fn surface_name(&self) -> *const c_char {
        self.surface_name
    }
    pub fn flags(&self) -> i32 {
        self.flags
    }
}

/// `UI_G2_SETSURFACEONOFF` MP UI imports syscall ABI token.
///
/// Source: `oracle/codemp/ui/ui_public.h:534`
pub struct UiG2Setsurfaceonoff;

impl OutboundSysCall for UiG2Setsurfaceonoff {
    type Import = MpUiImport;
    type Args = UiG2SetsurfaceonoffArgs;
    type Output = qboolean;

    const IMPORT: MpUiImport = MpUiImport::UI_G2_SETSURFACEONOFF;
}

impl EncodeSysCall for UiG2Setsurfaceonoff {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(a.ghoul2 as *const _),
            ptr_to_word(a.surface_name as *const _),
            a.flags as isize,
        ])
    }
}

impl DecodeSysCallReturn for UiG2Setsurfaceonoff {
    fn decode_return(word: isize) -> Self::Output {
        word as qboolean
    }
}
