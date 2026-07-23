//! `GameGlobals` — the remaining game-tier mutable file-scope globals
//! and file-statics as one owned GameWorld sub-struct: file-scope mutable
//! globals become GameWorld fields, grouped by owning `.c` file. Pass-2
//! porters read/write these through `ctx.world`; they
//! never add a field. Scalar decls carry their Rust type; non-scalar
//! decls (pointers/structs/arrays) are `()` placeholders with a
//! `//TODO: Port <type>` marker — the porter fills the real type when
//! porting that body (bg/qshared-owned globals and const tables are
//! intentionally excluded — not GameWorld state).
#![allow(non_snake_case, non_camel_case_types, unused)]

use core::ops::{Deref, DerefMut, Index, IndexMut};
use core::ptr::null_mut;
use std::alloc::{alloc_zeroed, handle_alloc_error, Layout};

use native_platform::zeroed_box;

use crate::botai::bot_state_s::bot_state_t;
use crate::level::bot_settings::bot_settings_t;

use crate::botai::nodeobject_s::nodeobject_t;
use crate::g_svcmds::ipFilter_t;
use crate::g_timer::{gtimer_t, MAX_GTIMERS};
use crate::game_cvars::GAME_CVAR_TABLE_LEN;
use crate::prelude::*;
use mp_qshared::shared::MAX_GENTITIES;

