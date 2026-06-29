use core::ffi::{c_char, c_int};

use super::super::SpCgameImport;
use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::ffi::types::qboolean;

/// Arguments for `CG_UI_GETMENUINFO`.
///
/// Raven wrapper: `cgi_UI_GetMenuInfo(menuFile,x,y,w,h);`
/// Raven transport: `Menus_FindByName((char *) VMA(1)); if (menu){ *(int *)VMA(2)=...; ...; result=qtrue; } else { result=qfalse; }`
///
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:623-625`
/// Args source: `oracle/oracle/code/cgame/cg_local.h:1222`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:895-917`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgUiGetmenuinfoArgs {
    menu_file: *const c_char,
    x: *mut c_int,
    y: *mut c_int,
    w: *mut c_int,
    h: *mut c_int,
}

impl CgUiGetmenuinfoArgs {
    /// # Safety
    /// `x`, `y`, `w`, and `h` must point to writable `int` outputs.
    pub const unsafe fn new(menu_file: *const c_char, x: *mut c_int, y: *mut c_int, w: *mut c_int, h: *mut c_int) -> Self {
        Self {
            menu_file,
            x,
            y,
            w,
            h,
        }
    }

    pub const fn menu_file(&self) -> *const c_char {
        self.menu_file
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
}

/// `CG_UI_GETMENUINFO` SP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/code/cgame/cg_public.h:205`
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:623-625`
/// Args source: `oracle/oracle/code/cgame/cg_local.h:1222`
/// Output source: `oracle/oracle/code/client/cl_cgame.cpp:895-917`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:895-917`
pub struct CgUiGetmenuinfo;

impl OutboundSysCall for CgUiGetmenuinfo {
    type Import = SpCgameImport;
    type Args = CgUiGetmenuinfoArgs;
    type Output = qboolean;

    const IMPORT: SpCgameImport = SpCgameImport::CG_UI_GETMENUINFO;
}

impl EncodeSysCall for CgUiGetmenuinfo {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(args.menu_file()),
            ptr_to_word(args.x()),
            ptr_to_word(args.y()),
            ptr_to_word(args.w()),
            ptr_to_word(args.h()),
        ])
    }
}

impl DecodeSysCallReturn for CgUiGetmenuinfo {
    fn decode_return(word: isize) -> Self::Output {
        word as qboolean
    }
}
