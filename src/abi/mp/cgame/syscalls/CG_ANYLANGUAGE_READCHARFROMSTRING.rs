use core::ffi::{c_char, c_int, c_uint};

use super::super::MpCgameImport;
use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::ffi::types::qboolean;

/// Arguments for `CG_ANYLANGUAGE_READCHARFROMSTRING`.
///
/// Raven wrapper: `return syscall( CG_ANYLANGUAGE_READCHARFROMSTRING, psText, piAdvanceCount, pbIsTrailingPunctuation);`
/// Raven transport: `return re.AnyLanguage_ReadCharFromString( (const char *) VMA(1), (int *) VMA(2), (qboolean *) VMA(3) );`
///
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:317-319`
/// Args source: `oracle/oracle/codemp/cgame/cg_local.h:2260`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:886-887`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgAnylanguageReadcharfromstringArgs {
    ps_text: *const c_char,
    pi_advance_count: *mut c_int,
    pb_is_trailing_punctuation: *mut qboolean,
}

impl CgAnylanguageReadcharfromstringArgs {
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
}

/// `CG_ANYLANGUAGE_READCHARFROMSTRING` MP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:128`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:317-319`
/// Output source: `oracle/oracle/codemp/cgame/cg_local.h:2260`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:886-887`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:886-887`
pub struct CgAnylanguageReadcharfromstring;

impl OutboundSysCall for CgAnylanguageReadcharfromstring {
    type Import = MpCgameImport;
    type Args = CgAnylanguageReadcharfromstringArgs;
    type Output = c_uint;

    const IMPORT: MpCgameImport = MpCgameImport::CG_ANYLANGUAGE_READCHARFROMSTRING;
}

impl EncodeSysCall for CgAnylanguageReadcharfromstring {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(args.ps_text),
            ptr_to_word(args.pi_advance_count),
            ptr_to_word(args.pb_is_trailing_punctuation),
        ])
    }
}

impl DecodeSysCallReturn for CgAnylanguageReadcharfromstring {
    fn decode_return(word: isize) -> Self::Output {
        word as c_uint
    }
}
