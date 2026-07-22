// PORT-COMPLETE: g_spawn.c
//! FAITHFUL port of `oracle/codemp/game/g_spawn.c` — entity spawn
//! dispatch, `level.spawnVars[]` parsing, and `worldspawn`.
//!
//! Filled by the jampgame mega-pass (settled fork rulings,
//! `docs/handoffs/jampgame-fork-discovery.md`).
//!
//! SPINE (per the settled fork rulings + `docs/architecture/engine-seam.md`,
//! precedent `g_client.rs`/`w_force.rs`): logic fns that reach `level`/cvars/
//! `g_entities`/traps thread the `GameContext<'_>` receiver (`.world: *mut
//! GameWorld`, `.engine`) as an ADDED first parameter (not present on the
//! staged raw-pointer skeleton). Globals are `GameWorld` fields:
//! `level` -> `ctx.world.level`, cvars -> `ctx.world.cvars`,
//! `g_entities[i]` -> `ctx.world.g_entities[i]`. Traps go through
//! `trap::X(ctx.engine, <Name>Args::new(...))`. Cross-file callees are invoked
//! with the packet's resolved raw-pointer signatures verbatim.
//!
//! `fields[]` (`BG_field_t[]`, g_spawn.c:54-149) is this file's own file-scope
//! table (not out-of-file) — transcribed below as `FIELDS`, offsets taken via
//! `offset_of!(gentity_t, ...)` against the already-ported, offset-asserted
//! `gentity_t` (mirrors the `g_client.rs`/`g_mover.rs`/`g_team.rs` precedent of
//! `offset_of!(gentity_t, targetname)` etc for `G_Find`).
//!
//! `GT_*`/`TEAM_*`/`BSET_SPAWN` constant spellings the packet does not
//! enumerate are transcribed as local consts by their faithful Raven values
//! (same convention as `g_team.rs`'s local `TEAM_RED`/`TEAM_BLUE`), resolved
//! at integration.
#![allow(non_snake_case, unused, clippy::all)]

use crate::q_shared::Q_strlen as strlen;
use std::ffi::CStr;

use crate::prelude::*;
use crate::ent_fn_enums::{dispatch_spawn, spawn_for_classname};
use crate::trap;
use crate::world::GameContext;
use native_string::atof_bytes;
use native_string::atoi_bytes;

use crate::g_items::G_SpawnItem;
use crate::g_main::{G_Error, G_Printf};
use crate::g_mem::G_Alloc;
use crate::g_misc::{SP_info_notnull, SP_info_null};
use crate::g_utils::{G_FreeEntity, G_SetOrigin, G_SoundIndex, G_SoundSetIndex, G_Spawn};
use crate::NPC_utils::G_ActivateBehavior;
use native_string::{Q_stricmp, Q_strncmp};
use mp_bg::bg_misc::{BG_FindItem, BG_ParseField};
use mp_bg::bg_panimate::BG_ParseAnimationFile;

use crate::ent_fn_enums::EntThink;
use mp_abi::game::syscalls::G_ICARUS_INITENT::GIcarusInitentArgs;
use mp_abi::game::syscalls::G_ICARUS_VALIDENT::GIcarusValidentArgs;

// Missing trap Args types - will be resolved by integration
use mp_abi::game::syscalls::G_G2_SETBOLTINFO::GG2SetboltinfoArgs as GG2SetBoltInfoArgs;
use mp_abi::game::syscalls::G_G2_SETSKIN::GG2SetskinArgs as GG2SetSkinArgs;
use mp_abi::game::syscalls::G_SET_SERVER_CULL::GSetServerCullArgs;

use mp_bg::public::bg_field::SpawnStringSetter;
use mp_bg::public::fieldtype::fieldtype_t;
use crate::q_shared;

// `TEAM_RED`/`TEAM_BLUE` (`bg_public.h`) — local consts, same convention as
// `g_team.rs`, resolved at integration.
const TEAM_RED: c_int = 1;
const TEAM_BLUE: c_int = 2;

// `GT_*` (`bg_public.h`) — local consts, resolved at integration.
const GT_FFA: c_int = 0;
const GT_SINGLE_PLAYER: c_int = 5;
const GT_TEAM: c_int = 6;
const GT_MAX_GAME_TYPE: c_int = 10;

// `BSET_SPAWN` (`g_public.h`'s `bSet_t`, `g_local.h`) — local const.
const BSET_SPAWN: c_int = 0;

// Configstring indices come from the canonical `mp_bg::public::configstring`
// module via the prelude glob (`CS_GAME_VERSION`=20, `CS_MUSIC`=2, `CS_MESSAGE`=3,
// `CS_MOTD`=4, `CS_WARMUP`=5, `CS_LEVEL_START_TIME`=21, `CS_GLOBAL_AMBIENT_SET`=32,
// `CS_LIGHT_STYLES`=`CS_EFFECTS+MAX_FX`). The former file-local block diverged from
// these oracle values and was removed.
// Source: `oracle/codemp/game/bg_public.h:59-114`

// Light style constants
// Source: `oracle/codemp/game/g_spawn.c` (SP_worldspawn light styles)
const LS_STYLES_START: c_int = 0;
const LS_NUM_STYLES: c_int = 32;

// `ENTITYNUM_WORLD` (== MAX_GENTITIES-2 == 1022) is canonical in
// `mp_qshared::shared::limits` and reaches here via the prelude glob. The
// former local decl was wrongly 0, which indexed g_entities[0] (a client slot)
// instead of the world entity in SP_worldspawn / worldspawn behaviorSet setup.
// Source: `oracle/codemp/game/q_shared.h:2015`

/// Raven `G_SpawnString`.
///
/// Source: `oracle/codemp/game/g_spawn.c:6-23`
/// Raven `MAX_AMBIENT_SETS`.
///
/// Raven: ambient soundsets must be sent over in config strings.
/// Source: `oracle/codemp/game/q_shared.h:2035`
pub const MAX_AMBIENT_SETS: c_int = 256;

pub fn G_SpawnString(ctx: &mut GameContext, key: &str, default: &str) -> (qboolean, String) {
    // Raven's `if ( !level.spawning ) { *out = default; }` is a no-op guard: the
    // `G_Error` return is commented out in Raven, so the search runs regardless
    // and the fall-through supplies `default` anyway.
    for (k, v) in &ctx.world.level.spawnVars {
        if Q_stricmp(k, key) == 0 {
            return (qtrue, v.clone());
        }
    }
    (qfalse, default.to_owned())
}

/// Raven `G_SpawnFloat`.
///
/// Source: `oracle/codemp/game/g_spawn.c:25-32`
pub fn G_SpawnFloat(
    ctx: &mut GameContext,
    key: *const c_char,
    defaultString: *const c_char,
    out: *mut f32,
) -> qboolean {
    unsafe {
        let (present, s) = G_SpawnString(ctx, &cstr_to_str(key), &cstr_to_str(defaultString));
        *out = atof_bytes(s.as_bytes()) as f32;
        present
    }
}

/// Raven `G_SpawnInt`.
///
/// Source: `oracle/codemp/game/g_spawn.c:34-41`
pub fn G_SpawnInt(
    ctx: &mut GameContext,
    key: *const c_char,
    defaultString: *const c_char,
    out: *mut c_int,
) -> qboolean {
    unsafe {
        let (present, s) = G_SpawnString(ctx, &cstr_to_str(key), &cstr_to_str(defaultString));
        *out = atoi_bytes(s.as_bytes());
        present
    }
}

/// Raven `G_SpawnVector`.
///
/// Source: `oracle/codemp/game/g_spawn.c:43-50`
pub fn G_SpawnVector(
    ctx: &mut GameContext,
    key: *const c_char,
    defaultString: *const c_char,
    out: *mut f32,
) -> qboolean {
    unsafe {
        let (present, s) = G_SpawnString(ctx, &cstr_to_str(key), &cstr_to_str(defaultString));
        // Unmatched components are left as whatever `*out` already held
        // (porting-rules §19) — read the current 3 floats, let `sscanf_str_3f`
        // overwrite only the ones libc `sscanf` would have matched.
        let mut vec: [f32; 3] = [*out.add(0), *out.add(1), *out.add(2)];
        sscanf_str_3f(&s, &mut vec);
        *out.add(0) = vec[0];
        *out.add(1) = vec[1];
        *out.add(2) = vec[2];
        present
    }
}

