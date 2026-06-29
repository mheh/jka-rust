use core::ffi::{c_int, c_void};

use crate::boundary::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::ffi::GameImport;

/// `BOTLIB_AAS_PREDICT_CLIENT_MOVEMENT` outbound game-to-engine syscall.
///
/// C ABI:
/// ```c
/// int trap_AAS_PredictClientMovement(
///     void *move, int entnum, vec3_t origin, int presencetype, int onground,
///     vec3_t velocity, vec3_t cmdmove, int cmdframes, int maxframes,
///     float frametime, int stopevent, int stopareanum, int visualize);
/// ```
#[derive(Debug)]
pub struct BotlibAasPredictClientMovementArgs {
    /// Out-param: engine writes `aas_clientmove_s` through this pointer.
    pub move_result: *mut c_void,
    pub entnum: c_int,
    /// Pointer to a 3-element float array (vec3_t).
    pub origin: *const f32,
    pub presencetype: c_int,
    pub onground: c_int,
    /// Pointer to a 3-element float array (vec3_t).
    pub velocity: *const f32,
    /// Pointer to a 3-element float array (vec3_t).
    pub cmdmove: *const f32,
    pub cmdframes: c_int,
    pub maxframes: c_int,
    /// Passed via PASSFLOAT in the C wrapper.
    pub frametime: f32,
    pub stopevent: c_int,
    pub stopareanum: c_int,
    pub visualize: c_int,
}

impl BotlibAasPredictClientMovementArgs {
    pub fn new(
        move_result: *mut c_void,
        entnum: c_int,
        origin: *const f32,
        presencetype: c_int,
        onground: c_int,
        velocity: *const f32,
        cmdmove: *const f32,
        cmdframes: c_int,
        maxframes: c_int,
        frametime: f32,
        stopevent: c_int,
        stopareanum: c_int,
        visualize: c_int,
    ) -> Self {
        Self {
            move_result,
            entnum,
            origin,
            presencetype,
            onground,
            velocity,
            cmdmove,
            cmdframes,
            maxframes,
            frametime,
            stopevent,
            stopareanum,
            visualize,
        }
    }

    pub fn move_result(&self) -> *mut c_void {
        self.move_result
    }
    pub fn entnum(&self) -> c_int {
        self.entnum
    }
    pub fn origin(&self) -> *const f32 {
        self.origin
    }
    pub fn presencetype(&self) -> c_int {
        self.presencetype
    }
    pub fn onground(&self) -> c_int {
        self.onground
    }
    pub fn velocity(&self) -> *const f32 {
        self.velocity
    }
    pub fn cmdmove(&self) -> *const f32 {
        self.cmdmove
    }
    pub fn cmdframes(&self) -> c_int {
        self.cmdframes
    }
    pub fn maxframes(&self) -> c_int {
        self.maxframes
    }
    pub fn frametime(&self) -> f32 {
        self.frametime
    }
    pub fn stopevent(&self) -> c_int {
        self.stopevent
    }
    pub fn stopareanum(&self) -> c_int {
        self.stopareanum
    }
    pub fn visualize(&self) -> c_int {
        self.visualize
    }
}

/// `BOTLIB_AAS_PREDICT_CLIENT_MOVEMENT` MP game imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:380`
pub struct BotlibAasPredictClientMovement;

impl OutboundSysCall for BotlibAasPredictClientMovement {
    type Import = GameImport;
    type Args = BotlibAasPredictClientMovementArgs;
    type Output = c_int;

    const IMPORT: GameImport = GameImport::BOTLIB_AAS_PREDICT_CLIENT_MOVEMENT;
}

impl EncodeSysCall for BotlibAasPredictClientMovement {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(a.move_result as *const _),
            a.entnum as isize,
            ptr_to_word(a.origin as *const _),
            a.presencetype as isize,
            a.onground as isize,
            ptr_to_word(a.velocity as *const _),
            ptr_to_word(a.cmdmove as *const _),
            a.cmdframes as isize,
            a.maxframes as isize,
            crate::ffi::syscalls::pass_float(a.frametime),
            a.stopevent as isize,
            a.stopareanum as isize,
            a.visualize as isize,
        ])
    }
}

impl DecodeSysCallReturn for BotlibAasPredictClientMovement {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
