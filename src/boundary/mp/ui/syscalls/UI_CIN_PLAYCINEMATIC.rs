use core::ffi::{c_char, c_int};

use super::super::MpUiImport;
use crate::boundary::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `UI_CIN_PLAYCINEMATIC`.
///
/// Raven: this returns a handle. arg0 is the name in the format "idlogo.roq",
/// set arg1 to NULL, alteredstates to qfalse (do not alter gamestate).
/// Raven wrapper: `syscall(UI_CIN_PLAYCINEMATIC, arg0, xpos, ypos, width, height, bits)`.
/// Raven transport: `CIN_PlayCinematic((const char *)VMA(1), args[2], args[3], args[4], args[5], args[6])`.
///
/// Args source: `oracle/oracle/codemp/ui/ui_syscalls.c:404-406`
/// Args source: `oracle/oracle/codemp/ui/ui_local.h:1002`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_ui.cpp:1183-1185`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UiCinPlaycinematicArgs {
    arg0: *const c_char,
    xpos: c_int,
    ypos: c_int,
    width: c_int,
    height: c_int,
    bits: c_int,
}

impl UiCinPlaycinematicArgs {
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

/// `UI_CIN_PLAYCINEMATIC` MP UI imports syscall boundary token.
///
/// Enum value source: `oracle/oracle/codemp/ui/ui_public.h:105`
/// Enum comment source: `oracle/oracle/codemp/ui/ui_public.h:105-109`
/// Args source: `oracle/oracle/codemp/ui/ui_syscalls.c:404-406`
/// Output source: `oracle/oracle/codemp/ui/ui_local.h:1002`
/// Output source: `oracle/oracle/codemp/client/cl_ui.cpp:1183-1185`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_ui.cpp:1183-1185`
pub struct UiCinPlaycinematic;

impl OutboundSysCall for UiCinPlaycinematic {
    type Import = MpUiImport;
    type Args = UiCinPlaycinematicArgs;
    type Output = c_int;

    const IMPORT: MpUiImport = MpUiImport::UI_CIN_PLAYCINEMATIC;
}

impl EncodeSysCall for UiCinPlaycinematic {
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

impl DecodeSysCallReturn for UiCinPlaycinematic {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
