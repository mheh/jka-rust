use core::ffi::c_char;

use super::super::SpCgameImport;
use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::shared::qboolean;

/// Arguments for `CG_UI_GETITEMTEXT`.
///
/// Raven wrapper: `cgi_UI_GetItemText(menuFile,itemName,text);`
/// Raven transport: `Q_strncpyz((char *) VMA(3), item->text, 256); return qtrue/qfalse`.
///
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:633-635`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:919-941`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgUiGetitemtextArgs {
    menu_file: *const c_char,
    item_name: *const c_char,
    text: *mut c_char,
}

impl CgUiGetitemtextArgs {
    /// # Safety
    /// `text` must point to a writable `char` buffer at least 256 bytes long.
    pub const unsafe fn new(
        menu_file: *const c_char,
        item_name: *const c_char,
        text: *mut c_char,
    ) -> Self {
        Self {
            menu_file,
            item_name,
            text,
        }
    }

    pub const fn menu_file(&self) -> *const c_char {
        self.menu_file
    }

    pub const fn item_name(&self) -> *const c_char {
        self.item_name
    }

    pub const fn text(&self) -> *mut c_char {
        self.text
    }
}

/// `CG_UI_GETITEMTEXT` SP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/code/cgame/cg_public.h:207`
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:633-635`
/// Output source: `oracle/oracle/code/client/cl_cgame.cpp:919-941`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:919-941`
pub struct CgUiGetitemtext;

impl OutboundSysCall for CgUiGetitemtext {
    type Import = SpCgameImport;
    type Args = CgUiGetitemtextArgs;
    type Output = qboolean;

    const IMPORT: SpCgameImport = SpCgameImport::CG_UI_GETITEMTEXT;
}

impl EncodeSysCall for CgUiGetitemtext {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(args.menu_file()),
            ptr_to_word(args.item_name()),
            ptr_to_word(args.text()),
        ])
    }
}

impl DecodeSysCallReturn for CgUiGetitemtext {
    fn decode_return(word: isize) -> Self::Output {
        word as qboolean
    }
}
