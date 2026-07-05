//! MP `bg_misc.c` custom siege-order sound name table.

#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]

use core::ffi::CStr;

/// Raven `bg_customSiegeSoundNames[MAX_CUSTOM_SIEGE_SOUNDS]` — custom siege
/// voice-order sound names; scanned by consumers until the `NULL` sentinel
/// (`None`) is hit.
///
/// Definition source: `oracle/oracle/codemp/game/bg_misc.c:113-145`
/// Extern decl source: `oracle/oracle/codemp/game/bg_public.h:143`
pub static bg_customSiegeSoundNames: [Option<&CStr>; 30] = [
    Some(c"*att_attack"),
    Some(c"*att_primary"),
    Some(c"*att_second"),
    Some(c"*def_guns"),
    Some(c"*def_position"),
    Some(c"*def_primary"),
    Some(c"*def_second"),
    Some(c"*reply_coming"),
    Some(c"*reply_go"),
    Some(c"*reply_no"),
    Some(c"*reply_stay"),
    Some(c"*reply_yes"),
    Some(c"*req_assist"),
    Some(c"*req_demo"),
    Some(c"*req_hvy"),
    Some(c"*req_medic"),
    Some(c"*req_sup"),
    Some(c"*req_tech"),
    Some(c"*spot_air"),
    Some(c"*spot_defenses"),
    Some(c"*spot_emplaced"),
    Some(c"*spot_sniper"),
    Some(c"*spot_troops"),
    Some(c"*tac_cover"),
    Some(c"*tac_fallback"),
    Some(c"*tac_follow"),
    Some(c"*tac_hold"),
    Some(c"*tac_split"),
    Some(c"*tac_together"),
    None,
];
