use core::ffi::c_int;

use super::super::MpCgameImport;
use crate::abi::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// Arguments for `CG_OPENUIMENU`.
///
/// Raven wrapper: `syscall( CG_OPENUIMENU, menuID );`
/// Raven transport: `VM_Call( uivm, UI_SET_ACTIVE_MENU, args[1] ); return 0;`
///
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:508-510`
/// Args source: `oracle/oracle/codemp/cgame/cg_local.h:2354`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:983-985`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgOpenuimenuArgs {
    menu_id: c_int,
}

impl CgOpenuimenuArgs {
    pub const fn new(menu_id: c_int) -> Self {
        Self { menu_id }
    }
}

/// `CG_OPENUIMENU` MP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:190`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:508-510`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:983-985`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:983-985`
pub struct CgOpenuimenu;

impl OutboundSysCall for CgOpenuimenu {
    type Import = MpCgameImport;
    type Args = CgOpenuimenuArgs;
    type Output = ();

    const IMPORT: MpCgameImport = MpCgameImport::CG_OPENUIMENU;
}

impl EncodeSysCall for CgOpenuimenu {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([args.menu_id as isize])
    }
}

impl DecodeSysCallReturn for CgOpenuimenu {
    fn decode_return(_word: isize) -> Self::Output {}
}
