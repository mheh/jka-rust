// PORT-COMPLETE: g_utils.c
//! FAITHFUL port of `oracle/codemp/game/g_utils.c`.
//!
//! Filled by the jampgame mega-pass. Pass-2 resolved the ctx threading and the
//! vec3 out-param reshape, so most stateful/vec3 functions are now
//! ported (backfilling four `GameGlobals` `()` placeholders along the way:
//! `remappedShaders`/`gClPtrs`/`gG2KillIndex`/`g_vehiclePoolOccupied`, see
//! `game_globals.rs`). The functions that remain parked block on
//! genuinely-unported dependencies (bg-owned const tables `weaponData`/
//! `ammoData`/`bgSiegeClasses`/`bgAllAnims`; the `CS_*` configstring
//! wire-index chain; `SVF_PLAYER_USABLE`; the LCG `rand()`/`Rng` seam;
//! the `g_vehiclePool` storage field; the scratch-buffer-return
//! idiom (`tv`/`vtos`/`BuildShaderStateConfig`); or fn-pointer dispatch (`TryUse`'s
//! touch-pointer comparison) per the fn-ID-enum ruling.
#![allow(non_snake_case, unused, clippy::all)]

use crate::prelude::*;
use crate::q_math::{_DotProduct, _VectorMA, _VectorSubtract};
use native_string::Q_stricmp;
use native_string::strncpyz_string;

use crate::client::gclient_t;
use crate::g_main::G_Printf;
use crate::trap;
use mp_abi::game::syscalls::G_ENTITIES_IN_BOX::GEntitiesInBoxArgs;

/// Raven `EV_EVENT_BIT1`.
///
/// Source: `oracle/codemp/game/bg_public.h:728`
pub const EV_EVENT_BIT1: c_int = 0x00000100;

/// Raven `EV_EVENT_BIT2`.
///
/// Source: `oracle/codemp/game/bg_public.h:729`
pub const EV_EVENT_BIT2: c_int = 0x00000200;

/// Raven `EV_EVENT_BITS`.
///
/// Source: `oracle/codemp/game/bg_public.h:730`
pub const EV_EVENT_BITS: c_int = EV_EVENT_BIT1 | EV_EVENT_BIT2;
use mp_abi::game::syscalls::G_G2_CLEANMODELS::GG2CleanmodelsArgs;
use mp_abi::game::syscalls::G_ICARUS_FREEENT::GIcarusFreeentArgs;
use mp_abi::game::syscalls::G_LINKENTITY::GLinkentityArgs;
use mp_abi::game::syscalls::G_LOCATE_GAME_DATA::GLocateGameDataArgs;
use mp_abi::game::syscalls::G_TRACE::GTraceArgs;
use mp_abi::game::syscalls::G_UNLINKENTITY::GUnlinkentityArgs;
use mp_bg::public::entity_event::{entity_event_t, entity_event_t::*};
use mp_qshared::common::mp::game::Q3_INFINITE;
use mp_qshared::common::mp::trace_t::trace_t;
use mp_qshared::shared::trajectory::trType_t;
use std::ffi::{CStr, CString};

// Raven `qboolean` is `c_int`; keep the source spelling at assignment sites.
// Source: `oracle/codemp/game/q_shared.h`

// Raven `#define PMF_TIME_KNOCKBACK 64` (`bg_public.h:409`) and
// `#define SVF_BROADCAST 0x00000020` (`g_local.h`) both resolve through the
// prelude glob now — `PMF_TIME_KNOCKBACK` from
// `mp_qshared::common::mp::qcommon::pm_flags` (`=64`) and `SVF_BROADCAST` from
// `crate::g_public_consts` (`=0x0000_0020`); the earlier local shadow consts
// (which claimed no canonical existed) are dropped in favor of those.
// Source: `oracle/codemp/game/bg_public.h:409`, `g_local.h`
use crate::game_globals::MAX_SHADER_REMAPS;
use mp_bg::bg_misc::snap_vector;

/// Raven `AddRemap`. `remapCount`/`remappedShaders` reached via
/// `ctx.world.globals` (backfilled from the `()` placeholder —
/// see `RemappedShaders` in `game_globals.rs`).
///
/// Source: `oracle/codemp/game/g_utils.c:20-37`
pub fn AddRemap(
    ctx: &mut GameContext,
    oldShader: *const c_char,
    newShader: *const c_char,
    timeOffset: f32,
) {
    // `oldShader`/`newShader` remain seam pointers (entity `targetShaderName*`
    // fields); read them once into owned strings. Raven's `strcpy` into the
    // `MAX_QPATH` fields is bounded here (its C overrun is UB); the compare is
    // `Q_stricmp == 0` (ASCII case-fold).
    let old_shader = unsafe { cstr_to_str(oldShader) };
    let new_shader = unsafe { cstr_to_str(newShader) };
    for i in 0..ctx.world.globals.remapCount as usize {
        if old_shader.eq_ignore_ascii_case(&ctx.world.globals.remappedShaders.0[i].oldShader) {
            ctx.world.globals.remappedShaders.0[i].newShader =
                strncpyz_string(new_shader.as_bytes(), MAX_QPATH as usize);
            ctx.world.globals.remappedShaders.0[i].timeOffset = timeOffset;
            return;
        }
    }
    if (ctx.world.globals.remapCount as usize) < MAX_SHADER_REMAPS {
        let i = ctx.world.globals.remapCount as usize;
        ctx.world.globals.remappedShaders.0[i].newShader =
            strncpyz_string(new_shader.as_bytes(), MAX_QPATH as usize);
        ctx.world.globals.remappedShaders.0[i].oldShader =
            strncpyz_string(old_shader.as_bytes(), MAX_QPATH as usize);
        ctx.world.globals.remappedShaders.0[i].timeOffset = timeOffset;
        ctx.world.globals.remapCount += 1;
    }
}

/// Raven `BuildShaderStateConfig`.
///
/// Source: `oracle/codemp/game/g_utils.c:39-50`
pub fn BuildShaderStateConfig(ctx: &GameContext) -> String {
    // Raven accumulates into a `static char buff[MAX_STRING_CHARS*4]` via
    // `Q_strcat` and returns it by pointer; the owned `String` return makes the
    // scratch buffer unnecessary. The `Q_strcat` total bound
    // (`MAX_STRING_CHARS*4 - 1` bytes) is preserved by truncating the
    // concatenation once at the end (once full, `Q_strcat` is a no-op — so a
    // final truncation is byte-identical to the per-append bound).
    // `MAX_STRING_CHARS` resolves via the crate prelude glob.
    let mut buff = String::new();
    for i in 0..ctx.world.globals.remapCount as usize {
        let old_shader_str = &ctx.world.globals.remappedShaders.0[i].oldShader;
        let new_shader_str = &ctx.world.globals.remappedShaders.0[i].newShader;
        let time_offset = ctx.world.globals.remappedShaders.0[i].timeOffset;

        buff.push_str(&format!("{}={}:{:5.2}@", old_shader_str, new_shader_str, time_offset));
    }
    strncpyz_string(buff.as_bytes(), MAX_STRING_CHARS * 4)
}

/// Raven `G_FindConfigstringIndex`.
///
/// Source: `oracle/codemp/game/g_utils.c:66-95`
pub fn G_FindConfigstringIndex(
    ctx: &mut GameContext,
    name: &str,
    start: c_int,
    max: c_int,
    create: qboolean,
) -> c_int {
    if name.is_empty() {
        return 0;
    }

    // `MAX_STRING_CHARS` resolves via the crate prelude glob.
    let mut i = 1;
    let mut s;
    while i < max {
        s = trap::GetConfigstring(ctx.engine, start + i, MAX_STRING_CHARS);
        if s.is_empty() {
            break;
        }
        // `Q_strcmp` is an exact byte compare; both sides are NUL-free Rust
        // strings, so `==` is byte-identical.
        if s == name {
            return i;
        }
        i += 1;
    }

    if create == qfalse {
        return 0;
    }

    if i == max {
        trap::Error(ctx.engine, "G_FindConfigstringIndex: overflow");
    }

    trap::SetConfigstring(ctx.engine, start + i, name);

    i
}

/// Raven `G_BoneIndex`.
///
/// Source: `oracle/codemp/game/g_utils.c:101-103`
pub fn G_BoneIndex(ctx: &mut GameContext, name: &str) -> c_int {
    G_FindConfigstringIndex(ctx, name, CS_G2BONES, MAX_G2BONES, qtrue)
}

/// Raven `G_ModelIndex`.
///
/// Source: `oracle/codemp/game/g_utils.c:108-130`
pub fn G_ModelIndex(name: &str) -> c_int {
    // Ctx-less bg-callable boundary fn (Raven reaches the engine through the
    // global syscall pointer); engine + world via the `g_strap` seam cells
    // (STAGE-2a: `GameContext::world` is a live `&mut GameWorld`, so it can no
    // longer be left null — `G_FindConfigstringIndex` still only issues trap
    // syscalls and never touches it). Oracle body omits the
    // `#ifdef _DEBUG_MODEL_PATH_ON_SERVER` section (not compiled in release).
    let mut ctx = GameContext {
        world: unsafe { &mut *crate::g_strap::strap_world() },
        engine: crate::g_strap::strap_engine(),
    };
    G_FindConfigstringIndex(&mut ctx, name, CS_MODELS, MAX_MODELS, qtrue)
}

/// Raven `G_IconIndex`.
///
/// Source: `oracle/codemp/game/g_utils.c:132-136`
pub fn G_IconIndex(ctx: &mut GameContext, name: &str) -> c_int {
    debug_assert!(!name.is_empty());
    G_FindConfigstringIndex(ctx, name, CS_ICONS, MAX_ICONS, qtrue)
}

/// Raven `G_SoundIndex`.
///
/// Source: `oracle/codemp/game/g_utils.c:138-141`
pub fn G_SoundIndex(name: &str) -> c_int {
    // Ctx-less boundary fn; engine via the `g_strap` seam cell (see G_ModelIndex).
    debug_assert!(!name.is_empty());
    let mut ctx = GameContext {
        world: unsafe { &mut *crate::g_strap::strap_world() },
        engine: crate::g_strap::strap_engine(),
    };
    G_FindConfigstringIndex(&mut ctx, name, CS_SOUNDS, MAX_SOUNDS, qtrue)
}

/// Raven `G_SoundSetIndex`.
///
/// Source: `oracle/codemp/game/g_utils.c:143-146`
pub fn G_SoundSetIndex(ctx: &mut GameContext, name: &str) -> c_int {
    G_FindConfigstringIndex(ctx, name, CS_AMBIENT_SET, MAX_AMBIENT_SETS, qtrue)
}

/// Raven `G_EffectIndex`.
///
/// Source: `oracle/codemp/game/g_utils.c:148-151`
pub fn G_EffectIndex(name: &str) -> c_int {
    // Ctx-less boundary fn; engine via the `g_strap` seam cell (see G_ModelIndex).
    let mut ctx = GameContext {
        world: unsafe { &mut *crate::g_strap::strap_world() },
        engine: crate::g_strap::strap_engine(),
    };
    G_FindConfigstringIndex(&mut ctx, name, CS_EFFECTS, MAX_FX, qtrue)
}

/// Raven `G_BSPIndex`.
///
/// Source: `oracle/codemp/game/g_utils.c:153-156`
pub fn G_BSPIndex(ctx: &mut GameContext, name: &str) -> c_int {
    G_FindConfigstringIndex(ctx, name, CS_BSP_MODELS, MAX_SUB_BSP, qtrue)
}

/// Raven `G_PlayerHasCustomSkeleton`.
///
/// Raven: the real siege-class-flag body is `#if 0`'d out upstream; the live
/// function unconditionally returns `qfalse` — ported faithfully as-is.
/// Source: `oracle/codemp/game/g_utils.c:162-188`
pub fn G_PlayerHasCustomSkeleton(ent: &gentity_t) -> qboolean {
    // STAGE-1: ctx-free leaf borrow &gentity_t (body ignores `ent` — `#if 0` stub).
    let _ = ent;
    qfalse
}

/// Raven `G_TeamCommand`.
///
/// Source: `oracle/codemp/game/g_utils.c:197-207`
pub fn G_TeamCommand(ctx: &mut GameContext, team: team_t, cmd: *mut c_char) {
    use crate::client::client_connected::CON_CONNECTED;

    unsafe {
        let text = CStr::from_ptr(cmd).to_string_lossy().into_owned();
        for i in 0..ctx.world.level.maxclients {
            let client = &ctx.world.clients[i as usize];
            if client.pers.connected == CON_CONNECTED && client.sess.sessionTeam == team {
                trap::SendServerCommand(ctx.engine, i, &text);
            }
        }
    }
}

/// Which entity string field `G_Find` walks — replaces Raven's `FOFS(x)` byte
/// offset (`G_Find`'s `int fieldofs`), covering the fields the ported tree
/// actually searches by. `Targetname`/`Classname`/`ScriptTargetname` read prefix
/// `*mut c_char` slots; `Target`/`Target2` read the owned `Option<String>` tail
/// fields (door triggers scan them).
///
/// Source: `oracle/codemp/game/g_utils.c:222-243`
pub enum EntFindField {
    /// `targetname` slot (`FOFS(targetname)`).
    Targetname,
    /// `classname` slot (`FOFS(classname)`).
    Classname,
    /// `script_targetname` slot (`FOFS(script_targetname)`).
    ScriptTargetname,
    /// `target` field (`FOFS(target)`).
    Target,
    /// `target2` field (`FOFS(target2)`).
    Target2,
}

