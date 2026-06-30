use core::ffi::{c_char, c_int};

use super::super::SpCgameImport;
use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::codemp::game::q_shared_h::vec4_t;
use crate::shared::qboolean;
use crate::shared::qhandle_t;

/// Arguments for `CG_UI_GETITEMINFO`.
///
/// Raven wrapper: `return(int) syscall(CG_UI_GETITEMINFO,menuFile,itemName,x,y,w,h,color,background);`
/// Raven transport: `Menus_FindByName((char *) VMA(1)); ... (*color)[0]...; *background = item->window.background;`
///
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:628-630`
/// Args source: `oracle/oracle/code/cgame/cg_local.h:1221`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:895-917`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgUiGetiteminfoArgs {
    menu_file: *const c_char,
    item_name: *const c_char,
    x: *mut c_int,
    y: *mut c_int,
    w: *mut c_int,
    h: *mut c_int,
    color: *mut vec4_t,
    background: *mut qhandle_t,
}

impl CgUiGetiteminfoArgs {
    /// `x`, `y`, `w`, `h`, `color`, and `background` should all point to writable buffers.
    pub const unsafe fn new(
        menu_file: *const c_char,
        item_name: *const c_char,
        x: *mut c_int,
        y: *mut c_int,
        w: *mut c_int,
        h: *mut c_int,
        color: *mut vec4_t,
        background: *mut qhandle_t,
    ) -> Self {
        Self {
            menu_file,
            item_name,
            x,
            y,
            w,
            h,
            color,
            background,
        }
    }

    pub const fn menu_file(&self) -> *const c_char {
        self.menu_file
    }

    pub const fn item_name(&self) -> *const c_char {
        self.item_name
    }

    pub const fn x(&self) -> *mut c_int {
        self.x
    }

    pub const fn y(&self) -> *mut c_int {
        self.y
    }

    pub const fn w(&self) -> *mut c_int {
        self.w
    }

    pub const fn h(&self) -> *mut c_int {
        self.h
    }

    pub const fn color(&self) -> *mut vec4_t {
        self.color
    }

    pub const fn background(&self) -> *mut qhandle_t {
        self.background
    }
}

/// `CG_UI_GETITEMINFO` SP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/code/cgame/cg_public.h:208`
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:628-630`
/// Args source: `oracle/oracle/code/cgame/cg_local.h:1221`
/// Output source: `oracle/oracle/code/client/cl_cgame.cpp:895-917`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:895-917`
pub struct CgUiGetiteminfo;

impl OutboundSysCall for CgUiGetiteminfo {
    type Import = SpCgameImport;
    type Args = CgUiGetiteminfoArgs;
    type Output = qboolean;

    const IMPORT: SpCgameImport = SpCgameImport::CG_UI_GETITEMINFO;
}

impl EncodeSysCall for CgUiGetiteminfo {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(args.menu_file()),
            ptr_to_word(args.item_name()),
            ptr_to_word(args.x()),
            ptr_to_word(args.y()),
            ptr_to_word(args.w()),
            ptr_to_word(args.h()),
            ptr_to_word(args.color()),
            ptr_to_word(args.background()),
        ])
    }
}

impl DecodeSysCallReturn for CgUiGetiteminfo {
    fn decode_return(word: isize) -> Self::Output {
        word as qboolean
    }
}
