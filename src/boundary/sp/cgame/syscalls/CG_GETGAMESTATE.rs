use core::ffi::c_void;

use super::super::SpCgameImport;
use crate::boundary::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `CG_GETGAMESTATE`.
///
/// Raven wrapper: `void trap_GetGameState(gameState_t *gamestate)`.
/// The SP client switch decodes that pointer as `(gameState_t *)VMA(1)`,
/// calls `CL_GetGameState`, and returns `0`.
///
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:446-447`
/// Output source: `oracle/oracle/code/client/cl_cgame.cpp:752-754`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:752-754`
#[derive(Debug)]
pub struct CgGetgamestateArgs {
    gamestate: *mut c_void,
}

impl CgGetgamestateArgs {
    /// Construct raw `trap_GetGameState` syscall args.
    ///
    /// # Safety
    /// `gamestate` must point to a writable `gameState_t` for the duration of
    /// the syscall.
    pub const unsafe fn new(gamestate: *mut c_void) -> Self {
        Self { gamestate }
    }

    pub const fn gamestate(&self) -> *mut c_void {
        self.gamestate
    }
}

/// `CG_GETGAMESTATE` SP cgame imports syscall boundary token.
///
/// Enum value source: `oracle/oracle/code/cgame/cg_public.h:151`
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:446-447`
/// Output source: `oracle/oracle/code/client/cl_cgame.cpp:752-754`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:752-754`
pub struct CgGetgamestate;

impl OutboundSysCall for CgGetgamestate {
    type Import = SpCgameImport;
    type Args = CgGetgamestateArgs;
    type Output = ();

    const IMPORT: SpCgameImport = SpCgameImport::CG_GETGAMESTATE;
}

impl EncodeSysCall for CgGetgamestate {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.gamestate() as *const c_void)])
    }
}

impl DecodeSysCallReturn for CgGetgamestate {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