// ---------------------------------------------------------------------
// Local helpers mirroring libc semantics used throughout this file
// (`atoi`/`sscanf("%f %f %f", ...)` — house rule: libc/other symbols use the
// Rust std equivalent, no resolved signature needed). `atof` is libc strtod —
// `native_string::atof` (retail's JK2_game.vcproj excludes bg_lib.c from the
// native DLL, so its QVM `atof` never linked); `sscanf_3f`/`sscanf_1f` route
// through the shared libc-`%f` scanner `native_string::sscanf::sscanf_f32s`.
// ---------------------------------------------------------------------

/// `sscanf(s, "%f %f %f", &out[0], &out[1], &out[2])` via the shared
/// libc-`%f`-faithful scanner. Unmatched components are left at whatever
/// value `out` already held (porting-rules §19) — callers pre-seed `out`
/// before calling. An empty `s` matches nothing, mirroring the old NULL-pointer
/// early return.
fn sscanf_str_3f(s: &str, out: &mut [f32; 3]) {
    sscanf_f32s(s, out);
}

/// `sscanf(s, "%f", out)` via the shared libc-`%f`-faithful scanner. Leaves
/// `*out` untouched on a failed match (porting-rules §19).
fn sscanf_str_1f(s: &str, out: &mut f32) {
    sscanf_f32s(s, std::slice::from_mut(out));
}

/// Raven `SP_item_botroam` — empty body (Raven's is a stub too).
///
/// Source: `oracle/codemp/game/g_spawn.c:368-370`
pub fn SP_item_botroam(ent: &mut gentity_t) {}

/// Raven `SP_gametype_item`.
///
/// Source: `oracle/codemp/game/g_spawn.c:372-431`
pub fn SP_gametype_item(ctx: &mut GameContext, id: EntityId) {
    unsafe {
        // Raven reads "teamfilter" into a local that is never used again (the
        // team override below consults `level.mTeamFilter`); the call is pure, so
        // its result is discarded.
        let _ = G_SpawnString(ctx, "teamfilter", "");

        let origin = ctx.entity(id).s.origin;
        G_SetOrigin(ctx.entity_mut(id), origin);

        // If a team filter is set then override any team settings for the spawns
        let mut team: c_int = -1;
        if !ctx.world.level.mTeamFilter.is_empty() {
            if ctx.world.level.mTeamFilter.eq_ignore_ascii_case("red") {
                team = TEAM_RED;
            } else if ctx.world.level.mTeamFilter.eq_ignore_ascii_case("blue") {
                team = TEAM_BLUE;
            }
        }

        let mut item: Option<ItemId> = None;
        if let Some(targetname) = ctx.entity(id).targetname_str().filter(|s| !s.is_empty()) {
            if team != -1 {
                if targetname.contains("flag") {
                    item = if team == TEAM_RED {
                        BG_FindItem("team_CTF_redflag")
                    } else {
                        // blue
                        BG_FindItem("team_CTF_blueflag")
                    };
                }
            } else if targetname.contains("red_flag") {
                item = BG_FindItem("team_CTF_redflag");
            } else if targetname.contains("blue_flag") {
                item = BG_FindItem("team_CTF_blueflag");
            } else {
                item = None;
            }

            if let Some(item) = item {
                ctx.ent_set(id, PrefixSet::Targetname(None));
                // Raven `ent->classname = item->classname`: alias the item table's
                // `'static` classname pointer (no pool copy).
                let classname: &'static CStr = CStr::from_ptr(item.classname_cstr());
                ctx.ent_set(id, PrefixSet::ClassnameStatic(classname));
                G_SpawnItem(ctx, id, item);
            }
        }
    }
}

/// Raven `G_CallSpawn`.
///
/// Finds the spawn function for the entity and calls it, returning `qfalse`
/// if not found. Checks item spawn functions first (from `bg_itemlist`), then
/// normal spawn functions (from `spawns[]` table).
///
/// Source: `oracle/codemp/game/g_spawn.c:683-714`
pub fn G_CallSpawn(ctx: &mut GameContext, id: EntityId) -> qboolean {
    if ctx.entity(id).classname_str().is_empty() {
        G_Printf(ctx, "G_CallSpawn: NULL classname\n");
        return qfalse;
    }

    // check item spawn functions
    let ent_classname = ctx.entity(id).classname_str();
    let mut i: c_int = 1;
    while i < bg_numItems {
        let item = ItemId::from_modelindex(i).unwrap();
        // Raven matches items with case-sensitive `strcmp`, not `Q_stricmp`.
        if item.item().classname.as_bytes() == ent_classname.as_bytes() {
            G_SpawnItem(ctx, id, item);
            return qtrue;
        }
        i += 1;
    }

    // check normal spawn functions
    let classname = ctx.entity(id).classname_str();
    if let Some(sp) = spawn_for_classname(&classname) {
        let healingsound = ctx.entity(id).healingsound.clone();
        if !healingsound.is_empty() {
            //yeah...this can be used for anything, so.. precache it if it's there
            G_SoundIndex(&healingsound);
        }
        let ent_ptr = ctx.entity_mut(id) as *mut gentity_t;
        dispatch_spawn(ctx, sp, ent_ptr);
        return qtrue;
    }
    let classname_disp = ctx.entity(id).classname_str();
    G_Printf(
        ctx,
        &format!("{} doesn't have a spawn function\n", classname_disp),
    );
    qfalse
}

/// Raven `G_NewString` — builds a copy of the string, translating `\n` to
/// real linefeeds so message texts can be multi-line.
///
/// Source: `oracle/codemp/game/g_spawn.c:724-749`
pub fn G_NewString(ctx: &mut GameContext, string: *const c_char) -> *mut c_char {
    unsafe {
        let mut l = 0isize;
        while *string.offset(l) != 0 {
            l += 1;
        }
        l += 1; // + 1 for the NUL, matching `strlen(string) + 1`

        let newb = G_Alloc(ctx, l as c_int) as *mut c_char;
        let mut new_p = newb;

        let mut i: isize = 0;
        while i < l {
            let c = *string.offset(i);
            if c == b'\\' as c_char && i < l - 1 {
                i += 1;
                let c2 = *string.offset(i);
                if c2 == b'n' as c_char {
                    *new_p = b'\n' as c_char;
                } else {
                    *new_p = b'\\' as c_char;
                }
                new_p = new_p.offset(1);
            } else {
                *new_p = c;
                new_p = new_p.offset(1);
            }
            i += 1;
        }

        newb
    }
}

