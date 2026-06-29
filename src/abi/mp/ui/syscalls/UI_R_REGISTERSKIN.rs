use std::ffi::CString;

use crate::codemp::game::q_shared_h::qhandle_t;
use crate::ffi::GameImport;

use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `UiR_REGISTERSKIN` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct GRRegisterskinArgs {
    name: CString,
}

impl GRRegisterskinArgs {
    pub fn new(name: CString) -> Self {
        Self { name }
    }

    pub fn name(&self) -> &CString {
        &self.name
    }
}

/// `UiR_REGISTERSKIN` MP game imports syscall ABI token.
///
/// Source: `oracle/oracle/codemp/ui/ui_public.h:506`
pub struct GRRegisterskin;

impl OutboundSysCall for GRRegisterskin {
    type Import = GameImport;
    type Args = GRRegisterskinArgs;
    type Output = qhandle_t;

    const IMPORT: GameImport = GameImport::UiR_REGISTERSKIN;
}

impl EncodeSysCall for GRRegisterskin {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(a.name.as_ptr())])
    }
}

impl DecodeSysCallReturn for GRRegisterskin {
    fn decode_return(word: isize) -> Self::Output {
        word as qhandle_t
    }
}