/// Raven `G_Find`. `field` selects the prefix slot to match (was `FOFS(x)`); the
/// slot is decoded through the entity's `_str` reader — a NULL slot never
/// matches, preserving the C `if (!s || Q_stricmp(...))` skip.
///
/// Source: `oracle/codemp/game/g_utils.c:222-243`
pub fn G_Find(
    ctx: &mut GameContext,
    from: Option<EntityId>,
    field: EntFindField,
    r#match: &str,
) -> *mut gentity_t {
    // STAGE-1: Option param, raw body re-derived verbatim (Stage-2 debt).
    let from: *mut gentity_t =
        unsafe { crate::ent_id::resolve(ctx.world.g_entities.as_mut_ptr(), from) };
    unsafe {
        let base = ctx.world.g_entities.as_mut_ptr();
        let num_entities = ctx.world.level.num_entities;

        let mut cur = if from.is_null() { base } else { from.add(1) };

        while cur < base.add(num_entities as usize) {
            if (*cur).inuse != qfalse {
                // `classname_str` collapses NULL to `""`; a classname search is
                // never issued with an empty `match`, so that never spuriously
                // hits — identical to the C null-skip for every real input.
                let hit = match field {
                    EntFindField::Classname => Q_stricmp(&(*cur).classname_str(), r#match) == 0,
                    EntFindField::Targetname => (*cur)
                        .targetname_str()
                        .is_some_and(|s| Q_stricmp(&s, r#match) == 0),
                    EntFindField::ScriptTargetname => (*cur)
                        .script_targetname_str()
                        .is_some_and(|s| Q_stricmp(&s, r#match) == 0),
                    EntFindField::Target => (*cur)
                        .target
                        .as_deref()
                        .is_some_and(|s| Q_stricmp(s, r#match) == 0),
                    EntFindField::Target2 => (*cur)
                        .target2
                        .as_deref()
                        .is_some_and(|s| Q_stricmp(s, r#match) == 0),
                };
                if hit {
                    return cur;
                }
            }
            cur = cur.add(1);
        }

        core::ptr::null_mut()
    }
}

/// Raven `G_RadiusList`.
///
/// Source: `oracle/codemp/game/g_utils.c:252-311`
pub fn G_RadiusList(
    ctx: &mut GameContext,
    origin: vec3_t,
    radius: f32,
    ignore: Option<EntityId>,
    takeDamage: qboolean,
    ent_list: *mut *mut gentity_t,
) -> c_int {
    // Safe-state 2c: `ignore` stays an Option<EntityId> handle; `ent_list` is a
    // raw out-array (kept), so matched entities are stored as raw pointers taken
    // from the accessor at the point of use.
    let radius = if radius < 1.0 { 1.0 } else { radius };

    let mut mins = [0.0f32; 3];
    let mut maxs = [0.0f32; 3];
    for i in 0..3 {
        mins[i] = origin[i] - radius;
        maxs[i] = origin[i] + radius;
    }

    let mut entity_list = [0 as c_int; MAX_GENTITIES];
    let num_listed_entities = trap::EntitiesInBox(
        ctx.engine,
        GEntitiesInBoxArgs::new(
            mins.as_ptr() as *const vec3_t,
            maxs.as_ptr() as *const vec3_t,
            entity_list.as_mut_ptr(),
            MAX_GENTITIES as c_int,
        ),
    );

    let mut ent_count: c_int = 0;
    for e in 0..num_listed_entities {
        let id = EntityId(entity_list[e as usize] as u32);
        {
            let ent = ctx.world.entity(id);
            if Some(id) == ignore || ent.inuse == qfalse || ent.takedamage != takeDamage {
                continue;
            }

            let mut v = [0.0f32; 3];
            for i in 0..3 {
                if origin[i] < ent.r.absmin[i] {
                    v[i] = ent.r.absmin[i] - origin[i];
                } else if origin[i] > ent.r.absmax[i] {
                    v[i] = origin[i] - ent.r.absmax[i];
                } else {
                    v[i] = 0.0;
                }
            }

            let dist = VectorLength(v);
            if dist >= radius {
                continue;
            }
        }

        let ent_ptr = ctx.world.entity_mut(id) as *mut gentity_t;
        unsafe {
            *ent_list.add(ent_count as usize) = ent_ptr;
        }
        ent_count += 1;
    }

    ent_count
}

/// Raven `G_Throw`. `targ->client` reached via the house
/// `(*ent).client` cast (see `g_combat.rs`/`g_items.rs`).
///
/// Source: `oracle/codemp/game/g_utils.c:315-370`
pub fn G_Throw(ctx: &mut GameContext, targ: EntityId, newDir: vec3_t, push: f32) {
    // Safe-state 2c: EntityId param, entity fields via accessor borrow.
    let mass = {
        let pb = ctx.world.entity(targ).physicsBounce;
        if pb > 0.0 {
            pb
        } else {
            200.0
        }
    };

    let g_gravity = ctx.world.cvars.g_gravity.value;
    let g_knockback = ctx.world.cvars.g_knockback.value;

    let mut kvel = [0.0f32; 3];
    if g_gravity > 0.0 {
        // C's trailing `* 0.8` / `* 1.5` are unsuffixed double literals, so the
        // scale is formed in f64 and VectorScale's `newDir[i] * scale` multiply
        // is in f64, narrowed once to the f32 kvel.
        for i in 0..3 {
            kvel[i] = (newDir[i] as f64 * ((g_knockback * push / mass) as f64 * 0.8)) as f32;
        }
        // Unlike VectorScale's `newDir[i] * scale`, C folds newDir[2] into the
        // f32 product `newDir[2]*gk*push/mass` (left-assoc, all f32); only the
        // trailing `* 1.5` promotes to f64, narrowing once at the store.
        // Source: `oracle/codemp/game/g_utils.c:333`
        let chain = newDir[2] * g_knockback * push / mass;
        kvel[2] = (chain as f64 * 1.5) as f32;
    } else {
        for i in 0..3 {
            kvel[i] = newDir[i] * (g_knockback * push / mass);
        }
    }

    // FLAG (task #7): NPC pool client (gClPtrs) — not a `level.clients` slot;
    // the pointer is read off the borrow and dereffed raw as Raven does.
    let client = ctx.world.entity(targ).client;
    if !client.is_null() {
        unsafe {
            for i in 0..3 {
                (*client).ps.velocity[i] += kvel[i];
            }
        }
    } else {
        let trType = ctx.world.entity(targ).s.pos.trType;
        if trType != trType_t::TR_STATIONARY
            && trType != trType_t::TR_LINEAR_STOP
            && trType != trType_t::TR_NONLINEAR_STOP
        {
            let level_time = ctx.world.level.time;
            let ent = ctx.world.entity_mut(targ);
            for i in 0..3 {
                ent.s.pos.trDelta[i] += kvel[i];
            }
            ent.s.pos.trBase = ent.r.currentOrigin;
            ent.s.pos.trTime = level_time;
        }
    }

    // set the timer so that the other client can't cancel
    // out the movement immediately
    // FLAG (task #7): NPC pool client — dereffed raw.
    let client = ctx.world.entity(targ).client;
    if !client.is_null() {
        unsafe {
            if (*client).ps.pm_time == 0 {
                let mut t = (push * 2.0) as c_int;
                if t < 50 {
                    t = 50;
                }
                if t > 200 {
                    t = 200;
                }
                (*client).ps.pm_time = t;
                (*client).ps.pm_flags |= PMF_TIME_KNOCKBACK;
            }
        }
    }
}

/// Raven `G_FreeFakeClient`.
///
/// Raven: the dynamic-free path is commented out upstream ("or not, the
/// dynamic stuff is busted somehow at the moment"); the live function body
/// is empty — ported faithfully as a no-op.
/// Source: `oracle/codemp/game/g_utils.c:376-381`
pub fn G_FreeFakeClient(cl: *mut *mut gclient_t) {}

/// Raven `G_AllocateVehicleObject`.
///
/// Source: `oracle/codemp/game/g_utils.c:388-410`
pub fn G_AllocateVehicleObject(ctx: &mut GameContext, pVeh: *mut *mut Vehicle_t) {
    unsafe {
        let mut i: c_int = 0;

        if ctx.world.globals.g_vehiclePoolInit == qfalse {
            ctx.world.globals.g_vehiclePoolInit = qtrue;
            for j in 0..crate::game_globals::MAX_VEHICLES_AT_A_TIME {
                ctx.world.globals.g_vehiclePoolOccupied.0[j] = qfalse;
            }
        }

        while i < crate::game_globals::MAX_VEHICLES_AT_A_TIME as c_int {
            // iterate through and try to find a free one
            if ctx.world.globals.g_vehiclePoolOccupied.0[i as usize] == qfalse {
                ctx.world.globals.g_vehiclePoolOccupied.0[i as usize] = qtrue;
                let slot = &mut ctx.world.globals.g_vehiclePool.0[i as usize] as *mut Vehicle_t;
                core::ptr::write_bytes(slot, 0, 1);
                *pVeh = slot;
                return;
            }
            i += 1;
        }
        trap::Error(ctx.engine, "Ran out of vehicle pool slots.");
    }
}

/// Raven `G_FreeVehicleObject`.
///
/// Source: `oracle/codemp/game/g_utils.c:413-426`
pub fn G_FreeVehicleObject(ctx: &mut GameContext, pVeh: *mut Vehicle_t) {
    let mut i: c_int = 0;
    while i < crate::game_globals::MAX_VEHICLES_AT_A_TIME as c_int {
        if ctx.world.globals.g_vehiclePoolOccupied.0[i as usize] == qtrue
            && core::ptr::eq(&ctx.world.globals.g_vehiclePool.0[i as usize], pVeh)
        {
            // guess this is it
            ctx.world.globals.g_vehiclePoolOccupied.0[i as usize] = qfalse;
            break;
        }
        i += 1;
    }
}

/// Raven `G_CreateFakeClient`. `gClPtrs[]` reached via `ctx.world.globals`
/// (backfilled from the `()` placeholder — see `GClPtrs` in
/// `game_globals.rs`). `BG_Alloc` is itself still parked (bg-shared
/// allocator); calling it here matches the mechanical dependency chain the
/// rest of this pass follows.
///
/// Source: `oracle/codemp/game/g_utils.c:430-438`
pub fn G_CreateFakeClient(ctx: &mut GameContext, entNum: c_int, cl: *mut *mut gclient_t) {
    unsafe {
        if ctx.world.globals.gClPtrs.0[entNum as usize].is_null() {
            // `gclient_t` holds pointer fields (align 8); pad to an 8-byte
            // boundary first (see `BG_AllocPad8`) so every `(*client).field`
            // access downstream is safely dereferenceable.
            mp_bg::bg_misc::BG_AllocPad8(&mut ctx.world.bg_state);
            ctx.world.globals.gClPtrs.0[entNum as usize] = mp_bg::bg_misc::BG_Alloc(
                core::mem::size_of::<gclient_t>() as c_int,
                &mut ctx.world.bg_state,
            ) as *mut gclient_t;
        }
        *cl = ctx.world.globals.gClPtrs.0[entNum as usize];
    }
}

/// Raven `G_CleanAllFakeClients`.
///
/// Source: `oracle/codemp/game/g_utils.c:450-465`
pub fn G_CleanAllFakeClients(ctx: &mut GameContext) {
    // Safe-state 2c: index-driven accessor borrows.
    let mut i = MAX_CLIENTS as usize;
    while i < mp_qshared::shared::MAX_GENTITIES {
        let id = EntityId(i as u32);
        let (inuse, is_npc, has_client) = {
            let ent = ctx.world.entity(id);
            (
                ent.inuse != qfalse,
                ent.s.eType == ET_NPC as c_int,
                !ent.client.is_null(),
            )
        };
        if inuse && is_npc && has_client {
            let cl = &mut ctx.world.entity_mut(id).client as *mut *mut gclient_t;
            G_FreeFakeClient(cl);
        }
        i += 1;
    }
}

/// Raven `G_SetAnim`.
///
/// Source: `oracle/codemp/game/g_utils.c:479-509`
pub fn G_SetAnim(
    ctx: &mut GameContext,
    ent: EntityId,
    _ucmd: *mut usercmd_t,
    setAnimParts: c_int,
    anim: c_int,
    setAnimFlags: c_int,
    blendTime: c_int,
) {
    // STAGE-1: EntityId param, raw body re-derived verbatim (Stage-2 debt).
    let ent: *mut gentity_t = ctx.entity_mut(ent);
    // Oracle body (the live `#else` "new clean and shining way"): the old
    // `#if 0` pmove path is dead, so `ucmd` is unused (kept for signature parity).
    // `BG_SetAnim` is a `PmoveContext` method (needs `bgAllAnims` off `BgState` +
    // the bg channel handles); build a per-call context from `ctx`, matching the
    // `BG_ParseAnimationFile` game-tier wrapper precedent.
    unsafe {
        debug_assert!(!ent.is_null() && !(*ent).client.is_null());
        let anims = (&ctx.world.bg_state.bgAllAnims)[(*ent).localAnimIndex as usize].anims;
        let ps = &mut (*((*ent).client)).ps as *mut playerState_t;
        let traps = crate::bg_channel::GameBgTraps::new(ctx.engine);
        let mut callbacks = crate::bg_channel::GameCallbacksImpl {
            // SEAM-BG-REENTRY (DEC-28, sanctioned) — GameCallbacksImpl.world is a `*mut GameWorld`
            // field held alongside the `&mut ctx.world.bg_state` borrow below.
            world: ctx.world_raw(),
            engine: ctx.engine,
        };
        let mut pmc =
            crate::bg_channel::PmoveContext::new(&mut ctx.world.bg_state, &traps, &mut callbacks);
        pmc.BG_SetAnim(ps, anims, setAnimParts, anim, setAnimFlags, blendTime);
    }
}

/// Raven `G_PickTarget`.
///
/// Source: `oracle/codemp/game/g_utils.c:521-550`
pub fn G_PickTarget(ctx: &mut GameContext, targetname: Option<&str>) -> *mut gentity_t {
    const MAXCHOICES: usize = 32;
    let mut choice: [*mut gentity_t; MAXCHOICES] = [core::ptr::null_mut(); MAXCHOICES];
    let mut num_choices: usize = 0;

    // `None` ≡ Raven's NULL `targetname` (the `if (!targetname)` guard).
    let Some(targetname) = targetname else {
        G_Printf(ctx, "G_PickTarget called with NULL targetname\n");
        return core::ptr::null_mut();
    };

    unsafe {
        let mut ent: *mut gentity_t = core::ptr::null_mut();
        loop {
            ent = G_Find(ctx, ctx.entity_id_of(ent), EntFindField::Targetname, targetname);
            if ent.is_null() {
                break;
            }
            choice[num_choices] = ent;
            num_choices += 1;
            if num_choices == MAXCHOICES {
                break;
            }
        }

        if num_choices == 0 {
            let msg = format!("G_PickTarget: target {targetname} not found\n");
            G_Printf(ctx, &msg);
            return core::ptr::null_mut();
        }

        let idx = (ctx.world.bg_state.rng.rand() % num_choices as c_int) as usize;
        choice[idx]
    }
}

/// Raven `GlobalUse`.
///
/// Source: `oracle/codemp/game/g_utils.c:552-564`
pub fn GlobalUse(
    ctx: &mut GameContext,
    self_: Option<EntityId>,
    other: Option<EntityId>,
    activator: Option<EntityId>,
) {
    // Safe-state 2c: Option handles; `self_` fields read via the accessor. The
    // fn-enum dispatch still takes raw gentity pointers, taken from the accessor
    // as values at the call (sanctioned seam, not a held reference).
    let Some(self_id) = self_ else {
        return;
    };
    {
        let ent = ctx.world.entity(self_id);
        if (ent.flags & crate::entity::flags::FL_INACTIVE) != 0 {
            return;
        }
        if ent.use_.is_none() {
            return;
        }
    }

    // Oracle: self->use(self, other, activator); (g_utils.c:563)
    if let Some(use_fn) = ctx.world.entity(self_id).use_.get() {
        let self_ptr = ctx.world.entity_mut(self_id) as *mut gentity_t;
        let other_ptr = match other {
            Some(id) => ctx.world.entity_mut(id) as *mut gentity_t,
            None => core::ptr::null_mut(),
        };
        let activator_ptr = match activator {
            Some(id) => ctx.world.entity_mut(id) as *mut gentity_t,
            None => core::ptr::null_mut(),
        };
        crate::ent_fn_enums::dispatch_use(ctx, use_fn, self_ptr, other_ptr, activator_ptr);
    }
}

/// Raven `G_UseTargets2`.
///
/// Source: `oracle/codemp/game/g_utils.c:566-597`
pub fn G_UseTargets2(
    ctx: &mut GameContext,
    ent: Option<EntityId>,
    activator: Option<EntityId>,
    string: Option<&str>,
) {
    // Safe-state 2c: Option handles; `ent`/`t` fields via the accessor.
    let Some(ent_id) = ent else {
        return;
    };

    let (tsn, tsnn) = {
        let e = ctx.world.entity(ent_id);
        (e.targetShaderName.clone(), e.targetShaderNewName.clone())
    };
    // Owned `String` fields now: `""` ≡ absent (was the paired non-NULL check).
    if !tsn.is_empty() && !tsnn.is_empty() {
        // C's `level.time * 0.001`: 0.001 is a double literal, so level.time
        // promotes to f64 and narrows once at the store.
        // Source: `oracle/codemp/game/g_utils.c:574`
        let f = (ctx.world.level.time as f64 * 0.001) as f32;
        let tsn_c = cstr(&tsn);
        let tsnn_c = cstr(&tsnn);
        AddRemap(ctx, tsn_c.as_ptr(), tsnn_c.as_ptr(), f);
        let config = BuildShaderStateConfig(ctx);
        trap::SetConfigstring(ctx.engine, CS_SHADERSTATE, &config);
    }

    // Raven returns early when `string` is NULL or empty — nothing to search.
    let string = match string {
        Some(s) if !s.is_empty() => s,
        _ => return,
    };

    let mut t: *mut gentity_t = core::ptr::null_mut();
    loop {
        t = G_Find(ctx, ctx.entity_id_of(t), EntFindField::Targetname, string);
        if t.is_null() {
            break;
        }
        let t_id = ctx.entity_id_of(t);

        if t_id == Some(ent_id) {
            G_Printf(ctx, "WARNING: Entity used itself.\n");
        } else if !ctx.world.entity(t_id.unwrap()).use_.is_none() {
            GlobalUse(ctx, t_id, Some(ent_id), activator);
        }

        if ctx.world.entity(ent_id).inuse == qfalse {
            G_Printf(ctx, "entity was removed while using targets\n");
            return;
        }
    }
}

/// Raven `G_UseTargets`.
///
/// Thin wrapper: null-checks `ent` then forwards to `G_UseTargets2` with
/// `ent->target` as the search string.
/// Source: `oracle/codemp/game/g_utils.c:609-616`
pub fn G_UseTargets(ctx: &mut GameContext, ent: Option<EntityId>, activator: Option<EntityId>) {
    // Safe-state 2c: Option handles; `ent->target` read via the accessor.
    let Some(ent_id) = ent else {
        return;
    };
    let target = ctx.world.entity(ent_id).target.clone();
    G_UseTargets2(ctx, Some(ent_id), activator, target.as_deref());
}

/// Raven `tv`.
///
/// Source: `oracle/codemp/game/g_utils.c:627-642`
pub fn tv(ctx: &mut GameContext, x: f32, y: f32, z: f32) -> *mut f32 {
    unsafe {
        // Raven's function-local `static int index` / `static vec3_t vecs[8]`
        // now live on `GameWorld.scratch` (safe-state Stage 3); the 8-slot ring
        // rotation is preserved exactly.
        let idx = ctx.world.scratch.tv_index as usize;
        ctx.world.scratch.tv_index = (ctx.world.scratch.tv_index + 1) & 7;

        let v = &mut ctx.world.scratch.tv_vecs[idx];
        v[0] = x;
        v[1] = y;
        v[2] = z;

        v.as_mut_ptr()
    }
}

/// Raven `vtos`.
///
/// Source: `oracle/codemp/game/g_utils.c:653-665`
pub fn vtos(ctx: &mut GameContext, v: vec3_t) -> String {
    // Raven returns one of 8 rotating `static char str[8][32]` buffers so that
    // several `vtos` results survive in one `printf`; an owned `String` per call
    // makes the ring unnecessary. The `Com_sprintf(s, 32, ...)` bound (31 bytes)
    // is preserved. `ctx` is unused now (the ring is gone) but kept so the ~13
    // call sites are untouched.
    let _ = ctx;
    let formatted = format!("({} {} {})", v[0] as c_int, v[1] as c_int, v[2] as c_int);
    strncpyz_string(formatted.as_bytes(), 32)
}

/// Raven `G_SetMovedir`. Reshape: both `angles`/`movedir` are written
/// through in the oracle body, so both become `&mut [f32;3]` out-params (no
/// same-file callers to fix up).
///
/// Source: `oracle/codemp/game/g_utils.c:678-692`
pub fn G_SetMovedir(angles: &mut vec3_t, movedir: &mut vec3_t) {
    const VEC_UP: vec3_t = [0.0, -1.0, 0.0];
    const MOVEDIR_UP: vec3_t = [0.0, 0.0, 1.0];
    const VEC_DOWN: vec3_t = [0.0, -2.0, 0.0];
    const MOVEDIR_DOWN: vec3_t = [0.0, 0.0, -1.0];

    if *angles == VEC_UP {
        *movedir = MOVEDIR_UP;
    } else if *angles == VEC_DOWN {
        *movedir = MOVEDIR_DOWN;
    } else {
        AngleVectors(*angles, Some(movedir), None, None);
    }
    *angles = [0.0, 0.0, 0.0];
}

/// Raven `G_InitGentity`.
///
/// Source: `oracle/codemp/game/g_utils.c:694-702`
pub fn G_InitGentity(ctx: &mut GameContext, e: EntityId) {
    // Safe-state 2c: EntityId param, fields via accessor borrow. Raven's
    // `ent->s.number = ent - g_entities` is the arena index (== `e.index()`).
    {
        let ent = ctx.world.entity_mut(e);
        ent.inuse = qtrue;
        ent.s.number = e.index() as c_int;
        ent.r.ownerNum = mp_qshared::shared::ENTITYNUM_NONE;
        ent.s.modelGhoul2 = 0; // assume not
    }

    // ICARUS information must be added after this point.
    let ent_ptr = ctx.world.entity_mut(e) as *mut gentity_t;
    // `classname` write through the prefix choke point (literal path).
    ctx.ent_set(e, PrefixSet::ClassnameStatic(c"noclass"));
    trap::ICARUS_FreeEnt(ctx.engine, GIcarusFreeentArgs::new(ent_ptr.cast()));
}

/// Raven `G_SpewEntList`.
///
/// Referee build defines neither `FINAL_BUILD` nor `Q3_VM`, so Raven's
/// `#ifndef VM_OR_FINAL_BUILD` `entspew.txt` file log is live there; ported
/// alongside the (always-compiled) `Com_Printf` reporting.
/// Source: `oracle/codemp/game/g_utils.c:705-787`
pub fn G_SpewEntList(ctx: &mut GameContext) {
    unsafe {
        let mut numNPC = 0;
        let mut numProjectile = 0;
        let mut numTempEnt = 0;
        let mut numTempEntST = 0;

        let mut fh: fileHandle_t = 0;
        trap::FS_FOpenFile(ctx.engine, "entspew.txt", &mut fh, FS_WRITE);

        for i in 0..mp_qshared::shared::ENTITYNUM_MAX_NORMAL as usize {
            let ent = &ctx.world.g_entities[i];
            if ent.inuse != qfalse {
                if ent.s.eType == ET_NPC as c_int {
                    numNPC += 1;
                } else if ent.s.eType == ET_MISSILE as c_int {
                    numProjectile += 1;
                } else if ent.freeAfterEvent != qfalse {
                    numTempEnt += 1;
                    if ent.s.eFlags & EF_SOUNDTRACKER != 0 {
                        numTempEntST += 1;
                    }

                    let s = format!(
                        "TEMPENT {:4}: EV {}\n",
                        ent.s.number,
                        ent.s.eType - mp_bg::public::entity_type::entityType_t::ET_EVENTS as c_int
                    );
                    Com_Printf(&s);
                    if fh != 0 {
                        let bytes = s.as_bytes();
                        trap::FS_Write(ctx.engine, bytes, fh);
                    }
                }

                let className = {
                    let c = ent.classname_str();
                    if c.is_empty() {
                        "Unknown".to_string()
                    } else {
                        c
                    }
                };
                let s = format!("ENT {:4}: Classname {}\n", ent.s.number, className);
                Com_Printf(&s);
                if fh != 0 {
                    let bytes = s.as_bytes();
                    trap::FS_Write(ctx.engine, bytes, fh);
                }
            }
        }

        let s = format!(
            "TempEnt count: {}\nTempEnt ST: {}\nNPC count: {}\nProjectile count: {}\n",
            numTempEnt, numTempEntST, numNPC, numProjectile
        );
        Com_Printf(&s);
        if fh != 0 {
            let bytes = s.as_bytes();
            trap::FS_Write(ctx.engine, bytes, fh);
            trap::FS_FCloseFile(ctx.engine, fh);
        }
    }
}

/// Raven `G_Spawn`.
///
/// Source: `oracle/codemp/game/g_utils.c:804-853`
pub fn G_Spawn(ctx: &mut GameContext) -> EntityId {
    unsafe {
        let mut e: *mut gentity_t = core::ptr::null_mut();
        let mut i: c_int = 0;

        for force in 0..2 {
            e = &mut ctx.world.g_entities[MAX_CLIENTS as usize] as *mut gentity_t;
            i = MAX_CLIENTS as c_int;
            while i < ctx.world.level.num_entities {
                if (*e).inuse == qfalse {
                    if force == 0
                        && (*e).freetime > ctx.world.level.startTime + 2000
                        && ctx.world.level.time - (*e).freetime < 1000
                    {
                        i += 1;
                        e = e.add(1);
                        continue;
                    }

                    // reuse this slot
                    let id = ctx.entity_id_of(e).unwrap();
                    G_InitGentity(ctx, id);
                    return id;
                }
                i += 1;
                e = e.add(1);
            }
            if i != mp_qshared::shared::MAX_GENTITIES as c_int {
                break;
            }
        }
        if i == mp_qshared::shared::ENTITYNUM_MAX_NORMAL {
            G_SpewEntList(ctx);
            trap::Error(ctx.engine, "G_Spawn: no free entities");
        }

        // open up a new slot
        ctx.world.level.num_entities += 1;

        // let the server system know that there are more entities
        let entities_base = ctx.world.g_entities.as_mut_ptr();
        let clients_base = &mut ctx.world.clients[0] as *mut gclient_t as *mut playerState_t;
        trap::LocateGameData(
            ctx.engine,
            GLocateGameDataArgs::new(
                entities_base.cast(),
                ctx.world.level.num_entities,
                core::mem::size_of::<gentity_t>() as c_int,
                clients_base,
                core::mem::size_of::<gclient_t>() as c_int,
            ),
        );

        let id = ctx.entity_id_of(e).unwrap();
        G_InitGentity(ctx, id);
        id
    }
}

/// Raven `G_EntitiesFree`.
///
/// Source: `oracle/codemp/game/g_utils.c:860-873`
pub fn G_EntitiesFree(ctx: &mut GameContext) -> qboolean {
    let mut i = MAX_CLIENTS as c_int;
    while i < ctx.world.level.num_entities {
        if ctx.world.g_entities[i as usize].inuse == qfalse {
            return qtrue;
        }
        i += 1;
    }
    qfalse
}

/// Raven `G_SendG2KillQueue`. `gG2KillIndex`/`gG2KillNum` reached via
/// `ctx.world.globals` (backfilled `GG2KillIndex`, see `game_globals.rs`).
///
/// Source: `oracle/codemp/game/g_utils.c:880-907`
pub fn G_SendG2KillQueue(ctx: &mut GameContext) {
    if ctx.world.globals.gG2KillNum == 0 {
        return;
    }

    let mut msg = String::from("kg2");
    let mut i = 0;
    while i < ctx.world.globals.gG2KillNum && i < 64 {
        msg.push_str(&format!(
            " {}",
            ctx.world.globals.gG2KillIndex.0[i as usize]
        ));
        i += 1;
    }

    trap::SendServerCommand(ctx.engine, -1, &msg);

    // Clear the count because we just sent off the whole queue
    ctx.world.globals.gG2KillNum -= i;
    if ctx.world.globals.gG2KillNum < 0 {
        // Raven: "hmm, should be impossible, but I'm paranoid as we're
        // far past beta." `assert(0)` in a debug build; faithfully clamp.
        debug_assert!(false, "gG2KillNum went negative");
        ctx.world.globals.gG2KillNum = 0;
    }
}

/// Raven `G_KillG2Queue`.
///
/// Source: `oracle/codemp/game/g_utils.c:909-923`
pub fn G_KillG2Queue(ctx: &mut GameContext, entNum: c_int) {
    if ctx.world.globals.gG2KillNum >= crate::game_globals::MAX_G2_KILL_QUEUE as c_int {
        // This would be considered a Bad Thing. Since we're out of queue
        // slots, just send it now as a separate command (eats more
        // bandwidth, but we have no choice).
        trap::SendServerCommand(ctx.engine, -1, &format!("kg2 {}", entNum));
        return;
    }

    ctx.world.globals.gG2KillIndex.0[ctx.world.globals.gG2KillNum as usize] = entNum;
    ctx.world.globals.gG2KillNum += 1;
}

/// Raven `G_FreeEntity`. `ed->client` reached via the house
/// `(*ent).client` cast.
///
/// Source: `oracle/codemp/game/g_utils.c:932-1043`
pub fn G_FreeEntity(ctx: &mut GameContext, ed: Option<EntityId>) {
    // Safe-state 2c: EntityId handle; entity fields via the accessor. `ed_ptr` is
    // taken fresh from the accessor for each trap / `write_bytes` needing a raw
    // gentity pointer. FLAG (task #7): the NPC pool client (gClPtrs, g_utils.c:430)
    // is dereffed raw exactly as Raven does — it is not a `level.clients` slot.
    // Raven derefs `ed` unconditionally (NULL → UB); callers are null-tolerant, so
    // treat None as a no-op (rule 19: pick the one defined behavior).
    let Some(ed_id) = ed else {
        return;
    };

    if ctx.world.entity(ed_id).isSaberEntity != qfalse {
        return;
    }

    let ed_ptr = ctx.world.entity_mut(ed_id) as *mut gentity_t;
    trap::UnlinkEntity(ctx.engine, GUnlinkentityArgs::new(ed_ptr.cast())); // unlink from world
    let ed_ptr = ctx.world.entity_mut(ed_id) as *mut gentity_t;
    trap::ICARUS_FreeEnt(ctx.engine, GIcarusFreeentArgs::new(ed_ptr.cast())); // ICARUS information must be added after this point

    if ctx.world.entity(ed_id).neverFree != qfalse {
        return;
    }

    // rww - this may seem a bit hackish, but unfortunately we have no
    // access to anything ghoul2-related on the server and thus must send
    // a message to let the client know he needs to clean up all the g2
    // stuff for this now-removed entity.
    if ctx.world.entity(ed_id).s.modelGhoul2 != 0 {
        let number = ctx.world.entity(ed_id).s.number;
        G_KillG2Queue(ctx, number);
    }

    // And, free the server instance too, if there is one.
    if !ctx.world.entity(ed_id).ghoul2.is_null() {
        let g2 = &mut ctx.world.entity_mut(ed_id).ghoul2 as *mut *mut c_void;
        trap::G2API_CleanGhoul2Models(ctx.engine, GG2CleanmodelsArgs::new(g2));
    }

    if ctx.world.entity(ed_id).s.eType == ET_NPC as c_int
        && !ctx.world.entity(ed_id).m_pVehicle.is_null()
    {
        // tell the "vehicle pool" that this one is now free
        let veh = ctx.world.entity(ed_id).m_pVehicle;
        G_FreeVehicleObject(ctx, veh);
    }

    if ctx.world.entity(ed_id).s.eType == ET_NPC as c_int
        && !ctx.world.entity(ed_id).client.is_null()
    {
        // this "client" structure is one of our dynamically allocated
        // ones, so free the memory. FLAG (task #7): NPC pool client, dereffed raw.
        let client = ctx.world.entity(ed_id).client;
        let mut saberEntNum: c_int = -1;
        unsafe {
            if (*client).ps.saberEntityNum != 0 {
                saberEntNum = (*client).ps.saberEntityNum;
            } else if (*client).saberStoredIndex != 0 {
                saberEntNum = (*client).saberStoredIndex;
            }
        }

        if saberEntNum > 0 && ctx.world.g_entities[saberEntNum as usize].inuse != qfalse {
            ctx.world.g_entities[saberEntNum as usize].neverFree = qfalse;
            // Faithful to Raven's `ent_id_opt` (pointer→index, no ENTITYNUM_NONE
            // filter): a valid in-range slot → `Some(EntityId(n))`.
            G_FreeEntity(ctx, Some(EntityId(saberEntNum as u32)));
        }

        unsafe {
            for i in 0..MAX_SABERS {
                if !(*client).weaponGhoul2[i].is_null() {
                    let have = trap::G2_HaveWeGhoul2Models(
                        ctx.engine,
                        mp_abi::game::syscalls::G_G2_HAVEWEGHOULMODELS::GG2HaveweghoulmodelsArgs::new(
                            (*client).weaponGhoul2[i],
                        ),
                    );
                    if have != qfalse {
                        trap::G2API_CleanGhoul2Models(
                            ctx.engine,
                            GG2CleanmodelsArgs::new(
                                &mut (*client).weaponGhoul2[i] as *mut *mut c_void,
                            ),
                        );
                    }
                }
            }
        }

        let cl = &mut ctx.world.entity_mut(ed_id).client as *mut *mut gclient_t;
        G_FreeFakeClient(cl);
    }

    if ctx.world.entity(ed_id).s.eFlags & EF_SOUNDTRACKER != 0 {
        let ed_number = ctx.world.entity(ed_id).s.number;
        let mut i = 0usize;
        while i < MAX_CLIENTS {
            let id = EntityId(i as u32);
            let inuse = ctx.world.entity(id).inuse != qfalse;
            // FLAG (task #7): pool client possible — read off the borrow, deref raw.
            let client = ctx.world.entity(id).client;
            if inuse && !client.is_null() {
                unsafe {
                    let mut ch = (trackchan_t::TRACK_CHANNEL_NONE as c_int - 50) as usize;
                    while ch < (trackchan_t::NUM_TRACK_CHANNELS as c_int - 50) as usize {
                        if (*client).ps.fd.killSoundEntIndex[ch] == ed_number {
                            (*client).ps.fd.killSoundEntIndex[ch] = 0;
                        }
                        ch += 1;
                    }
                }
            }
            i += 1;
        }

        // make sure clientside loop sounds are killed on the tracker and client
        let trick = ctx.world.entity(ed_id).s.trickedentindex;
        trap::SendServerCommand(ctx.engine, -1, &format!("kls {} {}", trick, ed_number));
    }

    let ed_ptr = ctx.world.entity_mut(ed_id) as *mut gentity_t;
    unsafe {
        // Owned-`String` tail fields are not zero-valid: drop them first
        // (`take_owned_strings`), then the byte-wise zero (Raven
        // `memset(ed, 0, sizeof(*ed))`), then re-seat valid empty `String`s into
        // those now-zeroed slots (`seat_owned_strings`) — never dropping the
        // invalid zeroed image. Mirrors the ClientSpawn dance.
        (*ed_ptr).take_owned_strings();
        core::ptr::write_bytes(ed_ptr, 0, 1);
        gentity_t::seat_owned_strings(ed_ptr);
    }
    // The byte-wise zero above leaves the FnId<EntXxx> handler fields as None by
    // construction (zero == None, std-guaranteed via Option<NonZeroU8>), matching
    // Raven's NULL fn ptrs.
    let level_time = ctx.world.level.time;
    ctx.ent_set(ed_id, PrefixSet::ClassnameStatic(c"freed"));
    let ed = ctx.world.entity_mut(ed_id);
    ed.freetime = level_time;
    ed.inuse = qfalse;
}

/// Raven `G_TempEntity`.
///
/// Source: `oracle/codemp/game/g_utils.c:1054-1077`
pub fn G_TempEntity(ctx: &mut GameContext, origin: vec3_t, event: c_int) -> EntityId {
    let e_id = G_Spawn(ctx);
    let level_time = ctx.world.level.time;
    {
        let ent = ctx.world.entity_mut(e_id);
        ent.s.eType = mp_bg::public::entity_type::entityType_t::ET_EVENTS as c_int + event;
        ent.eventTime = level_time;
        ent.freeAfterEvent = qtrue;
    }
    ctx.ent_set(e_id, PrefixSet::ClassnameStatic(c"tempEntity"));

    let mut snapped = origin;
    snap_vector(&mut snapped); // save network bandwidth
    G_SetOrigin(ctx.world.entity_mut(e_id), snapped);
    // WTF? Why aren't we setting the s.origin? (like below) — cg_events.c
    // code checks origin all over the place!!! Trying to save
    // bandwidth...? (Raven comment, preserved.)

    // find cluster for PVS
    let e_ptr = ctx.world.entity_mut(e_id) as *mut gentity_t;
    trap::LinkEntity(ctx.engine, GLinkentityArgs::new(e_ptr.cast()));

    e_id
}

/// Raven `G_SoundTempEntity`.
///
/// Source: `oracle/codemp/game/g_utils.c:1087-1108`
pub fn G_SoundTempEntity(
    ctx: &mut GameContext,
    origin: vec3_t,
    event: c_int,
    channel: c_int,
) -> EntityId {
    let e_id = G_Spawn(ctx);
    let level_time = ctx.world.level.time;
    {
        let ent = ctx.world.entity_mut(e_id);
        ent.s.eType = mp_bg::public::entity_type::entityType_t::ET_EVENTS as c_int + event;
        ent.inuse = qtrue;
        ent.classname = b"tempEntity\0".as_ptr() as *mut c_char;
        ent.eventTime = level_time;
        ent.freeAfterEvent = qtrue;
    }

    let mut snapped = origin;
    snap_vector(&mut snapped); // save network bandwidth
    G_SetOrigin(ctx.world.entity_mut(e_id), snapped);

    // find cluster for PVS
    let e_ptr = ctx.world.entity_mut(e_id) as *mut gentity_t;
    trap::LinkEntity(ctx.engine, GLinkentityArgs::new(e_ptr.cast()));

    e_id
}

/// Raven `G_ScaleNetHealth`.
///
/// Source: `oracle/codemp/game/g_utils.c:1112-1142`
pub fn G_ScaleNetHealth(self_: &mut gentity_t) {
    // Safe-state 2c: ctx-free leaf mutator — operates directly on the caller's
    // `&mut gentity_t` borrow (no raw re-derive).
    let maxHealth = self_.maxHealth;

    if maxHealth < 1000 {
        // it's good then
        self_.s.maxhealth = maxHealth;
        self_.s.health = self_.health;

        if self_.s.health < 0 {
            // don't let it wrap around
            self_.s.health = 0;
        }
        return;
    }

    // otherwise, scale it down
    self_.s.maxhealth = maxHealth / 100;
    self_.s.health = self_.health / 100;

    if self_.s.health < 0 {
        // don't let it wrap around
        self_.s.health = 0;
    }

    if self_.health > 0 && self_.s.health <= 0 {
        // don't let it scale to 0 if the thing is still not "dead"
        self_.s.health = 1;
    }
}

/// Raven `G_KillBox`. `ent->client` reached via the house
/// `(*ent).client` cast; `G_Damage` is itself the deferred
/// monster fn (`g_combat.rs`) — calling it here matches the mechanical
/// dependency chain the rest of this pass follows.
///
/// Source: `oracle/codemp/game/g_utils.c:1162-1193`
pub fn G_KillBox(ctx: &mut GameContext, ent: EntityId) {
    // Safe-state 2c: EntityId self; entity fields via the accessor. FLAG (task #7):
    // `ent->client`/`hit->client` dereffed raw (pool-client-safe) as Raven does.
    let client = ctx.world.entity(ent).client;
    let (r_mins, r_maxs) = {
        let e = ctx.world.entity(ent);
        (e.r.mins, e.r.maxs)
    };
    let mut mins = [0.0f32; 3];
    let mut maxs = [0.0f32; 3];
    unsafe {
        for i in 0..3 {
            mins[i] = (*client).ps.origin[i] + r_mins[i];
            maxs[i] = (*client).ps.origin[i] + r_maxs[i];
        }
    }

    let mut touch = [0 as c_int; MAX_GENTITIES];
    let num = trap::EntitiesInBox(
        ctx.engine,
        GEntitiesInBoxArgs::new(
            mins.as_ptr() as *const vec3_t,
            maxs.as_ptr() as *const vec3_t,
            touch.as_mut_ptr(),
            MAX_GENTITIES as c_int,
        ),
    );

    for i in 0..num {
        let hit_id = EntityId(touch[i as usize] as u32);
        if ctx.world.entity(hit_id).client.is_null() {
            continue;
        }

        let hit_number = ctx.world.entity(hit_id).s.number;
        let ent_number = ctx.world.entity(ent).s.number;
        let ent_ownerNum = ctx.world.entity(ent).r.ownerNum;

        if hit_number == ent_number {
            // don't telefrag yourself!
            continue;
        }

        if ent_ownerNum == hit_number {
            // don't telefrag your vehicle!
            continue;
        }

        // nail it
        crate::g_combat::G_Damage(
            ctx,
            Some(hit_id),
            Some(ent),
            Some(ent),
            None,
            [0.0, 0.0, 0.0],
            100000,
            DAMAGE_NO_PROTECTION,
            MOD_TELEFRAG as c_int,
        );
    }
}

/// Raven `G_AddPredictableEvent`.
///
/// Source: `oracle/codemp/game/g_utils.c:1206-1211`
pub fn G_AddPredictableEvent(ent: Option<&mut gentity_t>, event: c_int, eventParm: c_int) {
    // Safe-state 2c: nullable ctx-free leaf — `ent` fields via the borrow.
    let Some(ent) = ent else {
        return;
    };
    // FLAG (task #7): `ent->client` may be a `BG_Alloc`'d NPC pool client
    // (gClPtrs, g_utils.c:430), not a `level.clients` slot; read the pointer off
    // the safe borrow and deref it raw exactly as Raven does.
    let client = ent.client;
    if client.is_null() {
        return;
    }
    unsafe {
        mp_bg::bg_misc::BG_AddPredictableEventToPlayerstate(event, eventParm, &mut (*client).ps);
    }
}

/// Raven `G_AddEvent`.
///
/// Source: `oracle/codemp/game/g_utils.c:1221-1243`
pub fn G_AddEvent(ent: &mut gentity_t, event: c_int, eventParm: c_int) {
    // Safe-state 2c: ctx-free leaf — `ent` fields via the borrow. Ctx-less
    // boundary fn (Raven reads the `level` global directly); world via the
    // `g_strap` seam world cell, engine (for the zero-event G_Printf) via the
    // strap_engine() precedent (see G_ModelIndex/G_SoundIndex).
    if event == 0 {
        let msg = format!("G_AddEvent: zero event added for entity {}\n", ent.s.number);
        let mut ctx = GameContext {
            world: unsafe { &mut *crate::g_strap::strap_world() },
            engine: crate::g_strap::strap_engine(),
        };
        G_Printf(&mut ctx, &msg);
        return;
    }

    let level_time = unsafe { (*crate::g_strap::strap_world()).level.time };

    // FLAG (task #7): `ent->client` may be a `BG_Alloc`'d NPC pool client
    // (gClPtrs), not a `level.clients` slot; read off the borrow, deref raw.
    let client = ent.client;
    if !client.is_null() {
        unsafe {
            let mut bits = (*client).ps.externalEvent & EV_EVENT_BITS;
            bits = (bits + EV_EVENT_BIT1) & EV_EVENT_BITS;
            (*client).ps.externalEvent = event | bits;
            (*client).ps.externalEventParm = eventParm;
            (*client).ps.externalEventTime = level_time;
        }
    } else {
        let mut bits = ent.s.event & EV_EVENT_BITS;
        bits = (bits + EV_EVENT_BIT1) & EV_EVENT_BITS;
        ent.s.event = event | bits;
        ent.s.eventParm = eventParm;
    }
    ent.eventTime = level_time;
}

/// Raven `G_PlayEffect`.
///
/// Source: `oracle/codemp/game/g_utils.c:1250-1260`
pub fn G_PlayEffect(fxID: c_int, org: vec3_t, ang: vec3_t) -> *mut gentity_t {
    // Ctx-less boundary fn; ctx rebuilt from the `g_strap` seam cells (world +
    // engine) so `G_TempEntity` can allocate — mirrors Raven reaching the
    // `level`/`g_entities` globals directly (see G_AddEvent). Safe-state 2c: te
    // fields via the accessor.
    let mut ctx = GameContext {
        world: unsafe { &mut *crate::g_strap::strap_world() },
        engine: crate::g_strap::strap_engine(),
    };
    let te_id = G_TempEntity(&mut ctx, org, EV_PLAY_EFFECT as c_int);
    let te = ctx.entity_mut(te_id) as *mut gentity_t;
    let e = ctx.world.entity_mut(te_id);
    e.s.angles = ang;
    e.s.origin = org;
    e.s.eventParm = fxID;

    te
}

/// Raven `G_PlayEffectID`.
///
/// Source: `oracle/codemp/game/g_utils.c:1267-1284`
pub fn G_PlayEffectID(fxID: c_int, org: vec3_t, ang: vec3_t) -> *mut gentity_t {
    // play an effect by the G_EffectIndex'd ID instead of a predefined effect ID
    // Ctx-less boundary fn; ctx rebuilt from the `g_strap` seam cells (see
    // G_PlayEffect). Safe-state 2c: te fields via the accessor.
    let mut ctx = GameContext {
        world: unsafe { &mut *crate::g_strap::strap_world() },
        engine: crate::g_strap::strap_engine(),
    };
    let te_id = G_TempEntity(&mut ctx, org, EV_PLAY_EFFECT_ID as c_int);
    let te = ctx.entity_mut(te_id) as *mut gentity_t;
    {
        let e = ctx.world.entity_mut(te_id);
        e.s.angles = ang;
        e.s.origin = org;
        e.s.eventParm = fxID;

        if e.s.angles[0] == 0.0 && e.s.angles[1] == 0.0 && e.s.angles[2] == 0.0 {
            // play off this dir by default then.
            e.s.angles[1] = 1.0;
        }
    }

    te
}

/// Raven `G_ScreenShake`.
///
/// Source: `oracle/codemp/game/g_utils.c:1291-1315`
pub fn G_ScreenShake(
    ctx: &mut GameContext,
    org: vec3_t,
    target: Option<EntityId>,
    intensity: f32,
    duration: c_int,
    global: qboolean,
) -> *mut gentity_t {
    // Safe-state 2c: `target` handle; te fields via the accessor. `te` is a fresh
    // temp entity whose raw pointer is returned to the caller as before.
    let te_id = G_TempEntity(ctx, org, EV_SCREENSHAKE as c_int);
    let te = ctx.entity_mut(te_id) as *mut gentity_t;

    let modelindex = match target {
        Some(id) => ctx.world.entity(id).s.number + 1,
        None => 0,
    };

    {
        let e = ctx.world.entity_mut(te_id);
        e.s.origin = org;
        e.s.angles[0] = intensity;
        e.s.time = duration;
        e.s.modelindex = modelindex;
        if global != qfalse {
            e.r.svFlags |= SVF_BROADCAST;
        }
    }

    te
}

/// Raven `G_MuteSound`.
///
/// Source: `oracle/codemp/game/g_utils.c:1322-1338`
pub fn G_MuteSound(ctx: &mut GameContext, entnum: c_int, channel: c_int) {
    // Safe-state 2c: te + target fields via the accessor.
    let te_id = G_TempEntity(ctx, vec3_origin, EV_MUTE_SOUND as c_int);
    {
        let e = ctx.world.entity_mut(te_id);
        e.r.svFlags = SVF_BROADCAST;
        e.s.trickedentindex2 = entnum;
        e.s.trickedentindex = channel;
    }

    let e_id = EntityId(entnum as u32);
    if ctx.world.entity(e_id).s.eFlags & EF_SOUNDTRACKER != 0 {
        G_FreeEntity(ctx, Some(e_id));
        ctx.world.entity_mut(e_id).s.eFlags = 0;
    }
}

/// Raven `G_Sound`. `ent->client` reached via the house
/// `(*ent).client` cast.
///
/// Source: `oracle/codemp/game/g_utils.c:1345-1372`
pub fn G_Sound(ctx: &mut GameContext, ent: Option<EntityId>, channel: c_int, soundIndex: c_int) {
    // Safe-state 2c: `ent` handle; te fields via the accessor. FLAG (task #7):
    // ent->client pool client (gClPtrs) dereffed raw. Raven reads
    // ent->r.currentOrigin unconditionally (the later null-check is vacuous), so
    // treat None as a no-op. No assert on soundIndex — Raven's G_Sound accepts 0
    // (benign no-sound event); a porter-invented debug_assert here killed live
    // bot spawns.
    let Some(ent_id) = ent else {
        return;
    };

    let origin = ctx.world.entity(ent_id).r.currentOrigin;
    let te_id = G_SoundTempEntity(ctx, origin, EV_GENERAL_SOUND as c_int, channel);
    {
        let e = ctx.world.entity_mut(te_id);
        e.s.eventParm = soundIndex;
        e.s.saberEntityNum = channel;
    }

    let client = ctx.world.entity(ent_id).client;
    if !client.is_null() && channel > trackchan_t::TRACK_CHANNEL_NONE as c_int {
        // let the client remember the index of the player entity so he
        // can kill the most recent sound on request
        let idx = (channel - 50) as usize;
        let killIdx = unsafe { (*client).ps.fd.killSoundEntIndex[idx] };
        if ctx.world.g_entities[killIdx as usize].inuse != qfalse && killIdx > MAX_CLIENTS as c_int
        {
            G_MuteSound(ctx, killIdx, mp_qshared::shared::sound_channel::CHAN_VOICE);
            let client = ctx.world.entity(ent_id).client;
            let killIdx = unsafe { (*client).ps.fd.killSoundEntIndex[idx] };
            if killIdx > MAX_CLIENTS as c_int
                && ctx.world.g_entities[killIdx as usize].inuse != qfalse
            {
                G_FreeEntity(ctx, Some(EntityId(killIdx as u32)));
            }
            unsafe {
                (*client).ps.fd.killSoundEntIndex[idx] = 0;
            }
        }

        let te_number = ctx.world.entity(te_id).s.number;
        let ent_number = ctx.world.entity(ent_id).s.number;
        let client = ctx.world.entity(ent_id).client;
        unsafe {
            (*client).ps.fd.killSoundEntIndex[idx] = te_number;
        }
        let e = ctx.world.entity_mut(te_id);
        e.s.trickedentindex = ent_number;
        e.s.eFlags = EF_SOUNDTRACKER;
    }
}

/// Raven `G_SoundAtLoc`. `loc` is read-only (passed straight to
/// `G_TempEntity`, never written), so kept by-value.
///
/// Source: `oracle/codemp/game/g_utils.c:1379-1385`
pub fn G_SoundAtLoc(ctx: &mut GameContext, loc: vec3_t, channel: c_int, soundIndex: c_int) {
    unsafe {
        let te_eid = G_TempEntity(ctx, loc, EV_GENERAL_SOUND as c_int);
        let te = ctx.entity_mut(te_eid) as *mut gentity_t;
        (*te).s.eventParm = soundIndex;
        (*te).s.saberEntityNum = channel;
    }
}

/// Raven `G_EntitySound`.
///
/// Source: `oracle/codemp/game/g_utils.c:1392-1399`
pub fn G_EntitySound(ctx: &mut GameContext, ent: EntityId, channel: c_int, soundIndex: c_int) {
    // Safe-state 2c: EntityId param; te + ent fields via the accessor.
    let origin = ctx.world.entity(ent).r.currentOrigin;
    let te_id = G_TempEntity(ctx, origin, EV_ENTITY_SOUND as c_int);
    let ent_number = ctx.world.entity(ent).s.number;
    let e = ctx.world.entity_mut(te_id);
    e.s.eventParm = soundIndex;
    e.s.clientNum = ent_number;
    e.s.trickedentindex = channel;
}

/// Raven `G_SoundOnEnt`.
///
/// Source: `oracle/codemp/game/g_utils.c:1402-1411`
pub fn G_SoundOnEnt(
    ctx: &mut GameContext,
    ent: EntityId,
    channel: c_int,
    soundPath: &str,
) {
    // Safe-state 2c: EntityId param; te + ent fields via the accessor.
    let origin = ctx.world.entity(ent).r.currentOrigin;
    let te_id = G_TempEntity(ctx, origin, EV_ENTITY_SOUND as c_int);
    let ent_number = ctx.world.entity(ent).s.number;
    let sound = G_SoundIndex(soundPath);
    let e = ctx.world.entity_mut(te_id);
    e.s.eventParm = sound;
    e.s.clientNum = ent_number;
    e.s.trickedentindex = channel;
}

/// Raven `ValidUseTarget`.
///
/// Source: `oracle/codemp/game/g_utils.c:1453-1471`
pub fn ValidUseTarget(ent: Option<&gentity_t>) -> qboolean {
    // Safe-state 2c: nullable ctx-free leaf — reads fields off the borrow directly.
    let Some(ent) = ent else {
        return qfalse;
    };
    if ent.use_.is_none() {
        return qfalse;
    }

    if (ent.flags & crate::entity::flags::FL_INACTIVE) != 0 {
        return qfalse;
    }

    if (ent.r.svFlags & SVF_PLAYER_USABLE) == 0 {
        return qfalse;
    }

    qtrue
}

/// Raven `G_UseDispenserOn`.
///
/// Source: `oracle/codemp/game/g_utils.c:1474-1505`
pub fn G_UseDispenserOn(ctx: &mut GameContext, ent: EntityId, dispType: c_int, target: EntityId) {
    // Safe-state 2c: EntityId params; entity fields via the accessor. FLAG
    // (task #7): the `ent`/`target` clients are dereffed raw (pool-client-safe).
    // HI_HEALTHDISP (8) / HI_AMMODISP (9) come from the canonical prelude glob
    // (mp_bg::public::holdable) so the value matches the STAT_HOLDABLE_ITEMS bit
    // and the dispType TryUse passes in. STAT_HEALTH (0) / STAT_MAX_HEALTH (8)
    // are the canonical statIndex_t slots (mp_bg::public::stat_index) cast to
    // usize for indexing — the old local `STAT_MAX_HEALTH = 1` read STAT_HOLDABLE_ITEM.
    const STAT_HEALTH: usize = statIndex_t::STAT_HEALTH as usize;
    const STAT_MAX_HEALTH: usize = statIndex_t::STAT_MAX_HEALTH as usize;

    let level_time = ctx.world.level.time;

    if dispType == HI_HEALTHDISP {
        let client = ctx.world.entity(target).client;
        let health = unsafe {
            (*client).ps.stats[STAT_HEALTH] += 4;

            if (*client).ps.stats[STAT_HEALTH] > (*client).ps.stats[STAT_MAX_HEALTH] {
                (*client).ps.stats[STAT_HEALTH] = (*client).ps.stats[STAT_MAX_HEALTH];
            }

            (*client).isMedHealed = level_time + 500;
            (*client).ps.stats[STAT_HEALTH]
        };
        ctx.world.entity_mut(target).health = health;
    } else if dispType == HI_AMMODISP {
        let client = ctx.world.entity(ent).client;
        let tclient = ctx.world.entity(target).client;
        unsafe {
            if (*client).medSupplyDebounce < level_time {
                // do the next increment; based on the amount of ammo used per normal shot.
                let weap = (*tclient).ps.weapon as usize;
                let ammo_index = weaponData[weap].ammoIndex as usize;
                (*tclient).ps.ammo[ammo_index] += weaponData[weap].energyPerShot;

                if (*tclient).ps.ammo[ammo_index] > ammoData[ammo_index].max {
                    // cap it off
                    (*tclient).ps.ammo[ammo_index] = ammoData[ammo_index].max;
                }

                // base the next supply time on how long the weapon takes to fire.
                (*client).medSupplyDebounce = level_time + weaponData[weap].fireTime;
            }
            (*tclient).isMedSupplied = level_time + 500;
        }
    }
}

/// Raven `G_CanUseDispOn`.
///
/// Source: `oracle/codemp/game/g_utils.c:1508-1544`
pub fn G_CanUseDispOn(ctx: &mut GameContext, ent: Option<EntityId>, dispType: c_int) -> c_int {
    // Safe-state 2c: Option handle; entity fields via the accessor. FLAG (task #7):
    // the `ent` client is dereffed raw (pool-client-safe).
    // HI_HEALTHDISP (8) / HI_AMMODISP (9) come from the canonical prelude glob
    // (mp_bg::public::holdable) so the compared dispType matches TryUse's caller.
    // STAT_HEALTH (0) / STAT_MAX_HEALTH (8) are canonical statIndex_t slots
    // (mp_bg::public::stat_index); WP_NONE (0) is the canonical prelude value.
    // The old local `STAT_MAX_HEALTH = 1` read STAT_HOLDABLE_ITEM, so the
    // "he's hurt" check compared health against the wrong stat.
    const STAT_HEALTH: usize = statIndex_t::STAT_HEALTH as usize;
    const STAT_MAX_HEALTH: usize = statIndex_t::STAT_MAX_HEALTH as usize;
    // Raven `#define LAST_USEABLE_WEAPON WP_BRYAR_OLD` (bg_weapons.h:43); the
    // port has no shared const for it, so mirror it locally from canonical WP_BRYAR_OLD.
    const LAST_USEABLE_WEAPON: c_int = WP_BRYAR_OLD;

    let Some(ent_id) = ent else {
        return 0;
    };
    let client = ctx.world.entity(ent_id).client;
    let (inuse, health) = {
        let e = ctx.world.entity(ent_id);
        (e.inuse, e.health)
    };

    //dead or invalid (client.is_null() short-circuits before the raw deref)
    if client.is_null()
        || inuse == qfalse
        || health < 1
        || unsafe { (*client).ps.stats[STAT_HEALTH] } < 1
    {
        return 0;
    }

    if dispType == HI_HEALTHDISP {
        unsafe {
            if (*client).ps.stats[STAT_HEALTH] < (*client).ps.stats[STAT_MAX_HEALTH] {
                return 1;
            }
        }
        return 0;
    } else if dispType == HI_AMMODISP {
        unsafe {
            if (*client).ps.weapon <= WP_NONE || (*client).ps.weapon > LAST_USEABLE_WEAPON {
                // not a player-useable weapon
                return 0;
            }

            let weap = (*client).ps.weapon as usize;
            let ammo_index = weaponData[weap].ammoIndex as usize;
            if (*client).ps.ammo[ammo_index] < ammoData[ammo_index].max {
                // needs more ammo for current weapon
                return 1;
            }
        }

        // needs none
        return 0;
    }

    0
}

/// Raven `TryHeal`.
///
/// Source: `oracle/codemp/game/g_utils.c:1546-1602`
pub fn TryHeal(ctx: &mut GameContext, ent: Option<EntityId>, target: Option<EntityId>) -> qboolean {
    use mp_bg::public::anim_number::animNumber_t;
    use mp_bg::public::gametype::GT_SIEGE;
    use mp_bg::public::set_anim::{SETANIM_FLAG_HOLD, SETANIM_FLAG_OVERRIDE, SETANIM_TORSO};
    use mp_qshared::shared::sound_channel::CHAN_AUTO;
    const BOTH_BUTTON_HOLD: c_int = animNumber_t::BOTH_BUTTON_HOLD as c_int;
    const BOTH_CONSOLE1: c_int = animNumber_t::BOTH_CONSOLE1 as c_int;

    // Safe-state 2c: Option handles; entity fields via the accessor. FLAG (task #7):
    // ent->client pool client (gClPtrs) dereffed raw as Raven does.
    let Some(ent_id) = ent else {
        return qfalse;
    };
    let client = ctx.world.entity(ent_id).client;
    if client.is_null() {
        return qfalse;
    }

    // Raven checks g_gametype/siegeClass before target.is_null(); with `target`
    // an Option, None short-circuits to qfalse here (pure reads, no reorder of
    // side effects).
    let Some(target_id) = target else {
        return qfalse;
    };

    let g_gametype = ctx.world.cvars.g_gametype.integer;
    let siegeClass = unsafe { (*client).siegeClass };
    let bail = {
        let t = ctx.world.entity(target_id);
        // Owned `String` now: `""` ≡ absent (was the `!= 0` non-empty check).
        g_gametype != GT_SIEGE
            || siegeClass == -1
            || t.inuse == qfalse
            || t.maxHealth == 0
            || t.healingclass.is_empty()
            || t.health <= 0
            || t.health >= t.maxHealth
    };
    if bail {
        // it's not dead yet...
        return qfalse;
    }

    let scl_matches = {
        let scl = &ctx.world.bg_state.bgSiegeClasses[siegeClass as usize];
        scl.name.eq_ignore_ascii_case(&ctx.world.entity(target_id).healingclass)
    };
    if !scl_matches {
        return qfalse;
    }

    // this thing can be healed by the class this player is using
    let level_time = ctx.world.level.time;
    if ctx.world.entity(target_id).healingDebounce < level_time {
        // do the actual heal
        {
            let t = ctx.world.entity_mut(target_id);
            t.health += 10;
            if t.health > t.maxHealth {
                // don't go too high
                t.health = t.maxHealth;
            }
        }
        let healingrate = ctx.world.entity(target_id).healingrate;
        ctx.world.entity_mut(target_id).healingDebounce = level_time + healingrate;

        let healingsound = ctx.world.entity(target_id).healingsound.clone();
        if !healingsound.is_empty() {
            // play it
            let sound = G_SoundIndex(&healingsound);
            if ctx.world.entity(target_id).s.solid == SOLID_BMODEL {
                // ok, well, just play it on the client then.
                G_Sound(ctx, Some(ent_id), CHAN_AUTO as c_int, sound);
            } else {
                G_Sound(ctx, Some(target_id), CHAN_AUTO as c_int, sound);
            }
        }

        // update net health for bar
        G_ScaleNetHealth(ctx.world.entity_mut(target_id));
        if let Some(te_id) = ctx.world.entity(target_id).target_ent {
            if ctx.world.entity(te_id).maxHealth != 0 {
                let target_health = ctx.world.entity(target_id).health;
                ctx.world.entity_mut(te_id).health = target_health;
                G_ScaleNetHealth(ctx.world.entity_mut(te_id));
            }
        }
    }

    // keep them in the healing anim even when the healing debounce is not yet expired
    let torsoAnim = unsafe { (*client).ps.torsoAnim };
    if torsoAnim == BOTH_BUTTON_HOLD || torsoAnim == BOTH_CONSOLE1 {
        // extend the time
        unsafe {
            (*client).ps.torsoTimer = 500;
        }
    } else {
        G_SetAnim(
            ctx,
            ent_id,
            core::ptr::null_mut(),
            SETANIM_TORSO,
            BOTH_BUTTON_HOLD,
            SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
            0,
        );
    }

    qtrue
}

/// Raven `TryUse`.
///
/// Source: `oracle/codemp/game/g_utils.c:1618-1875`
pub fn TryUse(ctx: &mut GameContext, ent: Option<EntityId>) {
    // GT_SIEGE (7), TEAM_SPECTATOR (3), PMF_FOLLOW (4096), SETANIM_TORSO (1),
    // SETANIM_FLAG_OVERRIDE/HOLD, MASK_OPAQUE/CONTENTS_*, HI_HEALTHDISP (8)/
    // HI_AMMODISP (9) and ENTITYNUM_NONE (1023) all resolve to the port's
    // canonical constants via the prelude glob (mp_bg::public::{gametype,team,
    // set_anim}, mp_qshared::shared::{surface_flags,limits}, qcommon::pm_flags,
    // mp_bg::public::holdable). Only the enum-typed values need a local c_int
    // cast, the same pattern commit 09afce35 established inside TryHeal.
    const HANDEXTEND_NONE: c_int = forceHandAnims_t::HANDEXTEND_NONE as c_int;
    const HANDEXTEND_DRAGGING: c_int = forceHandAnims_t::HANDEXTEND_DRAGGING as c_int;
    const BOTH_BUTTON_HOLD: c_int = animNumber_t::BOTH_BUTTON_HOLD as c_int;
    const BOTH_CONSOLE1: c_int = animNumber_t::BOTH_CONSOLE1 as c_int;
    const CLASS_VEHICLE: c_int = class_t::CLASS_VEHICLE as c_int;
    const USE_DISTANCE: f32 = 64.0;

    // STAGE-1: Option param (body null-checks ent), raw body re-derived verbatim (Stage-2 debt).
    let ent: *mut gentity_t =
        unsafe { crate::ent_id::resolve(ctx.world.g_entities.as_mut_ptr(), ent) };
    unsafe {
        if ctx.world.globals.gSiegeRoundBegun == qfalse
            && ctx.world.cvars.g_gametype.integer == GT_SIEGE
        {
            return;
        }

        if ent.is_null()
            || (*ent).client.is_null()
            || ((*ent).client != core::ptr::null_mut()
                && ((*((*ent).client)).ps.weaponTime > 0
                    && (*((*ent).client)).ps.torsoAnim != BOTH_BUTTON_HOLD
                    && (*((*ent).client)).ps.torsoAnim != BOTH_CONSOLE1))
            || (*ent).health < 1
            || (((*((*ent).client)).ps.pm_flags & PMF_FOLLOW) != 0)
            || ((*((*ent).client)).sess.sessionTeam == TEAM_SPECTATOR)
            || ((*((*ent).client)).ps.forceHandExtend != HANDEXTEND_NONE
                && (*((*ent).client)).ps.forceHandExtend != HANDEXTEND_DRAGGING)
        {
            return;
        }

        // Check if on emplaced gun or using vehicle
        let client = (*ent).client;
        if (*client).ps.emplacedIndex != 0 {
            return;
        }

        // Check if in a vehicle
        if (*ent).s.number < MAX_CLIENTS as c_int && (*client).ps.m_iVehicleNum != 0 {
            let current_veh =
                &mut ctx.world.g_entities[(*client).ps.m_iVehicleNum as usize] as *mut gentity_t;
            if (*current_veh).inuse != qfalse && !(*current_veh).m_pVehicle.is_null() {
                let pVeh = (*current_veh).m_pVehicle;
                if (*pVeh).m_iBoarding == 0 {
                    crate::veh_dispatch::eject(ctx, pVeh, ent as *mut bgEntity_t, qfalse);
                }
                return;
            }
        }

        // Check jetpack
        if (*client).jetPackOn != qfalse {
            // tryJetPack label logic - implemented at end of function
            goto_tryJetPack(ctx, ctx.entity_id_of(ent).unwrap());
            return;
        }

        // Check body grab
        if (*client).bodyGrabIndex != ENTITYNUM_NONE {
            if (*client).bodyGrabTime < ctx.world.level.time {
                let grabbed =
                    &mut ctx.world.g_entities[(*client).bodyGrabIndex as usize] as *mut gentity_t;
                if (*grabbed).inuse != qfalse {
                    if !(*grabbed).client.is_null() {
                        let grabbed_client = (*grabbed).client;
                        (*grabbed_client).ps.ragAttach = 0;
                    } else {
                        (*grabbed).s.ragAttach = 0;
                    }
                }
            }
            (*client).bodyGrabIndex = ENTITYNUM_NONE;
            (*client).bodyGrabTime = ctx.world.level.time + 1000;
            return;
        }

        // Trace ahead
        let mut viewspot = (*client).ps.origin;
        viewspot[2] += (*client).ps.viewheight as f32;

        let src = viewspot;
        let mut vf = [0.0f32; 3];
        AngleVectors((*client).ps.viewangles, Some(&mut vf), None, None);

        let mut dest = src;
        _VectorMA(src, USE_DISTANCE, vf, &mut dest);

        // Trace to find target
        let mut trace: trace_t = core::mem::zeroed();
        trap::Trace(
            ctx.engine,
            GTraceArgs::new(
                &mut trace as *mut trace_t,
                &src as *const vec3_t,
                &vec3_origin as *const vec3_t,
                &vec3_origin as *const vec3_t,
                &dest as *const vec3_t,
                (*ent).s.number,
                MASK_OPAQUE | CONTENTS_SOLID | CONTENTS_BODY | CONTENTS_ITEM | CONTENTS_CORPSE,
            ),
        );

        if trace.fraction == 1.0 || trace.entityNum < 1 {
            goto_tryJetPack(ctx, ctx.entity_id_of(ent).unwrap());
            return;
        }

        let target = &mut ctx.world.g_entities[trace.entityNum as usize] as *mut gentity_t;

        // Check for vehicle target
        if !target.is_null()
            && !(*target).m_pVehicle.is_null()
            && !(*target).client.is_null()
            && (*target).s.NPC_class == CLASS_VEHICLE
            && ((*client).ps.zoomMode == qfalse)
        {
            //if target is a vehicle then perform appropriate checks
            let pVeh = (*target).m_pVehicle;
            if !(*pVeh).m_pVehicleInfo.is_null() {
                if (*ent).r.ownerNum == (*target).s.number {
                    //user is already on this vehicle so eject him
                    crate::veh_dispatch::eject(ctx, pVeh, ent as *mut bgEntity_t, qfalse);
                } else {
                    // Otherwise board this vehicle.
                    if ctx.world.cvars.g_gametype.integer < GT_TEAM
                        || (*target).alliedTeam == 0
                        || ((*target).alliedTeam == (*client).sess.sessionTeam)
                    {
                        //not belonging to a team, or client is on same team
                        crate::veh_dispatch::board(ctx, pVeh, ent as *mut bgEntity_t);
                    }
                }
                //clear the damn button!
                (*client).pers.cmd.buttons &= !BUTTON_USE;
                return;
            }
        }

        // Check for dispenser usage
        if ((*client).ps.stats[STAT_HOLDABLE_ITEMS as usize] & (1 << HI_HEALTHDISP)) != 0
            || ((*client).ps.stats[STAT_HOLDABLE_ITEMS as usize] & (1 << HI_AMMODISP)) != 0
        {
            if !target.is_null()
                && (*target).inuse != qfalse
                && !(*target).client.is_null()
                && (*target).health > 0
                && OnSameTeam(ctx, ctx.entity_id_of(ent), ctx.entity_id_of(target)) != qfalse
                && (G_CanUseDispOn(ctx, ctx.entity_id_of(target), HI_HEALTHDISP) != 0
                    || G_CanUseDispOn(ctx, ctx.entity_id_of(target), HI_AMMODISP) != 0)
            {
                if G_CanUseDispOn(ctx, ctx.entity_id_of(target), HI_HEALTHDISP) != 0 {
                    G_UseDispenserOn(
                        ctx,
                        ctx.entity_id_of(ent).unwrap(),
                        HI_HEALTHDISP,
                        ctx.entity_id_of(target).unwrap(),
                    );
                }
                if G_CanUseDispOn(ctx, ctx.entity_id_of(target), HI_AMMODISP) != 0 {
                    G_UseDispenserOn(
                        ctx,
                        ctx.entity_id_of(ent).unwrap(),
                        HI_AMMODISP,
                        ctx.entity_id_of(target).unwrap(),
                    );
                }

                if (*client).ps.torsoAnim == BOTH_BUTTON_HOLD {
                    (*client).ps.torsoTimer = 500;
                } else {
                    G_SetAnim(
                        ctx,
                        ctx.entity_id_of(ent).unwrap(),
                        core::ptr::null_mut(),
                        SETANIM_TORSO,
                        BOTH_BUTTON_HOLD,
                        SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                        0,
                    );
                }
                (*client).ps.weaponTime = (*client).ps.torsoTimer;
                return;
            }
        }

        // Check for valid use target
        if ValidUseTarget((target).as_ref()) != qfalse
            && (ctx.world.cvars.g_gametype.integer != GT_SIEGE
                || (*target).alliedTeam == 0
                || (*target).alliedTeam != (*client).sess.sessionTeam
                || ctx.world.cvars.g_ff_objectives.integer != 0)
        {
            if (*client).ps.torsoAnim == BOTH_BUTTON_HOLD || (*client).ps.torsoAnim == BOTH_CONSOLE1
            {
                (*client).ps.torsoTimer = 500;
            } else {
                G_SetAnim(
                    ctx,
                    ctx.entity_id_of(ent).unwrap(),
                    core::ptr::null_mut(),
                    SETANIM_TORSO,
                    BOTH_BUTTON_HOLD,
                    SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                    0,
                );
            }
            (*client).ps.weaponTime = (*client).ps.torsoTimer;

            let target_id = ctx.entity_id_of(target).unwrap();
            if ctx.world.entity(target_id).touch.get() == Some(EntTouch::Touch_Button) {
                // Raven: pretend we touched it (NULL trace).
                let target_ptr = ctx.world.entity_mut(target_id) as *mut gentity_t;
                crate::ent_fn_enums::dispatch_touch(
                    ctx,
                    EntTouch::Touch_Button,
                    target_ptr,
                    ent,
                    core::ptr::null_mut(),
                );
            } else if !(*target).use_.is_none() {
                GlobalUse(
                    ctx,
                    ctx.entity_id_of(target),
                    ctx.entity_id_of(ent),
                    ctx.entity_id_of(ent),
                );
            }
            return;
        }

        // Check for healing
        if TryHeal(ctx, ctx.entity_id_of(ent), ctx.entity_id_of(target)) != qfalse {
            return;
        }

        goto_tryJetPack(ctx, ctx.entity_id_of(ent).unwrap());
    }
}

fn goto_tryJetPack(ctx: &mut GameContext, ent: EntityId) {
    // Safe-state 2c: EntityId param; ent fields via the accessor. FLAG (task #7):
    // ent->client pool client dereffed raw.
    // HI_JETPACK (7), HI_AMMODISP (9) and ENTITYNUM_NONE (1023) resolve to the
    // port's canonical constants via the prelude glob (mp_bg::public::holdable,
    // mp_qshared::shared::limits) so the STAT_HOLDABLE_ITEMS bit tests and the
    // ItemUse_UseDisp / EV_USE_ITEM0 dispType all use the real values.
    let client = ctx.world.entity(ent).client;

    // Jetpack check
    if (unsafe { (*client).ps.stats[STAT_HOLDABLE_ITEMS as usize] } & (1 << HI_JETPACK)) != 0 {
        let ready = unsafe {
            (*client).jetPackOn != qfalse || (*client).ps.groundEntityNum == ENTITYNUM_NONE
        };
        if ready {
            ItemUse_Jetpack(ctx, ent);
            return;
        }
    }

    // Ammo dispenser check
    if (unsafe { (*client).ps.stats[STAT_HOLDABLE_ITEMS as usize] } & (1 << HI_AMMODISP)) != 0 {
        let mut tr_toss: trace_t = unsafe { core::mem::zeroed() };
        let mut f_ang = [0.0f32; 3];
        let mut fwd = [0.0f32; 3];

        f_ang[0] = 0.0f32;
        f_ang[1] = unsafe { (*client).ps.viewangles[YAW as usize] };
        f_ang[2] = 0.0f32;

        AngleVectors(f_ang, Some(&mut fwd), None, None);

        let origin = unsafe { (*client).ps.origin };
        _VectorMA(origin, 64.0f32, fwd, &mut fwd);
        let ent_number = ctx.world.entity(ent).s.number;
        let ent_clipmask = ctx.world.entity(ent).clipmask;
        trap::Trace(
            ctx.engine,
            GTraceArgs::new(
                &mut tr_toss as *mut trace_t,
                &origin as *const vec3_t,
                &playerMins as *const vec3_t,
                &playerMaxs as *const vec3_t,
                &fwd as *const vec3_t,
                ent_number,
                ent_clipmask,
            ),
        );

        if tr_toss.fraction == 1.0f32 && tr_toss.allsolid == 0 && tr_toss.startsolid == 0 {
            ItemUse_UseDisp(ctx, ent, HI_AMMODISP);
            let ev =
                mp_bg::public::entity_event::entity_event_t::EV_USE_ITEM0 as c_int + HI_AMMODISP;
            G_AddEvent(ctx.world.entity_mut(ent), ev, 0);
        }
    }
}

/// Raven `G_PointInBounds`.
///
/// Source: `oracle/codemp/game/g_utils.c:1877-1894`
pub fn G_PointInBounds(point: vec3_t, mins: vec3_t, maxs: vec3_t) -> qboolean {
    for i in 0..3 {
        if point[i] < mins[i] {
            return qfalse;
        }
        if point[i] > maxs[i] {
            return qfalse;
        }
    }
    qtrue
}

/// Raven `G_BoxInBounds`.
///
/// Source: `oracle/codemp/game/g_utils.c:1896-1924`
pub fn G_BoxInBounds(
    point: vec3_t,
    mins: vec3_t,
    maxs: vec3_t,
    boundsMins: vec3_t,
    boundsMaxs: vec3_t,
) -> qboolean {
    let boxMins = [point[0] + mins[0], point[1] + mins[1], point[2] + mins[2]];
    let boxMaxs = [point[0] + maxs[0], point[1] + maxs[1], point[2] + maxs[2]];

    if boxMaxs[0] > boundsMaxs[0] {
        return qfalse;
    }
    if boxMaxs[1] > boundsMaxs[1] {
        return qfalse;
    }
    if boxMaxs[2] > boundsMaxs[2] {
        return qfalse;
    }
    if boxMins[0] < boundsMins[0] {
        return qfalse;
    }
    if boxMins[1] < boundsMins[1] {
        return qfalse;
    }
    if boxMins[2] < boundsMins[2] {
        return qfalse;
    }

    // box is completely contained within bounds
    qtrue
}

/// Raven `G_SetAngles`.
///
/// Source: `oracle/codemp/game/g_utils.c:1927-1932`
pub fn G_SetAngles(ent: &mut gentity_t, angles: vec3_t) {
    // Safe-state 2c: ctx-free leaf mutator on the caller's `&mut gentity_t`.
    ent.r.currentAngles = angles;
    ent.s.angles = angles;
    ent.s.apos.trBase = angles;
}

/// Raven `G_ClearTrace`. All four vec3 params are read-only inputs
/// to `trap_Trace` (never written), so kept by-value.
///
/// Source: `oracle/codemp/game/g_utils.c:1934-1946`
pub fn G_ClearTrace(
    ctx: &mut GameContext,
    start: vec3_t,
    mins: vec3_t,
    maxs: vec3_t,
    end: vec3_t,
    ignore: c_int,
    clipmask: c_int,
) -> qboolean {
    unsafe {
        let mut tr: trace_t = core::mem::zeroed();
        trap::Trace(
            ctx.engine,
            GTraceArgs::new(
                &mut tr as *mut trace_t,
                &start as *const vec3_t,
                &mins as *const vec3_t,
                &maxs as *const vec3_t,
                &end as *const vec3_t,
                ignore,
                clipmask,
            ),
        );

        if tr.allsolid != 0 || tr.startsolid != 0 || tr.fraction < 1.0 {
            return qfalse;
        }

        qtrue
    }
}

/// Raven `G_SetOrigin`. `origin` is a read-only input here (copied
/// out, never written back), so kept by-value.
///
/// Source: `oracle/codemp/game/g_utils.c:1955-1963`
pub fn G_SetOrigin(ent: &mut gentity_t, origin: vec3_t) {
    // Safe-state 2c: ctx-free leaf mutator on the caller's `&mut gentity_t`.
    ent.s.pos.trBase = origin;
    ent.s.pos.trType = trType_t::TR_STATIONARY;
    ent.s.pos.trTime = 0;
    ent.s.pos.trDuration = 0;
    ent.s.pos.trDelta = [0.0, 0.0, 0.0];

    ent.r.currentOrigin = origin;
}

/// Raven `G_CheckInSolid`.
///
/// Source: `oracle/codemp/game/g_utils.c:1965-2001`
pub fn G_CheckInSolid(ctx: &mut GameContext, self_: EntityId, fix: qboolean) -> qboolean {
    // Safe-state 2c: EntityId param; entity fields via the accessor. The trace
    // reads are read-only inputs, copied into locals.
    let (currentOrigin, r_mins, r_maxs, number, clipmask) = {
        let e = ctx.world.entity(self_);
        (
            e.r.currentOrigin,
            e.r.mins,
            e.r.maxs,
            e.s.number,
            e.clipmask,
        )
    };

    let mut end = currentOrigin;
    end[2] += r_mins[2];
    let mut mins = r_mins;
    mins[2] = 0.0;

    let mut trace: trace_t = unsafe { core::mem::zeroed() };
    trap::Trace(
        ctx.engine,
        GTraceArgs::new(
            &mut trace as *mut trace_t,
            &currentOrigin as *const vec3_t,
            &mins as *const vec3_t,
            &r_maxs as *const vec3_t,
            &end as *const vec3_t,
            number,
            clipmask,
        ),
    );

    if trace.allsolid != 0 || trace.startsolid != 0 {
        return qtrue;
    }

    if trace.fraction < 1.0 {
        if fix != qfalse {
            // Put them at end of trace and check again
            let mut neworg = trace.endpos;
            neworg[2] -= r_mins[2];
            G_SetOrigin(ctx.world.entity_mut(self_), neworg);
            let self_ptr = ctx.world.entity_mut(self_) as *mut gentity_t;
            trap::LinkEntity(ctx.engine, GLinkentityArgs::new(self_ptr.cast()));

            return G_CheckInSolid(ctx, self_, qfalse);
        } else {
            return qtrue;
        }
    }

    qfalse
}

/// Raven `DebugLine`. `start`/`end` are read-only inputs (never
/// written), so kept by-value.
///
/// Source: `oracle/codemp/game/g_utils.c:2011-2037`
pub fn DebugLine(ctx: &mut GameContext, start: vec3_t, end: vec3_t, color: c_int) -> c_int {
    let mut points = [[0.0f32; 3]; 4];
    points[0] = start;
    points[1] = start;
    points[2] = end;
    points[3] = end;

    let mut dir = [end[0] - start[0], end[1] - start[1], end[2] - start[2]];
    VectorNormalize(&mut dir);
    let up = [0.0f32, 0.0, 1.0];
    let dot = _DotProduct(dir, up);
    let mut cross = [0.0f32; 3];
    // `0.99` is a bare double in the oracle; both compares promote (fn is dead —
    // zero callers in either tree — promoted for F3-class purity).
    if (dot as f64) > 0.99 || (dot as f64) < -0.99 {
        cross = [1.0, 0.0, 0.0];
    } else {
        CrossProduct(dir, up, &mut cross);
    }
    VectorNormalize(&mut cross);

    for i in 0..3 {
        points[0][i] += 2.0 * cross[i];
        points[1][i] += -2.0 * cross[i];
        points[2][i] += -2.0 * cross[i];
        points[3][i] += 2.0 * cross[i];
    }

    trap::DebugPolygonCreate(
        ctx.engine,
        mp_abi::game::syscalls::G_DEBUG_POLYGON_CREATE::GDebugPolygonCreateArgs::new(
            color,
            4,
            points.as_mut_ptr() as *mut vec3_t,
        ),
    )
}

/// Raven `G_ROFF_NotetrackCallback`.
///
/// Source: `oracle/codemp/game/g_utils.c:2039-2080`
pub fn G_ROFF_NotetrackCallback(
    ctx: &mut GameContext,
    cent: Option<EntityId>,
    notetrack: *const c_char,
) {
    // Safe-state 2c: Option handle; cent fields via the accessor. `notetrack` is a
    // raw c-string (seam), parsed under a tight unsafe block.
    let Some(cent_id) = cent else {
        return;
    };
    if notetrack.is_null() {
        return;
    }

    let (addlArg, is_loop) = unsafe {
        let mut ty = [0u8; 256];
        let mut i: usize = 0;
        while *notetrack.add(i) != 0 && *notetrack.add(i) != b' ' as c_char {
            ty[i] = *notetrack.add(i) as u8;
            i += 1;
        }
        ty[i] = 0;

        if i == 0 || ty[0] == 0 {
            return;
        }

        let addlArg = *notetrack.add(i) == b' ' as c_char;
        let type_str = CStr::from_bytes_with_nul(&ty[..=i]).unwrap();
        (addlArg, type_str.to_bytes() == b"loop")
    };

    if is_loop {
        if addlArg {
            // including an additional argument means reset to original
            // position before loop
            let e = ctx.world.entity_mut(cent_id);
            e.s.pos.trBase = e.s.origin2;
            e.r.currentOrigin = e.s.origin2;
            e.s.apos.trBase = e.s.angles2;
            e.r.currentAngles = e.s.angles2;
        }

        let (number, roffid) = {
            let e = ctx.world.entity(cent_id);
            (e.s.number, e.roffid)
        };
        trap::ROFF_Play(
            ctx.engine,
            mp_abi::game::syscalls::G_ROFF_PLAY::GRoffPlayArgs::new(number, roffid, qfalse),
        );
    }
}

/// Raven `G_SpeechEvent`.
///
/// Source: `oracle/codemp/game/g_utils.c:2082-2085`
pub fn G_SpeechEvent(ctx: &mut GameContext, self_: EntityId, event: c_int) {
    G_AddEvent(ctx.entity_mut(self_), event, 0);
}

/// Raven `G_ExpandPointToBBox`. Reshape: `point` is written through
/// (the final `VectorCopy(start, point)`), so it becomes `&mut [f32;3]`;
/// `mins`/`maxs` are read-only, kept by-value (no same-file callers to fix up).
///
/// Source: `oracle/codemp/game/g_utils.c:2087-2128`
pub fn G_ExpandPointToBBox(
    ctx: &mut GameContext,
    point: &mut vec3_t,
    mins: vec3_t,
    maxs: vec3_t,
    ignore: c_int,
    clipmask: c_int,
) -> qboolean {
    unsafe {
        let mut start = *point;

        for i in 0..3 {
            let mut end = start;
            end[i] += mins[i];
            let mut tr: trace_t = core::mem::zeroed();
            trap::Trace(
                ctx.engine,
                GTraceArgs::new(
                    &mut tr as *mut trace_t,
                    &start as *const vec3_t,
                    &vec3_origin as *const vec3_t,
                    &vec3_origin as *const vec3_t,
                    &end as *const vec3_t,
                    ignore,
                    clipmask,
                ),
            );
            if tr.allsolid != 0 || tr.startsolid != 0 {
                return qfalse;
            }
            if tr.fraction < 1.0 {
                end = start;
                end[i] += maxs[i] - mins[i] * tr.fraction;
                trap::Trace(
                    ctx.engine,
                    GTraceArgs::new(
                        &mut tr as *mut trace_t,
                        &start as *const vec3_t,
                        &vec3_origin as *const vec3_t,
                        &vec3_origin as *const vec3_t,
                        &end as *const vec3_t,
                        ignore,
                        clipmask,
                    ),
                );
                if tr.allsolid != 0 || tr.startsolid != 0 {
                    return qfalse;
                }
                if tr.fraction < 1.0 {
                    return qfalse;
                }
                start = end;
            }
        }
        // expanded it, now see if it's all clear
        let mut tr: trace_t = core::mem::zeroed();
        trap::Trace(
            ctx.engine,
            GTraceArgs::new(
                &mut tr as *mut trace_t,
                &start as *const vec3_t,
                &mins as *const vec3_t,
                &maxs as *const vec3_t,
                &start as *const vec3_t,
                ignore,
                clipmask,
            ),
        );
        if tr.allsolid != 0 || tr.startsolid != 0 {
            return qfalse;
        }
        *point = start;
        qtrue
    }
}

/// Raven `ShortestLineSegBewteen2LineSegs`. Reshape: `close_pnt1`/
/// `close_pnt2` are the written out-params, so they become `&mut [f32;3]`;
/// `start1`/`end1`/`start2`/`end2` are read-only, kept by-value (no
/// same-file callers to fix up).
///
/// Source: `oracle/codemp/game/g_utils.c:2131-2307`
pub fn ShortestLineSegBewteen2LineSegs(
    start1: vec3_t,
    end1: vec3_t,
    start2: vec3_t,
    end2: vec3_t,
    close_pnt1: &mut vec3_t,
    close_pnt2: &mut vec3_t,
) -> f32 {
    let mut start_dif = [0.0f32; 3];
    _VectorSubtract(start2, start1, &mut start_dif);
    let mut v1 = [0.0f32; 3];
    _VectorSubtract(end1, start1, &mut v1);
    let mut v2 = [0.0f32; 3];
    _VectorSubtract(end2, start2, &mut v2);

    let v1v1 = _DotProduct(v1, v1);
    let v2v2 = _DotProduct(v2, v2);
    let v1v2 = _DotProduct(v1, v2);

    let denom = (v1v2 * v1v2) - (v1v1 * v2v2);

    let mut current_dist;
    if denom.abs() > 0.001 {
        let s_num = -((v2v2 * _DotProduct(v1, start_dif)) - (v1v2 * _DotProduct(v2, start_dif)));
        let t_num = (v1v1 * _DotProduct(v2, start_dif)) - (v1v2 * _DotProduct(v1, start_dif));
        let mut s = s_num / denom;
        let mut t = t_num / denom;
        let mut done = true;

        if s < 0.0 {
            done = false;
            s = 0.0;
        }
        if s > 1.0 {
            done = false;
            s = 1.0;
        }
        if t < 0.0 {
            done = false;
            t = 0.0;
        }
        if t > 1.0 {
            done = false;
            t = 1.0;
        }

        _VectorMA(start1, s, v1, close_pnt1);
        _VectorMA(start2, t, v2, close_pnt2);

        current_dist = Distance(*close_pnt1, *close_pnt2);
        if done {
            return current_dist;
        }
    } else {
        // Raven uses `Q3_INFINITE` (16777216), not a true infinity, as the
        // parallel-line sentinel. Source: `oracle/codemp/game/g_utils.c:2212`
        current_dist = Q3_INFINITE as f32;
    }

    let mut new_dist = Distance(start1, start2);
    if new_dist < current_dist {
        *close_pnt1 = start1;
        *close_pnt2 = start2;
        current_dist = new_dist;
    }

    new_dist = Distance(start1, end2);
    if new_dist < current_dist {
        *close_pnt1 = start1;
        *close_pnt2 = end2;
        current_dist = new_dist;
    }

    new_dist = Distance(end1, start2);
    if new_dist < current_dist {
        *close_pnt1 = end1;
        *close_pnt2 = start2;
        current_dist = new_dist;
    }

    new_dist = Distance(end1, end2);
    if new_dist < current_dist {
        *close_pnt1 = end1;
        *close_pnt2 = end2;
        current_dist = new_dist;
    }

    let mut new_pnt = [0.0f32; 3];

    G_FindClosestPointOnLineSegment(start2, end2, start1, &mut new_pnt);
    new_dist = Distance(start1, new_pnt);
    if new_dist < current_dist {
        *close_pnt1 = start1;
        *close_pnt2 = new_pnt;
        current_dist = new_dist;
    }

    G_FindClosestPointOnLineSegment(start2, end2, end1, &mut new_pnt);
    new_dist = Distance(end1, new_pnt);
    if new_dist < current_dist {
        *close_pnt1 = end1;
        *close_pnt2 = new_pnt;
        current_dist = new_dist;
    }

    G_FindClosestPointOnLineSegment(start1, end1, start2, &mut new_pnt);
    new_dist = Distance(start2, new_pnt);
    if new_dist < current_dist {
        *close_pnt1 = new_pnt;
        *close_pnt2 = start2;
        current_dist = new_dist;
    }

    G_FindClosestPointOnLineSegment(start1, end1, end2, &mut new_pnt);
    new_dist = Distance(end2, new_pnt);
    if new_dist < current_dist {
        *close_pnt1 = new_pnt;
        *close_pnt2 = end2;
        current_dist = new_dist;
    }

    current_dist
}

/// Raven `GetAnglesForDirection`. Reshape: `out` is the written
/// out-param, so it becomes `&mut [f32;3]`; `p1`/`p2` are read-only, kept
/// by-value (no same-file callers to fix up).
///
/// Source: `oracle/codemp/game/g_utils.c:2309-2315`
pub fn GetAnglesForDirection(p1: vec3_t, p2: vec3_t, out: &mut vec3_t) {
    let v = [p2[0] - p1[0], p2[1] - p1[1], p2[2] - p1[2]];
    vectoangles(v, out);
}
