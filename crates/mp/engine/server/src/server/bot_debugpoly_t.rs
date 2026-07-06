#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_int;

use mp_qshared::shared::vec3_t;

/// Raven `bot_debugpoly_t` (`bot_debugpoly_s`) — a single bot-navigation debug
/// polygon, file-local to `sv_bot.cpp` (`SV_BotAllocDebugPolygon` et al.). The
/// `points[128]` bound is a literal in Raven (no named constant).
///
/// Type definition source: `oracle/oracle/codemp/server/sv_bot.cpp:6-14`
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct bot_debugpoly_t {
    pub inuse: c_int,
    pub color: c_int,
    pub numPoints: c_int,
    pub points: [vec3_t; 128],
}

const _: () = {
    use core::mem::{offset_of, size_of};
    assert!(size_of::<bot_debugpoly_t>() == 1548);
    assert!(offset_of!(bot_debugpoly_t, inuse) == 0);
    assert!(offset_of!(bot_debugpoly_t, color) == 4);
    assert!(offset_of!(bot_debugpoly_t, numPoints) == 8);
    assert!(offset_of!(bot_debugpoly_t, points) == 12);
};

/// C tag `bot_debugpoly_s` is the same type as the `bot_debugpoly_t` typedef.
pub type bot_debugpoly_s = bot_debugpoly_t;