/// Generates the array-wrapper newtypes `GameGlobals` needs for arrays larger
/// than stable Rust's 32-element `Default` limit. Each arm emits the `pub`
/// tuple struct (with the forwarded attributes/doc comments) plus a `Default`
/// impl for one initialization strategy, and optionally `Index`/`IndexMut` or
/// `Deref`/`DerefMut`. Modes: `null` (raw-pointer arrays), `zero` (integer/char
/// arrays), `zero2d` (rectangular integer arrays), `elem` (element-`Default`
/// arrays), `boxed` (heap-`zeroed_box` arrays).
macro_rules! array_newtype {
    // Raw-pointer array, null-initialized.
    (null; $(#[$meta:meta])* $vis:vis $name:ident, $elem:ty, $n:expr) => {
        $(#[$meta])*
        $vis struct $name(pub [$elem; $n]);
        impl Default for $name {
            fn default() -> Self {
                $name([null_mut(); $n])
            }
        }
    };
    (null, index; $(#[$meta:meta])* $vis:vis $name:ident, $elem:ty, $n:expr) => {
        array_newtype!(null; $(#[$meta])* $vis $name, $elem, $n);
        array_newtype!(@index $name, $elem);
    };
    // Integer/char array, zero-initialized.
    (zero; $(#[$meta:meta])* $vis:vis $name:ident, $elem:ty, $n:expr) => {
        $(#[$meta])*
        $vis struct $name(pub [$elem; $n]);
        impl Default for $name {
            fn default() -> Self {
                $name([0; $n])
            }
        }
    };
    (zero, deref; $(#[$meta:meta])* $vis:vis $name:ident, $elem:ty, $n:expr) => {
        array_newtype!(zero; $(#[$meta])* $vis $name, $elem, $n);
        impl Deref for $name {
            type Target = [$elem];
            fn deref(&self) -> &[$elem] {
                &self.0
            }
        }
        impl DerefMut for $name {
            fn deref_mut(&mut self) -> &mut [$elem] {
                &mut self.0
            }
        }
    };
    // Rectangular integer array, zero-initialized.
    (zero2d; $(#[$meta:meta])* $vis:vis $name:ident, $elem:ty, $c:expr, $r:expr) => {
        $(#[$meta])*
        $vis struct $name(pub [[$elem; $c]; $r]);
        impl Default for $name {
            fn default() -> Self {
                $name([[0; $c]; $r])
            }
        }
    };
    // Array of an element type that is itself `Copy + Default`.
    (elem; $(#[$meta:meta])* $vis:vis $name:ident, $elem:ty, $n:expr) => {
        $(#[$meta])*
        $vis struct $name(pub [$elem; $n]);
        impl Default for $name {
            fn default() -> Self {
                $name([<$elem>::default(); $n])
            }
        }
    };
    (elem, index; $(#[$meta:meta])* $vis:vis $name:ident, $elem:ty, $n:expr) => {
        array_newtype!(elem; $(#[$meta])* $vis $name, $elem, $n);
        array_newtype!(@index $name, $elem);
    };
    // Heap-allocated array, zero-initialized directly on the heap.
    (boxed; $(#[$meta:meta])* $vis:vis $name:ident, $elem:ty, $n:expr) => {
        $(#[$meta])*
        $vis struct $name(pub Box<[$elem; $n]>);
        impl Default for $name {
            fn default() -> Self {
                $name(zeroed_box())
            }
        }
    };
    (boxed, index; $(#[$meta:meta])* $vis:vis $name:ident, $elem:ty, $n:expr) => {
        array_newtype!(boxed; $(#[$meta])* $vis $name, $elem, $n);
        array_newtype!(@index $name, $elem);
    };
    // Shared `Index`/`IndexMut` over `self.0[i]`.
    (@index $name:ident, $elem:ty) => {
        impl Index<usize> for $name {
            type Output = $elem;
            fn index(&self, i: usize) -> &$elem {
                &self.0[i]
            }
        }
        impl IndexMut<usize> for $name {
            fn index_mut(&mut self, i: usize) -> &mut $elem {
                &mut self.0[i]
            }
        }
    };
}

/// `ipFilter_t ipFilters[MAX_IPFILTERS]` (`g_svcmds.c:54`). Newtype because a
/// 1024-element array has no library `Default` impl (only arrays up to 32
/// elements do in stable Rust).
#[derive(Clone, Copy)]
pub struct IpFilters(pub [ipFilter_t; MAX_IPFILTERS]);

impl Default for IpFilters {
    fn default() -> Self {
        IpFilters(
            [ipFilter_t {
                mask: 0,
                compare: 0,
            }; MAX_IPFILTERS],
        )
    }
}

impl core::ops::Index<usize> for IpFilters {
    type Output = ipFilter_t;
    fn index(&self, i: usize) -> &ipFilter_t {
        &self.0[i]
    }
}

impl core::ops::IndexMut<usize> for IpFilters {
    fn index_mut(&mut self, i: usize) -> &mut ipFilter_t {
        &mut self.0[i]
    }
}

// Raven `#define MAX_ITEMS 256`.
// Source: `oracle/codemp/game/bg_public.h:31`
pub const MAX_ITEMS: usize = 256;

// Raven `ai_wpnav.c` / `q_shared.h` waypoint-arena sizes. `MAX_WPARRAY_SIZE`
// canonical in `mp_qshared::shared::limits` (`c_int`, cast here);
// `MAX_NODETABLE_SIZE` canonical in `ai_wpnav` (`c_int`, cast here);
// `MAX_SPAWNPOINT_ARRAY` canonical in `ai_wpnav` (already `usize`, plain import).
// Source: `oracle/codemp/game/q_shared.h:993`,
//         `oracle/codemp/game/ai_main.h:15`,
//         `oracle/codemp/game/ai_wpnav.c:2505`
use crate::ai_wpnav::MAX_SPAWNPOINT_ARRAY;
const MAX_WPARRAY_SIZE: usize = mp_qshared::shared::limits::MAX_WPARRAY_SIZE as usize;
const MAX_NODETABLE_SIZE: usize = crate::ai_wpnav::MAX_NODETABLE_SIZE as usize;

// Raven `#define MAX_SHADER_REMAPS 128` / `MAX_G2_KILL_QUEUE 256` /
// `MAX_VEHICLES_AT_A_TIME 128` (`g_utils.c:15,875,384`). Pass-2 backfill of
// the `()` placeholders these fields carried (allowed: "replace a
// ()-placeholder field's type with the real one if your packet cites it").
pub(crate) const MAX_SHADER_REMAPS: usize = 128;
pub(crate) const MAX_G2_KILL_QUEUE: usize = 256;
pub(crate) const MAX_VEHICLES_AT_A_TIME: usize = 128;

// Raven `#define MAX_CHAT_BUFFER_SIZE 8192` (unless `_XBOX` is defined; MP
// uses the full 8192). `ai_main.h:19`.
// Source: `oracle/codemp/game/ai_main.h:15-18`
pub(crate) const MAX_CHAT_BUFFER_SIZE: usize = 8192;

// Raven `#define MAX_ARENAS 1024` / `MAX_BOTS 1024` / `BOT_SPAWN_QUEUE_DEPTH 16`
// (`g_bot.c:9,13,19`).
// Source: `oracle/codemp/game/bg_public.h:1022,1024`
//         `oracle/codemp/game/g_bot.c:19`
const MAX_ARENAS: usize = 1024;
const MAX_BOTS: usize = 1024;
const BOT_SPAWN_QUEUE_DEPTH: usize = 16;

// Raven `#define MAX_SABER_VICTIMS 16` (`w_saber.c:3503`) — the per-swing
// victim-tracking array bound shared by the `w_saber.c` file-statics.
// Source: `oracle/codemp/game/w_saber.c:3503`
const MAX_SABER_VICTIMS: usize = 16;

// Raven `#define MAX_SIEGE_INFO_SIZE 16384` (`bg_saga.h:1`) — sizes the
// `gParseObjectives` siege-config parse buffer. Canonical in
// `mp_bg::saga::siege_team_t` (`i32`, cast here).
// Source: `oracle/codemp/game/bg_saga.h:1`
const MAX_SIEGE_INFO_SIZE: usize = mp_bg::saga::siege_team_t::MAX_SIEGE_INFO_SIZE as usize;

/// `botSpawnQueue_t` — bot spawn queue entry (`g_bot.c:21-24`).
///
/// Source: `oracle/codemp/game/g_bot.c:21-24`
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct botSpawnQueue_t {
    pub clientNum: c_int,
    pub spawnTime: c_int,
}

/// `bot_state_t *botstates[MAX_CLIENTS]` — per-client bot AI state, now owned:
/// each slot is an `Option<Box<bot_state_t>>` (`None` ≡ Raven's null slot).
/// Raven allocated these off the `B_Alloc` bump pool; here the game owns them,
/// and the `Box`'s stable heap address feeds the raw-pointer body code in
/// `ai_main.c` unchanged via [`BotStates::ptr`].
/// Source: `oracle/codemp/game/ai_main.c:46`
pub struct BotStates(pub [Option<Box<bot_state_t>>; MAX_CLIENTS]);

impl Default for BotStates {
    fn default() -> Self {
        BotStates(core::array::from_fn(|_| None))
    }
}

impl BotStates {
    /// Raw `*mut bot_state_t` for slot `i` — the `Box`'s stable heap address, or
    /// null when the slot is `None`. `ai_main.c`'s body code reads bot state
    /// across `ctx`-mutating calls (STAGE-2b irreducible aliasing), so it takes
    /// this raw pointer rather than a checked borrow; `ptr(i).is_null()` is the
    /// faithful equivalent of Raven's `botstates[i] == NULL`.
    #[inline]
    pub fn ptr(&self, i: usize) -> *mut bot_state_t {
        match &self.0[i] {
            Some(b) => &**b as *const bot_state_t as *mut bot_state_t,
            None => null_mut(),
        }
    }
}

/// A fresh zeroed `bot_state_t` on the heap, mirroring Raven's
/// `memset(B_Alloc(sizeof(bot_state_t)), 0, ...)` in `BotAISetupClient`.
/// `bot_state_t` stopped being zero-valid when its `settings` field grew owned
/// `String`s, so this zeroes the POD bulk (`alloc_zeroed`) and then seats a
/// valid empty `settings` into the one owned slot — the codebase's
/// zeroed-then-seat convention (`zeroed_clients`).
/// Source: `oracle/codemp/game/ai_main.c:824-831`
pub fn zeroed_bot_state() -> Box<bot_state_t> {
    let layout = Layout::new::<bot_state_t>();
    // SAFETY: `alloc_zeroed` yields storage that is all-zero-valid for every
    // `bot_state_t` field except the `String`-bearing `settings`; the `ptr::write`
    // seats a valid empty `settings` (its zeroed bytes never dropped) before
    // ownership passes to the `Box`, so the whole state is initialized.
    unsafe {
        let p = alloc_zeroed(layout) as *mut bot_state_t;
        if p.is_null() {
            handle_alloc_error(layout);
        }
        core::ptr::write(
            core::ptr::addr_of_mut!((*p).settings),
            bot_settings_t::default(),
        );
        Box::from_raw(p)
    }
}

array_newtype!(null, index;
    /// `gNPC_t *gNPCPtrs[MAX_GENTITIES]` — per-entity NPC state pointers
    /// (`NPC_spawn.c` file-scope global). Newtype because a raw-pointer array has
    /// no library `Default` impl.
    /// Source: `oracle/codemp/game/NPC_spawn.c:1276`
    pub GNpcPtrs, *mut gNPC_t, MAX_GENTITIES);

/// Raven `#define MAX_NPC_DATA_SIZE 0x20000` (`NPC_stats.c:236`).
pub const MAX_NPC_DATA_SIZE: usize = 0x20000;

array_newtype!(zero;
    /// Raven `char NPCParms[MAX_NPC_DATA_SIZE]` / `char npcParseBuffer[MAX_NPC_DATA_SIZE]`
    /// (`NPC_stats.c:237-3238`) — a fixed 128 KB NPC-config parse buffer. Newtype so
    /// `GameGlobals` keeps a derive-shaped `Default` (arrays > 32 have no library `Default`);
    /// `#[repr(transparent)]` keeps the `&globals.NPCParms as *const _ as *const c_char`
    /// porter idiom valid — the wrapper's address is the buffer's first byte.
    /// Source: `oracle/codemp/game/NPC_stats.c:236-238`
    #[repr(transparent)]
    pub NpcDataBuffer, c_char, MAX_NPC_DATA_SIZE);

array_newtype!(elem, index;
    /// `botSpawnQueue_t botSpawnQueue[BOT_SPAWN_QUEUE_DEPTH]` — spawn queue array (`g_bot.c:27`).
    /// Newtype for consistent interface with other large arrays.
    /// Source: `oracle/codemp/game/g_bot.c:27`
    #[derive(Clone, Copy)]
    pub BotSpawnQueue, botSpawnQueue_t, BOT_SPAWN_QUEUE_DEPTH);

array_newtype!(zero;
    /// `itemRegistered[MAX_ITEMS]` (`g_items.c:2966`). A thin wrapper because
    /// `[qboolean; 256]` has no library `Default` impl (only arrays up to 32
    /// elements do in stable Rust).
    #[derive(Clone, Copy)]
    pub ItemRegistered, qboolean, MAX_ITEMS);

array_newtype!(boxed;
    /// `gBotChatBuffer[MAX_CLIENTS][MAX_CHAT_BUFFER_SIZE]` — bot personality
    /// chat message buffers, one per client. Boxed so the ~256 KB of bytes lives on
    /// the heap (not the `GameGlobals` stack image, which the engine's
    /// `vmMain(GAME_INIT)` builds on a constrained stack).
    /// Source: `oracle/codemp/game/ai_util.c:12`
    pub BotChatBuffer, [c_char; MAX_CHAT_BUFFER_SIZE], MAX_CLIENTS);

array_newtype!(null;
    /// `wpobject_t *gWPArray[MAX_WPARRAY_SIZE]` — the waypoint arena, faithfully a
    /// fixed array of raw pointers into the `B_Alloc` bump arena (individually
    /// allocated, never freed). Newtype because a 4096-element array has no
    /// library `Default` (>32) and the entries are raw pointers (null-init).
    /// Source: `oracle/codemp/game/ai_main.h:398`
    pub WpArray, *mut wpobject_t, MAX_WPARRAY_SIZE);

array_newtype!(null;
    /// `gentity_t *gSpawnPoints[MAX_SPAWNPOINT_ARRAY]` (RMG autopath spawn set).
    /// Source: `oracle/codemp/game/ai_wpnav.c:2507`
    pub SpawnPointArray, *mut gentity_t, MAX_SPAWNPOINT_ARRAY);

array_newtype!(zero2d;
    /// `int G_WeaponLogDamage[MAX_CLIENTS][MOD_MAX]` (`g_log.c:21`). Newtype
    /// because the inner `[c_int; MOD_MAX]` (45 elements) has no library
    /// `Default` impl (only arrays up to 32 elements do in stable Rust).
    #[derive(Clone, Copy)]
    pub WeaponLogDamage, c_int, meansOfDeath_t::MOD_MAX as usize, MAX_CLIENTS);

array_newtype!(zero2d;
    /// `int G_WeaponLogKills[MAX_CLIENTS][MOD_MAX]` (`g_log.c:22`). Same
    /// >32-inner-array `Default` gap as `WeaponLogDamage`.
    #[derive(Clone, Copy)]
    pub WeaponLogKills, c_int, meansOfDeath_t::MOD_MAX as usize, MAX_CLIENTS);

array_newtype!(boxed;
    /// `nodeobject_t nodetable[MAX_NODETABLE_SIZE]` — the 16384-entry node-graph
    /// scratch table. Boxed so the ~458 KB of POD lives on the heap (not the
    /// `GameWorld` stack image) and default-zeroed (`nodeobject_t` is `#[repr(C)]`
    /// POD, so an all-zero image is valid).
    /// Source: `oracle/codemp/game/ai_wpnav.c:19`
    pub NodeTable, nodeobject_t, MAX_NODETABLE_SIZE);

/// `waypointData_t tempWaypointList[MAX_STORED_WAYPOINTS]` (`g_nav.c:1660`).
/// Boxed so the array lives on the heap (not the `GameGlobals` stack image,
/// which the engine's `vmMain(GAME_INIT)` builds on a constrained stack); the
/// element owns `String`s (non-`Copy`, no zero image), so it is built
/// element-by-element on the heap via a `Vec`.
/// Source: `oracle/codemp/game/g_nav.c:1660`
pub struct TempWaypointList(pub Box<[waypointData_t; MAX_STORED_WAYPOINTS]>);

impl Default for TempWaypointList {
    fn default() -> Self {
        let items: Vec<waypointData_t> =
            (0..MAX_STORED_WAYPOINTS).map(|_| waypointData_t::default()).collect();
        let boxed: Box<[waypointData_t; MAX_STORED_WAYPOINTS]> =
            items.into_boxed_slice().try_into().ok().unwrap();
        TempWaypointList(boxed)
    }
}

impl Index<usize> for TempWaypointList {
    type Output = waypointData_t;
    fn index(&self, i: usize) -> &waypointData_t {
        &self.0[i]
    }
}

impl IndexMut<usize> for TempWaypointList {
    fn index_mut(&mut self, i: usize) -> &mut waypointData_t {
        &mut self.0[i]
    }
}

/// Raven `shaderRemap_t` (`g_utils.c:8-13`): `{ char oldShader[MAX_QPATH];
/// char newShader[MAX_QPATH]; float timeOffset; }`. `oldShader`/`newShader` are
/// owned `String`s (the `MAX_QPATH` byte bound is applied at the write sites in
/// `AddRemap`); the struct is game-internal, so layout is free.
/// Source: `oracle/codemp/game/g_utils.c:8-13`
#[derive(Clone, Default)]
pub struct shaderRemap_t {
    pub oldShader: String,
    pub newShader: String,
    pub timeOffset: f32,
}

/// `shaderRemap_t remappedShaders[MAX_SHADER_REMAPS]` (`g_utils.c:18`). Newtype
/// because a 128-element array has no library `Default` (>32); the non-`Copy`
/// element (owns `String`s) rules out the `[x; N]` repeat form, so it is built
/// element-by-element via `core::array::from_fn`.
pub struct RemappedShaders(pub [shaderRemap_t; MAX_SHADER_REMAPS]);

impl Default for RemappedShaders {
    fn default() -> Self {
        RemappedShaders(core::array::from_fn(|_| shaderRemap_t::default()))
    }
}

array_newtype!(null;
    /// `gclient_t *gClPtrs[MAX_GENTITIES]` (`g_utils.c:428`) — the dynamically
    /// allocated NPC `gclient_t` backing store, indexed by entity number.
    /// Source: `oracle/codemp/game/g_utils.c:428`
    pub GClPtrs, *mut gclient_t, MAX_GENTITIES);

array_newtype!(zero;
    /// `int gG2KillIndex[MAX_G2_KILL_QUEUE]` (`g_utils.c:877`).
    pub GG2KillIndex, c_int, MAX_G2_KILL_QUEUE);

array_newtype!(zero;
    /// `qboolean g_vehiclePoolOccupied[MAX_VEHICLES_AT_A_TIME]` (`g_utils.c:386`).
    pub VehiclePoolOccupied, qboolean, MAX_VEHICLES_AT_A_TIME);

array_newtype!(boxed;
    /// `static Vehicle_t g_vehiclePool[MAX_VEHICLES_AT_A_TIME]` (`g_utils.c:385`) —
    /// the fixed pool `G_AllocateVehicleObject` hands out slots from. Boxed so the
    /// ~122 KB slab (`976 * 128`) lives on the heap, not in the `GameGlobals` stack
    /// image the engine builds during `vmMain(GAME_INIT)`; the all-zero start
    /// matches Raven's zero-initialized `static`.
    pub VehiclePool, Vehicle_t, MAX_VEHICLES_AT_A_TIME);

array_newtype!(boxed;
    /// `gtimer_t g_timerPool[MAX_GTIMERS]` (`g_timer.c:17`) — the fixed timer pool,
    /// intrusively linked into a free list. Boxed so the ~384 KB pool lives on the
    /// heap (not the `GameGlobals` stack image, which the engine's
    /// `vmMain(GAME_INIT)` builds on a constrained stack); the all-null/zero start
    /// matches Raven's zero-initialized pool.
    pub GTimerPool, gtimer_t, MAX_GTIMERS);

array_newtype!(null;
    /// `gtimer_t *g_timers[MAX_GENTITIES]` (`g_timer.c:18`) — per-entity timer
    /// list heads, indexed by entity number.
    pub GTimers, *mut gtimer_t, MAX_GENTITIES);

/// `navInfo_t frameNavInfo` (`NPC_move.c:14`) — per-frame NPC nav-move
/// scratch state. Newtype because `navInfo_t` embeds `trace_t`/raw pointers
/// with no library `Default` impl.
pub struct FrameNavInfo(pub navInfo_t);

impl Default for FrameNavInfo {
    fn default() -> Self {
        // Matches the oracle's static zero-initialization of `frameNavInfo`
        // and every runtime `memset(&frameNavInfo, 0, sizeof(frameNavInfo))`.
        FrameNavInfo(unsafe { core::mem::zeroed() })
    }
}

array_newtype!(zero;
    /// Per-row `cvarTable_t.modificationCount` cache (`g_main.c:22`). Raven
    /// stores this inline on each `gameCvarTable` row; this crate's
    /// `GAME_CVAR_TABLE` is a `const`, so the per-call-spanning cache lives here
    /// instead, indexed identically to `GAME_CVAR_TABLE`.
    pub GameCvarModCounts, c_int, GAME_CVAR_TABLE_LEN);

/// `CheckCvars`' function-scope `static int lastMod = -1` (`g_main.c:3456`) —
/// a genuine cross-frame static, homed here. Newtype so `GameGlobals` keeps
/// `#[derive(Default)]` while this field seeds to `-1` (not 0), matching Raven's
/// initializer so the first `CheckCvars` call always fires.
pub struct CheckCvarsLastMod(pub c_int);

impl Default for CheckCvarsLastMod {
    fn default() -> Self {
        CheckCvarsLastMod(-1)
    }
}

/// `teamgame_t` — CTF flag-state file global (`g_team.c:18`).
///
/// Source: `oracle/codemp/game/g_team.c:18`
#[derive(Clone, Copy, Default)]
pub struct teamgame_t {
    pub last_flag_capture: f32,
    pub last_capture_team: c_int,
    pub redStatus: flagStatus_t,
    pub blueStatus: flagStatus_t,
    pub flagStatus: flagStatus_t,
    pub redTakenTime: c_int,
    pub blueTakenTime: c_int,
}

/// Raven game-tier mutable file-scope globals, grouped into GameWorld fields.
pub struct GameGlobals {
    // --- `NPC.c` file-scope globals ---
    // Pass-2 backfill: `gentity_t *NPC;`/`gNPC_t *NPCInfo;`/`gclient_t *client;`
    // are single-pointer file statics (not `**` — the placeholder comment
    // mis-described the level of indirection), null-init like the other raw
    // pointer fields above.
    /// `NPC`. Source: `oracle/codemp/game/NPC.c:33`
    pub NPC: *mut gentity_t,
    /// `NPCInfo`. Source: `oracle/codemp/game/NPC.c:34`
    pub NPCInfo: *mut gNPC_t,
    /// `_saved_NPC`. Source: `oracle/codemp/game/NPC.c:625`
    pub _saved_NPC: *mut gentity_t,
    /// `_saved_NPCInfo`. Source: `oracle/codemp/game/NPC.c:626`
    pub _saved_NPCInfo: *mut gNPC_t,
    /// `_saved_client`. Source: `oracle/codemp/game/NPC.c:627`
    pub _saved_client: *mut gclient_t,
    /// `client`. Source: `oracle/codemp/game/NPC.c:35`
    pub client: *mut gclient_t,
    /// `enemyVisibility` (pass-2 backfill of the `()` placeholder — porting-rules
    /// §E13: "replace a ()-placeholder field's type with the real one if your
    /// packet cites it").
    /// Source: `oracle/codemp/game/NPC.c:38`
    pub enemyVisibility: crate::npc::visibility_t::visibility_t,
    /// `ucmd`. Source: `oracle/codemp/game/NPC.c:36`
    pub ucmd: usercmd_t,
    /// `_saved_ucmd` — the `SaveNPCGlobals`/`RestoreNPCGlobals` shadow copy of
    /// `ucmd` (genuine cross-frame state).
    /// Source: `oracle/codemp/game/NPC.c:628`
    pub _saved_ucmd: usercmd_t,
    // --- `NPC_AI_GalakMech.c` file-scope globals ---
    /// `enemyCS4`. Source: `oracle/codemp/game/NPC_AI_GalakMech.c:34`
    pub enemyCS4: qboolean,
    /// `enemyDist4`. Source: `oracle/codemp/game/NPC_AI_GalakMech.c:39`
    pub enemyDist4: f32,
    /// `enemyLOS4`. Source: `oracle/codemp/game/NPC_AI_GalakMech.c:33`
    pub enemyLOS4: qboolean,
    /// `faceEnemy4`. Source: `oracle/codemp/game/NPC_AI_GalakMech.c:36`
    pub faceEnemy4: qboolean,
    /// `hitAlly4`. Source: `oracle/codemp/game/NPC_AI_GalakMech.c:35`
    pub hitAlly4: qboolean,
    /// `move4`. Source: `oracle/codemp/game/NPC_AI_GalakMech.c:37`
    pub move4: qboolean,
    /// `shoot4`. Source: `oracle/codemp/game/NPC_AI_GalakMech.c:38`
    pub shoot4: qboolean,
    // --- `NPC_AI_Grenadier.c` file-scope globals ---
    /// `enemyCS3`. Source: `oracle/codemp/game/NPC_AI_Grenadier.c:35`
    pub enemyCS3: qboolean,
    /// `enemyDist3`. Source: `oracle/codemp/game/NPC_AI_Grenadier.c:39`
    pub enemyDist3: f32,
    /// `enemyLOS3`. Source: `oracle/codemp/game/NPC_AI_Grenadier.c:34`
    pub enemyLOS3: qboolean,
    /// `faceEnemy3`. Source: `oracle/codemp/game/NPC_AI_Grenadier.c:36`
    pub faceEnemy3: qboolean,
    /// `move3`. Source: `oracle/codemp/game/NPC_AI_Grenadier.c:37`
    pub move3: qboolean,
    /// `shoot3`. Source: `oracle/codemp/game/NPC_AI_Grenadier.c:38`
    pub shoot3: qboolean,
    // --- `NPC_AI_Jedi.c` file-scope globals ---
    /// `jediSpeechDebounceTime`. Source: `oracle/codemp/game/NPC_AI_Jedi.c:94`
    // §19: every .npc-parsed NPC has playerTeam == -1 (Raven's "NPC%s" sprintf
    // bug), so Raven indexes [-1] here — OOB read/write. Consumers use
    // .get()/.get_mut(): out-of-range reads pass the debounce, writes are skipped.
    pub jediSpeechDebounceTime: [c_int; TEAM_NUM_TEAMS as usize],
    // --- `NPC_AI_Sniper.c` file-scope globals ---
    /// `enemyCS2`. Source: `oracle/codemp/game/NPC_AI_Sniper.c:30`
    pub enemyCS2: qboolean,
    /// `enemyDist2`. Source: `oracle/codemp/game/NPC_AI_Sniper.c:34`
    pub enemyDist2: f32,
    /// `enemyLOS2`. Source: `oracle/codemp/game/NPC_AI_Sniper.c:29`
    pub enemyLOS2: qboolean,
    /// `faceEnemy2`. Source: `oracle/codemp/game/NPC_AI_Sniper.c:31`
    pub faceEnemy2: qboolean,
    /// `move2`. Source: `oracle/codemp/game/NPC_AI_Sniper.c:32`
    pub move2: qboolean,
    /// `shoot2`. Source: `oracle/codemp/game/NPC_AI_Sniper.c:33`
    pub shoot2: qboolean,
    // --- `NPC_AI_Stormtrooper.c` file-scope globals ---
    /// `enemyCS`. Source: `oracle/codemp/game/NPC_AI_Stormtrooper.c:41`
    pub enemyCS: qboolean,
    /// `enemyDist`. Source: `oracle/codemp/game/NPC_AI_Stormtrooper.c:47`
    pub enemyDist: f32,
    /// `enemyInFOV`. Source: `oracle/codemp/game/NPC_AI_Stormtrooper.c:42`
    pub enemyInFOV: qboolean,
    /// `enemyLOS`. Source: `oracle/codemp/game/NPC_AI_Stormtrooper.c:40`
    pub enemyLOS: qboolean,
    /// `faceEnemy`. Source: `oracle/codemp/game/NPC_AI_Stormtrooper.c:44`
    pub faceEnemy: qboolean,
    /// `groupSpeechDebounceTime[TEAM_NUM_TEAMS]` — stops several group AI from
    /// speaking all at once.
    /// Source: `oracle/codemp/game/NPC_AI_Stormtrooper.c:50`
    pub groupSpeechDebounceTime: [c_int; mp_bg::public::team::TEAM_NUM_TEAMS as usize],
    /// `hitAlly`. Source: `oracle/codemp/game/NPC_AI_Stormtrooper.c:43`
    pub hitAlly: qboolean,
    /// `move`. Source: `oracle/codemp/game/NPC_AI_Stormtrooper.c:45`
    pub r#move: qboolean,
    /// `shoot`. Source: `oracle/codemp/game/NPC_AI_Stormtrooper.c:46`
    pub shoot: qboolean,
    /// `static vec3_t impactPos` — last shot impact point (Stormtrooper aim).
    /// Source: `oracle/codemp/game/NPC_AI_Stormtrooper.c:48`
    pub impactPos: vec3_t,
    // --- `NPC_move.c` file-scope globals ---
    /// `navInfo_t frameNavInfo` — per-frame NPC nav-move scratch state,
    /// reused across calls within a frame.
    /// Source: `oracle/codemp/game/NPC_move.c:14`
    pub frameNavInfo: FrameNavInfo,
    // --- `NPC_spawn.c` file-scope globals ---
    /// `gNPCPtrs` (`gNPC_t *[MAX_GENTITIES]`; null-init raw pointers).
    /// Source: `oracle/codemp/game/NPC_spawn.c:1276`
    pub gNPCPtrs: GNpcPtrs,
    /// `showBBoxes`. Source: `oracle/codemp/game/NPC_spawn.c:4182`
    pub showBBoxes: qboolean,
    // --- `NPC_stats.c` file-scope globals ---
    /// Raven `char NPCParms[MAX_NPC_DATA_SIZE]` — the loaded NPC-config text.
    /// Source: `oracle/codemp/game/NPC_stats.c:237`
    pub NPCParms: NpcDataBuffer,
    /// Raven `char npcParseBuffer[MAX_NPC_DATA_SIZE]` — scratch parse buffer.
    /// Source: `oracle/codemp/game/NPC_stats.c:3238`
    pub npcParseBuffer: NpcDataBuffer,
    /// Raven `char NPCFile[MAX_QPATH]` — the currently-loading NPC-config file
    /// name (parse-error text only; never written in MP, so an owned `String`).
    /// Source: `oracle/codemp/game/NPC_stats.c:238`
    pub NPCFile: String,
    // --- `ai_main.c` file-scope globals ---
    /// `botstates` (`bot_state_t *[MAX_CLIENTS]`; owned `Option<Box<_>>` slots).
    /// Source: `oracle/codemp/game/ai_main.c:46`
    pub botstates: BotStates,
    /// `droppedBlueFlag` (`gentity_t *`; raw pointer, matches the
    /// raw-pointer entity signatures used throughout the pass-2 shards).
    /// Source: `oracle/codemp/game/ai_main.c:94`
    pub droppedBlueFlag: *mut gentity_t,
    /// `droppedRedFlag` (`gentity_t *`).
    /// Source: `oracle/codemp/game/ai_main.c:92`
    pub droppedRedFlag: *mut gentity_t,
    /// `eFlagBlue` (`gentity_t *`). Source: `oracle/codemp/game/ai_main.c:93`
    pub eFlagBlue: *mut gentity_t,
    /// `eFlagRed` (`gentity_t *`). Source: `oracle/codemp/game/ai_main.c:91`
    pub eFlagRed: *mut gentity_t,
    /// `flagBlue` (`wpobject_t *`; points into `gWPArray`).
    /// Source: `oracle/codemp/game/ai_main.c:88`
    pub flagBlue: *mut wpobject_t,
    /// `flagRed` (`wpobject_t *`; points into `gWPArray`).
    /// Source: `oracle/codemp/game/ai_main.c:86`
    pub flagRed: *mut wpobject_t,
    /// `boteventtracker_t gBotEventTracker[MAX_CLIENTS]`.
    /// Source: `oracle/codemp/game/ai_main.c:59`
    pub gBotEventTracker:
        [crate::botai::boteventtracker_s::boteventtracker_t; mp_qshared::shared::MAX_CLIENTS],
    /// `gUpdateVars`. Source: `oracle/codemp/game/ai_main.c:7485`
    pub gUpdateVars: c_int,
    /// `CheckCvars`' `static int lastMod = -1` (`g_main.c:3456`).
    pub checkCvarsLastMod: CheckCvarsLastMod,
    /// `static int lastbotthink_time` — bot-think cadence latch (function-scope
    /// static in `BotAIStartFrame`; genuine cross-frame state).
    /// Source: `oracle/codemp/game/ai_main.c:7497`
    pub lastbotthink_time: c_int,
    /// `static int local_time` — bot-frame elapsed-time cursor (function-scope
    /// static in `BotAIStartFrame`; genuine cross-frame state).
    /// Source: `oracle/codemp/game/ai_main.c:7495`
    pub local_time: c_int,
    /// `numbots`. Source: `oracle/codemp/game/ai_main.c:48`
    pub numbots: c_int,
    /// `oFlagBlue` (`wpobject_t *`). Source: `oracle/codemp/game/ai_main.c:89`
    pub oFlagBlue: *mut wpobject_t,
    /// `oFlagRed` (`wpobject_t *`). Source: `oracle/codemp/game/ai_main.c:87`
    pub oFlagRed: *mut wpobject_t,
    /// `regularupdate_time`. Source: `oracle/codemp/game/ai_main.c:52`
    pub regularupdate_time: f32,
    // --- `ai_main.h` file-scope globals ---
    /// `wpobject_t *gWPArray[MAX_WPARRAY_SIZE]` (see `WpArray`).
    /// Source: `oracle/codemp/game/ai_main.h:398`
    pub gWPArray: WpArray,
    // --- `ai_util.c` file-scope globals ---
    /// `gBotChatBuffer[MAX_CLIENTS][MAX_CHAT_BUFFER_SIZE]` — bot chat message
    /// buffers, one per client. Newtype because a 32×8192 array has no library
    /// `Default` impl (only arrays up to 32 elements do in stable Rust).
    /// Source: `oracle/codemp/game/ai_util.c:12`
    pub gBotChatBuffer: BotChatBuffer,
    // --- `ai_wpnav.c` file-scope globals ---
    /// `gBotEdit`. Source: `oracle/codemp/game/ai_wpnav.c:8`
    pub gBotEdit: f32,
    /// `gDeactivated`. Source: `oracle/codemp/game/ai_wpnav.c:7`
    pub gDeactivated: f32,
    /// `gLastPrintedIndex`. Source: `oracle/codemp/game/ai_wpnav.c:16`
    pub gLastPrintedIndex: c_int,
    /// `gLevelFlags`. Source: `oracle/codemp/game/ai_wpnav.c:23`
    pub gLevelFlags: c_int,
    /// `gSpawnPointNum`. Source: `oracle/codemp/game/ai_wpnav.c:2506`
    pub gSpawnPointNum: c_int,
    /// `gentity_t *gSpawnPoints[MAX_SPAWNPOINT_ARRAY]` (see `SpawnPointArray`).
    /// Source: `oracle/codemp/game/ai_wpnav.c:2507`
    pub gSpawnPoints: SpawnPointArray,
    /// `gWPNum`. Source: `oracle/codemp/game/ai_wpnav.c:13`
    pub gWPNum: c_int,
    /// `gWPRenderTime`. Source: `oracle/codemp/game/ai_wpnav.c:6`
    pub gWPRenderTime: f32,
    /// `gWPRenderedFrame`. Source: `oracle/codemp/game/ai_wpnav.c:9`
    pub gWPRenderedFrame: c_int,
    /// `nodenum`. Source: `oracle/codemp/game/ai_wpnav.c:20`
    pub nodenum: c_int,
    /// `nodeobject_t nodetable[MAX_NODETABLE_SIZE]` (see `NodeTable`).
    /// Source: `oracle/codemp/game/ai_wpnav.c:19`
    pub nodetable: NodeTable,
    // --- `g_bot.c` file-scope globals ---
    /// `botSpawnQueue_t botSpawnQueue[BOT_SPAWN_QUEUE_DEPTH]`.
    /// Source: `oracle/codemp/game/g_bot.c:27`
    pub botSpawnQueue: BotSpawnQueue,
    /// `char *g_botInfos[MAX_BOTS]` — bot info strings, owned.
    /// Source: `oracle/codemp/game/g_bot.c:9`
    pub g_botInfos: Vec<String>,
    /// `char *g_arenaInfos[MAX_ARENAS]` — arena info strings, owned.
    /// Source: `oracle/codemp/game/g_bot.c:13`
    pub g_arenaInfos: Vec<String>,
    /// `g_numArenas`. Source: `oracle/codemp/game/g_bot.c:12`
    pub g_numArenas: c_int,
    /// `g_numBots`. Source: `oracle/codemp/game/g_bot.c:8`
    pub g_numBots: c_int,
    /// `vmCvar_t bot_minplayers` — minimum players cvar.
    /// Source: `oracle/codemp/game/g_bot.c:1226`
    pub bot_minplayers: vmCvar_t,
    /// `static int checkminimumplayers_time` — function-static debounce timer
    /// (folded into GameGlobals per threading pattern).
    /// Source: `oracle/codemp/game/g_bot.c:572`
    pub checkminimumplayers_time: c_int,
    // --- `g_client.c` file-scope globals ---
    /// `void *g2SaberInstance` — the server's shared template ghoul2 saber instance handle.
    /// Source: `oracle/codemp/game/g_client.c:1414`
    pub g2SaberInstance: *mut c_void,
    /// Raven `gentity_t *gJMSaberEnt` — the current Jedi-Master saber entity.
    /// Source: `oracle/codemp/game/g_client.c:471`
    //
    // The general rule stores `gentity_t*` as `EntityId`, but g_client.rs transcribes
    // entities as raw `*mut gentity_t` throughout (its resolved signatures keep
    // raw pointers); `Option<_>` gives the nullable-pointer semantics a Default
    // (`None`) that a bare `*mut` lacks under this struct's `#[derive(Default)]`.
    pub gJMSaberEnt: Option<*mut gentity_t>,
    // --- `g_cmds.c` file-scope globals ---
    /// `g_dontPenalizeTeam`. Source: `oracle/codemp/game/g_cmds.c:750`
    pub g_dontPenalizeTeam: qboolean,
    /// `g_preventTeamBegin`. Source: `oracle/codemp/game/g_cmds.c:751`
    pub g_preventTeamBegin: qboolean,
    // --- `g_combat.c` file-scope globals ---
    /// `static int i` in `player_die` — rotates the EV_DEATH1..3 anim pick
    /// across deaths (function-scope static; genuine cross-frame state).
    /// Source: `oracle/codemp/game/g_combat.c:2858`
    pub death_anim_i: c_int,
    /// `gGAvoidDismember`. Source: `oracle/codemp/game/g_combat.c:3753`
    pub gGAvoidDismember: c_int,
    /// `gPainHitLoc`. Source: `oracle/codemp/game/g_combat.c:4574`
    pub gPainHitLoc: c_int,
    /// `gPainMOD`. Source: `oracle/codemp/game/g_combat.c:4573`
    pub gPainMOD: c_int,
    /// `vec3_t gPainPoint` — location of the last registered pain hit.
    /// Source: `oracle/codemp/game/g_combat.c:4575`
    pub gPainPoint: vec3_t,
    // --- `g_items.c` file-scope globals ---
    /// `itemRegistered[MAX_ITEMS]` (`MAX_ITEMS` = 256, `bg_public.h:31`).
    /// Array `Default` isn't derivable past 32 elements in stable Rust, so
    /// this is a thin newtype with its own `Default` impl (below).
    /// Source: `oracle/codemp/game/g_items.c:2966`
    pub itemRegistered: ItemRegistered,
    /// `shieldActivateSound` (`qhandle_t` = `c_int`).
    /// Source: `oracle/codemp/game/g_items.c:103`
    pub shieldActivateSound: qhandle_t,
    /// `shieldAttachSound` (`qhandle_t` = `c_int`).
    /// Source: `oracle/codemp/game/g_items.c:102`
    pub shieldAttachSound: qhandle_t,
    /// `shieldDamageSound` (`qhandle_t` = `c_int`).
    /// Source: `oracle/codemp/game/g_items.c:105`
    pub shieldDamageSound: qhandle_t,
    /// `shieldDeactivateSound` (`qhandle_t` = `c_int`).
    /// Source: `oracle/codemp/game/g_items.c:104`
    pub shieldDeactivateSound: qhandle_t,
    /// `shieldLoopSound` (`qhandle_t` = `c_int`).
    /// Source: `oracle/codemp/game/g_items.c:101`
    pub shieldLoopSound: qhandle_t,
    // --- `g_log.c` file-scope globals ---
    // Pass-2 backfill of the `()` placeholders (allowed: "replace a
    // ()-placeholder field's type with the real one if your packet cites
    // it"); shapes are exactly what the `g_log.md` packet's TODO comments
    // spelled out.
    /// `qboolean G_WeaponLogClientTouch[MAX_CLIENTS]`.
    /// Source: `oracle/codemp/game/g_log.c:27`
    pub G_WeaponLogClientTouch: [qboolean; MAX_CLIENTS],
    /// `int G_WeaponLogDamage[MAX_CLIENTS][MOD_MAX]`.
    /// Source: `oracle/codemp/game/g_log.c:21`
    pub G_WeaponLogDamage: WeaponLogDamage,
    /// `int G_WeaponLogDeaths[MAX_CLIENTS][WP_NUM_WEAPONS]`.
    /// Source: `oracle/codemp/game/g_log.c:23`
    pub G_WeaponLogDeaths: [[c_int; WP_NUM_WEAPONS as usize]; MAX_CLIENTS],
    /// `int G_WeaponLogFired[MAX_CLIENTS][WP_NUM_WEAPONS]`.
    /// Source: `oracle/codemp/game/g_log.c:20`
    pub G_WeaponLogFired: [[c_int; WP_NUM_WEAPONS as usize]; MAX_CLIENTS],
    /// `int G_WeaponLogFrags[MAX_CLIENTS][MAX_CLIENTS]`.
    /// Source: `oracle/codemp/game/g_log.c:24`
    pub G_WeaponLogFrags: [[c_int; MAX_CLIENTS]; MAX_CLIENTS],
    /// `int G_WeaponLogItems[MAX_CLIENTS][PW_NUM_POWERUPS]`.
    /// Source: `oracle/codemp/game/g_log.c:29`
    pub G_WeaponLogItems: [[c_int; PW_NUM_POWERUPS as usize]; MAX_CLIENTS],
    /// `int G_WeaponLogKills[MAX_CLIENTS][MOD_MAX]`.
    /// Source: `oracle/codemp/game/g_log.c:22`
    pub G_WeaponLogKills: WeaponLogKills,
    /// `int G_WeaponLogLastTime[MAX_CLIENTS]`.
    /// Source: `oracle/codemp/game/g_log.c:26`
    pub G_WeaponLogLastTime: [c_int; MAX_CLIENTS],
    /// `int G_WeaponLogPickups[MAX_CLIENTS][WP_NUM_WEAPONS]`.
    /// Source: `oracle/codemp/game/g_log.c:19`
    pub G_WeaponLogPickups: [[c_int; WP_NUM_WEAPONS as usize]; MAX_CLIENTS],
    /// `int G_WeaponLogPowerups[MAX_CLIENTS][HI_NUM_HOLDABLE]`.
    /// Source: `oracle/codemp/game/g_log.c:28`
    pub G_WeaponLogPowerups: [[c_int; HI_NUM_HOLDABLE as usize]; MAX_CLIENTS],
    /// `int G_WeaponLogTime[MAX_CLIENTS][WP_NUM_WEAPONS]`.
    /// Source: `oracle/codemp/game/g_log.c:25`
    pub G_WeaponLogTime: [[c_int; WP_NUM_WEAPONS as usize]; MAX_CLIENTS],
    // --- `g_main.c` file-scope globals ---
    /// `cvarTable_t.modificationCount` per-row cache (see `GameCvarModCounts`).
    /// Source: `oracle/codemp/game/g_main.c:17-25`
    pub gameCvarModCounts: GameCvarModCounts,
    /// `eventClearTime`. Source: `oracle/codemp/game/g_main.c:11`
    pub eventClearTime: c_int,
    /// `gDidDuelStuff`. Source: `oracle/codemp/game/g_main.c:2305`
    pub gDidDuelStuff: qboolean,
    /// `gDoSlowMoDuel`. Source: `oracle/codemp/game/g_main.c:3517`
    pub gDoSlowMoDuel: qboolean,
    /// `gDuelExit`. Source: `oracle/codemp/game/g_main.c:30`
    pub gDuelExit: qboolean,
    /// `gQueueScoreMessage`. Source: `oracle/codemp/game/g_main.c:1691`
    pub gQueueScoreMessage: qboolean,
    /// `gQueueScoreMessageTime`. Source: `oracle/codemp/game/g_main.c:1692`
    pub gQueueScoreMessageTime: c_int,
    /// `gSlowMoDuelTime`. Source: `oracle/codemp/game/g_main.c:3518`
    pub gSlowMoDuelTime: c_int,
    /// `g_LastFrameTime`. Source: `oracle/codemp/game/g_main.c:3514`
    pub g_LastFrameTime: c_int,
    /// `g_TimeSinceLastFrame`. Source: `oracle/codemp/game/g_main.c:3515`
    pub g_TimeSinceLastFrame: c_int,
    /// `g_dontFrickinCheck`. Source: `oracle/codemp/game/g_main.c:1427`
    pub g_dontFrickinCheck: qboolean,
    /// `g_duelPrintTimer`. Source: `oracle/codemp/game/g_main.c:2947`
    pub g_duelPrintTimer: c_int,
    /// `g_endPDuel`. Source: `oracle/codemp/game/g_main.c:2587`
    pub g_endPDuel: qboolean,
    /// `g_noPDuelCheck`. Source: `oracle/codemp/game/g_main.c:1717`
    pub g_noPDuelCheck: qboolean,
    /// `g_siegeRespawnCheck`. Source: `oracle/codemp/game/g_main.c:3580`
    pub g_siegeRespawnCheck: c_int,
    /// `killPlayerTimer`. Source: `oracle/codemp/game/g_main.c:15`
    pub killPlayerTimer: c_int,
    /// `navCalcPathTime`. Source: `oracle/codemp/game/g_main.c:12`
    pub navCalcPathTime: c_int,
    // --- `g_misc.c` file-scope globals ---
    /// `gEscapeTime`. Source: `oracle/codemp/game/g_misc.c:2541`
    pub gEscapeTime: c_int,
    /// `gEscaping`. Source: `oracle/codemp/game/g_misc.c:2540`
    pub gEscaping: qboolean,
    /// `g_shooterClientInit`. Source: `oracle/codemp/game/g_misc.c:3352`
    pub g_shooterClientInit: qboolean,
    /// `shooterClient_t g_shooterClients[MAX_SHOOTERS]`.
    /// Source: `oracle/codemp/game/g_misc.c:3351`
    pub g_shooterClients: [crate::g_misc::shooterClient_t; crate::g_misc::MAX_SHOOTERS as usize],
    // --- `g_mover.c` file-scope globals ---
    /// `pushed_t pushed[MAX_GENTITIES]` / `pushed_p` save-stack
    /// (`g_mover.c:19-24`) — one saved position/angle/deltayaw snapshot per
    /// moved entity, so a blocked mover push can roll everything back.
    /// Modeled as an owned `Vec` + cursor index rather than a raw pointer
    /// pair into a fixed array (porting-rules B3/B5); `pushed_p` (Raven:
    /// `pushed_t *pushed_p`) becomes an index into `pushed`.
    /// Source: `oracle/codemp/game/g_mover.c:19-24`
    pub pushed: Vec<crate::g_mover::PushedEntry>,
    pub pushed_p: usize,
    // --- `g_nav.c` file-scope globals ---
    /// `NAVDEBUG_curGoal`. Source: `oracle/codemp/game/g_nav.c:1607`
    pub NAVDEBUG_curGoal: c_int,
    /// `NAVDEBUG_showCollision`. Source: `oracle/codemp/game/g_nav.c:1606`
    pub NAVDEBUG_showCollision: qboolean,
    /// `NAVDEBUG_showCombatPoints`. Source: `oracle/codemp/game/g_nav.c:1604`
    pub NAVDEBUG_showCombatPoints: qboolean,
    /// `NAVDEBUG_showEdges`. Source: `oracle/codemp/game/g_nav.c:1601`
    pub NAVDEBUG_showEdges: qboolean,
    /// `NAVDEBUG_showEnemyPath`. Source: `oracle/codemp/game/g_nav.c:1603`
    pub NAVDEBUG_showEnemyPath: qboolean,
    /// `NAVDEBUG_showNavGoals`. Source: `oracle/codemp/game/g_nav.c:1605`
    pub NAVDEBUG_showNavGoals: qboolean,
    /// `NAVDEBUG_showNodes`. Source: `oracle/codemp/game/g_nav.c:1599`
    pub NAVDEBUG_showNodes: qboolean,
    /// `NAVDEBUG_showRadius`. Source: `oracle/codemp/game/g_nav.c:1600`
    pub NAVDEBUG_showRadius: qboolean,
    /// `NAVDEBUG_showTestPath`. Source: `oracle/codemp/game/g_nav.c:1602`
    pub NAVDEBUG_showTestPath: qboolean,
    /// Raven `char *fatalErrorPointer` — rolling write cursor into
    /// `fatalErrorString`. Modeled as a byte offset (not a raw pointer) per
    /// the no-aliasing-pointers-into-owned-arrays convention (porting-rules
    /// §B5); `fatalErrorPointer - fatalErrorString` in the oracle is this
    /// value directly.
    /// Source: `oracle/codemp/game/g_nav.c:1616`
    pub fatalErrorPointer: usize,
    /// Raven `char fatalErrorString[4096]` — the rolling nav-error log, appended
    /// to at `fatalErrorPointer` (an owned `String`; the 4096-byte cap is a
    /// load-time guard in `NAV_WaypointsTooFar`).
    /// Source: `oracle/codemp/game/g_nav.c:1617`
    pub fatalErrorString: String,
    /// `fatalErrors`. Source: `oracle/codemp/game/g_nav.c:1615`
    pub fatalErrors: c_int,
    /// `navCalculatePaths`. Source: `oracle/codemp/game/g_nav.c:1597`
    pub navCalculatePaths: qboolean,
    /// `numStoredWaypoints`. Source: `oracle/codemp/game/g_nav.c:1658`
    pub numStoredWaypoints: c_int,
    /// `waypointData_t tempWaypointList[MAX_STORED_WAYPOINTS]`.
    /// Source: `oracle/codemp/game/g_nav.c:1660`
    pub tempWaypointList: TempWaypointList,
    // --- `g_saga.c` file-scope globals ---
    /// `gImperialCountdown`. Source: `oracle/codemp/game/g_saga.c:30`
    pub gImperialCountdown: c_int,
    /// `static char team1[512]` — theme text for siege team 1 (an owned
    /// `String`; the 511-byte bound is applied at the write sites in `G_SiegeInit`).
    /// Source: `oracle/codemp/game/g_saga.c:17`
    pub team1: String,
    /// `static char team2[512]` — theme text for siege team 2 (an owned
    /// `String`; the 511-byte bound is applied at the write sites in `G_SiegeInit`).
    /// Source: `oracle/codemp/game/g_saga.c:18`
    pub team2: String,
    /// `static char gObjectiveCfgStr[1024]` — assembled objective config string
    /// (an owned `String`; the 1024-byte bound is applied at the write site in
    /// `G_SiegeCompleteObjective`).
    /// Source: `oracle/codemp/game/g_saga.c:47`
    pub gObjectiveCfgStr: String,
    /// `static char gParseObjectives[MAX_SIEGE_INFO_SIZE]` — siege-config parse
    /// buffer (an owned `String`; the parse fns produce owned values).
    /// Source: `oracle/codemp/game/g_saga.c:46`
    pub gParseObjectives: String,
    /// `gRebelCountdown`. Source: `oracle/codemp/game/g_saga.c:31`
    pub gRebelCountdown: c_int,
    /// `gSiegeBeginTime`. Source: `oracle/codemp/game/g_saga.c:39`
    pub gSiegeBeginTime: c_int,
    /// `gSiegeRoundBegun`. Source: `oracle/codemp/game/g_saga.c:36`
    pub gSiegeRoundBegun: qboolean,
    /// `gSiegeRoundEnded`. Source: `oracle/codemp/game/g_saga.c:37`
    pub gSiegeRoundEnded: qboolean,
    /// `gSiegeRoundWinningTeam`. Source: `oracle/codemp/game/g_saga.c:38`
    pub gSiegeRoundWinningTeam: qboolean,
    /// `g_preroundState`. Source: `oracle/codemp/game/g_saga.c:41`
    pub g_preroundState: c_int,
    /// `g_siegePersistant` (`siegePers_t`) — cross-round siege team-switch
    /// persistence, mirrored to the engine via `trap_SiegePersGet`/`Set`.
    /// Source: `oracle/codemp/game/g_saga.c:20`
    pub g_siegePersistant: siegePers_t,
    /// `imperial_attackers`. Source: `oracle/codemp/game/g_saga.c:34`
    pub imperial_attackers: c_int,
    /// `imperial_goals_completed`. Source: `oracle/codemp/game/g_saga.c:23`
    pub imperial_goals_completed: c_int,
    /// `imperial_goals_required`. Source: `oracle/codemp/game/g_saga.c:22`
    pub imperial_goals_required: c_int,
    /// `imperial_time_limit`. Source: `oracle/codemp/game/g_saga.c:27`
    pub imperial_time_limit: c_int,
    /// `rebel_attackers`. Source: `oracle/codemp/game/g_saga.c:33`
    pub rebel_attackers: c_int,
    /// `rebel_goals_completed`. Source: `oracle/codemp/game/g_saga.c:25`
    pub rebel_goals_completed: c_int,
    /// `rebel_goals_required`. Source: `oracle/codemp/game/g_saga.c:24`
    pub rebel_goals_required: c_int,
    /// `rebel_time_limit`. Source: `oracle/codemp/game/g_saga.c:28`
    pub rebel_time_limit: c_int,
    // --- `g_spawn.c` file-scope globals ---
    /// `void *precachedKyle` — the server's precached Kyle template ghoul2 instance handle.
    /// Source: `oracle/codemp/game/g_spawn.c:1226`
    pub precachedKyle: *mut c_void,
    /// `float g_cullDistance` — server-cull distance, set once from the
    /// `worldspawn` `distanceCull` key and read by vehicle crosshair tracing.
    /// Source: `oracle/codemp/game/g_spawn.c:1258`
    pub g_cullDistance: f32,
    // --- `g_svcmds.c` file-scope globals ---
    /// `ipFilter_t ipFilters[MAX_IPFILTERS]`.
    /// Source: oracle/codemp/game/g_svcmds.c:54
    pub ipFilters: IpFilters,
    /// `numIPFilters`. Source: `oracle/codemp/game/g_svcmds.c:55`
    pub numIPFilters: c_int,
    // --- `g_target.c` file-scope globals ---
    /// `numNewICARUSEnts`. Source: `oracle/codemp/game/g_target.c:753`
    pub numNewICARUSEnts: c_int,
    // --- `g_team.c` file-scope globals ---
    /// `teamgame` — CTF flag state.
    /// Source: oracle/codemp/game/g_team.c:18
    pub teamgame: teamgame_t,
    // --- `g_timer.c` file-scope globals ---
    /// `gtimer_t *g_timerFreeList` — head of the free-list of unused pool slots.
    /// Source: `oracle/codemp/game/g_timer.c:19`
    pub g_timerFreeList: *mut crate::g_timer::gtimer_t,
    /// `gtimer_t g_timerPool[MAX_GTIMERS]` — the fixed timer pool.
    /// Source: `oracle/codemp/game/g_timer.c:17`
    pub g_timerPool: GTimerPool,
    /// `gtimer_t *g_timers[MAX_GENTITIES]` — per-entity timer list heads.
    /// Source: `oracle/codemp/game/g_timer.c:18`
    pub g_timers: GTimers,
    // --- `g_trigger.c` file-scope globals ---
    /// `gTrigFallSound`. Source: `oracle/codemp/game/g_trigger.c:6`
    pub gTrigFallSound: c_int,
    // --- `g_utils.c` file-scope globals ---
    /// `gclient_t *gClPtrs[MAX_GENTITIES]` (see `GClPtrs`). Restored to Raven's
    /// `gclient_t*` element typing (safe-state Stage 4); reads no longer cast.
    /// Source: `oracle/codemp/game/g_utils.c:428`
    pub gClPtrs: GClPtrs,
    /// `int gG2KillIndex[MAX_G2_KILL_QUEUE]` (see `GG2KillIndex`).
    /// Source: `oracle/codemp/game/g_utils.c:877`
    pub gG2KillIndex: GG2KillIndex,
    /// `gG2KillNum`. Source: `oracle/codemp/game/g_utils.c:878`
    pub gG2KillNum: c_int,
    /// `g_vehiclePoolInit`. Source: `oracle/codemp/game/g_utils.c:387`
    pub g_vehiclePoolInit: qboolean,
    /// `qboolean g_vehiclePoolOccupied[MAX_VEHICLES_AT_A_TIME]` (see
    /// `VehiclePoolOccupied`).
    /// Source: `oracle/codemp/game/g_utils.c:386`
    pub g_vehiclePoolOccupied: VehiclePoolOccupied,
    /// `Vehicle_t g_vehiclePool[MAX_VEHICLES_AT_A_TIME]` (see `VehiclePool`).
    /// Source: `oracle/codemp/game/g_utils.c:385`
    pub g_vehiclePool: VehiclePool,
    /// `remapCount`. Source: `oracle/codemp/game/g_utils.c:17`
    pub remapCount: c_int,
    /// `shaderRemap_t remappedShaders[MAX_SHADER_REMAPS]` (see
    /// `RemappedShaders`).
    /// Source: `oracle/codemp/game/g_utils.c:18`
    pub remappedShaders: RemappedShaders,
    // --- `g_weapon.c` file-scope globals ---
    /// `s_quadFactor`. Source: `oracle/codemp/game/g_weapon.c:12`
    pub s_quadFactor: f32,
    /// `static vec3_t forward` — fire-time forward axis shared across
    /// `WP_Fire*`/`CalcMuzzlePoint`.
    /// Source: `oracle/codemp/game/g_weapon.c:13`
    pub forward: vec3_t,
    /// `static vec3_t vright` — fire-time right axis.
    /// Source: `oracle/codemp/game/g_weapon.c:13`
    pub vright: vec3_t,
    /// `static vec3_t up` — fire-time up axis.
    /// Source: `oracle/codemp/game/g_weapon.c:13`
    pub up: vec3_t,
    /// `static vec3_t muzzle` — fire-time muzzle point.
    /// Source: `oracle/codemp/game/g_weapon.c:14`
    pub muzzle: vec3_t,
    // --- `w_saber.c` file-scope globals ---
    /// `static vec3_t dmgDir[MAX_SABER_VICTIMS]` — per-victim saber damage
    /// direction.
    /// Source: `oracle/codemp/game/w_saber.c:3507`
    pub dmgDir: [vec3_t; MAX_SABER_VICTIMS],
    /// `static vec3_t dmgSpot[MAX_SABER_VICTIMS]` — per-victim saber impact
    /// point.
    /// Source: `oracle/codemp/game/w_saber.c:3508`
    pub dmgSpot: [vec3_t; MAX_SABER_VICTIMS],
    /// `static qboolean dismemberDmg[MAX_SABER_VICTIMS]` — per-victim dismember flag.
    /// Source: oracle/codemp/game/w_saber.c:3509
    pub dismemberDmg: [qboolean; MAX_SABER_VICTIMS],
    /// `numVictims`. Source: `oracle/codemp/game/w_saber.c:3511`
    pub numVictims: c_int,
    /// `saberClashEventParm`. Source: `oracle/codemp/game/w_saber.c:3797`
    pub saberClashEventParm: c_int,
    /// `static vec3_t saberClashNorm` — surface normal at the last saber clash.
    /// Source: `oracle/codemp/game/w_saber.c:3796`
    pub saberClashNorm: vec3_t,
    /// `static vec3_t saberClashPos` — world position of the last saber clash.
    /// Source: `oracle/codemp/game/w_saber.c:3795`
    pub saberClashPos: vec3_t,
    /// `saberDoClashEffect`. Source: `oracle/codemp/game/w_saber.c:3794`
    pub saberDoClashEffect: qboolean,
    /// `saberHitFraction`. Source: `oracle/codemp/game/w_saber.c:3848`
    pub saberHitFraction: f32,
    /// `saberHitSaber`. Source: `oracle/codemp/game/w_saber.c:3847`
    pub saberHitSaber: qboolean,
    /// `saberHitWall`. Source: `oracle/codemp/game/w_saber.c:3846`
    pub saberHitWall: qboolean,
    /// `static int saberKnockbackFlags[MAX_SABER_VICTIMS]` — per-victim knockback flags.
    /// Source: oracle/codemp/game/w_saber.c:3510
    pub saberKnockbackFlags: [c_int; MAX_SABER_VICTIMS],
    /// `saberSpinSound`. Source: `oracle/codemp/game/w_saber.c:18`
    pub saberSpinSound: c_int,
    /// `static float totalDmg[MAX_SABER_VICTIMS]` — per-victim accumulated damage.
    /// `f32` to match the oracle: accumulation, the wall-scale multiply, and the
    /// magnitude comparisons all run in float; only the `G_Damage` `int damage`
    /// argument truncates, at that call site.
    /// Source: oracle/codemp/game/w_saber.c:3506
    pub totalDmg: [f32; MAX_SABER_VICTIMS],
    /// `static int victimEntityNum[MAX_SABER_VICTIMS]` — per-victim entity number.
    /// Source: oracle/codemp/game/w_saber.c:3504
    pub victimEntityNum: [c_int; MAX_SABER_VICTIMS],
    /// `static qboolean victimHitEffectDone[MAX_SABER_VICTIMS]` — per-victim hit-effect flag.
    /// Source: oracle/codemp/game/w_saber.c:3505
    pub victimHitEffectDone: [qboolean; MAX_SABER_VICTIMS],
}

impl Default for GameGlobals {
    fn default() -> Self {
        // Manual impl (not `#[derive(Default)]`): Raven's file-scope
        // `int gSiegeBeginTime = Q3_INFINITE;` seeds that one field non-zero at
        // load time; every other field keeps the derived zero default.
        // Source: `oracle/codemp/game/g_saga.c:39`
        Self {
            NPC: Default::default(),
            NPCInfo: Default::default(),
            _saved_NPC: Default::default(),
            _saved_NPCInfo: Default::default(),
            _saved_client: Default::default(),
            client: Default::default(),
            enemyVisibility: Default::default(),
            ucmd: Default::default(),
            _saved_ucmd: Default::default(),
            enemyCS4: Default::default(),
            enemyDist4: Default::default(),
            enemyLOS4: Default::default(),
            faceEnemy4: Default::default(),
            hitAlly4: Default::default(),
            move4: Default::default(),
            shoot4: Default::default(),
            enemyCS3: Default::default(),
            enemyDist3: Default::default(),
            enemyLOS3: Default::default(),
            faceEnemy3: Default::default(),
            move3: Default::default(),
            shoot3: Default::default(),
            jediSpeechDebounceTime: Default::default(),
            enemyCS2: Default::default(),
            enemyDist2: Default::default(),
            enemyLOS2: Default::default(),
            faceEnemy2: Default::default(),
            move2: Default::default(),
            r#move: Default::default(),
            shoot2: Default::default(),
            enemyCS: Default::default(),
            enemyDist: Default::default(),
            enemyInFOV: Default::default(),
            enemyLOS: Default::default(),
            faceEnemy: Default::default(),
            groupSpeechDebounceTime: Default::default(),
            hitAlly: Default::default(),
            shoot: Default::default(),
            impactPos: Default::default(),
            frameNavInfo: Default::default(),
            gNPCPtrs: Default::default(),
            showBBoxes: Default::default(),
            NPCParms: Default::default(),
            npcParseBuffer: Default::default(),
            NPCFile: Default::default(),
            botstates: Default::default(),
            droppedBlueFlag: Default::default(),
            droppedRedFlag: Default::default(),
            eFlagBlue: Default::default(),
            eFlagRed: Default::default(),
            flagBlue: Default::default(),
            flagRed: Default::default(),
            gBotEventTracker: Default::default(),
            gUpdateVars: Default::default(),
            checkCvarsLastMod: Default::default(),
            lastbotthink_time: Default::default(),
            local_time: Default::default(),
            numbots: Default::default(),
            oFlagBlue: Default::default(),
            oFlagRed: Default::default(),
            regularupdate_time: Default::default(),
            gWPArray: Default::default(),
            gBotChatBuffer: Default::default(),
            gBotEdit: Default::default(),
            gDeactivated: Default::default(),
            // Raven file-scope `= -1`. Source: `oracle/codemp/game/ai_wpnav.c:16`
            gLastPrintedIndex: -1,
            gLevelFlags: Default::default(),
            gSpawnPointNum: Default::default(),
            gSpawnPoints: Default::default(),
            gWPNum: Default::default(),
            gWPRenderTime: Default::default(),
            gWPRenderedFrame: Default::default(),
            nodenum: Default::default(),
            nodetable: Default::default(),
            botSpawnQueue: Default::default(),
            g_botInfos: Default::default(),
            g_arenaInfos: Default::default(),
            g_numArenas: Default::default(),
            g_numBots: Default::default(),
            bot_minplayers: Default::default(),
            checkminimumplayers_time: Default::default(),
            g2SaberInstance: Default::default(),
            gJMSaberEnt: Default::default(),
            g_dontPenalizeTeam: Default::default(),
            g_preventTeamBegin: Default::default(),
            death_anim_i: Default::default(),
            gGAvoidDismember: Default::default(),
            // Raven file-scope `= -1`. Source: `oracle/codemp/game/g_combat.c:4574`
            gPainHitLoc: -1,
            gPainMOD: Default::default(),
            gPainPoint: Default::default(),
            itemRegistered: Default::default(),
            shieldActivateSound: Default::default(),
            shieldAttachSound: Default::default(),
            shieldDamageSound: Default::default(),
            shieldDeactivateSound: Default::default(),
            shieldLoopSound: Default::default(),
            G_WeaponLogClientTouch: Default::default(),
            G_WeaponLogDamage: Default::default(),
            G_WeaponLogDeaths: Default::default(),
            G_WeaponLogFired: Default::default(),
            G_WeaponLogFrags: Default::default(),
            G_WeaponLogItems: Default::default(),
            G_WeaponLogKills: Default::default(),
            G_WeaponLogLastTime: Default::default(),
            G_WeaponLogPickups: Default::default(),
            G_WeaponLogPowerups: Default::default(),
            G_WeaponLogTime: Default::default(),
            gameCvarModCounts: Default::default(),
            eventClearTime: Default::default(),
            gDidDuelStuff: Default::default(),
            gDoSlowMoDuel: Default::default(),
            gDuelExit: Default::default(),
            gQueueScoreMessage: Default::default(),
            gQueueScoreMessageTime: Default::default(),
            gSlowMoDuelTime: Default::default(),
            g_LastFrameTime: Default::default(),
            g_TimeSinceLastFrame: Default::default(),
            g_dontFrickinCheck: Default::default(),
            g_duelPrintTimer: Default::default(),
            g_endPDuel: Default::default(),
            g_noPDuelCheck: Default::default(),
            g_siegeRespawnCheck: Default::default(),
            killPlayerTimer: Default::default(),
            navCalcPathTime: Default::default(),
            gEscapeTime: Default::default(),
            gEscaping: Default::default(),
            g_shooterClientInit: Default::default(),
            g_shooterClients: Default::default(),
            pushed: Default::default(),
            pushed_p: Default::default(),
            NAVDEBUG_curGoal: Default::default(),
            NAVDEBUG_showCollision: Default::default(),
            NAVDEBUG_showCombatPoints: Default::default(),
            NAVDEBUG_showEdges: Default::default(),
            NAVDEBUG_showEnemyPath: Default::default(),
            NAVDEBUG_showNavGoals: Default::default(),
            NAVDEBUG_showNodes: Default::default(),
            NAVDEBUG_showRadius: Default::default(),
            NAVDEBUG_showTestPath: Default::default(),
            fatalErrorPointer: Default::default(),
            fatalErrorString: Default::default(),
            fatalErrors: Default::default(),
            navCalculatePaths: Default::default(),
            numStoredWaypoints: Default::default(),
            tempWaypointList: Default::default(),
            gImperialCountdown: Default::default(),
            team1: Default::default(),
            team2: Default::default(),
            gObjectiveCfgStr: Default::default(),
            gParseObjectives: Default::default(),
            gRebelCountdown: Default::default(),
            gSiegeBeginTime: Q3_INFINITE,
            gSiegeRoundBegun: Default::default(),
            gSiegeRoundEnded: Default::default(),
            gSiegeRoundWinningTeam: Default::default(),
            g_preroundState: Default::default(),
            g_siegePersistant: Default::default(),
            imperial_attackers: Default::default(),
            imperial_goals_completed: Default::default(),
            imperial_goals_required: Default::default(),
            imperial_time_limit: Default::default(),
            rebel_attackers: Default::default(),
            rebel_goals_completed: Default::default(),
            rebel_goals_required: Default::default(),
            rebel_time_limit: Default::default(),
            precachedKyle: Default::default(),
            g_cullDistance: Default::default(),
            ipFilters: Default::default(),
            numIPFilters: Default::default(),
            numNewICARUSEnts: Default::default(),
            teamgame: Default::default(),
            g_timerFreeList: Default::default(),
            g_timerPool: Default::default(),
            g_timers: Default::default(),
            gTrigFallSound: Default::default(),
            gClPtrs: Default::default(),
            gG2KillIndex: Default::default(),
            gG2KillNum: Default::default(),
            g_vehiclePoolInit: Default::default(),
            g_vehiclePoolOccupied: Default::default(),
            g_vehiclePool: Default::default(),
            remapCount: Default::default(),
            remappedShaders: Default::default(),
            s_quadFactor: Default::default(),
            forward: Default::default(),
            vright: Default::default(),
            up: Default::default(),
            muzzle: Default::default(),
            dmgDir: Default::default(),
            dmgSpot: Default::default(),
            dismemberDmg: Default::default(),
            numVictims: Default::default(),
            // Raven file-scope `= 1`. Source: `oracle/codemp/game/w_saber.c:3797`
            saberClashEventParm: 1,
            saberClashNorm: Default::default(),
            saberClashPos: Default::default(),
            saberDoClashEffect: Default::default(),
            saberHitFraction: Default::default(),
            saberHitSaber: Default::default(),
            saberHitWall: Default::default(),
            saberKnockbackFlags: Default::default(),
            saberSpinSound: Default::default(),
            totalDmg: Default::default(),
            victimEntityNum: Default::default(),
            victimHitEffectDone: Default::default(),
        }
    }
}
