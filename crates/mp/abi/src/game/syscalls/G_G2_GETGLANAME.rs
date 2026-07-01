use core::ffi::{c_char, c_int, c_void};

use super::super::MpGameImport;

use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `G_G2_GETGLANAME` outbound game-to-engine syscall.
///
/// Copies the GLA (animation file) name for the given model index into the
/// caller-supplied buffer.  The engine writes through `fill_buf`; the call
/// returns nothing.
///
/// C ABI: `void trap_G2API_GetGLAName(void *ghoul2, int modelIndex, char *fillBuf)`
#[derive(Debug)]
pub struct GG2GetglanameArgs {
    /// Ghoul2 instance handle (opaque engine pointer).
    pub ghoul2: *mut c_void,
    /// Index of the model within the ghoul2 instance.
    pub model_index: c_int,
    /// Caller-allocated buffer; the engine writes the GLA name into it.
    pub fill_buf: *mut c_char,
}

impl GG2GetglanameArgs {
    pub fn new(ghoul2: *mut c_void, model_index: c_int, fill_buf: *mut c_char) -> Self {
        Self {
            ghoul2,
            model_index,
            fill_buf,
        }
    }

    pub fn ghoul2(&self) -> *mut c_void {
        self.ghoul2
    }
    pub fn model_index(&self) -> c_int {
        self.model_index
    }
    pub fn fill_buf(&self) -> *mut c_char {
        self.fill_buf
    }
}

/// `G_G2_GETGLANAME` MP game imports syscall ABI token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:522`
pub struct GG2Getglaname;

impl OutboundSysCall for GG2Getglaname {
    type Import = MpGameImport;
    type Args = GG2GetglanameArgs;
    type Output = ();

    const IMPORT: MpGameImport = MpGameImport::G_G2_GETGLANAME;
}

impl EncodeSysCall for GG2Getglaname {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(a.ghoul2),
            a.model_index as isize,
            ptr_to_word(a.fill_buf),
        ])
    }
}

impl DecodeSysCallReturn for GG2Getglaname {
    fn decode_return(_word: isize) -> Self::Output {}
}
