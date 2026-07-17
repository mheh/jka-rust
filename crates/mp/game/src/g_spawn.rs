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
use std::ffi::{CStr, CString};

use crate::prelude::*;
use crate::trap;
use crate::world::GameContext;

use crate::g_items::G_SpawnItem;
use crate::g_main::{G_Error, G_Printf};
use crate::g_mem::G_Alloc;
use crate::g_misc::{SP_info_notnull, SP_info_null};
use crate::g_utils::{G_FreeEntity, G_SetOrigin, G_SoundIndex, G_SoundSetIndex, G_Spawn};
use crate::q_shared::{Q_stricmp, Q_strncmp};
use crate::NPC_utils::G_ActivateBehavior;
use mp_bg::bg_misc::{BG_FindItem, BG_ParseField};
use mp_bg::bg_panimate::BG_ParseAnimationFile;

use crate::ent_fn_enums::EntThink;
use mp_abi::game::syscalls::G_GET_ENTITY_TOKEN::GGetEntityTokenArgs;
use mp_abi::game::syscalls::G_ICARUS_INITENT::GIcarusInitentArgs;
use mp_abi::game::syscalls::G_ICARUS_VALIDENT::GIcarusValidentArgs;

// Missing trap Args types - will be resolved by integration
use mp_abi::game::syscalls::G_CVAR_SET::GCvarSetArgs;
use mp_abi::game::syscalls::G_G2_ADDBOLT::GG2AddboltArgs;
use mp_abi::game::syscalls::G_G2_INITGHOUL2MODEL::GG2Initghoul2ModelArgs;
use mp_abi::game::syscalls::G_G2_SETBOLTINFO::GG2SetboltinfoArgs as GG2SetBoltInfoArgs;
use mp_abi::game::syscalls::G_G2_SETSKIN::GG2SetskinArgs as GG2SetSkinArgs;
use mp_abi::game::syscalls::G_R_REGISTERSKIN::GRRegisterskinArgs as GR_RegisterSkinArgs;
use mp_abi::game::syscalls::G_SET_CONFIGSTRING::GSetConfigstringArgs;
use mp_abi::game::syscalls::G_SET_SERVER_CULL::GSetServerCullArgs;

use mp_bg::public::fieldtype::fieldtype_t;

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

pub fn G_SpawnString(
    ctx: &mut GameContext,
    key: *const c_char,
    defaultString: *const c_char,
    out: *mut *mut c_char,
) -> qboolean {
    unsafe {
        let level = &ctx.world.level;

        if level.spawning == qfalse {
            *out = defaultString as *mut c_char;
            // G_Error(ctx,  "G_SpawnString(ctx) called while not spawning" ) — commented
            // out in Raven itself; preserved as a no-op per the source.
        }

        for i in 0..level.numSpawnVars {
            if Q_stricmp(key, level.spawnVars[i as usize][0]) == 0 {
                *out = level.spawnVars[i as usize][1];
                return qtrue;
            }
        }

        *out = defaultString as *mut c_char;
        qfalse
    }
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
        let mut s: *mut c_char = std::ptr::null_mut();
        let present = G_SpawnString(ctx, key, defaultString, &mut s);
        *out = atof(s) as f32;
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
        let mut s: *mut c_char = std::ptr::null_mut();
        let present = G_SpawnString(ctx, key, defaultString, &mut s);
        *out = atoi(s);
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
        let mut s: *mut c_char = std::ptr::null_mut();
        let present = G_SpawnString(ctx, key, defaultString, &mut s);
        // Unmatched components are left as whatever `*out` already held
        // (porting-rules §19) — read the current 3 floats, let `sscanf_3f`
        // overwrite only the ones libc `sscanf` would have matched.
        let mut vec: [f32; 3] = [*out.add(0), *out.add(1), *out.add(2)];
        sscanf_3f(s, &mut vec);
        *out.add(0) = vec[0];
        *out.add(1) = vec[1];
        *out.add(2) = vec[2];
        present
    }
}

// ---------------------------------------------------------------------
// Local helpers mirroring libc semantics used throughout this file
// (`atoi`/`sscanf("%f %f %f", ...)` — house rule: libc/other symbols use the
// Rust std equivalent, no resolved signature needed). `atof` itself now
// routes through `mp_bg::bg_lib::atof` (verified faithful to the oracle DLL's
// linked Raven `atof`, `bg_lib.c:774-839`); `sscanf_3f`/`sscanf_1f` route
// through the shared libc-`%f` scanner `cstr_util::sscanf_f32s`.
// ---------------------------------------------------------------------

