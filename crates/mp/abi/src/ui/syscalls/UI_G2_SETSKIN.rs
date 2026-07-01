use core::ffi::{c_int, c_void};

use super::super::MpUiImport;
use mp_qshared::shared::qboolean;

use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Args for the `UI_G2_SETSKIN` outbound game-to-engine syscall.
///
/// Mirrors `syscall!(UI_G2_SETSKIN, ghoul2, model_index, custom_skin, render_skin)`.
#[derive(Debug)]
pub struct UiG2SetskinArgs {
    /// Ghoul2 instance pointer.
    pub ghoul2: *mut c_void,
    /// Model index within the ghoul2 instance.
    pub model_index: c_int,
    /// Registered `.skin` handle used for surface on/off overrides (0 for none).
    pub custom_skin: c_int,
    /// Skin handle the renderer draws with.
    pub render_skin: c_int,
}

impl UiG2SetskinArgs {
    pub fn new(
        ghoul2: *mut c_void,
        model_index: c_int,
        custom_skin: c_int,
        render_skin: c_int,
    ) -> Self {
        Self {
            ghoul2,
            model_index,
            custom_skin,
            render_skin,
        }
    }

    pub fn ghoul2(&self) -> *mut c_void {
        self.ghoul2
    }
    pub fn model_index(&self) -> c_int {
        self.model_index
    }
    pub fn custom_skin(&self) -> c_int {
        self.custom_skin
    }
    pub fn render_skin(&self) -> c_int {
        self.render_skin
    }
}

/// `UI_G2_SETSKIN` MP UI imports syscall ABI token.
///
/// Source: `oracle/oracle/codemp/ui/ui_public.h:515`
pub struct UiG2Setskin;

impl OutboundSysCall for UiG2Setskin {
    type Import = MpUiImport;
    type Args = UiG2SetskinArgs;
    type Output = qboolean;

    const IMPORT: MpUiImport = MpUiImport::UI_G2_SETSKIN;
}

impl EncodeSysCall for UiG2Setskin {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(a.ghoul2),
            a.model_index as isize,
            a.custom_skin as isize,
            a.render_skin as isize,
        ])
    }
}

impl DecodeSysCallReturn for UiG2Setskin {
    fn decode_return(word: isize) -> Self::Output {
        word as qboolean
    }
}
