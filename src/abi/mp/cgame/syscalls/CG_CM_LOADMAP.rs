use core::ffi::c_char;

use super::super::MpCgameImport;
use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::shared::qboolean;

/// Arguments for `CG_CM_LOADMAP`.
///
/// Raven wrapper: `void trap_CM_LoadMap(const char *mapname, qboolean SubBSP)`.
/// Raven transport decodes `mapname` through `VMA(1)` and reads `SubBSP` from
/// `args[2]`. When `SubBSP` is true the client loads a sub BSP with
/// `CM_LoadSubBSP`; otherwise it calls `CL_CM_LoadMap`.
///
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:123-124`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:771-780`
#[derive(Debug)]
pub struct CgCmLoadmapArgs {
    /// Map name C string decoded by Raven as `(const char *)VMA(1)`.
    mapname: *const c_char,
    /// `qboolean` flag read by Raven as the raw `args[2]` syscall word.
    sub_bsp: qboolean,
}

impl CgCmLoadmapArgs {
    /// Construct raw `trap_CM_LoadMap` syscall args.
    ///
    /// # Safety
    /// `mapname` must point to a valid NUL-terminated C string for the duration
    /// of the syscall.
    pub const unsafe fn new(mapname: *const c_char, sub_bsp: qboolean) -> Self {
        Self { mapname, sub_bsp }
    }

    pub const fn mapname(&self) -> *const c_char {
        self.mapname
    }

    pub const fn sub_bsp(&self) -> qboolean {
        self.sub_bsp
    }
}

/// `CG_CM_LOADMAP` MP cgame imports syscall ABI token.
///
/// Raven wrapper: `syscall( CG_CM_LOADMAP, mapname, SubBSP );`
/// Raven transport: if `args[2]`, call `CM_LoadSubBSP` with `VMA(1)`;
/// otherwise call `CL_CM_LoadMap((const char *)VMA(1))`; the switch returns 0.
///
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:83`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:123-124`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:779-780`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:771-780`
pub struct CgCmLoadmap;

impl OutboundSysCall for CgCmLoadmap {
    type Import = MpCgameImport;
    type Args = CgCmLoadmapArgs;
    type Output = ();

    const IMPORT: MpCgameImport = MpCgameImport::CG_CM_LOADMAP;
}

impl EncodeSysCall for CgCmLoadmap {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.mapname()), args.sub_bsp() as isize])
    }
}

impl DecodeSysCallReturn for CgCmLoadmap {
    // `trap_CM_LoadMap` is `void`; Raven's switch returns 0 after loading.
    fn decode_return(_word: isize) -> Self::Output {}
}
