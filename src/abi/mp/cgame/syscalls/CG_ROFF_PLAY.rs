use core::ffi::c_int;

use super::super::MpCgameImport;
use crate::abi::generic::{
    DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::ffi::types::qboolean;

/// Arguments for `CG_ROFF_PLAY`.
///
/// Raven wrapper: `return syscall( CG_ROFF_PLAY, entID, roffID, doTranslation );`
/// Raven transport:
/// `return theROFFSystem.Play(args[1], args[2], (qboolean)args[3], qtrue );`
///
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:745-747`
/// Args source: `oracle/oracle/codemp/cgame/cg_local.h:2433`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1278-1279`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgRoffPlayArgs {
    ent_id: c_int,
    roff_id: c_int,
    do_translation: qboolean,
}

impl CgRoffPlayArgs {
    pub const fn new(ent_id: c_int, roff_id: c_int, do_translation: qboolean) -> Self {
        Self {
            ent_id,
            roff_id,
            do_translation,
        }
    }
}

/// `CG_ROFF_PLAY` MP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:245`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:745-747`
/// Output source: `oracle/oracle/codemp/cgame/cg_syscalls.c:745-747`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:1278-1279`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1278-1279`
pub struct CgRoffPlay;

impl OutboundSysCall for CgRoffPlay {
    type Import = MpCgameImport;
    type Args = CgRoffPlayArgs;
    type Output = qboolean;

    const IMPORT: MpCgameImport = MpCgameImport::CG_ROFF_PLAY;
}

impl EncodeSysCall for CgRoffPlay {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            args.ent_id as isize,
            args.roff_id as isize,
            args.do_translation as isize,
        ])
    }
}

impl DecodeSysCallReturn for CgRoffPlay {
    fn decode_return(word: isize) -> Self::Output {
        word as qboolean
    }
}
