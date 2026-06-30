use core::ffi::{c_char, c_int};

use super::super::SpUiImport;
use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `UI_CIN_PLAYCINEMATIC` SP UI imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/code/ui/ui_public.h:227`
/// Args source: `oracle/oracle/code/client/cl_ui.cpp:466-468`
/// Args source: `oracle/oracle/code/client/client.h:430`
/// Output source: `oracle/oracle/code/client/cl_ui.cpp:468`
/// Transport/switch source: `oracle/oracle/code/client/cl_ui.cpp:466-468`
pub struct UiCinPlaycinematic;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UiCinPlaycinematicArgs {
    arg0: *const c_char,
    xpos: c_int,
    ypos: c_int,
    width: c_int,
    height: c_int,
    bits: c_int,
    ps_audio_file: *const c_char,
}

impl UiCinPlaycinematicArgs {
    pub const fn new(
        arg0: *const c_char,
        xpos: c_int,
        ypos: c_int,
        width: c_int,
        height: c_int,
        bits: c_int,
        ps_audio_file: *const c_char,
    ) -> Self {
        Self {
            arg0,
            xpos,
            ypos,
            width,
            height,
            bits,
            ps_audio_file,
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

    pub const fn ps_audio_file(&self) -> *const c_char {
        self.ps_audio_file
    }
}

impl OutboundSysCall for UiCinPlaycinematic {
    type Import = SpUiImport;
    type Args = UiCinPlaycinematicArgs;
    type Output = c_int;

    const IMPORT: SpUiImport = SpUiImport::UI_CIN_PLAYCINEMATIC;
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
            ptr_to_word(args.ps_audio_file()),
        ])
    }
}

impl DecodeSysCallReturn for UiCinPlaycinematic {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