/// `fields[]` (`BG_field_t[]`) — this file's own file-scope table, feeding
/// `BG_ParseField` in `G_SpawnGEntityFromSpawnVars`/`SP_worldspawn`. Offsets
/// via `offset_of!` against the already offset-asserted `gentity_t`.
///
/// Source: `oracle/codemp/game/g_spawn.c:54-149`
pub static FIELDS: &[BG_field_t] = &[
    field(
        c"classname",
        core::mem::offset_of!(gentity_t, classname),
        fieldtype_t::F_LSTRING,
    ),
    field(
        c"teamnodmg",
        core::mem::offset_of!(gentity_t, teamnodmg),
        fieldtype_t::F_INT,
    ),
    field(
        c"teamowner",
        core::mem::offset_of!(gentity_t, s) + core::mem::offset_of!(entityState_t, teamowner),
        fieldtype_t::F_INT,
    ),
    field(
        c"teamuser",
        core::mem::offset_of!(gentity_t, alliedTeam),
        fieldtype_t::F_INT,
    ),
    field(
        c"alliedTeam",
        core::mem::offset_of!(gentity_t, alliedTeam),
        fieldtype_t::F_INT,
    ),
    // `roffname`/`rofftarget` fields deleted (zero readers); their spawn keys
    // parse silently as F_IGNORE so maps still load.
    field(c"roffname", 0, fieldtype_t::F_IGNORE),
    field(c"rofftarget", 0, fieldtype_t::F_IGNORE),
    field_owned(c"healingclass", set_healingclass),
    field_owned(c"healingsound", set_healingsound),
    field(
        c"healingrate",
        core::mem::offset_of!(gentity_t, healingrate),
        fieldtype_t::F_INT,
    ),
    field_owned(c"ownername", set_ownername),
    field(
        c"origin",
        core::mem::offset_of!(gentity_t, s) + core::mem::offset_of!(entityState_t, origin),
        fieldtype_t::F_VECTOR,
    ),
    field_owned(c"model", set_model),
    field_owned(c"model2", set_model2),
    field(
        c"spawnflags",
        core::mem::offset_of!(gentity_t, spawnflags),
        fieldtype_t::F_INT,
    ),
    field(
        c"speed",
        core::mem::offset_of!(gentity_t, speed),
        fieldtype_t::F_FLOAT,
    ),
    field_owned(c"target", set_target),
    field_owned(c"target2", set_target2),
    field_owned(c"target3", set_target3),
    field_owned(c"target4", set_target4),
    field_owned(c"target5", set_target5),
    field_owned(c"target6", set_target6),
    field_owned(c"NPC_targetname", set_NPC_targetname),
    field_owned(c"NPC_target", set_NPC_target),
    field_owned(c"NPC_target2", set_target2), // NPC_spawner only
    field_owned(c"NPC_target4", set_target4), // NPC_spawner only
    field_owned(c"NPC_type", set_NPC_type),
    field(
        c"targetname",
        core::mem::offset_of!(gentity_t, targetname),
        fieldtype_t::F_LSTRING,
    ),
    field_owned(c"message", set_message),
    field_owned(c"team", set_team),
    field(
        c"wait",
        core::mem::offset_of!(gentity_t, wait),
        fieldtype_t::F_FLOAT,
    ),
    field(
        c"delay",
        core::mem::offset_of!(gentity_t, delay),
        fieldtype_t::F_INT,
    ),
    field(
        c"random",
        core::mem::offset_of!(gentity_t, random),
        fieldtype_t::F_FLOAT,
    ),
    field(
        c"count",
        core::mem::offset_of!(gentity_t, count),
        fieldtype_t::F_INT,
    ),
    field(
        c"health",
        core::mem::offset_of!(gentity_t, health),
        fieldtype_t::F_INT,
    ),
    field(c"light", 0, fieldtype_t::F_IGNORE),
    field(
        c"dmg",
        core::mem::offset_of!(gentity_t, damage),
        fieldtype_t::F_INT,
    ),
    field(
        c"angles",
        core::mem::offset_of!(gentity_t, s) + core::mem::offset_of!(entityState_t, angles),
        fieldtype_t::F_VECTOR,
    ),
    field(
        c"angle",
        core::mem::offset_of!(gentity_t, s) + core::mem::offset_of!(entityState_t, angles),
        fieldtype_t::F_ANGLEHACK,
    ),
    field_owned(c"targetShaderName", set_targetShaderName),
    field_owned(c"targetShaderNewName", set_targetShaderNewName),
    field(
        c"linear",
        core::mem::offset_of!(gentity_t, alt_fire),
        fieldtype_t::F_INT,
    ), // for movers to use linear movement
    field_owned(c"closetarget", set_closetarget), // for doors
    field_owned(c"opentarget", set_opentarget),   // for doors
    field_owned(c"paintarget", set_paintarget),   // for doors
    field_owned(c"goaltarget", set_goaltarget), // for siege
    field_owned(c"idealclass", set_idealclass), // for siege spawnpoints
    // rww - icarus stuff:
    field(
        c"spawnscript",
        core::mem::offset_of!(gentity_t, behaviorSet)
            + BSET_SPAWN as usize * core::mem::size_of::<*mut c_char>(),
        fieldtype_t::F_LSTRING,
    ),
    field(
        c"usescript",
        core::mem::offset_of!(gentity_t, behaviorSet) + 1 * core::mem::size_of::<*mut c_char>(),
        fieldtype_t::F_LSTRING,
    ),
    field(
        c"awakescript",
        core::mem::offset_of!(gentity_t, behaviorSet) + 2 * core::mem::size_of::<*mut c_char>(),
        fieldtype_t::F_LSTRING,
    ),
    field(
        c"angerscript",
        core::mem::offset_of!(gentity_t, behaviorSet) + 3 * core::mem::size_of::<*mut c_char>(),
        fieldtype_t::F_LSTRING,
    ),
    field(
        c"attackscript",
        core::mem::offset_of!(gentity_t, behaviorSet) + 4 * core::mem::size_of::<*mut c_char>(),
        fieldtype_t::F_LSTRING,
    ),
    field(
        c"victoryscript",
        core::mem::offset_of!(gentity_t, behaviorSet) + 5 * core::mem::size_of::<*mut c_char>(),
        fieldtype_t::F_LSTRING,
    ),
    field(
        c"lostenemyscript",
        core::mem::offset_of!(gentity_t, behaviorSet) + 6 * core::mem::size_of::<*mut c_char>(),
        fieldtype_t::F_LSTRING,
    ),
    field(
        c"painscript",
        core::mem::offset_of!(gentity_t, behaviorSet) + 7 * core::mem::size_of::<*mut c_char>(),
        fieldtype_t::F_LSTRING,
    ),
    field(
        c"fleescript",
        core::mem::offset_of!(gentity_t, behaviorSet) + 8 * core::mem::size_of::<*mut c_char>(),
        fieldtype_t::F_LSTRING,
    ),
    field(
        c"deathscript",
        core::mem::offset_of!(gentity_t, behaviorSet) + 9 * core::mem::size_of::<*mut c_char>(),
        fieldtype_t::F_LSTRING,
    ),
    field(
        c"delayscript",
        core::mem::offset_of!(gentity_t, behaviorSet) + 10 * core::mem::size_of::<*mut c_char>(),
        fieldtype_t::F_LSTRING,
    ),
    field(
        c"delayscripttime",
        core::mem::offset_of!(gentity_t, delayScriptTime),
        fieldtype_t::F_INT,
    ),
    field(
        c"blockedscript",
        core::mem::offset_of!(gentity_t, behaviorSet) + 11 * core::mem::size_of::<*mut c_char>(),
        fieldtype_t::F_LSTRING,
    ),
    field(
        c"ffirescript",
        core::mem::offset_of!(gentity_t, behaviorSet) + 14 * core::mem::size_of::<*mut c_char>(),
        fieldtype_t::F_LSTRING,
    ),
    field(
        c"ffdeathscript",
        core::mem::offset_of!(gentity_t, behaviorSet) + 15 * core::mem::size_of::<*mut c_char>(),
        fieldtype_t::F_LSTRING,
    ),
    field(
        c"mindtrickscript",
        core::mem::offset_of!(gentity_t, behaviorSet) + 16 * core::mem::size_of::<*mut c_char>(),
        fieldtype_t::F_LSTRING,
    ),
    field(
        c"script_targetname",
        core::mem::offset_of!(gentity_t, script_targetname),
        fieldtype_t::F_LSTRING,
    ),
    field(
        c"fullName",
        core::mem::offset_of!(gentity_t, fullName),
        fieldtype_t::F_LSTRING,
    ),
    field_owned(c"soundSet", set_soundSet),
    field(
        c"radius",
        core::mem::offset_of!(gentity_t, radius),
        fieldtype_t::F_FLOAT,
    ),
    field(
        c"numchunks",
        core::mem::offset_of!(gentity_t, radius),
        fieldtype_t::F_FLOAT,
    ), // for func_breakables
    field(
        c"chunksize",
        core::mem::offset_of!(gentity_t, mass),
        fieldtype_t::F_FLOAT,
    ), // for func_breakables
    // Script parms
    field(c"parm1", 0, fieldtype_t::F_PARM1),
    field(c"parm2", 0, fieldtype_t::F_PARM2),
    field(c"parm3", 0, fieldtype_t::F_PARM3),
    field(c"parm4", 0, fieldtype_t::F_PARM4),
    field(c"parm5", 0, fieldtype_t::F_PARM5),
    field(c"parm6", 0, fieldtype_t::F_PARM6),
    field(c"parm7", 0, fieldtype_t::F_PARM7),
    field(c"parm8", 0, fieldtype_t::F_PARM8),
    field(c"parm9", 0, fieldtype_t::F_PARM9),
    field(c"parm10", 0, fieldtype_t::F_PARM10),
    field(c"parm11", 0, fieldtype_t::F_PARM11),
    field(c"parm12", 0, fieldtype_t::F_PARM12),
    field(c"parm13", 0, fieldtype_t::F_PARM13),
    field(c"parm14", 0, fieldtype_t::F_PARM14),
    field(c"parm15", 0, fieldtype_t::F_PARM15),
    field(c"parm16", 0, fieldtype_t::F_PARM16),
    // {NULL} terminator: BG_ParseField scans `for (f=l_fields; f->name; f++)`,
    // so the sentinel's `name` must be a genuine null pointer, not a pointer to
    // an empty string, or the scan runs off the end of the table.
    BG_field_t {
        name: std::ptr::null_mut(),
        ofs: 0,
        r#type: fieldtype_t::F_IGNORE,
        flags: 0,
        set: None,
    },
];

