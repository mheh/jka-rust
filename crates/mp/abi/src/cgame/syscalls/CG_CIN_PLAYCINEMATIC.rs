use core::ffi::{c_char, c_int};

use super::super::MpCgameImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `CG_CIN_PLAYCINEMATIC`.
///
/// Raven: this returns a handle. arg0 is the name in the format "idlogo.roq",
/// set arg1 to NULL, alteredstates to qfalse (do not alter gamestate).
/// Raven wrapper: `syscall(CG_CIN_PLAYCINEMATIC, arg0, xpos, ypos, width, height, bits)`.
/// Raven transport:
/// `CIN_PlayCinematic((const char *)VMA(1), args[2], args[3], args[4], args[5], args[6])`.
///
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:584-585`
/// Args source: `oracle/oracle/codemp/cgame/cg_local.h:2381`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1025-1026`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgCinPlaycinematicArgs {
    arg0: *const c_char,
    xpos: c_int,
    ypos: c_int,
    width: c_int,
    height: c_int,
    bits: c_int,
}

impl CgCinPlaycinematicArgs {
    pub const fn new(
        arg0: *const c_char,
        xpos: c_int,
        ypos: c_int,
        width: c_int,
        height: c_int,
        bits: c_int,
    ) -> Self {
        Self {
            arg0,
            xpos,
            ypos,
            width,
            height,
            bits,
        }
    }

    pub const fn arg0(&self) -> *const c_char {
        self.arg0
    }

    pub const fn xpos(&self) -> c_int {
        self.xpos
    }

    pub const fn ypos(&self) -> c_int {
        self.ypos
    }

    pub const fn width(&self) -> c_int {
        self.width
    }

    pub const fn height(&self) -> c_int {
        self.height
    }

    pub const fn bits(&self) -> c_int {
        self.bits
    }
}

/// `CG_CIN_PLAYCINEMATIC` MP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:210`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:584-585`
/// Output source: `oracle/oracle/codemp/cgame/cg_local.h:2381`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:1025-1026`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1025-1026`
pub struct CgCinPlaycinematic;

impl OutboundSysCall for CgCinPlaycinematic {
    type Import = MpCgameImport;
    type Args = CgCinPlaycinematicArgs;
    type Output = c_int;

    const IMPORT: MpCgameImport = MpCgameImport::CG_CIN_PLAYCINEMATIC;
}

impl EncodeSysCall for CgCinPlaycinematic {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(args.arg0()),
            args.xpos() as isize,
            args.ypos() as isize,
            args.width() as isize,
            args.height() as isize,
            args.bits() as isize,
        ])
    }
}

impl DecodeSysCallReturn for CgCinPlaycinematic {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
