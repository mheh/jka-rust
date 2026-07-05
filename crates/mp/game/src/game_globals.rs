//! `GameGlobals` — the remaining game-tier mutable file-scope globals
//! and file-statics as one owned GameWorld sub-struct (fork ruling 1:
//! file-scope mutable globals become GameWorld fields, grouped by owning
//! `.c` file). Pass-2 porters read/write these through `ctx.world`; they
//! never add a field. Scalar decls carry their Rust type; non-scalar
//! decls (pointers/structs/arrays) are `()` placeholders with a
//! `//TODO: Port <type>` marker — the porter fills the real type when
//! porting that body (bg/qshared-owned globals and const tables are
//! intentionally excluded — not GameWorld state).
#![allow(non_snake_case, non_camel_case_types, unused)]

use crate::prelude::*;
use crate::botai::nodeobject_s::nodeobject_t;
use crate::g_svcmds::ipFilter_t;

/// `ipFilter_t ipFilters[MAX_IPFILTERS]` (`g_svcmds.c:54`). Newtype because a
/// 1024-element array has no library `Default` impl (only arrays up to 32
/// elements do in stable Rust).
#[derive(Clone, Copy)]
pub struct IpFilters(pub [ipFilter_t; MAX_IPFILTERS]);

impl Default for IpFilters {
    fn default() -> Self {
        IpFilters([ipFilter_t { mask: 0, compare: 0 }; MAX_IPFILTERS])
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
// Source: `oracle/oracle/codemp/game/bg_public.h:31`
pub const MAX_ITEMS: usize = 256;

// Raven `ai_wpnav.c` / `q_shared.h` waypoint-arena sizes.
// Source: `oracle/oracle/codemp/game/q_shared.h:993`,
//         `oracle/oracle/codemp/game/ai_main.h:15`,
//         `oracle/oracle/codemp/game/ai_wpnav.c:2505`
const MAX_WPARRAY_SIZE: usize = 4096;
const MAX_NODETABLE_SIZE: usize = 16384;
const MAX_SPAWNPOINT_ARRAY: usize = 64;

// Raven `#define MAX_SHADER_REMAPS 128` / `MAX_G2_KILL_QUEUE 256` /
// `MAX_VEHICLES_AT_A_TIME 128` (`g_utils.c:15,875,384`). Pass-2 backfill of
// the `()` placeholders these fields carried (allowed: "replace a
// ()-placeholder field's type with the real one if your packet cites it").
pub(crate) const MAX_SHADER_REMAPS: usize = 128;
pub(crate) const MAX_G2_KILL_QUEUE: usize = 256;
pub(crate) const MAX_VEHICLES_AT_A_TIME: usize = 128;

// Raven `#define MAX_CHAT_BUFFER_SIZE 8192` (unless `_XBOX` is defined; MP
// uses the full 8192). `ai_main.h:19`.
// Source: `oracle/oracle/codemp/game/ai_main.h:15-18`
pub(crate) const MAX_CHAT_BUFFER_SIZE: usize = 8192;

// Raven `#define MAX_ARENAS 1024` / `MAX_BOTS 1024` / `BOT_SPAWN_QUEUE_DEPTH 16`
// (`g_bot.c:9,13,19`).
// Source: `oracle/oracle/codemp/game/bg_public.h:1022,1024`
//         `oracle/oracle/codemp/game/g_bot.c:19`
const MAX_ARENAS: usize = 1024;
const MAX_BOTS: usize = 1024;
const BOT_SPAWN_QUEUE_DEPTH: usize = 16;

// Raven `#define MAX_SABER_VICTIMS 16` (`w_saber.c:3503`) — the per-swing
// victim-tracking array bound shared by the `w_saber.c` file-statics.
// Source: `oracle/oracle/codemp/game/w_saber.c:3503`
const MAX_SABER_VICTIMS: usize = 16;

// Raven `#define MAX_SIEGE_INFO_SIZE 16384` (`bg_saga.h:1`) — sizes the
// `gParseObjectives` siege-config parse buffer.
// Source: `oracle/oracle/codemp/game/bg_saga.h:1`
const MAX_SIEGE_INFO_SIZE: usize = 16384;

/// `botSpawnQueue_t` — bot spawn queue entry (`g_bot.c:21-24`).
///
/// Source: `oracle/oracle/codemp/game/g_bot.c:21-24`
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct botSpawnQueue_t {
    pub clientNum: c_int,
    pub spawnTime: c_int,
}

/// `char *g_botInfos[MAX_BOTS]` — bot info strings (`g_bot.c:9`). Newtype because
/// a 1024-element array has no library `Default` impl (only arrays up to 32 elements do).
/// Source: `oracle/oracle/codemp/game/g_bot.c:9`
pub struct BotInfos(pub [*mut c_char; MAX_BOTS]);

impl Default for BotInfos {
    fn default() -> Self {
        BotInfos([core::ptr::null_mut(); MAX_BOTS])
    }
}

// Transparent indexing (`globals.g_botInfos[i]`) so call sites written
// against a plain `[*mut c_char; MAX_BOTS]` need no change.
impl core::ops::Index<usize> for BotInfos {
    type Output = *mut c_char;
    fn index(&self, i: usize) -> &*mut c_char {
        &self.0[i]
    }
}
impl core::ops::IndexMut<usize> for BotInfos {
    fn index_mut(&mut self, i: usize) -> &mut *mut c_char {
        &mut self.0[i]
    }
}

/// `char *g_arenaInfos[MAX_ARENAS]` — arena info strings (`g_bot.c:13`). Newtype because
/// a 1024-element array has no library `Default` impl (only arrays up to 32 elements do).
/// Source: `oracle/oracle/codemp/game/g_bot.c:13`
pub struct ArenaInfos(pub [*mut c_char; MAX_ARENAS]);

impl Default for ArenaInfos {
    fn default() -> Self {
        ArenaInfos([core::ptr::null_mut(); MAX_ARENAS])
    }
}

// Transparent indexing (`globals.g_arenaInfos[i]`) so call sites written
// against a plain `[*mut c_char; MAX_ARENAS]` need no change.
impl core::ops::Index<usize> for ArenaInfos {
    type Output = *mut c_char;
    fn index(&self, i: usize) -> &*mut c_char {
        &self.0[i]
    }
}
impl core::ops::IndexMut<usize> for ArenaInfos {
    fn index_mut(&mut self, i: usize) -> &mut *mut c_char {
        &mut self.0[i]
    }
}

/// Raven `#define MAX_NPC_DATA_SIZE 0x20000` (`NPC_stats.c:236`).
pub const MAX_NPC_DATA_SIZE: usize = 0x20000;

/// Raven `char NPCParms[MAX_NPC_DATA_SIZE]` / `char npcParseBuffer[MAX_NPC_DATA_SIZE]`
/// (`NPC_stats.c:237-3238`) — a fixed 128 KB NPC-config parse buffer. Newtype so
/// `GameGlobals` keeps `#[derive(Default)]` (arrays > 32 have no library `Default`);
/// `#[repr(transparent)]` keeps the `&globals.NPCParms as *const _ as *const c_char`
/// porter idiom valid — the wrapper's address is the buffer's first byte.
/// Source: `oracle/oracle/codemp/game/NPC_stats.c:236-238`
#[repr(transparent)]
pub struct NpcDataBuffer(pub [c_char; MAX_NPC_DATA_SIZE]);

impl Default for NpcDataBuffer {
    fn default() -> Self {
        NpcDataBuffer([0; MAX_NPC_DATA_SIZE])
    }
}

/// `botSpawnQueue_t botSpawnQueue[BOT_SPAWN_QUEUE_DEPTH]` — spawn queue array (`g_bot.c:27`).
/// Newtype for consistent interface with other large arrays.
/// Source: `oracle/oracle/codemp/game/g_bot.c:27`
#[derive(Clone, Copy)]
pub struct BotSpawnQueue(pub [botSpawnQueue_t; BOT_SPAWN_QUEUE_DEPTH]);

impl Default for BotSpawnQueue {
    fn default() -> Self {
        BotSpawnQueue([botSpawnQueue_t::default(); BOT_SPAWN_QUEUE_DEPTH])
    }
}

// Transparent indexing (`globals.botSpawnQueue[i]`) so call sites written
// against a plain `[botSpawnQueue_t; BOT_SPAWN_QUEUE_DEPTH]` need no change.
impl core::ops::Index<usize> for BotSpawnQueue {
    type Output = botSpawnQueue_t;
    fn index(&self, i: usize) -> &botSpawnQueue_t {
        &self.0[i]
    }
}
impl core::ops::IndexMut<usize> for BotSpawnQueue {
    fn index_mut(&mut self, i: usize) -> &mut botSpawnQueue_t {
        &mut self.0[i]
    }
}

/// `itemRegistered[MAX_ITEMS]` (`g_items.c:2966`). A thin wrapper because
/// `[qboolean; 256]` has no library `Default` impl (only arrays up to 32
/// elements do in stable Rust).
#[derive(Clone, Copy)]
pub struct ItemRegistered(pub [qboolean; MAX_ITEMS]);

impl Default for ItemRegistered {
    fn default() -> Self {
        ItemRegistered([0; MAX_ITEMS])
    }
}

/// `gBotChatBuffer[MAX_CLIENTS][MAX_CHAT_BUFFER_SIZE]` — bot personality
/// chat message buffers, one per client. Newtype because a 32×8192 array of
/// bytes has no library `Default` impl (only arrays up to 32 elements do in
/// stable Rust).
/// Source: `oracle/oracle/codemp/game/ai_util.c:12`
pub struct BotChatBuffer(pub [[c_char; MAX_CHAT_BUFFER_SIZE]; mp_qshared::shared::MAX_CLIENTS]);

impl Default for BotChatBuffer {
    fn default() -> Self {
        BotChatBuffer([[0; MAX_CHAT_BUFFER_SIZE]; mp_qshared::shared::MAX_CLIENTS])
    }
}

/// `wpobject_t *gWPArray[MAX_WPARRAY_SIZE]` — the waypoint arena, faithfully a
/// fixed array of raw pointers into the `B_Alloc` bump arena (individually
/// allocated, never freed). Newtype because a 4096-element array has no
/// library `Default` (>32) and the entries are raw pointers (null-init).
/// Source: `oracle/oracle/codemp/game/ai_main.h:398`
pub struct WpArray(pub [*mut wpobject_t; MAX_WPARRAY_SIZE]);

impl Default for WpArray {
    fn default() -> Self {
        WpArray([core::ptr::null_mut(); MAX_WPARRAY_SIZE])
    }
}

/// `gentity_t *gSpawnPoints[MAX_SPAWNPOINT_ARRAY]` (RMG autopath spawn set).
/// Source: `oracle/oracle/codemp/game/ai_wpnav.c:2507`
pub struct SpawnPointArray(pub [*mut gentity_t; MAX_SPAWNPOINT_ARRAY]);

impl Default for SpawnPointArray {
    fn default() -> Self {
        SpawnPointArray([core::ptr::null_mut(); MAX_SPAWNPOINT_ARRAY])
    }
}

/// `int G_WeaponLogDamage[MAX_CLIENTS][MOD_MAX]` (`g_log.c:21`). Newtype
/// because the inner `[c_int; MOD_MAX]` (45 elements) has no library
/// `Default` impl (only arrays up to 32 elements do in stable Rust).
#[derive(Clone, Copy)]
pub struct WeaponLogDamage(pub [[c_int; meansOfDeath_t::MOD_MAX as usize]; MAX_CLIENTS]);

impl Default for WeaponLogDamage {
    fn default() -> Self {
        WeaponLogDamage([[0; meansOfDeath_t::MOD_MAX as usize]; MAX_CLIENTS])
    }
}

/// `int G_WeaponLogKills[MAX_CLIENTS][MOD_MAX]` (`g_log.c:22`). Same
/// >32-inner-array `Default` gap as `WeaponLogDamage`.
#[derive(Clone, Copy)]
pub struct WeaponLogKills(pub [[c_int; meansOfDeath_t::MOD_MAX as usize]; MAX_CLIENTS]);

impl Default for WeaponLogKills {
    fn default() -> Self {
        WeaponLogKills([[0; meansOfDeath_t::MOD_MAX as usize]; MAX_CLIENTS])
    }
}

/// `nodeobject_t nodetable[MAX_NODETABLE_SIZE]` — the 16384-entry node-graph
/// scratch table. Boxed so the ~458 KB of POD lives on the heap (not the
/// `GameWorld` stack image) and default-zeroed (`nodeobject_t` is `#[repr(C)]`
/// POD, so an all-zero image is valid).
/// Source: `oracle/oracle/codemp/game/ai_wpnav.c:19`
pub struct NodeTable(pub Box<[nodeobject_t; MAX_NODETABLE_SIZE]>);

impl Default for NodeTable {
    fn default() -> Self {
        // SAFETY: `nodeobject_t` is `#[repr(C)]` POD (`vec3_t`/`f32`/`c_int`);
        // an all-zero bit pattern is a valid inhabitant.
        NodeTable(Box::new(unsafe { core::mem::zeroed() }))
    }
}

/// `waypointData_t tempWaypointList[MAX_STORED_WAYPOINTS]` (`g_nav.c:1660`).
/// Same >32-array `Default` gap as `NodeTable`; `waypointData_t` is
/// `#[repr(C)]` POD so an all-zero image is valid.
/// Source: `oracle/oracle/codemp/game/g_nav.c:1660`
#[derive(Clone, Copy)]
pub struct TempWaypointList(pub [waypointData_t; MAX_STORED_WAYPOINTS]);

impl Default for TempWaypointList {
    fn default() -> Self {
        // SAFETY: `waypointData_t` is `#[repr(C)]` POD (`c_char`/`c_int`
        // fields only); an all-zero bit pattern is a valid inhabitant.
        TempWaypointList(unsafe { core::mem::zeroed() })
    }
}

// Transparent indexing (`globals.tempWaypointList[i]`) so call sites written
// against a plain `[waypointData_t; MAX_STORED_WAYPOINTS]` need no change.
impl core::ops::Index<usize> for TempWaypointList {
    type Output = waypointData_t;
    fn index(&self, i: usize) -> &waypointData_t {
        &self.0[i]
    }
}
impl core::ops::IndexMut<usize> for TempWaypointList {
    fn index_mut(&mut self, i: usize) -> &mut waypointData_t {
        &mut self.0[i]
    }
}

/// `char fatalErrorString[4096]` — newtype because a 4096-byte array has no
/// library `Default` impl (only arrays up to 32 elements do in stable Rust),
/// same gap as `BotChatBuffer`.
/// Source: `oracle/oracle/codemp/game/g_nav.c:1617`
#[derive(Clone, Copy)]
pub struct FatalErrorString(pub [c_char; 4096]);

impl Default for FatalErrorString {
    fn default() -> Self {
        FatalErrorString([0; 4096])
    }
}

/// Raven `char NPCFile[MAX_QPATH]` (`NPC_stats.c:238`) — the currently-loading
/// NPC-config file name. `#[repr(transparent)]` keeps the porter idiom
/// `&globals.NPCFile as *const _ as *const c_char` valid (same treatment as
/// `NpcDataBuffer`); newtype so `GameGlobals` keeps `#[derive(Default)]`
/// (`MAX_QPATH` = 64 > 32 has no library `Default`).
/// Source: `oracle/oracle/codemp/game/NPC_stats.c:238`
#[repr(transparent)]
pub struct NpcFileBuffer(pub [c_char; MAX_QPATH]);

impl Default for NpcFileBuffer {
    fn default() -> Self {
        NpcFileBuffer([0; MAX_QPATH])
    }
}

/// Raven `char gObjectiveCfgStr[1024]` (`g_saga.c:47`). Newtype (>32 array has
/// no library `Default`); `Deref`/`DerefMut` to `[c_char]` keep the porter
/// idioms `.as_ptr()`/`.as_mut_ptr()` and `write_cstr_field(&mut field, …)`
/// valid.
/// Source: `oracle/oracle/codemp/game/g_saga.c:47`
pub struct ObjectiveCfgStr(pub [c_char; 1024]);

impl Default for ObjectiveCfgStr {
    fn default() -> Self {
        ObjectiveCfgStr([0; 1024])
    }
}

impl core::ops::Deref for ObjectiveCfgStr {
    type Target = [c_char];
    fn deref(&self) -> &[c_char] {
        &self.0
    }
}
impl core::ops::DerefMut for ObjectiveCfgStr {
    fn deref_mut(&mut self) -> &mut [c_char] {
        &mut self.0
    }
}

/// Raven `char gParseObjectives[MAX_SIEGE_INFO_SIZE]` (`g_saga.c:46`). Newtype
/// (>32 array has no library `Default`); `Deref`/`DerefMut` to `[c_char]` keep
/// the porter idiom `.as_mut_ptr()` valid.
/// Source: `oracle/oracle/codemp/game/g_saga.c:46`
pub struct ParseObjectivesBuffer(pub [c_char; MAX_SIEGE_INFO_SIZE]);

impl Default for ParseObjectivesBuffer {
    fn default() -> Self {
        ParseObjectivesBuffer([0; MAX_SIEGE_INFO_SIZE])
    }
}

impl core::ops::Deref for ParseObjectivesBuffer {
    type Target = [c_char];
    fn deref(&self) -> &[c_char] {
        &self.0
    }
}
impl core::ops::DerefMut for ParseObjectivesBuffer {
    fn deref_mut(&mut self) -> &mut [c_char] {
        &mut self.0
    }
}

/// Raven `shaderRemap_t` (`g_utils.c:8-13`): `{ char oldShader[MAX_QPATH];
/// char newShader[MAX_QPATH]; float timeOffset; }`.
/// Source: `oracle/oracle/codemp/game/g_utils.c:8-13`
#[derive(Clone, Copy)]
pub struct shaderRemap_t {
    pub oldShader: [c_char; MAX_QPATH],
    pub newShader: [c_char; MAX_QPATH],
    pub timeOffset: f32,
}

impl Default for shaderRemap_t {
    fn default() -> Self {
        shaderRemap_t {
            oldShader: [0; MAX_QPATH],
            newShader: [0; MAX_QPATH],
            timeOffset: 0.0,
        }
    }
}

/// `shaderRemap_t remappedShaders[MAX_SHADER_REMAPS]` (`g_utils.c:18`).
/// Newtype because a 128-element array of a non-`Copy`-array-friendly struct
/// has no library `Default` (>32).
pub struct RemappedShaders(pub [shaderRemap_t; MAX_SHADER_REMAPS]);

impl Default for RemappedShaders {
    fn default() -> Self {
        RemappedShaders([shaderRemap_t::default(); MAX_SHADER_REMAPS])
    }
}

/// `gclient_t *gClPtrs[MAX_GENTITIES]` (`g_utils.c:428`) — the dynamically
/// allocated NPC `gclient_t` backing store, indexed by entity number.
/// Source: `oracle/oracle/codemp/game/g_utils.c:428`
pub struct GClPtrs(pub [*mut c_void; mp_qshared::shared::MAX_GENTITIES]);

impl Default for GClPtrs {
    fn default() -> Self {
        GClPtrs([core::ptr::null_mut(); mp_qshared::shared::MAX_GENTITIES])
    }
}

/// `int gG2KillIndex[MAX_G2_KILL_QUEUE]` (`g_utils.c:877`).
pub struct GG2KillIndex(pub [c_int; MAX_G2_KILL_QUEUE]);

impl Default for GG2KillIndex {
    fn default() -> Self {
        GG2KillIndex([0; MAX_G2_KILL_QUEUE])
    }
}

/// `qboolean g_vehiclePoolOccupied[MAX_VEHICLES_AT_A_TIME]` (`g_utils.c:386`).
pub struct VehiclePoolOccupied(pub [qboolean; MAX_VEHICLES_AT_A_TIME]);

impl Default for VehiclePoolOccupied {
    fn default() -> Self {
        VehiclePoolOccupied([0; MAX_VEHICLES_AT_A_TIME])
    }
}

/// `teamgame_t` — CTF flag-state file global (`g_team.c:18`).
///
/// Source: `oracle/oracle/codemp/game/g_team.c:18`
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

/// Raven game-tier mutable file-scope globals (fork ruling 1).
#[derive(Default)]
pub struct GameGlobals {
    // --- `NPC.c` file-scope globals ---
    // Pass-2 backfill: `gentity_t *NPC;`/`gNPC_t *NPCInfo;`/`gclient_t *client;`
    // are single-pointer file statics (not `**` — the placeholder comment
    // mis-described the level of indirection), null-init like the other raw
    // pointer fields above.
    /// `NPC`. Source: `oracle/oracle/codemp/game/NPC.c:33`
    pub NPC: *mut gentity_t,
    /// `NPCInfo`. Source: `oracle/oracle/codemp/game/NPC.c:34`
    pub NPCInfo: *mut gNPC_t,
    /// `_saved_NPC`. Source: `oracle/oracle/codemp/game/NPC.c:625`
    pub _saved_NPC: *mut gentity_t,
    /// `_saved_NPCInfo`. Source: `oracle/oracle/codemp/game/NPC.c:626`
    pub _saved_NPCInfo: *mut gNPC_t,
    /// `_saved_client`. Source: `oracle/oracle/codemp/game/NPC.c:627`
    pub _saved_client: *mut gclient_t,
    /// `client`. Source: `oracle/oracle/codemp/game/NPC.c:35`
    pub client: *mut gclient_t,
    /// `enemyVisibility` (pass-2 backfill of the `()` placeholder — porting-rules
    /// §E13: "replace a ()-placeholder field's type with the real one if your
    /// packet cites it").
    /// Source: `oracle/oracle/codemp/game/NPC.c:38`
    pub enemyVisibility: crate::npc::visibility_t::visibility_t,
    /// `ucmd`. Source: `oracle/oracle/codemp/game/NPC.c:36`
    pub ucmd: usercmd_t,
    /// `_saved_ucmd` — the `SaveNPCGlobals`/`RestoreNPCGlobals` shadow copy of
    /// `ucmd` (fork ruling 1: genuine cross-frame state).
    /// Source: `oracle/oracle/codemp/game/NPC.c:628`
    pub _saved_ucmd: usercmd_t,
    // --- `NPC_AI_GalakMech.c` file-scope globals ---
    /// `enemyCS4`. Source: `oracle/oracle/codemp/game/NPC_AI_GalakMech.c:34`
    pub enemyCS4: qboolean,
    /// `enemyDist4`. Source: `oracle/oracle/codemp/game/NPC_AI_GalakMech.c:39`
    pub enemyDist4: f32,
    /// `enemyLOS4`. Source: `oracle/oracle/codemp/game/NPC_AI_GalakMech.c:33`
    pub enemyLOS4: qboolean,
    /// `faceEnemy4`. Source: `oracle/oracle/codemp/game/NPC_AI_GalakMech.c:36`
    pub faceEnemy4: qboolean,
    /// `hitAlly4`. Source: `oracle/oracle/codemp/game/NPC_AI_GalakMech.c:35`
    pub hitAlly4: qboolean,
    /// `move4`. Source: `oracle/oracle/codemp/game/NPC_AI_GalakMech.c:37`
    pub move4: qboolean,
    /// `shoot4`. Source: `oracle/oracle/codemp/game/NPC_AI_GalakMech.c:38`
    pub shoot4: qboolean,
    // --- `NPC_AI_Grenadier.c` file-scope globals ---
    /// `enemyCS3`. Source: `oracle/oracle/codemp/game/NPC_AI_Grenadier.c:35`
    pub enemyCS3: qboolean,
    /// `enemyDist3`. Source: `oracle/oracle/codemp/game/NPC_AI_Grenadier.c:39`
    pub enemyDist3: f32,
    /// `enemyLOS3`. Source: `oracle/oracle/codemp/game/NPC_AI_Grenadier.c:34`
    pub enemyLOS3: qboolean,
    /// `faceEnemy3`. Source: `oracle/oracle/codemp/game/NPC_AI_Grenadier.c:36`
    pub faceEnemy3: qboolean,
    /// `move3`. Source: `oracle/oracle/codemp/game/NPC_AI_Grenadier.c:37`
    pub move3: qboolean,
    /// `shoot3`. Source: `oracle/oracle/codemp/game/NPC_AI_Grenadier.c:38`
    pub shoot3: qboolean,
    // --- `NPC_AI_Jedi.c` file-scope globals ---
    //TODO: Port int[TEAM_NUM_TEAMS]
    // Source: oracle/oracle/codemp/game/NPC_AI_Jedi.c:94
    pub jediSpeechDebounceTime: (),
    // --- `NPC_AI_Sniper.c` file-scope globals ---
    /// `enemyCS2`. Source: `oracle/oracle/codemp/game/NPC_AI_Sniper.c:30`
    pub enemyCS2: qboolean,
    /// `enemyDist2`. Source: `oracle/oracle/codemp/game/NPC_AI_Sniper.c:34`
    pub enemyDist2: f32,
    /// `enemyLOS2`. Source: `oracle/oracle/codemp/game/NPC_AI_Sniper.c:29`
    pub enemyLOS2: qboolean,
    /// `faceEnemy2`. Source: `oracle/oracle/codemp/game/NPC_AI_Sniper.c:31`
    pub faceEnemy2: qboolean,
    /// `move2`. Source: `oracle/oracle/codemp/game/NPC_AI_Sniper.c:32`
    pub move2: qboolean,
    /// `shoot2`. Source: `oracle/oracle/codemp/game/NPC_AI_Sniper.c:33`
    pub shoot2: qboolean,
    // --- `NPC_AI_Stormtrooper.c` file-scope globals ---
    /// `enemyCS`. Source: `oracle/oracle/codemp/game/NPC_AI_Stormtrooper.c:41`
    pub enemyCS: qboolean,
    /// `enemyDist`. Source: `oracle/oracle/codemp/game/NPC_AI_Stormtrooper.c:47`
    pub enemyDist: f32,
    /// `enemyInFOV`. Source: `oracle/oracle/codemp/game/NPC_AI_Stormtrooper.c:42`
    pub enemyInFOV: qboolean,
    /// `enemyLOS`. Source: `oracle/oracle/codemp/game/NPC_AI_Stormtrooper.c:40`
    pub enemyLOS: qboolean,
    /// `faceEnemy`. Source: `oracle/oracle/codemp/game/NPC_AI_Stormtrooper.c:44`
    pub faceEnemy: qboolean,
    /// `groupSpeechDebounceTime[TEAM_NUM_TEAMS]` — stops several group AI from
    /// speaking all at once.
    /// Source: `oracle/oracle/codemp/game/NPC_AI_Stormtrooper.c:50`
    pub groupSpeechDebounceTime: [c_int; mp_bg::public::team::TEAM_NUM_TEAMS as usize],
    /// `hitAlly`. Source: `oracle/oracle/codemp/game/NPC_AI_Stormtrooper.c:43`
    pub hitAlly: qboolean,
    /// `move`. Source: `oracle/oracle/codemp/game/NPC_AI_Stormtrooper.c:45`
    pub r#move: qboolean,
    /// `shoot`. Source: `oracle/oracle/codemp/game/NPC_AI_Stormtrooper.c:46`
    pub shoot: qboolean,
    /// `static vec3_t impactPos` — last shot impact point (Stormtrooper aim).
    /// Source: `oracle/oracle/codemp/game/NPC_AI_Stormtrooper.c:48`
    pub impactPos: vec3_t,
    // --- `NPC_move.c` file-scope globals ---
    //TODO: Port navInfo_t
    // Source: oracle/oracle/codemp/game/NPC_move.c:14
    pub frameNavInfo: (),
    // --- `NPC_spawn.c` file-scope globals ---
    //TODO: Port gNPC_t *[MAX_GENTITIES]*
    // Source: oracle/oracle/codemp/game/NPC_spawn.c:1276
    pub gNPCPtrs: (),
    /// `showBBoxes`. Source: `oracle/oracle/codemp/game/NPC_spawn.c:4182`
    pub showBBoxes: qboolean,
    // --- `NPC_stats.c` file-scope globals (ruling 24 — file-scope mutable → globals) ---
    /// Raven `char NPCParms[MAX_NPC_DATA_SIZE]` — the loaded NPC-config text.
    /// Source: `oracle/oracle/codemp/game/NPC_stats.c:237`
    pub NPCParms: NpcDataBuffer,
    /// Raven `char npcParseBuffer[MAX_NPC_DATA_SIZE]` — scratch parse buffer.
    /// Source: `oracle/oracle/codemp/game/NPC_stats.c:3238`
    pub npcParseBuffer: NpcDataBuffer,
    /// Raven `char NPCFile[MAX_QPATH]` — the currently-loading NPC-config file
    /// name (parse cursor state).
    /// Source: `oracle/oracle/codemp/game/NPC_stats.c:238`
    pub NPCFile: NpcFileBuffer,
    // --- `ai_main.c` file-scope globals ---
    //TODO: Port bot_state_t *[MAX_CLIENTS]*
    // Source: oracle/oracle/codemp/game/ai_main.c:46
    pub botstates: (),
    /// `droppedBlueFlag` (`gentity_t *`; raw pointer, matches the
    /// raw-pointer entity signatures used throughout the pass-2 shards).
    /// Source: `oracle/oracle/codemp/game/ai_main.c:94`
    pub droppedBlueFlag: *mut gentity_t,
    /// `droppedRedFlag` (`gentity_t *`).
    /// Source: `oracle/oracle/codemp/game/ai_main.c:92`
    pub droppedRedFlag: *mut gentity_t,
    /// `eFlagBlue` (`gentity_t *`). Source: `oracle/oracle/codemp/game/ai_main.c:93`
    pub eFlagBlue: *mut gentity_t,
    /// `eFlagRed` (`gentity_t *`). Source: `oracle/oracle/codemp/game/ai_main.c:91`
    pub eFlagRed: *mut gentity_t,
    /// `flagBlue` (`wpobject_t *`; points into `gWPArray`).
    /// Source: `oracle/oracle/codemp/game/ai_main.c:88`
    pub flagBlue: *mut wpobject_t,
    /// `flagRed` (`wpobject_t *`; points into `gWPArray`).
    /// Source: `oracle/oracle/codemp/game/ai_main.c:86`
    pub flagRed: *mut wpobject_t,
    //TODO: Port boteventtracker_t[MAX_CLIENTS]
    // Source: oracle/oracle/codemp/game/ai_main.c:59
    pub gBotEventTracker: (),
    /// `gUpdateVars`. Source: `oracle/oracle/codemp/game/ai_main.c:7485`
    pub gUpdateVars: c_int,
    /// `static int lastbotthink_time` — bot-think cadence latch (function-scope
    /// static in `BotAIStartFrame`; genuine cross-frame state per ruling 5).
    /// Source: `oracle/oracle/codemp/game/ai_main.c:7497`
    pub lastbotthink_time: c_int,
    /// `static int local_time` — bot-frame elapsed-time cursor (function-scope
    /// static in `BotAIStartFrame`; genuine cross-frame state per ruling 5).
    /// Source: `oracle/oracle/codemp/game/ai_main.c:7495`
    pub local_time: c_int,
    /// `numbots`. Source: `oracle/oracle/codemp/game/ai_main.c:48`
    pub numbots: c_int,
    /// `oFlagBlue` (`wpobject_t *`). Source: `oracle/oracle/codemp/game/ai_main.c:89`
    pub oFlagBlue: *mut wpobject_t,
    /// `oFlagRed` (`wpobject_t *`). Source: `oracle/oracle/codemp/game/ai_main.c:87`
    pub oFlagRed: *mut wpobject_t,
    /// `regularupdate_time`. Source: `oracle/oracle/codemp/game/ai_main.c:52`
    pub regularupdate_time: f32,
    // --- `ai_main.h` file-scope globals ---
    /// `wpobject_t *gWPArray[MAX_WPARRAY_SIZE]` (see `WpArray`).
    /// Source: `oracle/oracle/codemp/game/ai_main.h:398`
    pub gWPArray: WpArray,
    // --- `ai_util.c` file-scope globals ---
    /// `gBotChatBuffer[MAX_CLIENTS][MAX_CHAT_BUFFER_SIZE]` — bot chat message
    /// buffers, one per client. Newtype because a 32×8192 array has no library
    /// `Default` impl (only arrays up to 32 elements do in stable Rust).
    /// Source: `oracle/oracle/codemp/game/ai_util.c:12`
    pub gBotChatBuffer: BotChatBuffer,
    // --- `ai_wpnav.c` file-scope globals ---
    /// `gBotEdit`. Source: `oracle/oracle/codemp/game/ai_wpnav.c:8`
    pub gBotEdit: f32,
    /// `gDeactivated`. Source: `oracle/oracle/codemp/game/ai_wpnav.c:7`
    pub gDeactivated: f32,
    /// `gLastPrintedIndex`. Source: `oracle/oracle/codemp/game/ai_wpnav.c:16`
    pub gLastPrintedIndex: c_int,
    /// `gLevelFlags`. Source: `oracle/oracle/codemp/game/ai_wpnav.c:23`
    pub gLevelFlags: c_int,
    /// `gSpawnPointNum`. Source: `oracle/oracle/codemp/game/ai_wpnav.c:2506`
    pub gSpawnPointNum: c_int,
    /// `gentity_t *gSpawnPoints[MAX_SPAWNPOINT_ARRAY]` (see `SpawnPointArray`).
    /// Source: `oracle/oracle/codemp/game/ai_wpnav.c:2507`
    pub gSpawnPoints: SpawnPointArray,
    /// `gWPNum`. Source: `oracle/oracle/codemp/game/ai_wpnav.c:13`
    pub gWPNum: c_int,
    /// `gWPRenderTime`. Source: `oracle/oracle/codemp/game/ai_wpnav.c:6`
    pub gWPRenderTime: f32,
    /// `gWPRenderedFrame`. Source: `oracle/oracle/codemp/game/ai_wpnav.c:9`
    pub gWPRenderedFrame: c_int,
    /// `nodenum`. Source: `oracle/oracle/codemp/game/ai_wpnav.c:20`
    pub nodenum: c_int,
    /// `nodeobject_t nodetable[MAX_NODETABLE_SIZE]` (see `NodeTable`).
    /// Source: `oracle/oracle/codemp/game/ai_wpnav.c:19`
    pub nodetable: NodeTable,
    // --- `g_bot.c` file-scope globals ---
    /// `botSpawnQueue_t botSpawnQueue[BOT_SPAWN_QUEUE_DEPTH]`.
    /// Source: `oracle/oracle/codemp/game/g_bot.c:27`
    pub botSpawnQueue: BotSpawnQueue,
    /// `char *g_botInfos[MAX_BOTS]` — bot info strings.
    /// Source: `oracle/oracle/codemp/game/g_bot.c:9`
    pub g_botInfos: BotInfos,
    /// `char *g_arenaInfos[MAX_ARENAS]` — arena info strings.
    /// Source: `oracle/oracle/codemp/game/g_bot.c:13`
    pub g_arenaInfos: ArenaInfos,
    /// `g_numArenas`. Source: `oracle/oracle/codemp/game/g_bot.c:12`
    pub g_numArenas: c_int,
    /// `g_numBots`. Source: `oracle/oracle/codemp/game/g_bot.c:8`
    pub g_numBots: c_int,
    /// `vmCvar_t bot_minplayers` — minimum players cvar.
    /// Source: `oracle/oracle/codemp/game/g_bot.c:1226`
    pub bot_minplayers: vmCvar_t,
    /// `static int checkminimumplayers_time` — function-static debounce timer
    /// (folded into GameGlobals per threading pattern).
    /// Source: `oracle/oracle/codemp/game/g_bot.c:572`
    pub checkminimumplayers_time: c_int,
    // --- `g_client.c` file-scope globals ---
    /// `void *g2SaberInstance` — the server's shared template ghoul2 saber instance handle.
    /// Source: `oracle/oracle/codemp/game/g_client.c:1414`
    pub g2SaberInstance: *mut c_void,
    /// Raven `gentity_t *gJMSaberEnt` — the current Jedi-Master saber entity.
    /// Source: `oracle/oracle/codemp/game/g_client.c:471`
    //
    // Fork-4 rules stored `gentity_t*` → `EntityId`, but g_client.rs transcribes
    // entities as raw `*mut gentity_t` throughout (its resolved signatures keep
    // raw pointers); `Option<_>` gives the nullable-pointer semantics a Default
    // (`None`) that a bare `*mut` lacks under this struct's `#[derive(Default)]`.
    pub gJMSaberEnt: Option<*mut gentity_t>,
    // --- `g_cmds.c` file-scope globals ---
    /// `g_dontPenalizeTeam`. Source: `oracle/oracle/codemp/game/g_cmds.c:750`
    pub g_dontPenalizeTeam: qboolean,
    /// `g_preventTeamBegin`. Source: `oracle/oracle/codemp/game/g_cmds.c:751`
    pub g_preventTeamBegin: qboolean,
    // --- `g_combat.c` file-scope globals ---
    /// `gGAvoidDismember`. Source: `oracle/oracle/codemp/game/g_combat.c:3753`
    pub gGAvoidDismember: c_int,
    /// `gPainHitLoc`. Source: `oracle/oracle/codemp/game/g_combat.c:4574`
    pub gPainHitLoc: c_int,
    /// `gPainMOD`. Source: `oracle/oracle/codemp/game/g_combat.c:4573`
    pub gPainMOD: c_int,
    /// `vec3_t gPainPoint` — location of the last registered pain hit.
    /// Source: `oracle/oracle/codemp/game/g_combat.c:4575`
    pub gPainPoint: vec3_t,
    // --- `g_items.c` file-scope globals ---
    /// `itemRegistered[MAX_ITEMS]` (`MAX_ITEMS` = 256, `bg_public.h:31`).
    /// Array `Default` isn't derivable past 32 elements in stable Rust, so
    /// this is a thin newtype with its own `Default` impl (below).
    /// Source: `oracle/oracle/codemp/game/g_items.c:2966`
    pub itemRegistered: ItemRegistered,
    /// `shieldActivateSound` (`qhandle_t` = `c_int`).
    /// Source: `oracle/oracle/codemp/game/g_items.c:103`
    pub shieldActivateSound: qhandle_t,
    /// `shieldAttachSound` (`qhandle_t` = `c_int`).
    /// Source: `oracle/oracle/codemp/game/g_items.c:102`
    pub shieldAttachSound: qhandle_t,
    /// `shieldDamageSound` (`qhandle_t` = `c_int`).
    /// Source: `oracle/oracle/codemp/game/g_items.c:105`
    pub shieldDamageSound: qhandle_t,
    /// `shieldDeactivateSound` (`qhandle_t` = `c_int`).
    /// Source: `oracle/oracle/codemp/game/g_items.c:104`
    pub shieldDeactivateSound: qhandle_t,
    /// `shieldLoopSound` (`qhandle_t` = `c_int`).
    /// Source: `oracle/oracle/codemp/game/g_items.c:101`
    pub shieldLoopSound: qhandle_t,
    // --- `g_log.c` file-scope globals ---
    // Pass-2 backfill of the `()` placeholders (allowed: "replace a
    // ()-placeholder field's type with the real one if your packet cites
    // it"); shapes are exactly what the `g_log.md` packet's TODO comments
    // spelled out.
    /// `qboolean G_WeaponLogClientTouch[MAX_CLIENTS]`.
    /// Source: `oracle/oracle/codemp/game/g_log.c:27`
    pub G_WeaponLogClientTouch: [qboolean; MAX_CLIENTS],
    /// `int G_WeaponLogDamage[MAX_CLIENTS][MOD_MAX]`.
    /// Source: `oracle/oracle/codemp/game/g_log.c:21`
    pub G_WeaponLogDamage: WeaponLogDamage,
    /// `int G_WeaponLogDeaths[MAX_CLIENTS][WP_NUM_WEAPONS]`.
    /// Source: `oracle/oracle/codemp/game/g_log.c:23`
    pub G_WeaponLogDeaths: [[c_int; WP_NUM_WEAPONS as usize]; MAX_CLIENTS],
    /// `int G_WeaponLogFired[MAX_CLIENTS][WP_NUM_WEAPONS]`.
    /// Source: `oracle/oracle/codemp/game/g_log.c:20`
    pub G_WeaponLogFired: [[c_int; WP_NUM_WEAPONS as usize]; MAX_CLIENTS],
    /// `int G_WeaponLogFrags[MAX_CLIENTS][MAX_CLIENTS]`.
    /// Source: `oracle/oracle/codemp/game/g_log.c:24`
    pub G_WeaponLogFrags: [[c_int; MAX_CLIENTS]; MAX_CLIENTS],
    /// `int G_WeaponLogItems[MAX_CLIENTS][PW_NUM_POWERUPS]`.
    /// Source: `oracle/oracle/codemp/game/g_log.c:29`
    pub G_WeaponLogItems: [[c_int; PW_NUM_POWERUPS as usize]; MAX_CLIENTS],
    /// `int G_WeaponLogKills[MAX_CLIENTS][MOD_MAX]`.
    /// Source: `oracle/oracle/codemp/game/g_log.c:22`
    pub G_WeaponLogKills: WeaponLogKills,
    /// `int G_WeaponLogLastTime[MAX_CLIENTS]`.
    /// Source: `oracle/oracle/codemp/game/g_log.c:26`
    pub G_WeaponLogLastTime: [c_int; MAX_CLIENTS],
    /// `int G_WeaponLogPickups[MAX_CLIENTS][WP_NUM_WEAPONS]`.
    /// Source: `oracle/oracle/codemp/game/g_log.c:19`
    pub G_WeaponLogPickups: [[c_int; WP_NUM_WEAPONS as usize]; MAX_CLIENTS],
    /// `int G_WeaponLogPowerups[MAX_CLIENTS][HI_NUM_HOLDABLE]`.
    /// Source: `oracle/oracle/codemp/game/g_log.c:28`
    pub G_WeaponLogPowerups: [[c_int; HI_NUM_HOLDABLE as usize]; MAX_CLIENTS],
    /// `int G_WeaponLogTime[MAX_CLIENTS][WP_NUM_WEAPONS]`.
    /// Source: `oracle/oracle/codemp/game/g_log.c:25`
    pub G_WeaponLogTime: [[c_int; WP_NUM_WEAPONS as usize]; MAX_CLIENTS],
    // --- `g_main.c` file-scope globals ---
    /// `eventClearTime`. Source: `oracle/oracle/codemp/game/g_main.c:11`
    pub eventClearTime: c_int,
    /// `gDidDuelStuff`. Source: `oracle/oracle/codemp/game/g_main.c:2305`
    pub gDidDuelStuff: qboolean,
    /// `gDoSlowMoDuel`. Source: `oracle/oracle/codemp/game/g_main.c:3517`
    pub gDoSlowMoDuel: qboolean,
    /// `gDuelExit`. Source: `oracle/oracle/codemp/game/g_main.c:30`
    pub gDuelExit: qboolean,
    /// `gQueueScoreMessage`. Source: `oracle/oracle/codemp/game/g_main.c:1691`
    pub gQueueScoreMessage: qboolean,
    /// `gQueueScoreMessageTime`. Source: `oracle/oracle/codemp/game/g_main.c:1692`
    pub gQueueScoreMessageTime: c_int,
    /// `gSlowMoDuelTime`. Source: `oracle/oracle/codemp/game/g_main.c:3518`
    pub gSlowMoDuelTime: c_int,
    /// `g_LastFrameTime`. Source: `oracle/oracle/codemp/game/g_main.c:3514`
    pub g_LastFrameTime: c_int,
    /// `g_TimeSinceLastFrame`. Source: `oracle/oracle/codemp/game/g_main.c:3515`
    pub g_TimeSinceLastFrame: c_int,
    /// `g_dontFrickinCheck`. Source: `oracle/oracle/codemp/game/g_main.c:1427`
    pub g_dontFrickinCheck: qboolean,
    /// `g_duelPrintTimer`. Source: `oracle/oracle/codemp/game/g_main.c:2947`
    pub g_duelPrintTimer: c_int,
    /// `g_endPDuel`. Source: `oracle/oracle/codemp/game/g_main.c:2587`
    pub g_endPDuel: qboolean,
    /// `g_noPDuelCheck`. Source: `oracle/oracle/codemp/game/g_main.c:1717`
    pub g_noPDuelCheck: qboolean,
    /// `g_siegeRespawnCheck`. Source: `oracle/oracle/codemp/game/g_main.c:3580`
    pub g_siegeRespawnCheck: c_int,
    /// `killPlayerTimer`. Source: `oracle/oracle/codemp/game/g_main.c:15`
    pub killPlayerTimer: c_int,
    /// `navCalcPathTime`. Source: `oracle/oracle/codemp/game/g_main.c:12`
    pub navCalcPathTime: c_int,
    // --- `g_misc.c` file-scope globals ---
    /// `gEscapeTime`. Source: `oracle/oracle/codemp/game/g_misc.c:2541`
    pub gEscapeTime: c_int,
    /// `gEscaping`. Source: `oracle/oracle/codemp/game/g_misc.c:2540`
    pub gEscaping: qboolean,
    /// `g_shooterClientInit`. Source: `oracle/oracle/codemp/game/g_misc.c:3352`
    pub g_shooterClientInit: qboolean,
    //TODO: Port shooterClient_t[MAX_SHOOTERS]
    // Source: oracle/oracle/codemp/game/g_misc.c:3351
    pub g_shooterClients: (),
    // --- `g_mover.c` file-scope globals ---
    /// `pushed_t pushed[MAX_GENTITIES]` / `pushed_p` save-stack
    /// (`g_mover.c:19-24`) — one saved position/angle/deltayaw snapshot per
    /// moved entity, so a blocked mover push can roll everything back.
    /// Modeled as an owned `Vec` + cursor index rather than a raw pointer
    /// pair into a fixed array (porting-rules B3/B5); `pushed_p` (Raven:
    /// `pushed_t *pushed_p`) becomes an index into `pushed`.
    /// Source: `oracle/oracle/codemp/game/g_mover.c:19-24`
    pub pushed: Vec<crate::g_mover::PushedEntry>,
    pub pushed_p: usize,
    // --- `g_nav.c` file-scope globals ---
    /// `NAVDEBUG_curGoal`. Source: `oracle/oracle/codemp/game/g_nav.c:1607`
    pub NAVDEBUG_curGoal: c_int,
    /// `NAVDEBUG_showCollision`. Source: `oracle/oracle/codemp/game/g_nav.c:1606`
    pub NAVDEBUG_showCollision: qboolean,
    /// `NAVDEBUG_showCombatPoints`. Source: `oracle/oracle/codemp/game/g_nav.c:1604`
    pub NAVDEBUG_showCombatPoints: qboolean,
    /// `NAVDEBUG_showEdges`. Source: `oracle/oracle/codemp/game/g_nav.c:1601`
    pub NAVDEBUG_showEdges: qboolean,
    /// `NAVDEBUG_showEnemyPath`. Source: `oracle/oracle/codemp/game/g_nav.c:1603`
    pub NAVDEBUG_showEnemyPath: qboolean,
    /// `NAVDEBUG_showNavGoals`. Source: `oracle/oracle/codemp/game/g_nav.c:1605`
    pub NAVDEBUG_showNavGoals: qboolean,
    /// `NAVDEBUG_showNodes`. Source: `oracle/oracle/codemp/game/g_nav.c:1599`
    pub NAVDEBUG_showNodes: qboolean,
    /// `NAVDEBUG_showRadius`. Source: `oracle/oracle/codemp/game/g_nav.c:1600`
    pub NAVDEBUG_showRadius: qboolean,
    /// `NAVDEBUG_showTestPath`. Source: `oracle/oracle/codemp/game/g_nav.c:1602`
    pub NAVDEBUG_showTestPath: qboolean,
    /// Raven `char *fatalErrorPointer` — rolling write cursor into
    /// `fatalErrorString`. Modeled as a byte offset (not a raw pointer) per
    /// the no-aliasing-pointers-into-owned-arrays convention (porting-rules
    /// §B5); `fatalErrorPointer - fatalErrorString` in the oracle is this
    /// value directly.
    /// Source: `oracle/oracle/codemp/game/g_nav.c:1616`
    pub fatalErrorPointer: usize,
    /// Raven `char fatalErrorString[4096]`.
    /// Source: `oracle/oracle/codemp/game/g_nav.c:1617`
    pub fatalErrorString: FatalErrorString,
    /// `fatalErrors`. Source: `oracle/oracle/codemp/game/g_nav.c:1615`
    pub fatalErrors: c_int,
    /// `navCalculatePaths`. Source: `oracle/oracle/codemp/game/g_nav.c:1597`
    pub navCalculatePaths: qboolean,
    /// `numStoredWaypoints`. Source: `oracle/oracle/codemp/game/g_nav.c:1658`
    pub numStoredWaypoints: c_int,
    /// `waypointData_t tempWaypointList[MAX_STORED_WAYPOINTS]`.
    /// Source: `oracle/oracle/codemp/game/g_nav.c:1660`
    pub tempWaypointList: TempWaypointList,
    // --- `g_saga.c` file-scope globals ---
    /// `gImperialCountdown`. Source: `oracle/oracle/codemp/game/g_saga.c:30`
    pub gImperialCountdown: c_int,
    /// `static char gObjectiveCfgStr[1024]` — assembled objective config string.
    /// Source: `oracle/oracle/codemp/game/g_saga.c:47`
    pub gObjectiveCfgStr: ObjectiveCfgStr,
    /// `static char gParseObjectives[MAX_SIEGE_INFO_SIZE]` — siege-config parse
    /// buffer.
    /// Source: `oracle/oracle/codemp/game/g_saga.c:46`
    pub gParseObjectives: ParseObjectivesBuffer,
    /// `gRebelCountdown`. Source: `oracle/oracle/codemp/game/g_saga.c:31`
    pub gRebelCountdown: c_int,
    /// `gSiegeBeginTime`. Source: `oracle/oracle/codemp/game/g_saga.c:39`
    pub gSiegeBeginTime: c_int,
    /// `gSiegeRoundBegun`. Source: `oracle/oracle/codemp/game/g_saga.c:36`
    pub gSiegeRoundBegun: qboolean,
    /// `gSiegeRoundEnded`. Source: `oracle/oracle/codemp/game/g_saga.c:37`
    pub gSiegeRoundEnded: qboolean,
    /// `gSiegeRoundWinningTeam`. Source: `oracle/oracle/codemp/game/g_saga.c:38`
    pub gSiegeRoundWinningTeam: qboolean,
    /// `g_preroundState`. Source: `oracle/oracle/codemp/game/g_saga.c:41`
    pub g_preroundState: c_int,
    /// `g_siegePersistant` (`siegePers_t`) — cross-round siege team-switch
    /// persistence, mirrored to the engine via `trap_SiegePersGet`/`Set`.
    /// Source: `oracle/oracle/codemp/game/g_saga.c:20`
    pub g_siegePersistant: siegePers_t,
    /// `imperial_attackers`. Source: `oracle/oracle/codemp/game/g_saga.c:34`
    pub imperial_attackers: c_int,
    /// `imperial_goals_completed`. Source: `oracle/oracle/codemp/game/g_saga.c:23`
    pub imperial_goals_completed: c_int,
    /// `imperial_goals_required`. Source: `oracle/oracle/codemp/game/g_saga.c:22`
    pub imperial_goals_required: c_int,
    /// `imperial_time_limit`. Source: `oracle/oracle/codemp/game/g_saga.c:27`
    pub imperial_time_limit: c_int,
    /// `rebel_attackers`. Source: `oracle/oracle/codemp/game/g_saga.c:33`
    pub rebel_attackers: c_int,
    /// `rebel_goals_completed`. Source: `oracle/oracle/codemp/game/g_saga.c:25`
    pub rebel_goals_completed: c_int,
    /// `rebel_goals_required`. Source: `oracle/oracle/codemp/game/g_saga.c:24`
    pub rebel_goals_required: c_int,
    /// `rebel_time_limit`. Source: `oracle/oracle/codemp/game/g_saga.c:28`
    pub rebel_time_limit: c_int,
    // --- `g_spawn.c` file-scope globals ---
    /// `void *precachedKyle` — the server's precached Kyle template ghoul2 instance handle.
    /// Source: `oracle/oracle/codemp/game/g_spawn.c:1226`
    pub precachedKyle: *mut c_void,
    // --- `g_svcmds.c` file-scope globals ---
    /// `ipFilter_t ipFilters[MAX_IPFILTERS]`.
    /// Source: oracle/oracle/codemp/game/g_svcmds.c:54
    pub ipFilters: IpFilters,
    /// `numIPFilters`. Source: `oracle/oracle/codemp/game/g_svcmds.c:55`
    pub numIPFilters: c_int,
    // --- `g_target.c` file-scope globals ---
    /// `numNewICARUSEnts`. Source: `oracle/oracle/codemp/game/g_target.c:753`
    pub numNewICARUSEnts: c_int,
    // --- `g_team.c` file-scope globals ---
    /// `teamgame` — CTF flag state.
    /// Source: oracle/oracle/codemp/game/g_team.c:18
    pub teamgame: teamgame_t,
    // --- `g_timer.c` file-scope globals ---
    //TODO: Port gtimer_t **
    // Source: oracle/oracle/codemp/game/g_timer.c:19
    pub g_timerFreeList: (),
    //TODO: Port gtimer_t[ MAX_GTIMERS ]
    // Source: oracle/oracle/codemp/game/g_timer.c:17
    pub g_timerPool: (),
    //TODO: Port gtimer_t *[ MAX_GENTITIES ]*
    // Source: oracle/oracle/codemp/game/g_timer.c:18
    pub g_timers: (),
    // --- `g_trigger.c` file-scope globals ---
    /// `gTrigFallSound`. Source: `oracle/oracle/codemp/game/g_trigger.c:6`
    pub gTrigFallSound: c_int,
    // --- `g_utils.c` file-scope globals ---
    /// `gclient_t *gClPtrs[MAX_GENTITIES]` (see `GClPtrs`). Held as
    /// `*mut c_void` — same tiering rationale as `gentity_t.client`
    /// (`gentity.rs`): the real `gclient_t` pointee type isn't nameable from
    /// this field's declaration site without a cast at each use.
    /// Source: `oracle/oracle/codemp/game/g_utils.c:428`
    pub gClPtrs: GClPtrs,
    /// `int gG2KillIndex[MAX_G2_KILL_QUEUE]` (see `GG2KillIndex`).
    /// Source: `oracle/oracle/codemp/game/g_utils.c:877`
    pub gG2KillIndex: GG2KillIndex,
    /// `gG2KillNum`. Source: `oracle/oracle/codemp/game/g_utils.c:878`
    pub gG2KillNum: c_int,
    /// `g_vehiclePoolInit`. Source: `oracle/oracle/codemp/game/g_utils.c:387`
    pub g_vehiclePoolInit: qboolean,
    /// `qboolean g_vehiclePoolOccupied[MAX_VEHICLES_AT_A_TIME]` (see
    /// `VehiclePoolOccupied`).
    /// Source: `oracle/oracle/codemp/game/g_utils.c:386`
    pub g_vehiclePoolOccupied: VehiclePoolOccupied,
    /// `remapCount`. Source: `oracle/oracle/codemp/game/g_utils.c:17`
    pub remapCount: c_int,
    /// `shaderRemap_t remappedShaders[MAX_SHADER_REMAPS]` (see
    /// `RemappedShaders`).
    /// Source: `oracle/oracle/codemp/game/g_utils.c:18`
    pub remappedShaders: RemappedShaders,
    // --- `g_weapon.c` file-scope globals ---
    /// `s_quadFactor`. Source: `oracle/oracle/codemp/game/g_weapon.c:12`
    pub s_quadFactor: f32,
    /// `static vec3_t forward` — fire-time forward axis shared across
    /// `WP_Fire*`/`CalcMuzzlePoint` (ruling 29).
    /// Source: `oracle/oracle/codemp/game/g_weapon.c:13`
    pub forward: vec3_t,
    /// `static vec3_t vright` — fire-time right axis (ruling 29).
    /// Source: `oracle/oracle/codemp/game/g_weapon.c:13`
    pub vright: vec3_t,
    /// `static vec3_t up` — fire-time up axis (ruling 29).
    /// Source: `oracle/oracle/codemp/game/g_weapon.c:13`
    pub up: vec3_t,
    /// `static vec3_t muzzle` — fire-time muzzle point (ruling 29).
    /// Source: `oracle/oracle/codemp/game/g_weapon.c:14`
    pub muzzle: vec3_t,
    // --- `w_saber.c` file-scope globals ---
    /// `static vec3_t dmgDir[MAX_SABER_VICTIMS]` — per-victim saber damage
    /// direction.
    /// Source: `oracle/oracle/codemp/game/w_saber.c:3507`
    pub dmgDir: [vec3_t; MAX_SABER_VICTIMS],
    /// `static vec3_t dmgSpot[MAX_SABER_VICTIMS]` — per-victim saber impact
    /// point.
    /// Source: `oracle/oracle/codemp/game/w_saber.c:3508`
    pub dmgSpot: [vec3_t; MAX_SABER_VICTIMS],
    /// `static qboolean dismemberDmg[MAX_SABER_VICTIMS]` — per-victim dismember flag.
    /// Source: oracle/oracle/codemp/game/w_saber.c:3509
    pub dismemberDmg: [qboolean; MAX_SABER_VICTIMS],
    /// `numVictims`. Source: `oracle/oracle/codemp/game/w_saber.c:3511`
    pub numVictims: c_int,
    /// `saberClashEventParm`. Source: `oracle/oracle/codemp/game/w_saber.c:3797`
    pub saberClashEventParm: c_int,
    /// `static vec3_t saberClashNorm` — surface normal at the last saber clash.
    /// Source: `oracle/oracle/codemp/game/w_saber.c:3796`
    pub saberClashNorm: vec3_t,
    /// `static vec3_t saberClashPos` — world position of the last saber clash.
    /// Source: `oracle/oracle/codemp/game/w_saber.c:3795`
    pub saberClashPos: vec3_t,
    /// `saberDoClashEffect`. Source: `oracle/oracle/codemp/game/w_saber.c:3794`
    pub saberDoClashEffect: qboolean,
    /// `saberHitFraction`. Source: `oracle/oracle/codemp/game/w_saber.c:3848`
    pub saberHitFraction: f32,
    /// `saberHitSaber`. Source: `oracle/oracle/codemp/game/w_saber.c:3847`
    pub saberHitSaber: qboolean,
    /// `saberHitWall`. Source: `oracle/oracle/codemp/game/w_saber.c:3846`
    pub saberHitWall: qboolean,
    /// `static int saberKnockbackFlags[MAX_SABER_VICTIMS]` — per-victim knockback flags.
    /// Source: oracle/oracle/codemp/game/w_saber.c:3510
    pub saberKnockbackFlags: [c_int; MAX_SABER_VICTIMS],
    /// `saberSpinSound`. Source: `oracle/oracle/codemp/game/w_saber.c:18`
    pub saberSpinSound: c_int,
    /// `static float totalDmg[MAX_SABER_VICTIMS]` — per-victim accumulated damage.
    /// Stored as `c_int` (matches the ported bodies, which accumulate integer
    /// `trDmg` and feed `G_Damage`'s `int damage`); the wall-scale multiply
    /// widens to `f32` at the site, as in the oracle.
    /// Source: oracle/oracle/codemp/game/w_saber.c:3506
    pub totalDmg: [c_int; MAX_SABER_VICTIMS],
    /// `static int victimEntityNum[MAX_SABER_VICTIMS]` — per-victim entity number.
    /// Source: oracle/oracle/codemp/game/w_saber.c:3504
    pub victimEntityNum: [c_int; MAX_SABER_VICTIMS],
    /// `static qboolean victimHitEffectDone[MAX_SABER_VICTIMS]` — per-victim hit-effect flag.
    /// Source: oracle/oracle/codemp/game/w_saber.c:3505
    pub victimHitEffectDone: [qboolean; MAX_SABER_VICTIMS],
}