// `behaviorSet[i]` is a `[*mut c_char; NUM_BSETS]` array of pointer-sized slots;
// the `+ i * size_of::<*mut c_char>()` strides above mirror Raven's
// `FOFS(behaviorSet[BSET_X])` at the target's pointer width (8 on LP64, 4 on
// ILP32) without indexing through a non-const array-index `offset_of!` (not yet
// stable for computed indices).
const fn field(name: &'static CStr, ofs: usize, r#type: fieldtype_t) -> BG_field_t {
    BG_field_t {
        name: name.as_ptr() as *mut c_char,
        ofs: ofs as c_int,
        r#type,
        flags: 0,
        set: None,
    }
}

/// Owned-tail-field entry: [`fieldtype_t::F_STRING_OWNED`] with no offset — the
/// typed `set`ter stores the decoded value into the entity's owned
/// `String`/`Option<String>` field. Replaces the `F_LSTRING` (pool-pointer)
/// entry for every migrated tail field.
const fn field_owned(name: &'static CStr, set: SpawnStringSetter) -> BG_field_t {
    BG_field_t {
        name: name.as_ptr() as *mut c_char,
        ofs: 0,
        r#type: fieldtype_t::F_STRING_OWNED,
        flags: 0,
        set: Some(set),
    }
}

// Typed setters for the owned tail fields migrated in this batch (all plain
// `String`: `""` ≡ absent per the migration's ruling C). Each casts the
// type-erased entity base and stores the decoded value; the setter matches the
// [`SpawnStringSetter`] shape bg's `BG_ParseField` invokes.
//
// # Safety (all): `ent` is the base of one live `gentity_t`, as `BG_ParseField`
// passes it.
fn set_healingclass(ent: *mut byte, val: &str) {
    unsafe { (*(ent as *mut gentity_t)).healingclass = val.to_owned() };
}
fn set_healingsound(ent: *mut byte, val: &str) {
    unsafe { (*(ent as *mut gentity_t)).healingsound = val.to_owned() };
}
fn set_ownername(ent: *mut byte, val: &str) {
    unsafe { (*(ent as *mut gentity_t)).ownername = val.to_owned() };
}
fn set_NPC_target(ent: *mut byte, val: &str) {
    unsafe { (*(ent as *mut gentity_t)).NPC_target = val.to_owned() };
}
// `NPC_type` is `Option<String>` (`None` ≡ Raven NULL); a present spawn key —
// even `""` — is `Some(..)`, matching Raven's non-NULL pool pointer.
fn set_NPC_type(ent: *mut byte, val: &str) {
    unsafe { (*(ent as *mut gentity_t)).NPC_type = Some(val.to_owned()) };
}
fn set_NPC_targetname(ent: *mut byte, val: &str) {
    unsafe { (*(ent as *mut gentity_t)).NPC_targetname = val.to_owned() };
}
fn set_target3(ent: *mut byte, val: &str) {
    unsafe { (*(ent as *mut gentity_t)).target3 = val.to_owned() };
}
fn set_target4(ent: *mut byte, val: &str) {
    unsafe { (*(ent as *mut gentity_t)).target4 = val.to_owned() };
}
fn set_target5(ent: *mut byte, val: &str) {
    unsafe { (*(ent as *mut gentity_t)).target5 = val.to_owned() };
}
fn set_target6(ent: *mut byte, val: &str) {
    unsafe { (*(ent as *mut gentity_t)).target6 = val.to_owned() };
}
fn set_model2(ent: *mut byte, val: &str) {
    unsafe { (*(ent as *mut gentity_t)).model2 = val.to_owned() };
}
fn set_soundSet(ent: *mut byte, val: &str) {
    unsafe { (*(ent as *mut gentity_t)).soundSet = val.to_owned() };
}
fn set_targetShaderName(ent: *mut byte, val: &str) {
    unsafe { (*(ent as *mut gentity_t)).targetShaderName = val.to_owned() };
}
fn set_targetShaderNewName(ent: *mut byte, val: &str) {
    unsafe { (*(ent as *mut gentity_t)).targetShaderNewName = val.to_owned() };
}
fn set_goaltarget(ent: *mut byte, val: &str) {
    unsafe { (*(ent as *mut gentity_t)).goaltarget = val.to_owned() };
}
fn set_idealclass(ent: *mut byte, val: &str) {
    unsafe { (*(ent as *mut gentity_t)).idealclass = val.to_owned() };
}
// `target`/`target2`/`team` are `Option<String>` (`None` ≡ Raven NULL); a
// present spawn key — even `""` — is `Some(..)`, matching Raven's non-NULL pool
// pointer.
fn set_target(ent: *mut byte, val: &str) {
    unsafe { (*(ent as *mut gentity_t)).target = Some(val.to_owned()) };
}
fn set_target2(ent: *mut byte, val: &str) {
    unsafe { (*(ent as *mut gentity_t)).target2 = Some(val.to_owned()) };
}
fn set_team(ent: *mut byte, val: &str) {
    unsafe { (*(ent as *mut gentity_t)).team = Some(val.to_owned()) };
}

// `model`/`closetarget`/`opentarget`/`paintarget` are `Option<String>` (`None` ≡
// Raven NULL); a present spawn key — even `""` — is `Some(..)`, matching Raven's
// non-NULL pool pointer.
fn set_model(ent: *mut byte, val: &str) {
    unsafe { (*(ent as *mut gentity_t)).model = Some(val.to_owned()) };
}
fn set_closetarget(ent: *mut byte, val: &str) {
    unsafe { (*(ent as *mut gentity_t)).closetarget = Some(val.to_owned()) };
}
fn set_opentarget(ent: *mut byte, val: &str) {
    unsafe { (*(ent as *mut gentity_t)).opentarget = Some(val.to_owned()) };
}
fn set_paintarget(ent: *mut byte, val: &str) {
    unsafe { (*(ent as *mut gentity_t)).paintarget = Some(val.to_owned()) };
}

// `message` is `Option<String>` too, but its old `F_LSTRING` write ran through
// `G_NewString`, whose `\n`-escape translation must be reproduced here or
// multi-line message text regresses (G1 flag).
fn set_message(ent: *mut byte, val: &str) {
    unsafe { (*(ent as *mut gentity_t)).message = Some(translate_newlines(val)) };
}

