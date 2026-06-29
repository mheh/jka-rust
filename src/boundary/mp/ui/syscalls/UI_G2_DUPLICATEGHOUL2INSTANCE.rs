use core::ffi::c_void;

use crate::ffi::GameImport;

use crate::boundary::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `UiG2_DUPLICATEGHOUL2INSTANCE` outbound game-to-engine syscall.
///
/// Deep-copies a ghoul2 instance (`g2_from`) into the location pointed to by `g2_to`.
/// The engine allocates the new instance and writes the pointer through `g2_to`.
#[derive(Debug)]
pub struct GG2Duplicateghoul2InstanceArgs {
    /// Source ghoul2 instance handle.
    g2_from: *mut c_void,
    /// Pointer to the destination handle slot; engine writes the new instance pointer here.
    g2_to: *mut *mut c_void,
}

impl GG2Duplicateghoul2InstanceArgs {
    pub fn new(g2_from: *mut c_void, g2_to: *mut *mut c_void) -> Self {
        Self { g2_from, g2_to }
    }

    pub fn g2_from(&self) -> *mut c_void {
        self.g2_from
    }

    pub fn g2_to(&self) -> *mut *mut c_void {
        self.g2_to
    }
}

/// `UiG2_DUPLICATEGHOUL2INSTANCE` MP game imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/ui/ui_public.h:525`
pub struct GG2Duplicateghoul2Instance;

impl OutboundSysCall for GG2Duplicateghoul2Instance {
    type Import = GameImport;
    type Args = GG2Duplicateghoul2InstanceArgs;
    type Output = ();

    const IMPORT: GameImport = GameImport::UiG2_DUPLICATEGHOUL2INSTANCE;
}

impl EncodeSysCall for GG2Duplicateghoul2Instance {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(a.g2_from()), ptr_to_word(a.g2_to())])
    }
}

impl DecodeSysCallReturn for GG2Duplicateghoul2Instance {
    fn decode_return(_word: isize) -> Self::Output {}
}
