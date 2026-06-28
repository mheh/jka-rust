use core::ffi::c_char;

use crate::ffi::types::qboolean;
use crate::ffi::GameImport;
use crate::boundary::generic::{ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `G_ICARUS_REGISTERSCRIPT` outbound game-to-engine syscall.
///
/// Precaches/validates the script `name`. `b_called_during_interrogate`
/// distinguishes the interrogation precache pass from a normal registration.
#[derive(Debug)]
pub struct GIcarusRegisterscriptArgs {
    name: *const c_char,
    b_called_during_interrogate: qboolean,
}

impl GIcarusRegisterscriptArgs {
    pub fn new(name: *const c_char, b_called_during_interrogate: qboolean) -> Self {
        Self { name, b_called_during_interrogate }
    }

    pub fn name(&self) -> *const c_char {
        self.name
    }

    pub fn b_called_during_interrogate(&self) -> qboolean {
        self.b_called_during_interrogate
    }
}

pub struct GIcarusRegisterscript;

impl OutboundSysCall for GIcarusRegisterscript {
    type Import = GameImport;
    type Args = GIcarusRegisterscriptArgs;
    type Output = qboolean;

    const IMPORT: GameImport = GameImport::G_ICARUS_REGISTERSCRIPT;
}

impl EncodeSysCall for GIcarusRegisterscript {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(a.name),
            a.b_called_during_interrogate as isize,
        ])
    }
}

impl DecodeSysCallReturn for GIcarusRegisterscript {
    fn decode_return(word: isize) -> Self::Output {
        word as qboolean
    }
}