/// Reproduces `G_NewString`'s `\n`-escape translation as an owned `String`
/// (no pool allocation): a `\` followed by `n` becomes a real linefeed, any
/// other `\x` collapses to a lone `\` (the escaped char is dropped), matching
/// the C copy loop byte-for-byte. Shared by the owned-`String` setters whose
/// Raven write went through `G_NewString`.
///
/// Source: `oracle/codemp/game/g_spawn.c:724-749`
pub fn translate_newlines(src: &str) -> String {
    let bytes = src.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        let c = bytes[i];
        if c == b'\\' && i < bytes.len() - 1 {
            i += 1;
            out.push(if bytes[i] == b'n' { b'\n' } else { b'\\' });
        } else {
            out.push(c);
        }
        i += 1;
    }
    // `src` is valid UTF-8 and the translation only ever emits `\n`/`\\`/copied
    // input bytes, so the result is still valid UTF-8.
    String::from_utf8(out).unwrap()
}

/// Raven `G_SpawnGEntityFromSpawnVars` — spawns an entity and fills in all of
/// the level fields from `level.spawnVars[]`, then calls the class-specific
/// spawn function.
///
/// Source: `oracle/codemp/game/g_spawn.c:766-842`
pub fn G_SpawnGEntityFromSpawnVars(ctx: &mut GameContext, inSubBSP: qboolean) {
    unsafe {
        // static char *gametypeNames[] — fn-scope const table.
        const GAMETYPE_NAMES: [&CStr; 10] = [
            c"ffa",
            c"holocron",
            c"jedimaster",
            c"duel",
            c"powerduel",
            c"single",
            c"team",
            c"siege",
            c"ctf",
            c"cty",
        ];

        // get the next free entity
        let ent_eid = G_Spawn(ctx);
        let ent = ctx.entity_mut(ent_eid) as *mut gentity_t;

        let mut callbacks = crate::bg_channel::GameCallbacksImpl {
            // SEAM-BG-REENTRY (DEC-28, sanctioned) — GameCallbacksImpl.world is a `*mut GameWorld`
            // field; a raw store is required (bg-seam re-entry).
            world: ctx.world_raw(),
            engine: ctx.engine,
        };
        let num_spawn_vars = ctx.world.level.spawnVars.len();
        for i in 0..num_spawn_vars {
            let key = cstr(&ctx.world.level.spawnVars[i].0);
            let value = cstr(&ctx.world.level.spawnVars[i].1);
            BG_ParseField(
                &mut callbacks,
                FIELDS.as_ptr() as *mut BG_field_t,
                key.as_ptr(),
                value.as_ptr(),
                ent as *mut byte,
            );
        }

        // check for "notsingle" flag
        let mut i: c_int = 0;
        if ctx.world.cvars.g_gametype.integer == GT_SINGLE_PLAYER {
            G_SpawnInt(ctx, c"notsingle".as_ptr(), c"0".as_ptr(), &mut i);
            if i != 0 {
                G_FreeEntity(ctx, Some(ent_eid));
                return;
            }
        }
        // check for "notteam" flag (GT_FFA, GT_DUEL, GT_SINGLE_PLAYER)
        if ctx.world.cvars.g_gametype.integer >= GT_TEAM {
            G_SpawnInt(ctx, c"notteam".as_ptr(), c"0".as_ptr(), &mut i);
            if i != 0 {
                G_FreeEntity(ctx, Some(ent_eid));
                return;
            }
        } else {
            G_SpawnInt(ctx, c"notfree".as_ptr(), c"0".as_ptr(), &mut i);
            if i != 0 {
                G_FreeEntity(ctx, Some(ent_eid));
                return;
            }
        }

        G_SpawnInt(ctx, c"notta".as_ptr(), c"0".as_ptr(), &mut i);
        if i != 0 {
            G_FreeEntity(ctx, Some(ent_eid));
            return;
        }

        let (present, value) = G_SpawnString(ctx, "gametype", "");
        if present != qfalse {
            let gt = ctx.world.cvars.g_gametype.integer;
            if gt >= GT_FFA && gt < GT_MAX_GAME_TYPE {
                let gametype_name = GAMETYPE_NAMES[gt as usize];
                if !value.contains(gametype_name.to_str().unwrap()) {
                    G_FreeEntity(ctx, Some(ent_eid));
                    return;
                }
            }
        }

        // move editor origin to pos
        let id = ent_eid;
        {
            let e = ctx.world.entity_mut(id);
            let origin = e.s.origin;
            e.s.pos.trBase = origin;
            e.r.currentOrigin = origin;
        }

        // if we didn't get a classname, don't bother spawning anything
        if G_CallSpawn(ctx, id) == qfalse {
            G_FreeEntity(ctx, Some(id));
        }

        // Tag on the ICARUS scripting information only to valid recipients
        if trap::ICARUS_ValidEnt(ctx.engine, GIcarusValidentArgs::new(ent.cast())) != qfalse {
            trap::ICARUS_InitEnt(ctx.engine, GIcarusInitentArgs::new(ent.cast()));

            let classname = ctx.entity(id).classname_str();
            if !classname.is_empty() && Q_strncmp("NPC_", &classname, 4) != 0 {
                // Not an NPC_spawner (rww - probably don't even care for MP, but whatever)
                G_ActivateBehavior(ctx, Some(id), BSET_SPAWN);
            }
        }
    }
}

// Raven `G_AddSpawnVarToken` (g_spawn.c:851-866) deleted: it bump-copied a token
// into the fixed `spawnVarChars` pool and returned a pointer. `spawnVars` is now
// an owned `Vec<(String, String)>`, so tokens are pushed as owned strings and the
// pool (and its `MAX_SPAWN_CHARS` overflow error) no longer exist.

/// Raven `AddSpawnField`.
///
/// Source: `oracle/codemp/game/g_spawn.c:868-884`
pub fn AddSpawnField(ctx: &mut GameContext, field: &str, value: &str) {
    for kv in &mut ctx.world.level.spawnVars {
        if Q_stricmp(&kv.0, field) == 0 {
            kv.1 = value.to_owned();
            return;
        }
    }
    ctx.world.level.spawnVars.push((field.to_owned(), value.to_owned()));
}

pub const NOVALUE: &CStr = c"novalue";

