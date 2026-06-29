use core::ffi::{c_char, c_int, c_uint};

use super::super::SpCgameImport;
use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::ffi::types::qboolean;

/// Arguments for `CG_ANYLANGUAGE_READFROMSTRING`.
///
/// Raven wrapper: `return syscall( CG_ANYLANGUAGE_READFROMSTRING, psText, piAdvanceCount, pbIsTrailingPunctuation );`
/// Raven transport: `return re.AnyLanguage_ReadCharFromString((const char *) VMA(1), (int *) VMA(2), (qboolean *) VMA(3));`
///
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:347-349`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:678-679`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgAnylanguageReadfromstringArgs {
    ps_text: *const c_char,
    pi_advance_count: *mut c_int,
    pb_is_trailing_punctuation: *mut qboolean,
}

impl CgAnylanguageReadfromstringArgs {
    pub const fn new(
        ps_text: *const c_char,
        pi_advance_count: *mut c_int,
        pb_is_trailing_punctuation: *mut qboolean,
    ) -> Self {
        Self {
            ps_text,
            pi_advance_count,
            pb_is_trailing_punctuation,
        }
    }

    pub const fn ps_text(&self) -> *const c_char {
        self.ps_text
    }

    pub const fn pi_advance_count(&self) -> *mut c_int {
        self.pi_advance_count
    }

    pub const fn pb_is_trailing_punctuation(&self) -> *mut qboolean {
        self.pb_is_trailing_punctuation
    }
}

/// `CG_ANYLANGUAGE_READFROMSTRING` SP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/code/cgame/cg_public.h:129`
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:347-349`
/// Output source: `oracle/oracle/code/cgame/cg_syscalls.cpp:347-349`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:678-679`
pub struct CgAnylanguageReadfromstring;

impl OutboundSysCall for CgAnylanguageReadfromstring {
    type Import = SpCgameImport;
    type Args = CgAnylanguageReadfromstringArgs;
    type Output = c_uint;

    const IMPORT: SpCgameImport = SpCgameImport::CG_ANYLANGUAGE_READFROMSTRING;
}

impl EncodeSysCall for CgAnylanguageReadfromstring {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(args.ps_text()),
            ptr_to_word(args.pi_advance_count()),
            ptr_to_word(args.pb_is_trailing_punctuation()),
        ])
    }
}

impl DecodeSysCallReturn for CgAnylanguageReadfromstring {
    fn decode_return(word: isize) -> Self::Output {
        word as c_uint
    }
}
