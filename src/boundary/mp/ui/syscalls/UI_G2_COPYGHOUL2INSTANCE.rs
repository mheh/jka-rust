use core::ffi::c_int;
use core::ffi::c_void;

use crate::ffi::GameImport;

use crate::boundary::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `UiG2_COPYGHOUL2INSTANCE` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct GG2Copyghoul2InstanceArgs {
    /// Source ghoul2 instance.
    pub g2_from: *mut c_void,
    /// Destination ghoul2 instance.
    pub g2_to: *mut c_void,
    /// Model index within the source instance to copy.
    pub model_index: c_int,
}

impl GG2Copyghoul2InstanceArgs {
    pub fn new(g2_from: *mut c_void, g2_to: *mut c_void, model_index: c_int) -> Self {
        Self {
            g2_from,
            g2_to,
            model_index,
        }
    }

    pub fn g2_from(&self) -> *mut c_void {
        self.g2_from
    }
    pub fn g2_to(&self) -> *mut c_void {
        self.g2_to
    }
    pub fn model_index(&self) -> c_int {
        self.model_index
    }
}

/// `UiG2_COPYGHOUL2INSTANCE` MP game imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/ui/ui_public.h:523`
pub struct GG2Copyghoul2Instance;

impl OutboundSysCall for GG2Copyghoul2Instance {
    type Import = GameImport;
    type Args = GG2Copyghoul2InstanceArgs;
    type Output = c_int;

    const IMPORT: GameImport = GameImport::UiG2_COPYGHOUL2INSTANCE;
}

impl EncodeSysCall for GG2Copyghoul2Instance {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(a.g2_from),
            ptr_to_word(a.g2_to),
            a.model_index as isize,
        ])
    }
}

impl DecodeSysCallReturn for GG2Copyghoul2Instance {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