/// Raven `HandleEntityAdjustment` (file-static) — sub-BSP instance origin/
/// angle/name-prefix rewriting.
///
/// Source: `oracle/codemp/game/g_spawn.c:888-1006`
fn HandleEntityAdjustment(ctx: &mut GameContext) {
    unsafe {
        let mut new_origin: vec3_t = [0.0; 3];

        let (_, value) = G_SpawnString(ctx, "origin", "novalue");
        // `origin` is pre-seeded 0.0 (matching the else-branch below); any
        // component `sscanf_str_3f` fails to match is left at that seed rather
        // than picking up C's stack garbage (porting-rules §19).
        let mut origin: vec3_t = [0.0, 0.0, 0.0];
        if Q_stricmp(&value, "novalue") != 0 {
            sscanf_str_3f(&value, &mut origin);
        }

        // `DEG2RAD(a)` is `(a * M_PI) / 180.0F`; M_PI resolves to glibc's double
        // (math.h at q_shared.h:82 precedes the `#ifndef M_PI` float redefine at
        // :547), so `float * double / float` evaluates entirely in f64 and
        // narrows once at the f32 store. `cos`/`sin` are the double libm
        // functions and each `origin[k]*cos(...)` term likewise evaluates in f64.
        // Source: `oracle/codemp/game/q_shared.h:547-548,1174`
        let rotation =
            ((ctx.world.level.mRotationAdjust as f64 * std::f64::consts::PI) / 180.0f64) as f32;
        let cos_r = (rotation as f64).cos();
        let sin_r = (rotation as f64).sin();
        new_origin[0] = (origin[0] as f64 * cos_r - origin[1] as f64 * sin_r) as f32;
        new_origin[1] = (origin[0] as f64 * sin_r + origin[1] as f64 * cos_r) as f32;
        new_origin[2] = origin[2];
        let origin_adjust = ctx.world.level.mOriginAdjust;
        new_origin[0] += origin_adjust[0];
        new_origin[1] += origin_adjust[1];
        new_origin[2] += origin_adjust[2];

        // damn VMs don't handle outputing a float that is compatible with sscanf
        // in all cases — Com_sprintf("%0.0f %0.0f %0.0f", ...) inlined directly
        // (Com_sprintf itself is parked variadic-c-abi; same `COM_DefaultExtension`
        // precedent as `q_shared.rs`).
        let temp = format!(
            "{:.0} {:.0} {:.0}",
            new_origin[0], new_origin[1], new_origin[2]
        );
        AddSpawnField(ctx, "origin", &temp);

        let (_, value) = G_SpawnString(ctx, "angles", "novalue");
        if Q_stricmp(&value, "novalue") != 0 {
            let mut angles: vec3_t = [0.0, 0.0, 0.0];
            sscanf_str_3f(&value, &mut angles);

            // `fmod` is a double-precision truncated remainder whose sign follows
            // the dividend; `rem_euclid` (least non-negative) differs by 360 for a
            // negative sum.
            angles[1] = ((angles[1] + ctx.world.level.mRotationAdjust) as f64 % 360.0) as f32;
            let temp = format!("{:.0} {:.0} {:.0}", angles[0], angles[1], angles[2]);
            AddSpawnField(ctx, "angles", &temp);
        } else {
            let (_, value) = G_SpawnString(ctx, "angle", "novalue");
            let mut angle1: f32 = 0.0;
            if Q_stricmp(&value, "novalue") != 0 {
                sscanf_str_1f(&value, &mut angle1);
            }
            angle1 = ((angle1 + ctx.world.level.mRotationAdjust) as f64 % 360.0) as f32;
            let temp = format!("{:.0}", angle1);
            AddSpawnField(ctx, "angle", &temp);
        }

        // RJR experimental code for handling "direction" field of breakable
        // brushes, though direction is rarely ever used.
        let (_, value) = G_SpawnString(ctx, "direction", "novalue");
        let mut direction: vec3_t = [0.0, 0.0, 0.0];
        if Q_stricmp(&value, "novalue") != 0 {
            sscanf_str_3f(&value, &mut direction);
        }
        direction[1] = ((direction[1] + ctx.world.level.mRotationAdjust) as f64 % 360.0) as f32;
        let temp = format!(
            "{:.0} {:.0} {:.0}",
            direction[0], direction[1], direction[2]
        );
        AddSpawnField(ctx, "direction", &temp);

        let target_adjust = ctx.world.level.mTargetAdjust;
        let target_adjust_str = if target_adjust.is_null() {
            String::new()
        } else {
            CStr::from_ptr(target_adjust).to_string_lossy().into_owned()
        };

        AddSpawnField(ctx, "BSPInstanceID", &target_adjust_str);

        for key in [
            "targetname",
            "target",
            "killtarget",
            "brushparent",
            "brushchild",
            "enemy",
            "ICARUSname",
        ] {
            let (_, value) = G_SpawnString(ctx, key, "novalue");
            if Q_stricmp(&value, "novalue") != 0 {
                let temp = format!("{}{}", target_adjust_str, value);
                AddSpawnField(ctx, key, &temp);
            }
        }
    }
}

/// Raven `G_ParseSpawnVars` — parses a brace-bounded set of key/value pairs
/// out of the level's entity strings into `level.spawnVars[]`. Does not
/// actually spawn an entity.
///
/// Source: `oracle/codemp/game/g_spawn.c:1018-1067`
pub fn G_ParseSpawnVars(ctx: &mut GameContext, inSubBSP: qboolean) -> qboolean {
    // `MAX_TOKEN_CHARS` (value 1024) canonical in `mp_qshared::shared::limits`,
    // reaches this file via the crate prelude glob.
    ctx.world.level.spawnVars.clear();

    // parse the opening brace
    let Some(com_token) = trap::GetEntityToken(ctx.engine, MAX_TOKEN_CHARS) else {
        // end of spawn string
        return qfalse;
    };
    if com_token.as_bytes().first() != Some(&b'{') {
        panic!("G_ParseSpawnVars: found {{ ... }} mismatch"); // G_Error -> panic (frozen Group A)
    }

    // go through all the key / value pairs
    loop {
        // parse key
        let Some(keyname) = trap::GetEntityToken(ctx.engine, MAX_TOKEN_CHARS) else {
            panic!("G_ParseSpawnVars: EOF without closing brace");
        };

        if keyname.as_bytes().first() == Some(&b'}') {
            break;
        }

        // parse value
        let Some(com_token) = trap::GetEntityToken(ctx.engine, MAX_TOKEN_CHARS) else {
            panic!("G_ParseSpawnVars: EOF without closing brace");
        };

        if com_token.as_bytes().first() == Some(&b'}') {
            panic!("G_ParseSpawnVars: closing brace without data");
        }
        if ctx.world.level.spawnVars.len() == mp_bg::MAX_SPAWN_VARS {
            panic!("G_ParseSpawnVars: MAX_SPAWN_VARS");
        }
        ctx.world.level.spawnVars.push((keyname, com_token));
    }

    // Oracle calls HandleEntityAdjustment exactly once, after the loop, gated on
    // inSubBSP. Source: `oracle/codemp/game/g_spawn.c:1061-1064`
    if inSubBSP != qfalse {
        HandleEntityAdjustment(ctx);
    }

    qtrue
}

