use core::ffi::{c_int, c_void};

use super::super::MpGameImport;

use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `BOTLIB_AAS_PREDICT_ROUTE` outbound game-to-engine syscall.
///
/// C signature:
/// ```c
/// int trap_AAS_PredictRoute(void *route, int areanum, vec3_t origin,
///     int goalareanum, int travelflags, int maxareas, int maxtime,
///     int stopevent, int stopcontents, int stoptfl, int stopareanum);
/// ```
#[derive(Debug)]
pub struct BotlibAasPredictRouteArgs {
    /// Out-param: engine writes the predicted route into this struct.
    pub route: *mut c_void,
    pub areanum: c_int,
    /// Passed as a pointer into the syscall ABI (vec3_t = [f32; 3]).
    pub origin: *const [f32; 3],
    pub goalareanum: c_int,
    pub travelflags: c_int,
    pub maxareas: c_int,
    pub maxtime: c_int,
    pub stopevent: c_int,
    pub stopcontents: c_int,
    pub stoptfl: c_int,
    pub stopareanum: c_int,
}

impl BotlibAasPredictRouteArgs {
    pub fn new(
        route: *mut c_void,
        areanum: c_int,
        origin: *const [f32; 3],
        goalareanum: c_int,
        travelflags: c_int,
        maxareas: c_int,
        maxtime: c_int,
        stopevent: c_int,
        stopcontents: c_int,
        stoptfl: c_int,
        stopareanum: c_int,
    ) -> Self {
        Self {
            route,
            areanum,
            origin,
            goalareanum,
            travelflags,
            maxareas,
            maxtime,
            stopevent,
            stopcontents,
            stoptfl,
            stopareanum,
        }
    }

    pub fn route(&self) -> *mut c_void {
        self.route
    }
    pub fn areanum(&self) -> c_int {
        self.areanum
    }
    pub fn origin(&self) -> *const [f32; 3] {
        self.origin
    }
    pub fn goalareanum(&self) -> c_int {
        self.goalareanum
    }
    pub fn travelflags(&self) -> c_int {
        self.travelflags
    }
    pub fn maxareas(&self) -> c_int {
        self.maxareas
    }
    pub fn maxtime(&self) -> c_int {
        self.maxtime
    }
    pub fn stopevent(&self) -> c_int {
        self.stopevent
    }
    pub fn stopcontents(&self) -> c_int {
        self.stopcontents
    }
    pub fn stoptfl(&self) -> c_int {
        self.stoptfl
    }
    pub fn stopareanum(&self) -> c_int {
        self.stopareanum
    }
}

/// `BOTLIB_AAS_PREDICT_ROUTE` MP game imports syscall ABI token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:495`
pub struct BotlibAasPredictRoute;

impl OutboundSysCall for BotlibAasPredictRoute {
    type Import = MpGameImport;
    type Args = BotlibAasPredictRouteArgs;
    type Output = c_int;

    const IMPORT: MpGameImport = MpGameImport::BOTLIB_AAS_PREDICT_ROUTE;
}

impl EncodeSysCall for BotlibAasPredictRoute {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(a.route as *const _),
            a.areanum as isize,
            ptr_to_word(a.origin as *const _),
            a.goalareanum as isize,
            a.travelflags as isize,
            a.maxareas as isize,
            a.maxtime as isize,
            a.stopevent as isize,
            a.stopcontents as isize,
            a.stoptfl as isize,
            a.stopareanum as isize,
        ])
    }
}

impl DecodeSysCallReturn for BotlibAasPredictRoute {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
