use core::ffi::{c_char, c_int};

use super::super::MpGameImport;
use mp_qshared::common::mp::gentity_t;

use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `G_ICARUS_RUNSCRIPT` outbound game-to-engine syscall.
///
/// ABI: `( gentity_t *ent, const char *name ) -> int`
#[derive(Debug)]
pub struct GIcarusRunscriptArgs {
    /// Entity to run the script on.
    ent: *mut gentity_t,
    /// Script name (null-terminated C string).
    name: *const c_char,
}

impl GIcarusRunscriptArgs {
    pub fn new(ent: *mut gentity_t, name: *const c_char) -> Self {
        Self { ent, name }
    }

    pub fn ent(&self) -> *mut gentity_t {
        self.ent
    }

    pub fn name(&self) -> *const c_char {
        self.name
    }
}

/// `G_ICARUS_RUNSCRIPT` MP game imports syscall ABI token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:252`
pub struct GIcarusRunscript;

impl OutboundSysCall for GIcarusRunscript {
    type Import = MpGameImport;
    type Args = GIcarusRunscriptArgs;
    type Output = c_int;

    const IMPORT: MpGameImport = MpGameImport::G_ICARUS_RUNSCRIPT;
}

impl EncodeSysCall for GIcarusRunscript {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(a.ent as *const _),
            ptr_to_word(a.name as *const _),
        ])
    }
}

impl DecodeSysCallReturn for GIcarusRunscript {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