/// `sscanf(s, "%f %f %f", &out[0], &out[1], &out[2])` via the shared
/// libc-`%f`-faithful scanner. Unmatched components are left at whatever
/// value `out` already held (porting-rules §19) — callers pre-seed `out`
/// before calling.
unsafe fn sscanf_3f(s: *const c_char, out: &mut [f32; 3]) {
    if s.is_null() {
        return;
    }
    let text = CStr::from_ptr(s).to_string_lossy();
    sscanf_f32s(&text, out);
}

/// `sscanf(s, "%f", out)` via the shared libc-`%f`-faithful scanner. Leaves
/// `*out` untouched on a failed match (porting-rules §19).
unsafe fn sscanf_1f(s: *const c_char, out: &mut f32) {
    if s.is_null() {
        return;
    }
    let text = CStr::from_ptr(s).to_string_lossy();
    sscanf_f32s(&text, std::slice::from_mut(out));
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
        let mut value: *mut c_char = std::ptr::null_mut();
        G_SpawnString(ctx, c"teamfilter".as_ptr(), c"".as_ptr(), &mut value);

        let origin = ctx.entity(id).s.origin;
        G_SetOrigin(ctx.entity_mut(id), origin);

        // If a team filter is set then override any team settings for the spawns
        let mut team: c_int = -1;
        let mTeamFilter = ctx.world.level.mTeamFilter.as_ptr();
        if *mTeamFilter != 0 {
            if Q_stricmp(mTeamFilter, c"red".as_ptr()) == 0 {
                team = TEAM_RED;
            } else if Q_stricmp(mTeamFilter, c"blue".as_ptr()) == 0 {
                team = TEAM_BLUE;
            }
        }

        let mut item: Option<ItemId> = None;
        let targetname_ptr = ctx.entity(id).targetname;
        if !targetname_ptr.is_null() && *targetname_ptr != 0 {
            let targetname = CStr::from_ptr(targetname_ptr).to_string_lossy();
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
                ctx.entity_mut(id).targetname = std::ptr::null_mut();
                ctx.entity_mut(id).classname = item.classname_cstr() as *mut c_char;
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
    unsafe {
        if ctx.entity(id).classname.is_null() {
            G_Printf(ctx, c"G_CallSpawn: NULL classname\n".as_ptr());
            return qfalse;
        }

        // check item spawn functions
        let ent_classname = CStr::from_ptr(ctx.entity(id).classname);
        let mut i: c_int = 1;
        while i < bg_numItems {
            let item = ItemId::from_modelindex(i).unwrap();
            // Raven matches items with case-sensitive `strcmp`, not `Q_stricmp`.
            if item.item().classname.as_bytes() == ent_classname.to_bytes() {
                G_SpawnItem(ctx, id, item);
                return qtrue;
            }
            i += 1;
        }

        // check normal spawn functions
        let classname = std::ffi::CStr::from_ptr(ctx.entity(id).classname).to_string_lossy();
        if let Some(sp) = crate::ent_fn_enums::spawn_for_classname(&classname) {
            let healingsound = ctx.entity(id).healingsound;
            if !healingsound.is_null() && *healingsound != 0 {
                //yeah...this can be used for anything, so.. precache it if it's there
                G_SoundIndex(healingsound);
            }
            let ent_ptr = ctx.entity_mut(id) as *mut gentity_t;
            crate::ent_fn_enums::dispatch_spawn(ctx, sp, ent_ptr);
            return qtrue;
        }
        let classname_disp = CStr::from_ptr(ctx.entity(id).classname).to_string_lossy();
        G_Printf(
            ctx,
            cstr(&format!(
                "{} doesn't have a spawn function\n",
                classname_disp
            ))
            .as_ptr(),
        );
        qfalse
    }
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
    field(
        c"roffname",
        core::mem::offset_of!(gentity_t, roffname),
        fieldtype_t::F_LSTRING,
    ),
    field(
        c"rofftarget",
        core::mem::offset_of!(gentity_t, rofftarget),
        fieldtype_t::F_LSTRING,
    ),
    field(
        c"healingclass",
        core::mem::offset_of!(gentity_t, healingclass),
        fieldtype_t::F_LSTRING,
    ),
    field(
        c"healingsound",
        core::mem::offset_of!(gentity_t, healingsound),
        fieldtype_t::F_LSTRING,
    ),
    field(
        c"healingrate",
        core::mem::offset_of!(gentity_t, healingrate),
        fieldtype_t::F_INT,
    ),
    field(
        c"ownername",
        core::mem::offset_of!(gentity_t, ownername),
        fieldtype_t::F_LSTRING,
    ),
    field(
        c"origin",
        core::mem::offset_of!(gentity_t, s) + core::mem::offset_of!(entityState_t, origin),
        fieldtype_t::F_VECTOR,
    ),
    field(
        c"model",
        core::mem::offset_of!(gentity_t, model),
        fieldtype_t::F_LSTRING,
    ),
    field(
        c"model2",
        core::mem::offset_of!(gentity_t, model2),
        fieldtype_t::F_LSTRING,
    ),
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
    field(
        c"target",
        core::mem::offset_of!(gentity_t, target),
        fieldtype_t::F_LSTRING,
    ),
    field(
        c"target2",
        core::mem::offset_of!(gentity_t, target2),
        fieldtype_t::F_LSTRING,
    ),
    field(
        c"target3",
        core::mem::offset_of!(gentity_t, target3),
        fieldtype_t::F_LSTRING,
    ),
    field(
        c"target4",
        core::mem::offset_of!(gentity_t, target4),
        fieldtype_t::F_LSTRING,
    ),
    field(
        c"target5",
        core::mem::offset_of!(gentity_t, target5),
        fieldtype_t::F_LSTRING,
    ),
    field(
        c"target6",
        core::mem::offset_of!(gentity_t, target6),
        fieldtype_t::F_LSTRING,
    ),
    field(
        c"NPC_targetname",
        core::mem::offset_of!(gentity_t, NPC_targetname),
        fieldtype_t::F_LSTRING,
    ),
    field(
        c"NPC_target",
        core::mem::offset_of!(gentity_t, NPC_target),
        fieldtype_t::F_LSTRING,
    ),
    field(
        c"NPC_target2",
        core::mem::offset_of!(gentity_t, target2),
        fieldtype_t::F_LSTRING,
    ), // NPC_spawner only
    field(
        c"NPC_target4",
        core::mem::offset_of!(gentity_t, target4),
        fieldtype_t::F_LSTRING,
    ), // NPC_spawner only
    field(
        c"NPC_type",
        core::mem::offset_of!(gentity_t, NPC_type),
        fieldtype_t::F_LSTRING,
    ),
    field(
        c"targetname",
        core::mem::offset_of!(gentity_t, targetname),
        fieldtype_t::F_LSTRING,
    ),
    field(
        c"message",
        core::mem::offset_of!(gentity_t, message),
        fieldtype_t::F_LSTRING,
    ),
    field(
        c"team",
        core::mem::offset_of!(gentity_t, team),
        fieldtype_t::F_LSTRING,
    ),
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
    field(
        c"targetShaderName",
        core::mem::offset_of!(gentity_t, targetShaderName),
        fieldtype_t::F_LSTRING,
    ),
    field(
        c"targetShaderNewName",
        core::mem::offset_of!(gentity_t, targetShaderNewName),
        fieldtype_t::F_LSTRING,
    ),
    field(
        c"linear",
        core::mem::offset_of!(gentity_t, alt_fire),
        fieldtype_t::F_INT,
    ), // for movers to use linear movement
    field(
        c"closetarget",
        core::mem::offset_of!(gentity_t, closetarget),
        fieldtype_t::F_LSTRING,
    ), // for doors
    field(
        c"opentarget",
        core::mem::offset_of!(gentity_t, opentarget),
        fieldtype_t::F_LSTRING,
    ), // for doors
    field(
        c"paintarget",
        core::mem::offset_of!(gentity_t, paintarget),
        fieldtype_t::F_LSTRING,
    ), // for doors
    field(
        c"goaltarget",
        core::mem::offset_of!(gentity_t, goaltarget),
        fieldtype_t::F_LSTRING,
    ), // for siege
    field(
        c"idealclass",
        core::mem::offset_of!(gentity_t, idealclass),
        fieldtype_t::F_LSTRING,
    ), // for siege spawnpoints
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
    field(
        c"soundSet",
        core::mem::offset_of!(gentity_t, soundSet),
        fieldtype_t::F_LSTRING,
    ),
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
    }
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
        let ent = G_Spawn(ctx);

        let mut callbacks = crate::bg_channel::GameCallbacksImpl {
            // SEAM-BG-REENTRY (DEC-28, sanctioned) — GameCallbacksImpl.world is a `*mut GameWorld`
            // field; a raw store is required (bg-seam re-entry).
            world: ctx.world_raw(),
            engine: ctx.engine,
        };
        let num_spawn_vars = ctx.world.level.numSpawnVars;
        for i in 0..num_spawn_vars {
            let key = ctx.world.level.spawnVars[i as usize][0];
            let value = ctx.world.level.spawnVars[i as usize][1];
            BG_ParseField(
                &mut callbacks,
                FIELDS.as_ptr() as *mut BG_field_t,
                key,
                value,
                ent as *mut byte,
            );
        }

        // check for "notsingle" flag
        let mut i: c_int = 0;
        if ctx.world.cvars.g_gametype.integer == GT_SINGLE_PLAYER {
            G_SpawnInt(ctx, c"notsingle".as_ptr(), c"0".as_ptr(), &mut i);
            if i != 0 {
                G_FreeEntity(ctx, ctx.entity_id_of(ent));
                return;
            }
        }
        // check for "notteam" flag (GT_FFA, GT_DUEL, GT_SINGLE_PLAYER)
        if ctx.world.cvars.g_gametype.integer >= GT_TEAM {
            G_SpawnInt(ctx, c"notteam".as_ptr(), c"0".as_ptr(), &mut i);
            if i != 0 {
                G_FreeEntity(ctx, ctx.entity_id_of(ent));
                return;
            }
        } else {
            G_SpawnInt(ctx, c"notfree".as_ptr(), c"0".as_ptr(), &mut i);
            if i != 0 {
                G_FreeEntity(ctx, ctx.entity_id_of(ent));
                return;
            }
        }

        G_SpawnInt(ctx, c"notta".as_ptr(), c"0".as_ptr(), &mut i);
        if i != 0 {
            G_FreeEntity(ctx, ctx.entity_id_of(ent));
            return;
        }

        let mut value: *mut c_char = std::ptr::null_mut();
        if G_SpawnString(ctx, c"gametype".as_ptr(), std::ptr::null(), &mut value) != qfalse {
            let gt = ctx.world.cvars.g_gametype.integer;
            if gt >= GT_FFA && gt < GT_MAX_GAME_TYPE {
                let gametype_name = GAMETYPE_NAMES[gt as usize];
                let value_str = CStr::from_ptr(value).to_string_lossy();
                if !value_str.contains(gametype_name.to_str().unwrap()) {
                    G_FreeEntity(ctx, ctx.entity_id_of(ent));
                    return;
                }
            }
        }

        // move editor origin to pos
        let id = ctx.entity_id_of(ent).unwrap();
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

            let classname = ctx.entity(id).classname;
            if !classname.is_null() && *classname != 0 {
                if Q_strncmp(c"NPC_".as_ptr(), classname, 4) != 0 {
                    // Not an NPC_spawner (rww - probably don't even care for MP, but whatever)
                    G_ActivateBehavior(ctx, Some(id), BSET_SPAWN);
                }
            }
        }
    }
}