/// Raven `SP_worldspawn`.
///
/// Spawns the world entity and initializes the level. Parses worldspawn-specific
/// spawn variables, loads animations and ghoul2 models, and sets up configstrings.
///
/// Source: `oracle/codemp/game/g_spawn.c:1259-1386`
pub fn SP_worldspawn(ctx: &mut GameContext) {
    unsafe {
        let mut text: String = String::new();
        let mut temp = [0i8; 32];
        let mut lengthRed: c_int;
        let mut lengthBlue: c_int;
        let mut lengthGreen: c_int;

        // STAGE-2b: irreducible — `g_cullDistance` is a `&mut f32` out-param
        // aliasing `ctx.world` while `ctx` is also passed to `G_SpawnFloat`.
        let cull_out = &mut (*ctx.world_raw()).globals.g_cullDistance;
        // I want to "cull" entities out of net sends to clients to reduce
        // net traffic on our larger open maps -rww
        G_SpawnFloat(ctx, c"distanceCull".as_ptr(), c"6000.0".as_ptr(), cull_out);
        trap::SetServerCull(
            ctx.engine,
            GSetServerCullArgs::new(ctx.world.globals.g_cullDistance),
        );

        text = G_SpawnString(ctx, "classname", "").1;
        if Q_stricmp(&text, "worldspawn") != 0 {
            G_Error(ctx, "SP_worldspawn: The first entity isn't 'worldspawn'");
        }

        let mut callbacks = crate::bg_channel::GameCallbacksImpl {
            // SEAM-BG-REENTRY (DEC-28, sanctioned) — GameCallbacksImpl.world is a `*mut GameWorld`
            // field; a raw store is required (bg-seam re-entry).
            world: ctx.world_raw(),
            engine: ctx.engine,
        };
        for i in 0..ctx.world.level.spawnVars.len() {
            if Q_stricmp("spawnscript", &ctx.world.level.spawnVars[i].0) == 0 {
                let field_key = cstr(&ctx.world.level.spawnVars[i].0);
                let field_value = cstr(&ctx.world.level.spawnVars[i].1);
                let ent_base = ctx.world.g_entities.as_mut_ptr() as *mut byte;
                // Only let them set spawnscript, we don't want them setting an angle or something on the world.
                BG_ParseField(
                    &mut callbacks,
                    FIELDS.as_ptr() as *mut BG_field_t,
                    field_key.as_ptr(),
                    field_value.as_ptr(),
                    ent_base,
                );
            }
        }

        // The server will precache the standard model and animations, so that there is no hit
        // when the first client connects.
        if ctx.world.bg_state.BGPAFtextLoaded == qfalse {
            let traps = crate::bg_channel::GameBgTraps::new(ctx.engine);
            let mut callbacks = crate::bg_channel::GameCallbacksImpl {
                // SEAM-BG-REENTRY (DEC-28, sanctioned) — GameCallbacksImpl.world is a `*mut GameWorld`
                // field aliasing the `bg_state` args below; the callbacks handle and
                // both `&mut bg_state`/`bgHumanoidAnimations` derefs alias one world,
                // so the whole `BG_ParseAnimationFile` seam stays a raw-pointer group.
                world: ctx.world_raw(),
                engine: ctx.engine,
            };
            BG_ParseAnimationFile(
                &mut (*ctx.world_raw()).bg_state,
                &traps,
                &mut callbacks,
                c"models/players/_humanoid/animation.cfg".as_ptr(),
                (*ctx.world_raw())
                    .bg_state
                    .bgHumanoidAnimations
                    .as_mut_ptr(),
                qtrue,
            );
        }

        if ctx.world.globals.precachedKyle.is_null() {
            let mut defSkin: c_int;

            trap::G2API_InitGhoul2Model(
                ctx.engine,
                &mut ctx.world.globals.precachedKyle as *mut *mut c_void,
                "models/players/kyle/model.glm",
                0,
                0,
                -20,
                0,
                0,
            );

            if !ctx.world.globals.precachedKyle.is_null() {
                defSkin = trap::R_RegisterSkin(ctx.engine, "models/players/kyle/model_default.skin");
                trap::G2API_SetSkin(
                    ctx.engine,
                    GG2SetSkinArgs::new(ctx.world.globals.precachedKyle, 0, defSkin, defSkin),
                );
            }
        }

        if ctx.world.globals.g2SaberInstance.is_null() {
            trap::G2API_InitGhoul2Model(
                ctx.engine,
                &mut ctx.world.globals.g2SaberInstance as *mut *mut c_void,
                "models/weapons2/saber/saber_w.glm",
                0,
                0,
                -20,
                0,
                0,
            );

            if !ctx.world.globals.g2SaberInstance.is_null() {
                // indicate we will be bolted to model 0 (ie the player) on bolt 0 (always the right hand) when we get copied
                trap::G2API_SetBoltInfo(
                    ctx.engine,
                    GG2SetBoltInfoArgs::new(ctx.world.globals.g2SaberInstance, 0, 0),
                );
                // now set up the gun bolt on it
                trap::G2API_AddBolt(ctx.engine, ctx.world.globals.g2SaberInstance, 0, "*blade1");
            }
        }

        if ctx.world.cvars.g_gametype.integer == GT_SIEGE {
            // a tad bit of a hack, but..
            EWebPrecache(ctx);
        }

        // make some data visible to connecting client
        // `#define GAME_VERSION "basejka-1"`.
        trap::SetConfigstring(ctx.engine, CS_GAME_VERSION, "basejka-1");

        trap::SetConfigstring(
            ctx.engine,
            CS_LEVEL_START_TIME,
            &format!("{}", ctx.world.level.startTime),
        );

        text = G_SpawnString(ctx, "music", "").1;
        trap::SetConfigstring(ctx.engine, CS_MUSIC, &text);

        text = G_SpawnString(ctx, "message", "").1;
        trap::SetConfigstring(ctx.engine, CS_MESSAGE, &text); // map specific message

        trap::SetConfigstring(
            ctx.engine,
            CS_MOTD,
            &cstr_from_chars(&ctx.world.cvars.g_motd.string).to_string_lossy(),
        ); // message of the day

        text = G_SpawnString(ctx, "gravity", "800").1;
        trap::Cvar_Set(ctx.engine, "g_gravity", &text);

        text = G_SpawnString(ctx, "enableBreath", "0").1;
        trap::Cvar_Set(ctx.engine, "g_enableBreath", &text);

        text = G_SpawnString(ctx, "soundSet", "default").1;
        trap::SetConfigstring(
            ctx.engine,
            mp_bg::public::configstring::CS_GLOBAL_AMBIENT_SET,
            &text,
        );

        ctx.world.g_entities[ENTITYNUM_WORLD as usize].s.number = ENTITYNUM_WORLD;
        ctx.ent_set(EntityId(ENTITYNUM_WORLD as u32), PrefixSet::ClassnameStatic(c"worldspawn"));

        // see if we want a warmup time
        trap::SetConfigstring(ctx.engine, CS_WARMUP, "");
        if ctx.world.cvars.g_restarted.integer != 0 {
            trap::Cvar_Set(ctx.engine, "g_restarted", "0");
            ctx.world.level.warmupTime = 0;
        }

        trap::SetConfigstring(
            ctx.engine,
            CS_LIGHT_STYLES + (LS_STYLES_START as c_int * 3) as c_int,
            &cstr_to_str(defaultStyles[0][0]),
        );
        trap::SetConfigstring(
            ctx.engine,
            CS_LIGHT_STYLES + (LS_STYLES_START as c_int * 3 + 1) as c_int,
            &cstr_to_str(defaultStyles[0][1]),
        );
        trap::SetConfigstring(
            ctx.engine,
            CS_LIGHT_STYLES + (LS_STYLES_START as c_int * 3 + 2) as c_int,
            &cstr_to_str(defaultStyles[0][2]),
        );

        for i in 1..LS_NUM_STYLES {
            let red_key = format!("ls_{}r", i);
            text = G_SpawnString(ctx, &red_key, &cstr_to_str(defaultStyles[i as usize][0])).1;
            lengthRed = text.len() as i32;
            trap::SetConfigstring(
                ctx.engine,
                CS_LIGHT_STYLES + ((i + LS_STYLES_START) as c_int * 3) as c_int,
                &text,
            );

            let green_key = format!("ls_{}g", i);
            text = G_SpawnString(ctx, &green_key, &cstr_to_str(defaultStyles[i as usize][1])).1;
            lengthGreen = text.len() as i32;
            trap::SetConfigstring(
                ctx.engine,
                CS_LIGHT_STYLES + ((i + LS_STYLES_START) as c_int * 3 + 1) as c_int,
                &text,
            );

            let blue_key = format!("ls_{}b", i);
            text = G_SpawnString(ctx, &blue_key, &cstr_to_str(defaultStyles[i as usize][2])).1;
            lengthBlue = text.len() as i32;
            trap::SetConfigstring(
                ctx.engine,
                CS_LIGHT_STYLES + ((i + LS_STYLES_START) as c_int * 3 + 2) as c_int,
                &text,
            );

            if lengthRed != lengthGreen || lengthGreen != lengthBlue {
                panic!(
                    "Style {} has inconsistent lengths: R {}, G {}, B {}",
                    i, lengthRed, lengthGreen, lengthBlue
                );
            }
        }
    }
}

/// Raven `SP_bsp_worldspawn` — rww: planning on having something here?
///
/// Source: `oracle/codemp/game/g_spawn.c:1389-1392`
pub fn SP_bsp_worldspawn() -> qboolean {
    qtrue
}

/// Raven `G_PrecacheSoundsets`.
///
/// Source: `oracle/codemp/game/g_spawn.c:1394-1415`
pub fn G_PrecacheSoundsets(ctx: &mut GameContext) {
    let mut counted_sets: c_int = 0;

    for i in 0..MAX_GENTITIES {
        // `soundSet` is now an owned `String` (`""` ≡ Raven's NULL-or-empty
        // guard `!soundSet || !soundSet[0]`).
        if ctx.world.g_entities[i].inuse != qfalse && !ctx.world.g_entities[i].soundSet.is_empty() {
            if counted_sets >= MAX_AMBIENT_SETS {
                panic!("MAX_AMBIENT_SETS was exceeded! (too many soundsets)\n");
                // Com_Error(ERR_DROP, ...) -> panic
            }

            let soundSet = ctx.world.g_entities[i].soundSet.clone();
            let idx = G_SoundSetIndex(ctx, &soundSet);
            ctx.world.g_entities[i].s.soundSetIndex = idx;
            counted_sets += 1;
        }
    }
}

