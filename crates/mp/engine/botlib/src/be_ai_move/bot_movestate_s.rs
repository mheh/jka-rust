#![allow(non_camel_case_types, non_snake_case)]

use mp_qshared::common::mp::botlib::bot_avoidspot_s::bot_avoidspot_t;
use mp_qshared::shared::vec3_t;

/// `MAX_AVOIDREACH`.
///
/// Source: `oracle/codemp/botlib/be_ai_move.cpp` (avoid-reach ring size).
pub const MAX_AVOIDREACH: usize = 1;

/// `MAX_AVOIDSPOTS`.
///
/// Source: `oracle/codemp/botlib/be_ai_move.cpp` (avoid-spot ring size).
pub const MAX_AVOIDSPOTS: usize = 32;

/// Raven `bot_movestate_t` — the movement state of a single bot.
///
/// Type definition source: `oracle/codemp/botlib/be_ai_move.cpp:42-71`
#[repr(C)]
pub struct bot_movestate_t {
    /// origin of the bot
    pub origin: vec3_t,
    /// velocity of the bot
    pub velocity: vec3_t,
    /// view offset
    pub viewoffset: vec3_t,
    /// entity number of the bot
    pub entitynum: i32,
    /// client number of the bot
    pub client: i32,
    /// time the bot thinks
    pub thinktime: f32,
    /// presencetype of the bot
    pub presencetype: i32,
    /// view angles of the bot
    pub viewangles: vec3_t,
    /// area the bot is in
    pub areanum: i32,
    /// last area the bot was in
    pub lastareanum: i32,
    /// last goal area number
    pub lastgoalareanum: i32,
    /// last reachability number
    pub lastreachnum: i32,
    /// origin previous cycle
    pub lastorigin: vec3_t,
    /// area number of the reachabilty
    pub reachareanum: i32,
    /// movement flags
    pub moveflags: i32,
    /// set when jumped
    pub jumpreach: i32,
    /// last time the grapple was visible
    pub grapplevisible_time: f32,
    /// last distance to the grapple end
    pub lastgrappledist: f32,
    /// time to use current reachability
    pub reachability_time: f32,
    /// reachabilities to avoid
    pub avoidreach: [i32; MAX_AVOIDREACH],
    /// times to avoid the reachabilities
    pub avoidreachtimes: [f32; MAX_AVOIDREACH],
    /// number of tries before avoiding
    pub avoidreachtries: [i32; MAX_AVOIDREACH],
    /// spots to avoid
    pub avoidspots: [bot_avoidspot_t; MAX_AVOIDSPOTS],
    pub numavoidspots: i32,
}

pub type bot_movestate_s = bot_movestate_t;

const _: () = assert!(core::mem::size_of::<bot_movestate_t>() == 772);
const _: () = assert!(core::mem::offset_of!(bot_movestate_t, origin) == 0);
const _: () = assert!(core::mem::offset_of!(bot_movestate_t, velocity) == 12);
const _: () = assert!(core::mem::offset_of!(bot_movestate_t, viewoffset) == 24);
const _: () = assert!(core::mem::offset_of!(bot_movestate_t, entitynum) == 36);
const _: () = assert!(core::mem::offset_of!(bot_movestate_t, client) == 40);
const _: () = assert!(core::mem::offset_of!(bot_movestate_t, thinktime) == 44);
const _: () = assert!(core::mem::offset_of!(bot_movestate_t, presencetype) == 48);
const _: () = assert!(core::mem::offset_of!(bot_movestate_t, viewangles) == 52);
const _: () = assert!(core::mem::offset_of!(bot_movestate_t, areanum) == 64);
const _: () = assert!(core::mem::offset_of!(bot_movestate_t, lastareanum) == 68);
const _: () = assert!(core::mem::offset_of!(bot_movestate_t, lastgoalareanum) == 72);
const _: () = assert!(core::mem::offset_of!(bot_movestate_t, lastreachnum) == 76);
const _: () = assert!(core::mem::offset_of!(bot_movestate_t, lastorigin) == 80);
const _: () = assert!(core::mem::offset_of!(bot_movestate_t, reachareanum) == 92);
const _: () = assert!(core::mem::offset_of!(bot_movestate_t, moveflags) == 96);
const _: () = assert!(core::mem::offset_of!(bot_movestate_t, jumpreach) == 100);
const _: () = assert!(core::mem::offset_of!(bot_movestate_t, grapplevisible_time) == 104);
const _: () = assert!(core::mem::offset_of!(bot_movestate_t, lastgrappledist) == 108);
const _: () = assert!(core::mem::offset_of!(bot_movestate_t, reachability_time) == 112);
const _: () = assert!(core::mem::offset_of!(bot_movestate_t, avoidreach) == 116);
const _: () = assert!(core::mem::offset_of!(bot_movestate_t, avoidreachtimes) == 120);
const _: () = assert!(core::mem::offset_of!(bot_movestate_t, avoidreachtries) == 124);
const _: () = assert!(core::mem::offset_of!(bot_movestate_t, avoidspots) == 128);
const _: () = assert!(core::mem::offset_of!(bot_movestate_t, numavoidspots) == 768);
