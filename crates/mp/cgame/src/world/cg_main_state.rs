//! `CgMainState` — `cg_main.c`'s mutable file-scope globals as one `CgWorld`
//! sub-struct.

#![allow(non_snake_case)]

use core::ffi::c_int;

use mp_qshared::common::mp::cgame::ref_entity_t::refEntity_t;
use mp_qshared::shared::vec3_t;

/// One `CG_DrawMiscEnts` draw record — Raven's three lockstep arrays
/// `MiscEnts[MAX_MISC_ENTS]` / `Radius[MAX_MISC_ENTS]` / `zOffset[MAX_MISC_ENTS]`
/// plus the `NumMiscEnts` counter, folded into a single `Vec` entry (`Vec::len`
/// stands in for `NumMiscEnts`; §C9 — internal shape is free, this wave has no
/// `MAX_MISC_ENTS`-sized-array consumer to match).
///
/// Source: `oracle/codemp/cgame/cg_main.c:130-136`
#[derive(Clone, Copy)]
pub struct CgMiscEnt {
    /// Raven `MiscEnts[i]`.
    pub ent: refEntity_t,
    /// Raven `Radius[i]`.
    pub radius: f32,
    /// Raven `zOffset[i]`.
    pub zOffset: f32,
}

/// `cg_main.c`'s mutable file-scope globals, grouped by owning `.c` file
/// (§B3: file-scope globals become owned state, they never become Rust
/// globals).
///
/// Fields fold in as the waves transcribe `cg_main.c`'s file-scope statics
/// (DEC-46.1); a wave transcriber only ever touches its own TU's two files —
/// the function file and this one — and never `cg_world.rs`. Raven's read-only
/// tables beside them are compiled-in data, not state; they land as `const`s
/// beside the functions that read them (§C8).
///
/// The `strPool` buffer itself isn't here: `CG_StrPool_Alloc` hands back an
/// owned buffer instead of a pointer into a bump arena, so only the budget
/// offset below survives (`CG_StrPool_Reset` rewinding it to 0 IS the free).
///
/// Source: `oracle/codemp/cgame/cg_main.c:113,115,145-149,695-696,3310-3311,3411-3414,3417,3429-3431`
#[derive(Clone)]
pub struct CgMainState {
    /// Raven `refEntity_t MiscEnts[]` / `float Radius[]` / `float zOffset[]` /
    /// `int NumMiscEnts` — see [`CgMiscEnt`].
    /// Source: `oracle/codemp/cgame/cg_main.c:130-136`
    pub miscEnts: Vec<CgMiscEnt>,

    /// Raven `int forceModelModificationCount` — `cg_forceModel`'s last-seen
    /// `vmCvar_t::modificationCount`, set by `CG_RegisterCvars`.
    /// Source: `oracle/codemp/cgame/cg_main.c:115`
    pub forceModelModificationCount: c_int,

    /// Raven `centity_t *cg_permanents[MAX_GENTITIES]` plus its
    /// `int cg_numpermanents` count — the RMG's permanent entities, latched by
    /// `CG_TransitionPermanent` and walked by `cg_ents.c`/`cg_predict.c`.
    /// Entity numbers, not pointers (§B5); `Vec::len` stands in for
    /// `cg_numpermanents`, the same fold [`CgMiscEnt`] got.
    /// Source: `oracle/codemp/cgame/cg_main.c:695-696`
    pub cg_permanents: Vec<usize>,

    /// Raven `static int cg_strPoolSize` — bump offset into the `cg_strPool`
    /// intern buffer. `CG_StrPool_Reset` rewinding it to 0 IS the free.
    /// Source: `oracle/codemp/cgame/cg_main.c:3310`
    pub cg_strPoolSize: c_int,

    /// Raven `static int cg_numSpawnVarChars` — how many bytes of the
    /// `MAX_SPAWN_VARS_CHARS` spawn-var budget `CG_AddSpawnVarToken` has handed
    /// out this map load.
    ///
    /// Raven's `cg_spawnVarChars[]` backing buffer itself is gone: it only
    /// existed to give a parsed token stable storage, and the port's tokens are
    /// owned `String`s (the same fold `G_AddSpawnVarToken` got in
    /// `crates/mp/game/src/g_spawn.rs:828-831`). The counter stays because the
    /// budget overrun is an observable `CG_Error`.
    /// Source: `oracle/codemp/cgame/cg_main.c:3412`
    pub cg_numSpawnVarChars: c_int,

    /// Raven `static char *cg_spawnVars[MAX_SPAWN_VARS][2]` plus its
    /// `int cg_numSpawnVars` count — one map entity's key/value pairs, refilled
    /// by `CG_ParseSpawnVars` per entity. Owned `String`s rather than pointers
    /// into `cg_spawnVarChars[]` (`CG_AddSpawnVarToken` hands back the owned
    /// copy); `Vec::len` stands in for `cg_numSpawnVars`, the same fold
    /// [`CgMiscEnt`] got.
    /// Source: `oracle/codemp/cgame/cg_main.c:3411,3413`
    pub cg_spawnVars: Vec<[String; 2]>,

    /// Raven `qboolean cg_noFogOutsidePortal` — the map's sky portal asked for
    /// all global fog to live inside it (`cg_view.c` reads it).
    /// Source: `oracle/codemp/cgame/cg_main.c:3417`
    pub cg_noFogOutsidePortal: bool,

    /// Raven `qboolean cg_skyOri` — the map placed a sky portal origin, so the
    /// portal view parallaxes with the player.
    /// Source: `oracle/codemp/cgame/cg_main.c:3429`
    pub cg_skyOri: bool,

    /// Raven `vec3_t cg_skyOriPos`.
    /// Source: `oracle/codemp/cgame/cg_main.c:3430`
    pub cg_skyOriPos: vec3_t,

    /// Raven `float cg_skyOriScale`.
    /// Source: `oracle/codemp/cgame/cg_main.c:3431`
    pub cg_skyOriScale: f32,
}

impl Default for CgMainState {
    /// Raven's initializers — zeroed BSS except `forceModelModificationCount`,
    /// which starts at `-1` so the first `CG_RegisterCvars` pass always sees a
    /// force-model change.
    /// Source: `oracle/codemp/cgame/cg_main.c:115`
    fn default() -> Self {
        CgMainState {
            miscEnts: Vec::new(),
            forceModelModificationCount: -1,
            cg_permanents: Vec::new(),
            cg_strPoolSize: 0,
            cg_numSpawnVarChars: 0,
            cg_spawnVars: Vec::new(),
            cg_noFogOutsidePortal: false,
            cg_skyOri: false,
            cg_skyOriPos: [0.0; 3],
            cg_skyOriScale: 0.0,
        }
    }
}