/// Raven `G_AddSpawnVarToken`.
///
/// Source: `oracle/codemp/game/g_spawn.c:851-866`
pub fn G_AddSpawnVarToken(ctx: &mut GameContext, string: *const c_char) -> *mut c_char {
    unsafe {
        let mut l: isize = 0;
        while *string.offset(l) != 0 {
            l += 1;
        }

        let level = &mut ctx.world.level;
        if level.numSpawnVarChars + (l as c_int) + 1 > mp_bg::MAX_SPAWN_VARS_CHARS as c_int {
            // G_Error(ctx,  "G_AddSpawnVarToken: MAX_SPAWN_CHARS" ) — fatal, panic
            // per frozen Group A (Com_Error/G_Error -> panic).
            panic!("G_AddSpawnVarToken: MAX_SPAWN_CHARS");
        }

        let dest = level
            .spawnVarChars
            .as_mut_ptr()
            .offset(level.numSpawnVarChars as isize);
        std::ptr::copy_nonoverlapping(string, dest, (l + 1) as usize);

        level.numSpawnVarChars += l as c_int + 1;

        dest
    }
}

/// Raven `AddSpawnField`.
///
/// Source: `oracle/codemp/game/g_spawn.c:868-884`
pub fn AddSpawnField(ctx: &mut GameContext, field: *mut c_char, value: *mut c_char) {
    let num_spawn_vars = ctx.world.level.numSpawnVars;
    for i in 0..num_spawn_vars {
        if Q_stricmp(ctx.world.level.spawnVars[i as usize][0], field) == 0 {
            let token = G_AddSpawnVarToken(ctx, value);
            ctx.world.level.spawnVars[i as usize][1] = token;
            return;
        }
    }

    let n = ctx.world.level.numSpawnVars;
    let key_tok = G_AddSpawnVarToken(ctx, field);
    let val_tok = G_AddSpawnVarToken(ctx, value);
    ctx.world.level.spawnVars[n as usize][0] = key_tok;
    ctx.world.level.spawnVars[n as usize][1] = val_tok;
    ctx.world.level.numSpawnVars += 1;
}

