use super::super::MpCgameImport;
use core::ffi::c_void;

use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `CG_GETGAMESTATE`.
///
/// Raven cgame calls `syscall( CG_GETGAMESTATE, gamestate )`; the MP client
/// switch decodes that first argument as `(gameState_t *)VMA(1)`, fills it via
/// `CL_GetGameState`, and returns `0`.
///
/// Args source: `oracle/codemp/cgame/cg_syscalls.c:465-466`
/// Transport/switch source: `oracle/codemp/client/cl_cgame.cpp:957-959`
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

/// `CG_GETGAMESTATE` MP cgame imports syscall ABI token.
///
/// Raven: `( gameState_t *gamestate )`; result is written into the out pointer.
/// Enum value source: `oracle/codemp/cgame/cg_public.h:180`
/// Args source: `oracle/codemp/cgame/cg_syscalls.c:465-466`
/// Output source: `oracle/codemp/client/cl_cgame.cpp:957-959`
/// Transport/switch source: `oracle/codemp/client/cl_cgame.cpp:957-959`
pub struct CgGetgamestate;

impl OutboundSysCall for CgGetgamestate {
    type Import = MpCgameImport;
    type Args = CgGetgamestateArgs;
    type Output = ();

    const IMPORT: MpCgameImport = MpCgameImport::CG_GETGAMESTATE;
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