/// Raven `G_SpawnEntitiesFromString` — parses textual entity definitions out
/// of an entstring and spawns gentities.
///
/// Source: `oracle/codemp/game/g_spawn.c:1424-1478`
pub fn G_SpawnEntitiesFromString(ctx: &mut GameContext, inSubBSP: qboolean) {
    unsafe {
        // allow calls to G_Spawn*()
        ctx.world.level.spawning = qtrue;
        ctx.world.level.spawnVars.clear();

        // the worldspawn is not an actual entity, but it still
        // has a "spawn" function to perform any global setup
        // needed by a level (setting configstrings or cvars, etc)
        if G_ParseSpawnVars(ctx, qfalse) == qfalse {
            G_Error(ctx, "SpawnEntities: no entities");
        }

        if inSubBSP == qfalse {
            SP_worldspawn(ctx);
        } else {
            // Skip this guy if its worldspawn fails
            if SP_bsp_worldspawn() == qfalse {
                return;
            }
        }

        // parse ents
        while G_ParseSpawnVars(ctx, inSubBSP) != qfalse {
            G_SpawnGEntityFromSpawnVars(ctx, inSubBSP);
        }

        let world_bset = ctx.world.g_entities[ENTITYNUM_WORLD as usize]
            .behavior_set_str(BSET_SPAWN as usize)
            .filter(|s| !s.is_empty());
        if let Some(world_bset) = world_bset {
            // World has a spawn script, but we don't want the world in ICARUS and running scripts,
            // so make a scriptrunner and start it going.
            let script_runner_eid = G_Spawn(ctx);
            let script_runner = ctx.entity_mut(script_runner_eid) as *mut gentity_t;
            if !script_runner.is_null() {
                let id = script_runner_eid;
                let next_think = ctx.world.level.time + 100;
                // Raven aliased the world's spawn-script pointer into the runner's
                // BSET_USESCRIPT (`behaviorSet[1]`) slot; the set copy is content-identical.
                ctx.ent_set(id, PrefixSet::BehaviorSet(1, Some(&world_bset)));
                {
                    let e = ctx.world.entity_mut(id);
                    e.count = 1;
                    e.think = Some(EntThink::scriptrunner_run).into();
                    e.nextthink = next_think;
                }

                if ctx.world.entity(id).inuse != qfalse {
                    trap::ICARUS_InitEnt(ctx.engine, GIcarusInitentArgs::new(script_runner.cast()));
                }
            }
        }

        if inSubBSP == qfalse {
            ctx.world.level.spawning = qfalse; // any future calls to G_Spawn*() will be errors
        }

        G_PrecacheSoundsets(ctx);
    }
}

/// Raven `defaultStyles[32][3]` — per-style light-pattern strings indexed by
/// `[styleIndex][fixture 0..2]`; entries 14-31 are empty strings (Raven never
/// filled them in).
///
/// Source: `oracle/codemp/game/g_spawn.c:1070-1236`
pub const defaultStyles: [[*const c_char; 3]; 32] = [
    [c"z".as_ptr(), c"z".as_ptr(), c"z".as_ptr()], // 0 normal
    [
        c"mmnmmommommnonmmonqnmmo".as_ptr(),
        c"mmnmmommommnonmmonqnmmo".as_ptr(),
        c"mmnmmommommnonmmonqnmmo".as_ptr(),
    ], // 1 FLICKER (first variety)
    [
        c"abcdefghijklmnopqrstuvwxyzyxwvutsrqponmlkjihgfedcb".as_ptr(),
        c"abcdefghijklmnopqrstuvwxyzyxwvutsrqponmlkjihgfedcb".as_ptr(),
        c"abcdefghijklmnopqrstuvwxyzyxwvutsrqponmlkjihgfedcb".as_ptr(),
    ], // 2 SLOW STRONG PULSE
    [
        c"mmmmmaaaaammmmmaaaaaabcdefgabcdefg".as_ptr(),
        c"mmmmmaaaaammmmmaaaaaabcdefgabcdefg".as_ptr(),
        c"mmmmmaaaaammmmmaaaaaabcdefgabcdefg".as_ptr(),
    ], // 3 CANDLE (first variety)
    [
        c"mamamamamama".as_ptr(),
        c"mamamamamama".as_ptr(),
        c"mamamamamama".as_ptr(),
    ], // 4 FAST STROBE
    [
        c"jklmnopqrstuvwxyzyxwvutsrqponmlkj".as_ptr(),
        c"jklmnopqrstuvwxyzyxwvutsrqponmlkj".as_ptr(),
        c"jklmnopqrstuvwxyzyxwvutsrqponmlkj".as_ptr(),
    ], // 5 GENTLE PULSE 1
    [
        c"nmonqnmomnmomomno".as_ptr(),
        c"nmonqnmomnmomomno".as_ptr(),
        c"nmonqnmomnmomomno".as_ptr(),
    ], // 6 FLICKER (second variety)
    [
        c"mmmaaaabcdefgmmmmaaaammmaamm".as_ptr(),
        c"mmmaaaabcdefgmmmmaaaammmaamm".as_ptr(),
        c"mmmaaaabcdefgmmmmaaaammmaamm".as_ptr(),
    ], // 7 CANDLE (second variety)
    [
        c"mmmaaammmaaammmabcdefaaaammmmabcdefmmmaaaa".as_ptr(),
        c"mmmaaammmaaammmabcdefaaaammmmabcdefmmmaaaa".as_ptr(),
        c"mmmaaammmaaammmabcdefaaaammmmabcdefmmmaaaa".as_ptr(),
    ], // 8 CANDLE (third variety)
    [
        c"aaaaaaaazzzzzzzz".as_ptr(),
        c"aaaaaaaazzzzzzzz".as_ptr(),
        c"aaaaaaaazzzzzzzz".as_ptr(),
    ], // 9 SLOW STROBE (fourth variety)
    [
        c"mmamammmmammamamaaamammma".as_ptr(),
        c"mmamammmmammamamaaamammma".as_ptr(),
        c"mmamammmmammamamaaamammma".as_ptr(),
    ], // 10 FLUORESCENT FLICKER
    [
        c"abcdefghijklmnopqrrqponmlkjihgfedcba".as_ptr(),
        c"abcdefghijklmnopqrrqponmlkjihgfedcba".as_ptr(),
        c"abcdefghijklmnopqrrqponmlkjihgfedcba".as_ptr(),
    ], // 11 SLOW PULSE NOT FADE TO BLACK
    [
        c"mkigegik".as_ptr(),
        c"mkigegik".as_ptr(),
        c"mkigegik".as_ptr(),
    ], // 12 FAST PULSE FOR JEREMY
    [
        c"abcdefghijklmqrstuvwxyz".as_ptr(),
        c"zyxwvutsrqmlkjihgfedcba".as_ptr(),
        c"aammbbzzccllcckkffyyggp".as_ptr(),
    ], // 13 Test Blending
    [c"".as_ptr(), c"".as_ptr(), c"".as_ptr()],    // 14
    [c"".as_ptr(), c"".as_ptr(), c"".as_ptr()],    // 15
    [c"".as_ptr(), c"".as_ptr(), c"".as_ptr()],    // 16
    [c"".as_ptr(), c"".as_ptr(), c"".as_ptr()],    // 17
    [c"".as_ptr(), c"".as_ptr(), c"".as_ptr()],    // 18
    [c"".as_ptr(), c"".as_ptr(), c"".as_ptr()],    // 19
    [c"".as_ptr(), c"".as_ptr(), c"".as_ptr()],    // 20
    [c"".as_ptr(), c"".as_ptr(), c"".as_ptr()],    // 21
    [c"".as_ptr(), c"".as_ptr(), c"".as_ptr()],    // 22
    [c"".as_ptr(), c"".as_ptr(), c"".as_ptr()],    // 23
    [c"".as_ptr(), c"".as_ptr(), c"".as_ptr()],    // 24
    [c"".as_ptr(), c"".as_ptr(), c"".as_ptr()],    // 25
    [c"".as_ptr(), c"".as_ptr(), c"".as_ptr()],    // 26
    [c"".as_ptr(), c"".as_ptr(), c"".as_ptr()],    // 27
    [c"".as_ptr(), c"".as_ptr(), c"".as_ptr()],    // 28
    [c"".as_ptr(), c"".as_ptr(), c"".as_ptr()],    // 29
    [c"".as_ptr(), c"".as_ptr(), c"".as_ptr()],    // 30
    [c"".as_ptr(), c"".as_ptr(), c"".as_ptr()],    // 31
];