pub const NOVALUE: &CStr = c"novalue";

/// Raven `HandleEntityAdjustment` (file-static) — sub-BSP instance origin/
/// angle/name-prefix rewriting.
///
/// Source: `oracle/codemp/game/g_spawn.c:888-1006`
fn HandleEntityAdjustment(ctx: &mut GameContext) {
    unsafe {
        let mut value: *mut c_char = std::ptr::null_mut();
        let mut new_origin: vec3_t = [0.0; 3];

        G_SpawnString(ctx, c"origin".as_ptr(), NOVALUE.as_ptr(), &mut value);
        // `origin` is pre-seeded 0.0 (matching the else-branch below); any
        // component `sscanf_3f` fails to match is left at that seed rather
        // than picking up C's stack garbage (porting-rules §19).
        let mut origin: vec3_t = [0.0, 0.0, 0.0];
        if Q_stricmp(value, NOVALUE.as_ptr()) != 0 {
            sscanf_3f(value, &mut origin);
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
        let temp_c = CString::new(temp).unwrap();
        AddSpawnField(
            ctx,
            c"origin".as_ptr() as *mut c_char,
            temp_c.as_ptr() as *mut c_char,
        );

        G_SpawnString(ctx, c"angles".as_ptr(), NOVALUE.as_ptr(), &mut value);
        if Q_stricmp(value, NOVALUE.as_ptr()) != 0 {
            let mut angles: vec3_t = [0.0, 0.0, 0.0];
            sscanf_3f(value, &mut angles);

            // `fmod` is a double-precision truncated remainder whose sign follows
            // the dividend; `rem_euclid` (least non-negative) differs by 360 for a
            // negative sum.
            angles[1] = ((angles[1] + ctx.world.level.mRotationAdjust) as f64 % 360.0) as f32;
            let temp = format!("{:.0} {:.0} {:.0}", angles[0], angles[1], angles[2]);
            let temp_c = CString::new(temp).unwrap();
            AddSpawnField(
                ctx,
                c"angles".as_ptr() as *mut c_char,
                temp_c.as_ptr() as *mut c_char,
            );
        } else {
            G_SpawnString(ctx, c"angle".as_ptr(), NOVALUE.as_ptr(), &mut value);
            let mut angle1: f32 = 0.0;
            if Q_stricmp(value, NOVALUE.as_ptr()) != 0 {
                sscanf_1f(value, &mut angle1);
            }
            angle1 = ((angle1 + ctx.world.level.mRotationAdjust) as f64 % 360.0) as f32;
            let temp = format!("{:.0}", angle1);
            let temp_c = CString::new(temp).unwrap();
            AddSpawnField(
                ctx,
                c"angle".as_ptr() as *mut c_char,
                temp_c.as_ptr() as *mut c_char,
            );
        }

        // RJR experimental code for handling "direction" field of breakable
        // brushes, though direction is rarely ever used.
        G_SpawnString(ctx, c"direction".as_ptr(), NOVALUE.as_ptr(), &mut value);
        let mut direction: vec3_t = [0.0, 0.0, 0.0];
        if Q_stricmp(value, NOVALUE.as_ptr()) != 0 {
            sscanf_3f(value, &mut direction);
        }
        direction[1] = ((direction[1] + ctx.world.level.mRotationAdjust) as f64 % 360.0) as f32;
        let temp = format!(
            "{:.0} {:.0} {:.0}",
            direction[0], direction[1], direction[2]
        );
        let temp_c = CString::new(temp).unwrap();
        AddSpawnField(
            ctx,
            c"direction".as_ptr() as *mut c_char,
            temp_c.as_ptr() as *mut c_char,
        );

        let target_adjust = ctx.world.level.mTargetAdjust;
        let target_adjust_str = if target_adjust.is_null() {
            String::new()
        } else {
            CStr::from_ptr(target_adjust).to_string_lossy().into_owned()
        };

        AddSpawnField(ctx, c"BSPInstanceID".as_ptr() as *mut c_char, target_adjust);

        for (key, out_key) in [
            (c"targetname", c"targetname"),
            (c"target", c"target"),
            (c"killtarget", c"killtarget"),
            (c"brushparent", c"brushparent"),
            (c"brushchild", c"brushchild"),
            (c"enemy", c"enemy"),
            (c"ICARUSname", c"ICARUSname"),
        ] {
            G_SpawnString(ctx, key.as_ptr(), NOVALUE.as_ptr(), &mut value);
            if Q_stricmp(value, NOVALUE.as_ptr()) != 0 {
                let value_str = CStr::from_ptr(value).to_string_lossy();
                let temp = format!("{}{}", target_adjust_str, value_str);
                let temp_c = CString::new(temp).unwrap();
                AddSpawnField(
                    ctx,
                    out_key.as_ptr() as *mut c_char,
                    temp_c.as_ptr() as *mut c_char,
                );
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
    let mut keyname = [0 as c_char; MAX_TOKEN_CHARS];
    let mut com_token = [0 as c_char; MAX_TOKEN_CHARS];

    ctx.world.level.numSpawnVars = 0;
    ctx.world.level.numSpawnVarChars = 0;

    // parse the opening brace
    if trap::GetEntityToken(
        ctx.engine,
        GGetEntityTokenArgs::new(com_token.as_mut_ptr(), MAX_TOKEN_CHARS as c_int),
    ) == qfalse
    {
        // end of spawn string
        return qfalse;
    }
    if com_token[0] != b'{' as c_char {
        panic!("G_ParseSpawnVars: found {{ ... }} mismatch"); // G_Error -> panic (frozen Group A)
    }

    // go through all the key / value pairs
    loop {
        // parse key
        if trap::GetEntityToken(
            ctx.engine,
            GGetEntityTokenArgs::new(keyname.as_mut_ptr(), MAX_TOKEN_CHARS as c_int),
        ) == qfalse
        {
            panic!("G_ParseSpawnVars: EOF without closing brace");
        }

        if keyname[0] == b'}' as c_char {
            break;
        }

        // parse value
        if trap::GetEntityToken(
            ctx.engine,
            GGetEntityTokenArgs::new(com_token.as_mut_ptr(), MAX_TOKEN_CHARS as c_int),
        ) == qfalse
        {
            panic!("G_ParseSpawnVars: EOF without closing brace");
        }

        if com_token[0] == b'}' as c_char {
            panic!("G_ParseSpawnVars: closing brace without data");
        }
        if ctx.world.level.numSpawnVars == mp_bg::MAX_SPAWN_VARS as c_int {
            panic!("G_ParseSpawnVars: MAX_SPAWN_VARS");
        }
        let n = ctx.world.level.numSpawnVars;
        let key_tok = G_AddSpawnVarToken(ctx, keyname.as_ptr() as *const c_char);
        let val_tok = G_AddSpawnVarToken(ctx, com_token.as_ptr() as *const c_char);
        ctx.world.level.spawnVars[n as usize][0] = key_tok;
        ctx.world.level.spawnVars[n as usize][1] = val_tok;
        ctx.world.level.numSpawnVars += 1;
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
        let mut text: *mut c_char = std::ptr::null_mut();
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

        G_SpawnString(ctx, c"classname".as_ptr(), c"".as_ptr(), &mut text);
        if Q_stricmp(text, c"worldspawn".as_ptr()) != 0 {
            G_Error(
                ctx,
                c"SP_worldspawn: The first entity isn't 'worldspawn'".as_ptr(),
            );
        }

        let mut callbacks = crate::bg_channel::GameCallbacksImpl {
            // SEAM-BG-REENTRY (DEC-28, sanctioned) — GameCallbacksImpl.world is a `*mut GameWorld`
            // field; a raw store is required (bg-seam re-entry).
            world: ctx.world_raw(),
            engine: ctx.engine,
        };
        for i in 0..ctx.world.level.numSpawnVars {
            if Q_stricmp(
                c"spawnscript".as_ptr(),
                ctx.world.level.spawnVars[i as usize][0],
            ) == 0
            {
                let field_key = ctx.world.level.spawnVars[i as usize][0];
                let field_value = ctx.world.level.spawnVars[i as usize][1];
                let ent_base = ctx.world.g_entities.as_mut_ptr() as *mut byte;
                // Only let them set spawnscript, we don't want them setting an angle or something on the world.
                BG_ParseField(
                    &mut callbacks,
                    FIELDS.as_ptr() as *mut BG_field_t,
                    field_key,
                    field_value,
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
                GG2Initghoul2ModelArgs::new(
                    &mut ctx.world.globals.precachedKyle as *mut *mut c_void,
                    c"models/players/kyle/model.glm".to_owned(),
                    0,
                    0,
                    -20,
                    0,
                    0,
                ),
            );

            if !ctx.world.globals.precachedKyle.is_null() {
                defSkin = trap::R_RegisterSkin(
                    ctx.engine,
                    GR_RegisterSkinArgs::new(c"models/players/kyle/model_default.skin".to_owned()),
                );
                trap::G2API_SetSkin(
                    ctx.engine,
                    GG2SetSkinArgs::new(ctx.world.globals.precachedKyle, 0, defSkin, defSkin),
                );
            }
        }

        if ctx.world.globals.g2SaberInstance.is_null() {
            trap::G2API_InitGhoul2Model(
                ctx.engine,
                GG2Initghoul2ModelArgs::new(
                    &mut ctx.world.globals.g2SaberInstance as *mut *mut c_void,
                    c"models/weapons2/saber/saber_w.glm".to_owned(),
                    0,
                    0,
                    -20,
                    0,
                    0,
                ),
            );

            if !ctx.world.globals.g2SaberInstance.is_null() {
                // indicate we will be bolted to model 0 (ie the player) on bolt 0 (always the right hand) when we get copied
                trap::G2API_SetBoltInfo(
                    ctx.engine,
                    GG2SetBoltInfoArgs::new(ctx.world.globals.g2SaberInstance, 0, 0),
                );
                // now set up the gun bolt on it
                trap::G2API_AddBolt(
                    ctx.engine,
                    GG2AddboltArgs::new(
                        ctx.world.globals.g2SaberInstance,
                        0,
                        c"*blade1".to_owned(),
                    ),
                );
            }
        }

        if ctx.world.cvars.g_gametype.integer == GT_SIEGE {
            // a tad bit of a hack, but..
            EWebPrecache(ctx);
        }

        // make some data visible to connecting client
        trap::SetConfigstring(
            ctx.engine,
            // `#define GAME_VERSION "basejka-1"`.
            GSetConfigstringArgs::new(CS_GAME_VERSION, c"basejka-1".to_owned()),
        );

        let level_start_time_str = format!("{}", ctx.world.level.startTime);
        let level_start_time_c = CString::new(level_start_time_str).unwrap();
        trap::SetConfigstring(
            ctx.engine,
            GSetConfigstringArgs::new(CS_LEVEL_START_TIME, level_start_time_c),
        );

        G_SpawnString(ctx, c"music".as_ptr(), c"".as_ptr(), &mut text);
        trap::SetConfigstring(
            ctx.engine,
            GSetConfigstringArgs::new(CS_MUSIC, CStr::from_ptr(text).to_owned()),
        );

        G_SpawnString(ctx, c"message".as_ptr(), c"".as_ptr(), &mut text);
        trap::SetConfigstring(
            ctx.engine,
            GSetConfigstringArgs::new(CS_MESSAGE, CStr::from_ptr(text).to_owned()),
        ); // map specific message

        trap::SetConfigstring(
            ctx.engine,
            GSetConfigstringArgs::new(
                CS_MOTD,
                cstr_from_chars(&ctx.world.cvars.g_motd.string).to_owned(),
            ),
        ); // message of the day

        G_SpawnString(ctx, c"gravity".as_ptr(), c"800".as_ptr(), &mut text);
        trap::Cvar_Set(
            ctx.engine,
            GCvarSetArgs::new(c"g_gravity".to_owned(), CStr::from_ptr(text).to_owned()),
        );

        G_SpawnString(ctx, c"enableBreath".as_ptr(), c"0".as_ptr(), &mut text);
        trap::Cvar_Set(
            ctx.engine,
            GCvarSetArgs::new(
                c"g_enableBreath".to_owned(),
                CStr::from_ptr(text).to_owned(),
            ),
        );

        G_SpawnString(ctx, c"soundSet".as_ptr(), c"default".as_ptr(), &mut text);
        trap::SetConfigstring(
            ctx.engine,
            GSetConfigstringArgs::new(
                mp_bg::public::configstring::CS_GLOBAL_AMBIENT_SET,
                CStr::from_ptr(text).to_owned(),
            ),
        );

        ctx.world.g_entities[ENTITYNUM_WORLD as usize].s.number = ENTITYNUM_WORLD;
        ctx.world.g_entities[ENTITYNUM_WORLD as usize].classname =
            c"worldspawn".as_ptr() as *mut c_char;

        // see if we want a warmup time
        trap::SetConfigstring(
            ctx.engine,
            GSetConfigstringArgs::new(CS_WARMUP, c"".to_owned()),
        );
        if ctx.world.cvars.g_restarted.integer != 0 {
            trap::Cvar_Set(
                ctx.engine,
                GCvarSetArgs::new(c"g_restarted".to_owned(), c"0".to_owned()),
            );
            ctx.world.level.warmupTime = 0;
        }

        trap::SetConfigstring(
            ctx.engine,
            GSetConfigstringArgs::new(
                CS_LIGHT_STYLES + (LS_STYLES_START as c_int * 3) as c_int,
                CStr::from_ptr(defaultStyles[0][0]).to_owned(),
            ),
        );
        trap::SetConfigstring(
            ctx.engine,
            GSetConfigstringArgs::new(
                CS_LIGHT_STYLES + (LS_STYLES_START as c_int * 3 + 1) as c_int,
                CStr::from_ptr(defaultStyles[0][1]).to_owned(),
            ),
        );
        trap::SetConfigstring(
            ctx.engine,
            GSetConfigstringArgs::new(
                CS_LIGHT_STYLES + (LS_STYLES_START as c_int * 3 + 2) as c_int,
                CStr::from_ptr(defaultStyles[0][2]).to_owned(),
            ),
        );

        for i in 1..LS_NUM_STYLES {
            let red_key = format!("ls_{}r", i);
            let red_key_c = CString::new(red_key).unwrap();
            G_SpawnString(
                ctx,
                red_key_c.as_ptr(),
                defaultStyles[i as usize][0],
                &mut text,
            );
            lengthRed = (strlen(text)) as i32;
            trap::SetConfigstring(
                ctx.engine,
                GSetConfigstringArgs::new(
                    CS_LIGHT_STYLES + ((i + LS_STYLES_START) as c_int * 3) as c_int,
                    CStr::from_ptr(text).to_owned(),
                ),
            );

            let green_key = format!("ls_{}g", i);
            let green_key_c = CString::new(green_key).unwrap();
            G_SpawnString(
                ctx,
                green_key_c.as_ptr(),
                defaultStyles[i as usize][1],
                &mut text,
            );
            lengthGreen = (strlen(text)) as i32;
            trap::SetConfigstring(
                ctx.engine,
                GSetConfigstringArgs::new(
                    CS_LIGHT_STYLES + ((i + LS_STYLES_START) as c_int * 3 + 1) as c_int,
                    CStr::from_ptr(text).to_owned(),
                ),
            );

            let blue_key = format!("ls_{}b", i);
            let blue_key_c = CString::new(blue_key).unwrap();
            G_SpawnString(
                ctx,
                blue_key_c.as_ptr(),
                defaultStyles[i as usize][2],
                &mut text,
            );
            lengthBlue = (strlen(text)) as i32;
            trap::SetConfigstring(
                ctx.engine,
                GSetConfigstringArgs::new(
                    CS_LIGHT_STYLES + ((i + LS_STYLES_START) as c_int * 3 + 2) as c_int,
                    CStr::from_ptr(text).to_owned(),
                ),
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
    unsafe {
        let mut counted_sets: c_int = 0;

        for i in 0..(mp_qshared::shared::MAX_GENTITIES as usize) {
            let soundSet = ctx.world.g_entities[i].soundSet;

            if ctx.world.g_entities[i].inuse != qfalse && !soundSet.is_null() && *soundSet != 0 {
                if counted_sets >= MAX_AMBIENT_SETS {
                    panic!("MAX_AMBIENT_SETS was exceeded! (too many soundsets)\n");
                    // Com_Error(ERR_DROP, ...) -> panic
                }

                let idx = G_SoundSetIndex(ctx, soundSet);
                ctx.world.g_entities[i].s.soundSetIndex = idx;
                counted_sets += 1;
            }
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
        ctx.world.level.numSpawnVars = 0;

        // the worldspawn is not an actual entity, but it still
        // has a "spawn" function to perform any global setup
        // needed by a level (setting configstrings or cvars, etc)
        if G_ParseSpawnVars(ctx, qfalse) == qfalse {
            G_Error(ctx, c"SpawnEntities: no entities".as_ptr());
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

        if !ctx.world.g_entities[ENTITYNUM_WORLD as usize].behaviorSet[BSET_SPAWN as usize]
            .is_null()
            && *ctx.world.g_entities[ENTITYNUM_WORLD as usize].behaviorSet[BSET_SPAWN as usize] != 0
        {
            // World has a spawn script, but we don't want the world in ICARUS and running scripts,
            // so make a scriptrunner and start it going.
            let script_runner = G_Spawn(ctx);
            if !script_runner.is_null() {
                let id = ctx.entity_id_of(script_runner).unwrap();
                let world_bset =
                    ctx.world.g_entities[ENTITYNUM_WORLD as usize].behaviorSet[BSET_SPAWN as usize];
                let next_think = ctx.world.level.time + 100;
                {
                    let e = ctx.world.entity_mut(id);
                    e.behaviorSet[1] = world_bset;
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
