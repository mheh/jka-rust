use core::ffi::{c_int, c_void};

use crate::ffi::GameImport;
use crate::ffi::types::qboolean;

use crate::boundary::generic::{ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// Args for the `G_G2_SETSKIN` outbound game-to-engine syscall.
///
/// Mirrors `syscall!(G_G2_SETSKIN, ghoul2, model_index, custom_skin, render_skin)`.
#[derive(Debug)]
pub struct GG2SetskinArgs {
    /// Ghoul2 instance pointer.
    pub ghoul2: *mut c_void,
    /// Model index within the ghoul2 instance.
    pub model_index: c_int,
    /// Registered `.skin` handle used for surface on/off overrides (0 for none).
    pub custom_skin: c_int,
    /// Skin handle the renderer draws with.
    pub render_skin: c_int,
}

impl GG2SetskinArgs {
    pub fn new(
        ghoul2: *mut c_void,
        model_index: c_int,
        custom_skin: c_int,
        render_skin: c_int,
    ) -> Self {
        Self { ghoul2, model_index, custom_skin, render_skin }
    }

    pub fn ghoul2(&self) -> *mut c_void { self.ghoul2 }
    pub fn model_index(&self) -> c_int { self.model_index }
    pub fn custom_skin(&self) -> c_int { self.custom_skin }
    pub fn render_skin(&self) -> c_int { self.render_skin }
}

/// `G_G2_SETSKIN` outbound game-to-engine syscall.
pub struct GG2Setskin;

impl OutboundSysCall for GG2Setskin {
    type Import = GameImport;
    type Args = GG2SetskinArgs;
    type Output = qboolean;

    const IMPORT: GameImport = GameImport::G_G2_SETSKIN;
}

impl EncodeSysCall for GG2Setskin {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(a.ghoul2),
            a.model_index as isize,
            a.custom_skin as isize,
            a.render_skin as isize,
        ])
    }
}

impl DecodeSysCallReturn for GG2Setskin {
    fn decode_return(word: isize) -> Self::Output {
        word as qboolean
    }
}
