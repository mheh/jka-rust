//! `Navigator` (Raven `CNavigator`, C-prefix dropped — NAV-D3 / RULING 40) —
//! the engine-side nav graph owner: the node/edge arenas, failed-edge /
//! checked-node bookkeeping, the priority-queue pathfinder, `Load`/`Save`,
//! and the full `G_NAV_*` pub surface the `SV_GameSystemCalls` switch
//! dispatches into (wave 20 wires the call sites — Seam definition,
//! `docs/subsystems/npcnav.md`).
//!
//! Host-taking methods take `(&mut self, host: &mut impl EngineHost)`
//! (NAV-D3 / RULING 11/24/36); the five ent-taking arms carry
//! `*mut sharedEntity_t` exactly as the trap marshals it (NAV-D3 / RULING 30)
//! and deref it directly (`ent->s.number`, `ent->waypoint`,
//! `ent->failedWaypoints`, …), writing back through the same borrow. Per
//! NAV-D4 (RULING 48) `clear_failed_edge` is host-free — its body reaches no
//! engine service, matching its sibling `clear_all_failed_edges`.
//!
//! **§20 dropped surface (zero-caller / not-taken — module-doc note only, no
//! stub emitted):**
//! - `CNavigator::GetChar`/`GetFloat` (navigator.cpp:512-519,542-549) —
//!   declared but never called anywhere in navigator.cpp (only
//!   `GetInt`/`GetLong` are, at :614/623/631).
//! - `CNavigator::GetNodeLeadDistance` (navigator.h:182) — declared, **no**
//!   definition in navigator.cpp, no caller/trap arm.
//! - `NAV_CvarInit`'s `Cvar_Get` **registration** half (navigator.cpp:41-42)
//!   — the frozen `EngineHost` trait has no registration method (Seam cvar
//!   note); only the `->integer` read-back ports, via
//!   `EngineHost::cvar_integer`. The ctor's lazy once-only *timing* still
//!   matters (RULING 12) — modeled by the `cvars_initialized` flag below —
//!   but there is nothing left to call.
//! - `~CNavigator` (navigator.cpp:486-488) hand-frees `m_nodes`/clears the
//!   map; the owned `Vec`/`BTreeMap` fields below do that on drop, no `Drop`
//!   impl needed.
//!
//! **`SV_inPVS` (server.h:356) is NOT on the frozen `EngineHost` trait** — the
//! doc defers it to the server-spine work (Seam note), not to npcnav. The
//! trace/PVS-dependent methods here reach it through the private
//! [`Navigator::sv_in_pvs`] shim, which conservatively reports "potentially
//! visible" (the authoritative trace that follows is the ground truth, and the
//! only sites where the check gates real control flow already re-verify with a
//! host trace / `TestNodePath`) until the service lands. Reported under
//! `problems`.
//!
//! Type definition source: `oracle/codemp/server/NPCNav/navigator.h:130-249`
//! Method source: `oracle/codemp/server/NPCNav/navigator.cpp`

use std::collections::BTreeMap;
use std::io::Read;

use mp_host_interface::EngineHost;
use mp_qshared::common::mp::game::{Q3_INFINITE, WAYPOINT_NONE};
use mp_qshared::common::mp::gentity::MAX_FAILED_NODES;
use mp_qshared::common::mp::qcommon::failedEdge_t;
use mp_qshared::common::mp::qcommon::shared_entity_t::sharedEntity_t;
use mp_qshared::common::mp::trace_t::trace_t;
use mp_qshared::shared::q_math::{_DotProduct, _VectorSubtract, VectorNormalize};
use mp_qshared::shared::{
    qboolean, qfalse, qtrue, vec3_t, CONTENTS_BODY, CONTENTS_BOTCLIP, CONTENTS_MONSTERCLIP,
    CONTENTS_SOLID, ENTITYNUM_NONE, ENTITYNUM_WORLD, MASK_NPCSOLID, MASK_SOLID, MAX_GENTITIES,
    WORLD_SIZE,
};

use super::callbacks::{
    GNavCallback_CP_FindCombatPointWaypoints, GNavCallback_G_EntIsBreakable,
    GNavCallback_G_EntIsDoor, GNavCallback_G_EntIsRemovableUsable,
    GNavCallback_G_EntIsUnlockedDoor, GNavCallback_NAV_ClearPathToPoint, GNavCallback_NPC_ClearLOS,
};
use super::{
    Edge, Node, PriorityQueue, CHECKED_FAILED, CHECKED_NO, CHECKED_PASSED, EFLAG_BLOCKED,
    EFLAG_NONE, MAX_FAILED_EDGES, NAV_HEADER_ID, NF_CLEAR_PATH, NF_RECALC, NODE_NONE, WP_MAXS,
    WP_MINS,
};

// --- Non-nav-owned constants the trace/PVS surface consumes -----------------
//
// These are NOT nav globals and are NOT reachable from `mp_engine_server` (they
// live in `mp_game`/`mp_bg`, off this crate's dep graph, or are file-scope
// `#define`s inside navigator.cpp itself). They are transcribed here as private
// module consts with their Raven cites — internal values, not seam types.

/// Raven `MAX_STORED_WAYPOINTS`.
/// Source: `oracle/codemp/game/g_nav.h:9`
const MAX_STORED_WAYPOINTS: i32 = 512;

/// Raven file-scope `#define MAX_Z_DELTA 18` (navigator.cpp:1326,1532) — used
/// in float `fabs(...)` comparisons, so carried as `f32`.
/// Source: `oracle/codemp/server/NPCNav/navigator.cpp:1326`
const MAX_Z_DELTA: f32 = 18.0;

/// Raven `#define NODE_COLLECT_MAX 16`.
/// Source: `oracle/codemp/server/NPCNav/navigator.cpp:1245`
const NODE_COLLECT_MAX: i32 = 16;

/// Raven `#define NODE_COLLECT_RADIUS 512`.
/// Source: `oracle/codemp/server/NPCNav/navigator.cpp:1246`
const NODE_COLLECT_RADIUS: i32 = 512;

/// Raven `#define CHECK_FAILED_EDGE_INTERVAL 1000`.
/// Source: `oracle/codemp/server/NPCNav/navigator.cpp:1721`
const CHECK_FAILED_EDGE_INTERVAL: i32 = 1000;

/// Raven `#define CHECK_FAILED_EDGE_INTITIAL 5000` (Raven's own spelling).
/// Source: `oracle/codemp/server/NPCNav/navigator.cpp:1722`
const CHECK_FAILED_EDGE_INTITIAL: i32 = 5000;

/// Raven `#define DEFAULT_MINS_2 -24` — used in a float `VectorSet`, so `f32`.
/// Source: `oracle/codemp/game/bg_public.h:41`
const DEFAULT_MINS_2: f32 = -24.0;

/// Raven `#define DEFAULT_MAXS_2 40`.
/// Source: `oracle/codemp/game/bg_public.h:42`
const DEFAULT_MAXS_2: f32 = 40.0;

/// Raven `entityType_t::ET_PLAYER` (2nd enumerator, value 1).
/// Source: `oracle/codemp/game/bg_public.h:1192`
const ET_PLAYER: i32 = 1;

/// Raven `entityType_t::ET_NPC` (14th enumerator, value 13).
/// Source: `oracle/codemp/game/bg_public.h:1204`
const ET_NPC: i32 = 13;

/// Raven `#define EF_DEAD (1<<1)`.
/// Source: `oracle/codemp/game/bg_public.h:561`
const EF_DEAD: i32 = 1 << 1;

/// A zeroed `trace_t` out-param — the analogue of Raven's uninitialised
/// `trace_t trace;` the host `SV_Trace`/`host.trace` fills. `trace_t` is a
/// `#[repr(C)]` POD, so an all-zero bit pattern is a valid value.
fn zeroed_trace() -> trace_t {
    // SAFETY: `trace_t` is a `#[repr(C)]` plain-old-data struct (floats, ints,
    // a `cplane_t` of floats/ints) with no niche/invariant — all-zero is valid.
    unsafe { core::mem::zeroed() }
}

/// Raven `DistanceSquared( p1, p2 )` — the squared distance as a `float`
/// (`q_shared.h` macro over `_DistanceSquared`); a free helper here since only
/// the `_`-prefixed vec3 primitives were migrated into `mp_qshared` (NAV-D3).
fn distance_squared(a: vec3_t, b: vec3_t) -> f32 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    let dz = a[2] - b[2];
    dx * dx + dy * dy + dz * dz
}

/// Raven `Distance( p1, p2 )` — `VectorLength( VectorSubtract )`; the `sqrt`
/// runs in `double` then rounds to `float`, matching `VectorNormalize`.
fn distance(a: vec3_t, b: vec3_t) -> f32 {
    (distance_squared(a, b) as f64).sqrt() as f32
}

/// Reads one little-endian `i32` off the front of a byte cursor, advancing it
/// 4 bytes — the shared in-memory analogue of `FS_Read( &value, 4, file )`
/// used for the `failedEdges[]` array in [`Navigator::load`].
fn take_i32(sl: &mut &[u8]) -> i32 {
    let (head, tail) = sl.split_at(4);
    *sl = tail;
    i32::from_le_bytes([head[0], head[1], head[2], head[3]])
}

/// Raven `Navigator` (C-prefix dropped, NAV-D3 / RULING 40) — see module
/// doc for the dropped/elided surface.
///
/// State-ownership per `docs/subsystems/npcnav.md`: a plain `Default`-init
/// direct field on `mp_engine_core::Engine.nav` (RULING 12), reached through
/// the `EngineHostView` split-borrow (`Engine::nav_call`, NAV-D3 /
/// RULING 43).
///
/// Type definition source: `oracle/codemp/server/NPCNav/navigator.h:134-249`
pub struct Navigator {
    /// Raven `m_nodes` (`vector<CNode*>`) — the owned node arena; node id ==
    /// index (NAV-D3, §B5: arena, not a pointer graph).
    ///
    /// Source: `oracle/codemp/server/NPCNav/navigator.h:247`
    pub nodes: Vec<Node>,

    /// Raven `m_edgeLookupMap` (`multimap<int,int>`) — a failed edge's
    /// `startID` → indices into [`failed_edges`](Self::failed_edges).
    /// `BTreeMap` (not `HashMap`) so `EdgeFailed`'s `equal_range` first-match
    /// scan (navigator.cpp:1876-1898) reproduces deterministically (NAV-D3 /
    /// RULING 18).
    ///
    /// Source: `oracle/codemp/server/NPCNav/navigator.h:248`
    pub edge_lookup: BTreeMap<i32, Vec<usize>>,

    /// Raven `failedEdges[MAX_FAILED_EDGES]`.
    ///
    /// Source: `oracle/codemp/server/NPCNav/navigator.h:245`
    pub failed_edges: [failedEdge_t; MAX_FAILED_EDGES],

    /// Raven `pathsCalculated` — a **public** field on `CNavigator`;
    /// `G_NAV_GETPATHSCALCULATED`/`G_NAV_SETPATHSCALCULATED` read/write it
    /// directly, no method needed.
    ///
    /// Source: `oracle/codemp/server/NPCNav/navigator.h:215`
    pub paths_calculated: qboolean,

    /// Raven file-scope `static map<int,byte> CheckedNodes`
    /// (navigator.cpp:1687), keyed `wayPoint*MAX_GENTITIES+ent` — genuine
    /// cross-call state (fork-3 kind-3), promoted to a `Navigator` field;
    /// `BTreeMap` for deterministic iteration/lookup (plan §3d).
    ///
    /// Source: `oracle/codemp/server/NPCNav/navigator.cpp:1687`
    pub checked_nodes: BTreeMap<i32, u8>,

    /// Models the ctor's lazy `NAV_CvarInit` once-only timing (RULING 12);
    /// the `Cvar_Get` registration it used to run has no frozen-trait
    /// counterpart and is elided (module doc), so this flag only gates the
    /// once-only semantics, nothing else.
    ///
    /// Source: `oracle/codemp/server/NPCNav/navigator.cpp:478-484`
    pub cvars_initialized: bool,

    /// Raven `pathsCalculated` — same public `CNavigator` field as
    /// [`paths_calculated`](Self::paths_calculated) above; kept under its
    /// verbatim Raven spelling too because `sv_game.rs`'s
    /// `G_NAV_GETPATHSCALCULATED`/`G_NAV_SETPATHSCALCULATED` arms reference
    /// it by that exact name (state-struct field-merge round, not a design
    /// choice — reconciling the two spellings into one is call-site work,
    /// out of scope for this file).
    ///
    /// Source: `oracle/codemp/server/NPCNav/navigator.h:215`
    pub pathsCalculated: qboolean,
}

impl Default for Navigator {
    /// Raven `CNavigator::CNavigator( void )` — lazily runs `NAV_CvarInit`
    /// (:39-43) once; its `Cvar_Get` registration side is elided (module
    /// doc), only the once-only timing is modeled. `~CNavigator` needs no
    /// counterpart (module doc).
    ///
    /// Source: `oracle/codemp/server/NPCNav/navigator.cpp:478-488`
    fn default() -> Self {
        // The global `CNavigator navigator` is zero-initialised (static
        // storage), then the ctor runs `NAV_CvarInit` once — whose only ported
        // effect is flipping the init flag (registration elided, module doc).
        Navigator {
            nodes: Vec::new(),
            edge_lookup: BTreeMap::new(),
            failed_edges: [failedEdge_t {
                startID: 0,
                endID: 0,
                checkTime: 0,
                entID: 0,
            }; MAX_FAILED_EDGES],
            paths_calculated: qfalse,
            checked_nodes: BTreeMap::new(),
            cvars_initialized: true,
            pathsCalculated: qfalse,
        }
    }
}

impl Navigator {
    // ---- Lifecycle / build (Seam: Inbound game -> engine) ------------

    /// Raven `CNavigator::Init( void )` — calls `Free()`. `G_NAV_INIT`.
    ///
    /// Source: `oracle/codemp/server/NPCNav/navigator.cpp:572-575`
    pub fn init(&mut self) {
        self.free();
    }

    /// Raven `CNavigator::Free( void )` — `delete`s every `CNode*`, clears
    /// `m_nodes` + `m_edgeLookupMap`. `G_NAV_FREE` (also `NAV_Free`'s
    /// counterpart, navigator.cpp:47-48).
    ///
    /// Source: `oracle/codemp/server/NPCNav/navigator.cpp:583-594`
    pub fn free(&mut self) {
        // Owned `Vec`/`BTreeMap` drop glue frees every `Node` (Raven's
        // `delete (*ni)` loop) as they clear.
        self.nodes.clear();
        self.edge_lookup.clear();
    }

    /// Raven `bool CNavigator::Load( const char *filename, int checksum )` —
    /// validates `NAV_HEADER_ID`/checksum, reads `numNodes` `CNode`s (each
    /// via `Node::load` over a shared in-memory cursor built from **one**
    /// `EngineHost::fs_read_file` — NAV-D3 / RULING 36), the `failedEdges[]`
    /// array, and rebuilds `edge_lookup`. `G_NAV_LOAD`.
    ///
    /// Source: `oracle/codemp/server/NPCNav/navigator.cpp:602-657`
    pub fn load(&mut self, host: &mut impl EngineHost, filename: &str, checksum: i32) -> bool {
        // One whole-file read (NAV-D3 / RULING 36); `None` = open failure
        // (Raven's `file == NULL` → return false).
        let data = match host.fs_read_file(&format!("maps/{}.nav", filename)) {
            Some(d) => d,
            None => return false,
        };

        let num_nodes;
        let start_off;
        {
            let mut cur = std::io::Cursor::new(data.as_slice());

            // Check the header id (NAV-D1 / RULING 44: 4-byte `u32`).
            let nav_id = self.get_long(&mut cur);
            if (nav_id as u32) != (NAV_HEADER_ID as u32) {
                return false;
            }

            // Check the checksum to see if this file is out of date.
            let check = self.get_int(&mut cur);
            if check != checksum {
                return false;
            }

            num_nodes = self.get_int(&mut cur);
            start_off = cur.position() as usize;
        }

        // The remaining bytes drive `Node::load` + the `failedEdges[]` read off
        // the same shared in-memory cursor (a `&mut &[u8]`, the node-loop
        // analogue of the oracle's shared `fileHandle_t`).
        let mut sl: &[u8] = &data[start_off..];

        for _ in 0..num_nodes {
            let mut node = Node::new();
            if !node.load(num_nodes, &mut sl) {
                return false;
            }
            self.nodes.push(node);
        }

        // Read in the failed edges (`FS_Read( &failedEdges, sizeof(...) )`,
        // :647) then rebuild `m_edgeLookupMap` (:648-651). Matches Raven: no
        // pre-clear here (Init/Free own that).
        for j in 0..MAX_FAILED_EDGES {
            let start_id = take_i32(&mut sl);
            let end_id = take_i32(&mut sl);
            let check_time = take_i32(&mut sl);
            let ent_id = take_i32(&mut sl);
            self.failed_edges[j] = failedEdge_t {
                startID: start_id,
                endID: end_id,
                checkTime: check_time,
                entID: ent_id,
            };
            self.edge_lookup
                .entry(self.failed_edges[j].startID)
                .or_default()
                .push(j);
        }

        true
    }

    /// Raven `bool CNavigator::Save( const char *filename, int checksum )` —
    /// builds one byte buffer (NAV header id as a 4-byte `u32`, NAV-D1 /
    /// RULING 44; checksum; `numNodes`; each `Node::save` over the shared
    /// buffer; `failedEdges[]`), then **one** `EngineHost::fs_write_file`
    /// (NAV-D3 / RULING 36). `G_NAV_SAVE`.
    ///
    /// Source: `oracle/codemp/server/NPCNav/navigator.cpp:665-702`
    pub fn save(&mut self, host: &mut impl EngineHost, filename: &str, checksum: i32) -> bool {
        let mut buf: Vec<u8> = Vec::new();

        // Write out the header id (NAV-D1: 4-byte `u32`), checksum, node count.
        let id: u32 = NAV_HEADER_ID as u32;
        buf.extend_from_slice(&id.to_le_bytes());
        buf.extend_from_slice(&checksum.to_le_bytes());

        let num_nodes = self.nodes.len() as i32;
        buf.extend_from_slice(&num_nodes.to_le_bytes());

        // Write out all the nodes over the one shared buffer (:691-694).
        for node in &self.nodes {
            node.save(num_nodes, &mut buf);
        }

        // Write out failed edges (`FS_Write( &failedEdges, sizeof(...) )`, :697).
        for e in &self.failed_edges {
            buf.extend_from_slice(&e.startID.to_le_bytes());
            buf.extend_from_slice(&e.endID.to_le_bytes());
            buf.extend_from_slice(&e.checkTime.to_le_bytes());
            buf.extend_from_slice(&e.entID.to_le_bytes());
        }

        // The single whole-file host call (NAV-D3 / RULING 36); `false` mirrors
        // Raven's `file == NULL` open-failure early return.
        host.fs_write_file(&format!("maps/{}.nav", filename), &buf)
    }

    /// Raven `int CNavigator::AddRawPoint( vec3_t point, int flags, int radius )`
    /// — appends a `Node` and returns its id; the `Com_Error` NULL-check
    /// branch is dead (D-3, `Node::create` is infallible). `G_NAV_ADDRAWPOINT`.
    /// Host-free (D-3, no `Com_Error` reachable in practice) but Seam pins a
    /// host param for the dead-branch shape's sake.
    ///
    /// Source: `oracle/codemp/server/NPCNav/navigator.cpp:710-726`
    pub fn add_raw_point(
        &mut self,
        host: &mut impl EngineHost,
        point: vec3_t,
        flags: i32,
        radius: i32,
    ) -> i32 {
        let _ = host; // D-3: the `Com_Error` NULL branch is unreachable, so no service is used.
        let id = self.nodes.len() as i32;
        let node = Node::create(point, flags, radius, id);
        // D-3: `CNode::Create` (C++ `new`) never returns NULL, so the
        // `Com_Error( ERR_DROP, ... )` branch (:714-718) is dead — omitted.
        self.nodes.push(node);
        id
    }

    /// Raven `void CNavigator::CalculatePaths( qboolean recalc=qfalse )` —
    /// allocates each node's rank table (`Node::init_ranks`) then runs
    /// [`calculate_path`](Self::calculate_path) from every node; calls
    /// `GNavCallback_CP_FindCombatPointWaypoints` (unless `recalc`) and sets
    /// `paths_calculated = qtrue`. `G_NAV_CALCULATEPATHS`.
    ///
    /// Source: `oracle/codemp/server/NPCNav/navigator.cpp:884-908`
    pub fn calculate_paths(&mut self, host: &mut impl EngineHost, recalc: qboolean) {
        let n = self.nodes.len() as i32;

        for i in 0..self.nodes.len() {
            self.nodes[i].init_ranks(n);
        }

        for i in 0..self.nodes.len() {
            self.calculate_path(i as i32);
        }

        if recalc == qfalse {
            // Mike says doesn't need to happen on recalc.
            GNavCallback_CP_FindCombatPointWaypoints(host);
        }

        self.paths_calculated = qtrue;
    }

    /// Raven `void CNavigator::HardConnect( int first, int second )`
    /// (`#if _HARD_CONNECT`) — wires an edge directly (bypassing
    /// `CalculatePath`'s trace-checked connect), then adds it bidirectionally.
    /// `G_NAV_HARDCONNECT`.
    ///
    /// Source: `oracle/codemp/server/NPCNav/navigator.cpp:1113-1140`
    pub fn hard_connect(&mut self, host: &mut impl EngineHost, first: i32, second: i32) {
        let p1 = self.nodes[first as usize].get_position();
        let p2 = self.nodes[second as usize].get_position();

        let mut trace = zeroed_trace();
        let mut flags = EFLAG_NONE;

        host.trace(
            &mut trace,
            &p1,
            &WP_MINS,
            &WP_MAXS,
            &p2,
            ENTITYNUM_NONE,
            MASK_SOLID | CONTENTS_BOTCLIP | CONTENTS_MONSTERCLIP,
            false,
            0,
            10,
        );

        let cost = distance(p1, p2) as i32;

        if trace.fraction != 1.0 || trace.startsolid != 0 || trace.allsolid != 0 {
            flags |= EFLAG_BLOCKED;
        }

        self.nodes[first as usize].add_edge(second, cost, flags);
        self.nodes[second as usize].add_edge(first, cost, flags);
    }

    /// Raven `void CNavigator::ShowNodes( void )` — draw calls stripped
    /// (renderer, §20); keeps the `SV_GentityNum(0)` player-relative
    /// PVS/distance control flow via `EngineHost::gentity` (NAV-D3).
    /// `G_NAV_SHOWNODES`.
    ///
    /// Source: `oracle/codemp/server/NPCNav/navigator.cpp:916-975`
    pub fn show_nodes(&mut self, host: &mut impl EngineHost) {
        for i in 0..self.nodes.len() {
            let position = self.nodes[i].get_position();

            // `NAVDEBUG_showRadius` branch is `if (0)` — dead; the live `else`
            // only computes `dist` (renderer stripped, §20).
            let show_radius = qfalse;
            let player = host.gentity(0);
            let player_origin = unsafe { (*player).r.currentOrigin };
            let dist = distance_squared(player_origin, position);

            if dist < 1048576.0 && self.sv_in_pvs(player_origin, position) {
                // Raven `(*ni)->Draw( showRadius )` — a no-op (renderer stripped).
                self.nodes[i].draw(show_radius);
            }
        }
    }

    /// Raven `void CNavigator::ShowEdges( void )` — draw calls stripped
    /// (renderer, §20); keeps the `SV_GentityNum(0)` PVS/distance control
    /// flow (NAV-D3). `G_NAV_SHOWEDGES`.
    ///
    /// Source: `oracle/codemp/server/NPCNav/navigator.cpp:977-1027`
    pub fn show_edges(&mut self, host: &mut impl EngineHost) {
        let n = self.nodes.len();
        // Raven `drawMap = new drawMap_m[ m_nodes.size() ]` — a per-node
        // `map<int,bool>` used to dedup the (stripped) edge draws.
        let mut draw_map: Vec<BTreeMap<i32, bool>> = vec![BTreeMap::new(); n];

        let player = host.gentity(0);
        let player_origin = unsafe { (*player).r.currentOrigin };

        for ni in 0..n {
            let start = self.nodes[ni].get_position();
            if distance_squared(player_origin, start) >= 1048576.0 {
                continue;
            }
            if !self.sv_in_pvs(player_origin, start) {
                continue;
            }

            let ni_id = self.nodes[ni].get_id();
            let num_edges = self.nodes[ni].get_num_edges();

            for i in 0..num_edges {
                let id = self.nodes[ni].get_edge(i);
                if id == -1 {
                    continue;
                }

                // Already drawn?
                if draw_map[ni_id as usize].contains_key(&id) {
                    continue;
                }

                let end = self.nodes[id as usize].get_position();

                // Set this as drawn.
                draw_map[id as usize].insert(ni_id, true);

                if distance_squared(player_origin, end) >= 1048576.0 {
                    continue;
                }
                if !self.sv_in_pvs(player_origin, end) {
                    continue;
                }
                // `CG_DrawEdge( start, end, ... )` — renderer stripped (§20).
            }
        }
    }

    /// Raven `void CNavigator::ShowPath( int start, int end )` — walks the
    /// rank table printing the path via `EngineHost::print` (`Com_Printf`).
    /// `G_NAV_SHOWPATH`.
    ///
    /// Source: `oracle/codemp/server/NPCNav/navigator.cpp:1632-1685`
    pub fn show_path(&mut self, host: &mut impl EngineHost, start: i32, end: i32) {
        let n = self.nodes.len() as i32;

        // Validate the start/end positions.
        if start < 0 || start >= n {
            return;
        }
        if end < 0 || end >= n {
            return;
        }

        let mut move_id = start;
        let mut run_away = 0;

        // Draw out our path.
        while move_id != end {
            let best_node = self.get_best_node(move_id, end, NODE_NONE);

            // Some nodes may be fragmented.
            if best_node == -1 {
                host.print(&format!(
                    "No connection possible between node {} and {}\n",
                    start, end
                ));
                return;
            }

            // Draw the edge (renderer stripped, §20). Take a new best node.
            move_id = best_node;

            if run_away > 64 {
                host.print("Potential Run-away path!\n");
                return;
            }
            run_away += 1;
        }
    }

    // ---- Queries (host-free pure graph = golden surface, EXCEPT --------
    // ---- get_nearest_node / get_edge_cost, which are trace-dependent) ---

    /// Raven `int CNavigator::GetNearestNode( sharedEntity_t *ent, int lastID,
    /// int flags, int targetID )` — trace/PVS-dependent search (3c surface);
    /// `ent` is the raw `*mut sharedEntity_t` the trap marshals
    /// (`(sharedEntity_t*)VMA(1)`, sv_game.cpp:865, NAV-D3), dereferenced
    /// directly (never calls `AddFailedNode`/`CheckFailedNodes` — those have
    /// no in-file caller here). `G_NAV_GETNEARESTNODE`.
    ///
    /// Source: `oracle/codemp/server/NPCNav/navigator.cpp:1320-1624`
    pub fn get_nearest_node(
        &mut self,
        host: &mut impl EngineHost,
        ent: *mut sharedEntity_t,
        last_id: i32,
        flags: i32,
        target_id: i32,
    ) -> i32 {
        let mut best_node = NODE_NONE;

        // Must have nodes.
        if self.nodes.is_empty() {
            return NODE_NONE;
        }

        if target_id == NODE_NONE {
            // Try and find an early match using our last node.
            best_node = self.test_best_first(host, ent, last_id, flags);
            if best_node != NODE_NONE {
                return best_node;
            }
        } // else can't rely on testing last, we want best to targetID

        let mut node_chain: Vec<(i32, u32)> = Vec::new();
        let ent_origin = unsafe { (*ent).r.currentOrigin };
        let ent_number = unsafe { (*ent).s.number };

        // Collect all nodes within a certain radius.
        self.collect_nearest_nodes(
            ent_origin,
            NODE_COLLECT_RADIUS,
            NODE_COLLECT_MAX,
            &mut node_chain,
        );

        let mut best_dist = Q3_INFINITE;

        for &(node_id, ndist) in node_chain.iter() {
            let position = self.nodes[node_id as usize].get_position();
            let radius = self.nodes[node_id as usize].get_radius();

            if self.node_failed(ent, node_id) != qfalse {
                continue;
            }

            // Are we within the known clear radius of this node?
            if ndist < (radius * radius) as u32 {
                // Do a z-difference sanity check.
                if (position[2] - ent_origin[2]).abs() < MAX_Z_DELTA {
                    // Found one.
                    return node_id;
                }
            }

            // We're not *within* this node, so... (D-5: the second `else if`
            // duplicates the first `CHECKED_FAILED` test — a preserved Raven
            // bug, kept bug-for-bug per §20.)
            let cn = self.checked_node(node_id, ent_number);
            if cn == CHECKED_FAILED {
                continue;
            } else if cn == CHECKED_FAILED {
                continue;
            } else {
                // Do we need a clear path?
                if flags & NF_CLEAR_PATH != 0
                    && self.test_node_path(host, ent, ENTITYNUM_NONE, position, qfalse) == 0
                {
                    self.set_checked_node(node_id, ent_number, CHECKED_FAILED);
                    continue;
                }
                self.set_checked_node(node_id, ent_number, CHECKED_PASSED);
            }

            if target_id != WAYPOINT_NONE {
                // We want to find the one with the shortest route here.
                let dist = self.get_path_cost(node_id, target_id) as i32;
                if dist < best_dist {
                    best_dist = dist;
                    best_node = node_id;
                }
            } else {
                // First one we find is fine.
                best_node = node_id;
                break;
            }
        }

        best_node
    }

    /// Raven `int CNavigator::GetBestNode( int startID, int endID, int rejectID
    /// = NODE_NONE )` — rank-table lookup, pure graph query (golden surface).
    /// `G_NAV_GETBESTNODE`.
    ///
    /// Source: `oracle/codemp/server/NPCNav/navigator.cpp:2377-2686`
    pub fn get_best_node(&mut self, start_id: i32, end_id: i32, reject_id: i32) -> i32 {
        let n = self.nodes.len() as i32;

        if start_id < 0 || start_id >= n {
            return WAYPOINT_NONE;
        }
        if end_id < 0 || end_id >= n {
            return WAYPOINT_NONE;
        }
        if start_id == end_id {
            return start_id;
        }

        let mut best_node = -1;
        let mut best_rank = Q3_INFINITE;
        let mut reject_rank = 0;

        let num_edges = self.nodes[start_id as usize].get_num_edges();

        if reject_id != WAYPOINT_NONE {
            for i in 0..num_edges {
                let edge_id = self.nodes[start_id as usize].get_edge(i);
                if edge_id == reject_id {
                    reject_rank = self.nodes[end_id as usize].get_rank(edge_id);
                    break;
                }
            }
        }

        for i in 0..num_edges {
            let edge_id = self.nodes[start_id as usize].get_edge(i);

            // Found one.
            if edge_id == end_id {
                return edge_id;
            }

            let test_rank = self.nodes[end_id as usize].get_rank(edge_id);

            if test_rank <= reject_rank {
                continue;
            }

            // No possible connection.
            if test_rank == NODE_NONE {
                return NODE_NONE;
            }

            // Found a better one.
            if test_rank < best_rank {
                best_node = edge_id;
                best_rank = test_rank;
            }
        }

        best_node
    }

    /// Raven `int CNavigator::GetNodePosition( int nodeID, vec3_t out )` —
    /// range-guards `nodeID`, writes `Node::get_position`. Pure graph query
    /// (golden surface). `G_NAV_GETNODEPOSITION`.
    ///
    /// Source: `oracle/codemp/server/NPCNav/navigator.cpp:2447`
    pub fn get_node_position(&self, node_id: i32, out: &mut vec3_t) -> i32 {
        // Validate the number.
        if node_id < 0 || node_id >= self.nodes.len() as i32 {
            return false as i32;
        }

        *out = self.nodes[node_id as usize].get_position();

        true as i32
    }

    /// Raven `int CNavigator::GetNodeNumEdges( int nodeID )` — range-guarded
    /// pure graph query (golden surface). `G_NAV_GETNODENUMEDGES`.
    ///
    /// Source: `oracle/codemp/server/NPCNav/navigator.cpp:2465`
    pub fn get_node_num_edges(&self, node_id: i32) -> i32 {
        if node_id < 0 || node_id >= self.nodes.len() as i32 {
            return -1;
        }

        self.nodes[node_id as usize].get_num_edges()
    }

    /// Raven `int CNavigator::GetNodeEdge( int nodeID, int edge )` —
    /// range-guarded pure graph query (golden surface). `G_NAV_GETNODEEDGE`.
    ///
    /// Source: `oracle/codemp/server/NPCNav/navigator.cpp:2483`
    pub fn get_node_edge(&self, node_id: i32, edge: i32) -> i32 {
        if node_id < 0 || node_id >= self.nodes.len() as i32 {
            return -1;
        }

        self.nodes[node_id as usize].get_edge(edge)
    }

    /// Raven `int CNavigator::GetNumNodes( void ) const { return
    /// m_nodes.size(); }` (navigator.h:184, inline) — `nodes.len()`.
    /// `G_NAV_GETNUMNODES`.
    ///
    /// Source: `oracle/codemp/server/NPCNav/navigator.h:184`
    pub fn get_num_nodes(&self) -> i32 {
        self.nodes.len() as i32
    }

    /// Raven `bool CNavigator::Connected( int startID, int endID )` — pure
    /// graph query (golden surface). `G_NAV_CONNECTED`.
    ///
    /// Source: `oracle/codemp/server/NPCNav/navigator.cpp:2377-2686`
    pub fn connected(&self, start_id: i32, end_id: i32) -> bool {
        let n = self.nodes.len() as i32;

        if start_id < 0 || start_id >= n {
            return false;
        }
        if end_id < 0 || end_id >= n {
            return false;
        }
        if start_id == end_id {
            return true;
        }

        let num_edges = self.nodes[start_id as usize].get_num_edges();

        for i in 0..num_edges {
            let edge_id = self.nodes[start_id as usize].get_edge(i);

            // Found one.
            if edge_id == end_id {
                return true;
            }

            if self.nodes[end_id as usize].get_rank(edge_id) != NODE_NONE {
                return true;
            }
        }

        false
    }

    /// Raven `unsigned int CNavigator::GetPathCost( int startID, int endID )`
    /// — reads a rank-table entry populated by
    /// [`calculate_path`](Self::calculate_path); pure graph query (golden
    /// surface). `G_NAV_GETPATHCOST`.
    ///
    /// Source: `oracle/codemp/server/NPCNav/navigator.cpp:2377-2686`
    pub fn get_path_cost(&self, start_id: i32, end_id: i32) -> u32 {
        let n = self.nodes.len() as i32;

        if start_id < 0 || start_id >= n {
            return Q3_INFINITE as u32;
        }
        if end_id < 0 || end_id >= n {
            return Q3_INFINITE as u32;
        }

        if self.nodes[start_id as usize].get_num_edges() == 0 {
            // WTF? Solitary waypoint! Bad designer!
            return Q3_INFINITE as u32;
        }

        let mut move_id = start_id;
        let mut path_cost = 0i32;
        let mut dont_screw_up = 0;

        while move_id != end_id {
            let mut best_rank = WORLD_SIZE as i32;
            let mut best_node = -1;
            let mut best_cost = 0;

            let num_edges = self.nodes[move_id as usize].get_num_edges();
            for i in 0..num_edges {
                let edge_id = self.nodes[move_id as usize].get_edge(i);

                // Done.
                if edge_id == end_id {
                    return (path_cost + self.nodes[move_id as usize].get_edge_cost(i)) as u32;
                }

                let test_rank = self.nodes[end_id as usize].get_rank(edge_id);

                // No possible connection.
                if test_rank == NODE_NONE {
                    return Q3_INFINITE as u32;
                }

                // Found a better one.
                if test_rank < best_rank {
                    best_node = edge_id;
                    best_rank = test_rank;
                    best_cost = self.nodes[move_id as usize].get_edge_cost(i);
                }
            }

            path_cost += best_cost;

            // Take a new best node.
            move_id = best_node;
            dont_screw_up += 1;

            if dont_screw_up > 40000 {
                // Ok, I think something probably screwed up.
                break;
            }
        }

        path_cost as u32
    }

    /// Raven `unsigned int CNavigator::GetEdgeCost( int startID, int endID )`
    /// — validates ids then **unconditionally** delegates to the trace form
    /// [`get_edge_cost_trace`](Self::get_edge_cost_trace) (:2634),
    /// trace-dependent (3c surface). `G_NAV_GETEDGECOST`.
    ///
    /// Source: `oracle/codemp/server/NPCNav/navigator.cpp:2621-2635`
    pub fn get_edge_cost(&mut self, host: &mut impl EngineHost, start_id: i32, end_id: i32) -> u32 {
        let n = self.nodes.len() as i32;

        if start_id < 0 || start_id >= n {
            return Q3_INFINITE as u32;
        }
        if end_id < 0 || end_id >= n {
            return Q3_INFINITE as u32;
        }

        self.get_edge_cost_trace(host, start_id, end_id) as u32
    }

    /// Raven `int CNavigator::GetProjectedNode( vec3_t origin, int nodeID )`
    /// — pure graph query (golden surface); `Q3_INFINITE`/vec3 primitives
    /// imported from `mp_qshared` (NAV-D3 / RULING 39d). `G_NAV_GETPROJECTEDNODE`.
    ///
    /// Source: `oracle/codemp/server/NPCNav/navigator.cpp:2377-2686`
    pub fn get_projected_node(&self, origin: vec3_t, node_id: i32) -> i32 {
        // Validate the start position.
        if node_id < 0 || node_id >= self.nodes.len() as i32 {
            return NODE_NONE;
        }

        let mut best_dot = 0.0f32;
        let mut best_node = NODE_NONE;

        // Setup our target direction.
        let base_pos = self.nodes[node_id as usize].get_position();

        let mut target_dir = [0.0f32; 3];
        _VectorSubtract(origin, base_pos, &mut target_dir);
        VectorNormalize(&mut target_dir);

        let num_edges = self.nodes[node_id as usize].get_num_edges();

        for i in 0..num_edges {
            let edge_id = self.nodes[node_id as usize].get_edge(i);
            let temp_pos = self.nodes[edge_id as usize].get_position();

            let mut temp_dir = [0.0f32; 3];
            _VectorSubtract(temp_pos, base_pos, &mut temp_dir);
            VectorNormalize(&mut temp_dir);

            let dot = _DotProduct(target_dir, temp_dir);

            if dot < 0.0 {
                continue;
            }

            if dot > best_dot {
                best_dot = dot;
                best_node = self.nodes[edge_id as usize].get_id();
            }
        }

        best_node
    }

    /// Raven `int CNavigator::GetNodeRadius( int nodeID )` — guards only the
    /// empty-graph case in Raven (D-8 UB on out-of-range `nodeID` otherwise);
    /// the Rust port adds the sibling range guard, returning the function's
    /// own empty-graph sentinel `0` (§19, D-8). Host-free pure query (golden
    /// surface). `G_NAV_GETNODERADIUS`.
    ///
    /// Source: `oracle/codemp/server/NPCNav/navigator.cpp:1029-1034`
    pub fn get_node_radius(&self, node_id: i32) -> i32 {
        if self.nodes.is_empty() {
            return 0;
        }
        // §19 / D-8: Raven's `m_nodes[nodeID]` is unchecked on a non-empty graph
        // — add the sibling range guard, returning its own `0` sentinel.
        if node_id < 0 || node_id >= self.nodes.len() as i32 {
            return 0;
        }
        self.nodes[node_id as usize].get_radius()
    }

    // ---- Failed-node bookkeeping (deref *mut sharedEntity_t from VMA(1)) ---

    /// Raven `void CNavigator::CheckFailedNodes( sharedEntity_t *ent )` —
    /// re-tests `ent->failedWaypoints[]`, stamping
    /// `svs.time + CHECK_FAILED_EDGE_INTERVAL + Q_irand(0,1000)` via
    /// `host.sv_time`/`host.irand` (:1763, NAV-D3 / RULING 39c).
    /// `G_NAV_CHECKFAILEDNODES`.
    ///
    /// Source: `oracle/codemp/server/NPCNav/navigator.cpp:1724-1766`
    pub fn check_failed_nodes(&mut self, host: &mut impl EngineHost, ent: *mut sharedEntity_t) {
        // Must have nodes.
        if self.nodes.is_empty() {
            return;
        }

        let check_time = unsafe { (*ent).failedWaypointCheckTime };
        if check_time != 0 && check_time < host.sv_time() {
            let mut failed = 0;
            let mins = unsafe { (*ent).r.mins };
            let maxs = unsafe { (*ent).r.maxs };

            // Do this only once every 1 second.
            for j in 0..MAX_FAILED_NODES {
                if unsafe { (*ent).failedWaypoints[j] } != 0 {
                    failed += 1;
                    // -1 because 0 is a valid node but also the default.
                    let idx = (unsafe { (*ent).failedWaypoints[j] } - 1) as usize;
                    let node_pos = self.nodes[idx].get_position();

                    if GNavCallback_NAV_ClearPathToPoint(
                        host,
                        ent,
                        &mins,
                        &maxs,
                        &node_pos,
                        CONTENTS_SOLID | CONTENTS_MONSTERCLIP | CONTENTS_BOTCLIP,
                        ENTITYNUM_NONE,
                    ) == qfalse
                    {
                        // No path clear of architecture, so clear this.
                        unsafe { (*ent).failedWaypoints[j] = 0 };
                        failed -= 1;
                    } else if GNavCallback_NAV_ClearPathToPoint(
                        host,
                        ent,
                        &mins,
                        &maxs,
                        &node_pos,
                        CONTENTS_BODY,
                        ENTITYNUM_NONE,
                    ) != qfalse
                    {
                        // Clear of ents, too, so all clear, clear this one out.
                        unsafe { (*ent).failedWaypoints[j] = 0 };
                        failed -= 1;
                    }
                }
            }

            if failed == 0 {
                unsafe { (*ent).failedWaypointCheckTime = 0 };
            } else {
                unsafe {
                    (*ent).failedWaypointCheckTime =
                        host.sv_time() + CHECK_FAILED_EDGE_INTERVAL + host.irand(0, 1000)
                };
            }
        }
    }

    /// Raven `void CNavigator::AddFailedNode( sharedEntity_t *ent, int nodeID
    /// )` — stamps `ent->failedWaypointCheckTime = svs.time +
    /// CHECK_FAILED_EDGE_INTITIAL` via `host.sv_time` (:1778,1797 — a
    /// **different** constant from `CheckFailedNodes`, **no** `Q_irand`
    /// jitter). `G_NAV_ADDFAILEDNODE`.
    ///
    /// Source: `oracle/codemp/server/NPCNav/navigator.cpp:1768-1799`
    pub fn add_failed_node(
        &mut self,
        host: &mut impl EngineHost,
        ent: *mut sharedEntity_t,
        node_id: i32,
    ) {
        let _ = &self; // self is unused: the failed-node array lives on `ent`.
        let mut j = 0;
        while j < MAX_FAILED_NODES {
            let fw = unsafe { (*ent).failedWaypoints[j] };
            if fw == 0 {
                // +1 because 0 is the default value and a valid node.
                unsafe { (*ent).failedWaypoints[j] = node_id + 1 };
                if unsafe { (*ent).failedWaypointCheckTime } == 0 {
                    unsafe {
                        (*ent).failedWaypointCheckTime = host.sv_time() + CHECK_FAILED_EDGE_INTITIAL
                    };
                }
                return;
            }
            if fw == node_id + 1 {
                // Already have this one marked as failed.
                return;
            }
            j += 1;
        }

        // Ran out of failed nodes, get rid of first one, shift rest up.
        for k in 0..MAX_FAILED_NODES - 1 {
            unsafe { (*ent).failedWaypoints[k] = (*ent).failedWaypoints[k + 1] };
        }
        unsafe { (*ent).failedWaypoints[MAX_FAILED_NODES - 1] = node_id + 1 };
        if unsafe { (*ent).failedWaypointCheckTime } == 0 {
            unsafe { (*ent).failedWaypointCheckTime = host.sv_time() + CHECK_FAILED_EDGE_INTITIAL };
        }
    }

    /// Raven `qboolean CNavigator::NodeFailed( sharedEntity_t *ent, int nodeID
    /// )` — reads `ent->failedWaypoints[]`, host-free. `G_NAV_NODEFAILED`.
    ///
    /// Source: `oracle/codemp/server/NPCNav/navigator.cpp:1801-1811`
    pub fn node_failed(&self, ent: *mut sharedEntity_t, node_id: i32) -> qboolean {
        for j in 0..MAX_FAILED_NODES {
            if (unsafe { (*ent).failedWaypoints[j] } - 1) == node_id {
                return qtrue;
            }
        }
        qfalse
    }

    /// Raven `qboolean CNavigator::NodesAreNeighbors( int startID, int endID )`
    /// — scans `nodes[startID].edges` for `endID`; host-free.
    /// `G_NAV_NODESARENEIGHBORS`.
    ///
    /// Source: `oracle/codemp/server/NPCNav/navigator.cpp:1813-1833`
    pub fn nodes_are_neighbors(&self, start_id: i32, end_id: i32) -> qboolean {
        if start_id == end_id {
            return qfalse;
        }

        // NOTE: we only check start because we assume all connections are 2-way.
        let num_edges = self.nodes[start_id as usize].get_num_edges();
        for i in 0..num_edges {
            if self.nodes[start_id as usize].get_edge(i) == end_id {
                return qtrue;
            }
        }
        qfalse
    }

    // ---- Failed-edge bookkeeping (failedEdge_t crosses by pointer via VMA) -

    /// Raven `void CNavigator::ClearFailedEdge( failedEdge_t *failedEdge )` —
    /// **host-free** (NAV-D4 / RULING 48): calls only the host-free
    /// [`set_edge_cost`](Self::set_edge_cost) (pure `nodes`/distance/`AddEdge`
    /// work) and writes `startID = endID = WAYPOINT_NONE`, `entID =
    /// ENTITYNUM_NONE`, `checkTime = 0`. Matches sibling
    /// [`clear_all_failed_edges`](Self::clear_all_failed_edges).
    /// `G_NAV_CLEARFAILEDEDGE`.
    ///
    /// Source: `oracle/codemp/server/NPCNav/navigator.cpp:1835-1865`
    pub fn clear_failed_edge(&mut self, e: &mut failedEdge_t) {
        // Raven's leading `if ( !failedEdge ) return;` null guard has no Rust
        // counterpart — a `&mut` reference is never null.

        // Clear failedEdge info (the commented-out edge-flag block, :1843-1859,
        // is dead and omitted).
        self.set_edge_cost(e.startID, e.endID, -1);
        e.startID = WAYPOINT_NONE;
        e.endID = WAYPOINT_NONE;
        e.entID = ENTITYNUM_NONE;
        e.checkTime = 0;
    }

    /// Raven `void CNavigator::ClearAllFailedEdges( void )` — `memset`s
    /// `failedEdges[]` to `WAYPOINT_NONE` (:1869) then calls
    /// [`clear_failed_edge`](Self::clear_failed_edge) per slot; host-free
    /// (its sole callee is). `G_NAV_CLEARALLFAILEDEDGES`.
    ///
    /// Source: `oracle/codemp/server/NPCNav/navigator.cpp:1867-1874`
    pub fn clear_all_failed_edges(&mut self) {
        // Raven `memset( &failedEdges, WAYPOINT_NONE, sizeof(...) )` — every
        // byte 0xFF, so each `int` field becomes -1 (`WAYPOINT_NONE`).
        for e in &mut self.failed_edges {
            e.startID = WAYPOINT_NONE;
            e.endID = WAYPOINT_NONE;
            e.checkTime = WAYPOINT_NONE;
            e.entID = WAYPOINT_NONE;
        }
        for j in 0..MAX_FAILED_EDGES {
            // Copy-out / copy-back to break the `&mut self` + `&mut e` alias
            // (`clear_failed_edge` also mutates `self` via `set_edge_cost`);
            // behaviorally identical — `clear_failed_edge`'s edits to `*e` are
            // written back.
            let mut e = self.failed_edges[j];
            self.clear_failed_edge(&mut e);
            self.failed_edges[j] = e;
        }
    }

    /// Raven `int CNavigator::EdgeFailed( int startID, int endID )` —
    /// `edge_lookup` `equal_range` first-match scan (NAV-D3); host-free.
    /// `G_NAV_EDGEFAILED`.
    ///
    /// Source: `oracle/codemp/server/NPCNav/navigator.cpp:1876-1923`
    pub fn edge_failed(&self, start_id: i32, end_id: i32) -> i32 {
        // OPTIMIZED WAY (bjg 01/02) — find in lookup map.
        if let Some(indices) = self.edge_lookup.get(&start_id) {
            for &idx in indices {
                if self.failed_edges[idx].endID == end_id {
                    return idx as i32;
                }
            }
        }
        if let Some(indices) = self.edge_lookup.get(&end_id) {
            for &idx in indices {
                if self.failed_edges[idx].endID == start_id {
                    return idx as i32;
                }
            }
        }

        -1
    }

    /// Raven `void CNavigator::AddFailedEdge( int entID, int startID, int
    /// endID )` — `d_altRoutes`-sibling `d_patched` read via
    /// `host.cvar_integer` (:1933, NAV-D3), `Com_Printf` diagnostics
    /// (:1945-2053) via `host.print`, stamps `svs.time +
    /// CHECK_FAILED_EDGE_INTERVAL + Q_irand(0,1000)` via `host.sv_time` +
    /// `host.irand` (:1987/2010, RULING 39c). `G_NAV_ADDFAILEDEDGE`.
    ///
    /// Source: `oracle/codemp/server/NPCNav/navigator.cpp:1925-2055`
    pub fn add_failed_edge(
        &mut self,
        host: &mut impl EngineHost,
        ent_id: i32,
        start_id: i32,
        end_id: i32,
    ) {
        let n = self.nodes.len() as i32;

        // Must have nodes.
        if n == 0 {
            return;
        }

        if host.cvar_integer("d_patched") != 0 {
            // Use patch-style navigation.
            if start_id == end_id {
                // Not an edge!
                return;
            }
        }

        // Validate the ent number. (`#ifndef FINAL_BUILD` prints — that macro
        // is not in the WinDed Release set; the trailing `assert(0)` is an
        // always-fail debug marker, release-elided under NDEBUG (D-6), omitted
        // to preserve the shipping print-then-return behavior.)
        if ent_id < 0 || ent_id > ENTITYNUM_NONE {
            host.print(&format!("^1NAV ERROR: envalid ent {}\n", ent_id));
            return;
        }

        // Validate the start position.
        if start_id < 0 || start_id >= n {
            host.print(&format!(
                "^1NAV ERROR: tried to fail invalid waypoint {}\n",
                start_id
            ));
            return;
        }

        // Validate the end position.
        if end_id < 0 || end_id >= n {
            host.print(&format!(
                "^1NAV ERROR: tried to fail invalid waypoint {}\n",
                end_id
            ));
            return;
        }

        // First see if we already have this one.
        let existing = self.edge_failed(start_id, end_id);
        if existing != -1 {
            // Just remember this guy instead.
            self.failed_edges[existing as usize].entID = ent_id;
            return;
        }

        // Okay, new one, find an empty slot.
        for j in 0..MAX_FAILED_EDGES {
            if self.failed_edges[j].startID == WAYPOINT_NONE {
                self.failed_edges[j].startID = start_id;
                self.failed_edges[j].endID = end_id;
                // Check one second from now to see if it's clear.
                self.failed_edges[j].checkTime =
                    host.sv_time() + CHECK_FAILED_EDGE_INTERVAL + host.irand(0, 1000);

                self.edge_lookup.entry(start_id).or_default().push(j);

                // Remember who needed it.
                self.failed_edges[j].entID = ent_id;

                // Now recalc all the paths!
                if self.paths_calculated != qfalse {
                    // Reconnect the nodes and mark every node's flag NF_RECALC.
                    self.set_edge_cost(start_id, end_id, Q3_INFINITE);
                    self.flag_all_nodes(NF_RECALC);
                }
                return;
            }
        }

        host.print(&format!(
            "^1NAV ERROR: too many blocked waypoint connections ({})!!!\n",
            MAX_FAILED_EDGES
        ));
    }

    /// Raven `qboolean CNavigator::CheckFailedEdge( failedEdge_t *failedEdge )`
    /// — trace/PVS-dependent (3c surface); the live `#else` `SV_Trace` arm is
    /// taken, the `#if 0` `NAVNEW_ClearPathBetweenPoints` arm does not compile
    /// (D-4, dead). Stamps `svs.time + Q_irand(0,1000)` via `host.sv_time` +
    /// `host.irand` (:2065/2137). `G_NAV_CHECKFAILEDEDGE`.
    ///
    /// Source: `oracle/codemp/server/NPCNav/navigator.cpp:2057-2142`
    pub fn check_failed_edge(
        &mut self,
        host: &mut impl EngineHost,
        e: &mut failedEdge_t,
    ) -> qboolean {
        // Raven's leading `if ( !failedEdge ) return qfalse;` — a `&mut` is
        // never null, guard omitted.

        // Every 1 second, see if our failed edges are clear.
        if e.checkTime < host.sv_time() && e.startID != WAYPOINT_NONE {
            let mut mins;
            let mut maxs;
            let ignore;
            let clipmask;

            let ent = host.gentity(e.entID);
            let bad = ent.is_null() || {
                let e_type = unsafe { (*ent).s.eType };
                let e_flags = unsafe { (*ent).s.eFlags };
                (e_type != ET_PLAYER && e_type != ET_NPC) || (e_flags & EF_DEAD) != 0
            };

            if bad {
                mins = [-15.0, -15.0, DEFAULT_MINS_2 + stepsize()];
                maxs = [15.0, 15.0, DEFAULT_MAXS_2];
                ignore = ENTITYNUM_NONE;
                clipmask = MASK_NPCSOLID;
            } else {
                mins = unsafe { (*ent).r.mins };
                mins[2] += stepsize();
                maxs = unsafe { (*ent).r.maxs };
                ignore = e.entID;
                clipmask = MASK_SOLID; // Raven rww note: ent->clipmask; share clipmask?
            }

            if maxs[2] < mins[2] {
                // Don't invert bounding box.
                maxs[2] = mins[2];
            }

            let start = self.nodes[e.startID as usize].get_position();
            let end = self.nodes[e.endID as usize].get_position();

            // D-4: the `#if 0` `NAVNEW_ClearPathBetweenPoints` arm does not
            // compile — only the live `#else` `SV_Trace` arm is ported.

            // Test if they're even conceivably close to one another.
            if !self.sv_in_pvs(start, end) {
                return qfalse;
            }

            let mut trace = zeroed_trace();
            host.trace(
                &mut trace,
                &start,
                &mins,
                &maxs,
                &end,
                ignore,
                clipmask | CONTENTS_MONSTERCLIP | CONTENTS_BOTCLIP,
                false,
                0,
                10,
            );

            if trace.startsolid != 0 || trace.allsolid != 0 {
                return qfalse;
            }
            let mut hit_ent_num = trace.entityNum as i32;

            // If we did hit something, see if it's just an auto-door.
            if hit_ent_num != ENTITYNUM_NONE
                && GNavCallback_G_EntIsUnlockedDoor(host, hit_ent_num) != qfalse
            {
                hit_ent_num = ENTITYNUM_NONE;
            } else if hit_ent_num == e.entID {
                // Don't hit the person who initially marked the edge failed.
                hit_ent_num = ENTITYNUM_NONE;
            }

            if hit_ent_num == ENTITYNUM_NONE {
                // If so, clear it.
                self.clear_failed_edge(e);
                return qtrue;
            } else {
                // Check again in one second.
                e.checkTime = host.sv_time() + CHECK_FAILED_EDGE_INTERVAL + host.irand(0, 1000);
            }
        }
        qfalse
    }

    /// Raven `void CNavigator::CheckAllFailedEdges( void )` — loops
    /// [`check_failed_edge`](Self::check_failed_edge) over `failedEdges[]`.
    /// `G_NAV_CHECKALLFAILEDEDGES`.
    ///
    /// Source: `oracle/codemp/server/NPCNav/navigator.cpp:2144-2168`
    pub fn check_all_failed_edges(&mut self, host: &mut impl EngineHost) {
        // Must have nodes.
        if self.nodes.is_empty() {
            return;
        }

        let mut cleared_any = qfalse;

        for j in 0..MAX_FAILED_EDGES {
            // Copy-out / copy-back to break the `&mut self` + `&mut failedEdge`
            // alias (`check_failed_edge` takes `&mut self`); `check_failed_edge`
            // edits `*e`, which we persist.
            let mut e = self.failed_edges[j];
            let r = self.check_failed_edge(host, &mut e);
            self.failed_edges[j] = e;
            cleared_any = if r != qfalse { qtrue } else { cleared_any };
        }

        if cleared_any != qfalse {
            // Need to recalc the paths.
            if self.paths_calculated != qfalse {
                self.flag_all_nodes(NF_RECALC);
            }
        }
    }

    /// Raven `qboolean CNavigator::RouteBlocked( int startID, int testEdgeID,
    /// int endID, int rejectRank )` — rank-guided `while(1)` walk; host-free
    /// pure graph work. `G_NAV_ROUTEBLOCKED`.
    ///
    /// Source: `oracle/codemp/server/NPCNav/navigator.cpp:2170-2253`
    pub fn route_blocked(
        &self,
        start_id: i32,
        test_edge_id: i32,
        end_id: i32,
        reject_rank: i32,
    ) -> qboolean {
        let mut best_next_id = NODE_NONE;
        let mut best_rank = reject_rank;

        if self.edge_failed(start_id, test_edge_id) != -1 {
            return qtrue;
        }

        if test_edge_id == end_id {
            // Neighbors, checked out, all clear.
            return qfalse;
        }

        // Okay, first edge is clear, now check rest of route!
        let mut next_id = test_edge_id;
        let mut last_id = start_id;

        loop {
            let mut all_edges_failed = true;

            let num_edges = self.nodes[next_id as usize].get_num_edges();
            for i in 0..num_edges {
                let edge_id = self.nodes[next_id as usize].get_edge(i);

                if edge_id == last_id {
                    // Don't backtrack.
                    continue;
                }
                if edge_id == start_id {
                    // Don't loop around.
                    continue;
                }
                if self.edge_failed(next_id, edge_id) != -1 {
                    // This edge blocked, check next.
                    continue;
                }
                if edge_id == end_id {
                    // We got there all clear!
                    return qfalse;
                }

                // Still going...
                let test_rank = self.nodes[end_id as usize].get_rank(edge_id);

                if test_rank < 0 {
                    // No route this way.
                    continue;
                }

                // Is the rank good enough?
                if test_rank < best_rank {
                    best_next_id = edge_id;
                    best_rank = test_rank;
                    all_edges_failed = false;
                }
            }

            if all_edges_failed {
                // This route has no clear way of getting to end.
                return qtrue;
            } else {
                last_id = next_id;
                next_id = best_next_id;
            }
        }
    }

    /// Raven `int CNavigator::GetBestNodeAltRoute( int startID, int endID, int
    /// *pathCost, int rejectID = NODE_NONE )` (4-arg overload) — `d_altRoutes`
    /// read via `host.cvar_integer` (:2278/2323/2346, NAV-D3).
    /// `G_NAV_GETBESTNODEALTROUTE`.
    ///
    /// Source: `oracle/codemp/server/NPCNav/navigator.cpp:2261-2370`
    pub fn get_best_node_alt_route(
        &mut self,
        host: &mut impl EngineHost,
        start_id: i32,
        end_id: i32,
        path_cost: &mut i32,
        reject_id: i32,
    ) -> i32 {
        let n = self.nodes.len() as i32;

        // Must have nodes.
        if n == 0 {
            return WAYPOINT_NONE;
        }
        if start_id < 0 || start_id >= n {
            return WAYPOINT_NONE;
        }
        if end_id < 0 || end_id >= n {
            return WAYPOINT_NONE;
        }

        // Is it the same node?
        if start_id == end_id {
            if host.cvar_integer("d_altRoutes") == 0 || self.edge_failed(start_id, end_id) == -1 {
                return start_id;
            } else {
                return WAYPOINT_NONE;
            }
        }

        let mut best_node = -1;
        let mut best_rank = Q3_INFINITE;
        let mut reject_rank = Q3_INFINITE;
        let mut best_cost = Q3_INFINITE;

        *path_cost = 0;

        let num_edges = self.nodes[start_id as usize].get_num_edges();

        // Find the minimum rank of the edge(s) we want to reject as paths.
        if reject_id != WAYPOINT_NONE {
            for i in 0..num_edges {
                if self.nodes[start_id as usize].get_edge(i) == reject_id {
                    reject_rank = self.get_path_cost(start_id, end_id) as i32;
                    break;
                }
            }
        }

        for i in 0..num_edges {
            let edge_id = self.nodes[start_id as usize].get_edge(i);

            let test_rank = self.get_path_cost(edge_id, end_id) as i32;

            // Make sure it's not worse than our reject rank.
            if test_rank >= reject_rank {
                continue;
            }

            // Found one.
            if edge_id == end_id {
                if host.cvar_integer("d_altRoutes") == 0
                    || self.route_blocked(start_id, edge_id, end_id, reject_rank) == qfalse
                {
                    *path_cost += self.nodes[start_id as usize].get_edge_cost(i);
                    return edge_id;
                } else {
                    // This is blocked, can't consider it.
                    continue;
                }
            }

            // No possible connection.
            if test_rank == NODE_NONE {
                *path_cost = Q3_INFINITE;
                return NODE_NONE;
            }

            // Found a better one.
            if test_rank < best_rank
                && (host.cvar_integer("d_altRoutes") == 0
                    || self.route_blocked(start_id, edge_id, end_id, reject_rank) == qfalse)
            {
                best_node = edge_id;
                best_rank = test_rank;
                best_cost = self.nodes[start_id as usize].get_edge_cost(i) + test_rank;
            }
        }

        *path_cost = best_cost;

        best_node
    }

    /// Raven `int CNavigator::GetBestNodeAltRoute( int startID, int endID, int
    /// rejectID = NODE_NONE )` (3-arg overload) — delegates to the 4-arg form
    /// (discarding `pathCost`). `G_NAV_GETBESTNODEALT2` (the second arm the
    /// game issues for this overloaded name).
    ///
    /// Source: `oracle/codemp/server/NPCNav/navigator.cpp:2261-2370`
    pub fn get_best_node_alt_route2(
        &mut self,
        host: &mut impl EngineHost,
        start_id: i32,
        end_id: i32,
        reject_id: i32,
    ) -> i32 {
        let mut junk = 0;
        self.get_best_node_alt_route(host, start_id, end_id, &mut junk, reject_id)
    }

    /// Raven `int CNavigator::GetBestPathBetweenEnts( sharedEntity_t *ent,
    /// sharedEntity_t *goal, int flags )` — trace/PVS-dependent (3c surface);
    /// `ent`/`goal` are the raw `*mut sharedEntity_t` the trap marshals
    /// (`VMA(1)`+`VMA(2)`, sv_game.cpp:917, NAV-D3).
    /// `G_NAV_GETBESTPATHBETWEENENTS`.
    ///
    /// Source: `oracle/codemp/server/NPCNav/navigator.cpp:1320-1624`
    pub fn get_best_path_between_ents(
        &mut self,
        host: &mut impl EngineHost,
        ent: *mut sharedEntity_t,
        goal: *mut sharedEntity_t,
        flags: i32,
    ) -> i32 {
        // Must have nodes.
        if self.nodes.is_empty() {
            return NODE_NONE;
        }

        let ent_origin = unsafe { (*ent).r.currentOrigin };
        let goal_origin = unsafe { (*goal).r.currentOrigin };
        let ent_number = unsafe { (*ent).s.number };
        let goal_number = unsafe { (*goal).s.number };

        let mut node_chain: Vec<(i32, u32)> = Vec::new();
        let mut node_chain2: Vec<(i32, u32)> = Vec::new();

        // Collect all nodes within a certain radius.
        self.collect_nearest_nodes(
            ent_origin,
            NODE_COLLECT_RADIUS,
            NODE_COLLECT_MAX,
            &mut node_chain,
        );
        self.collect_nearest_nodes(
            goal_origin,
            NODE_COLLECT_RADIUS,
            NODE_COLLECT_MAX,
            &mut node_chain2,
        );

        let mut best_cost = Q3_INFINITE;
        let mut best_node = NODE_NONE;
        let mut next_node = NODE_NONE;

        unsafe { (*ent).waypoint = NODE_NONE };
        unsafe { (*goal).waypoint = NODE_NONE };

        // Look through all nodes.
        for &(node_id, ndist) in node_chain.iter() {
            let position = self.nodes[node_id as usize].get_position();

            let cn = self.checked_node(node_id, ent_number);
            if cn == CHECKED_FAILED {
                // Already checked this node against ent and it failed.
                continue;
            }
            if cn == CHECKED_PASSED {
                // Already checked this node against ent and it passed.
            } else {
                // Haven't checked this node against ent yet.
                if self.node_failed(ent, node_id) != qfalse {
                    self.set_checked_node(node_id, ent_number, CHECKED_FAILED);
                    continue;
                }

                let radius = self.nodes[node_id as usize].get_radius();

                // If we're not within the known clear radius OR out of Z range.
                if ndist >= (radius * radius) as u32
                    || (position[2] - ent_origin[2]).abs() >= MAX_Z_DELTA
                {
                    // Need a clear path or LOS?
                    if flags & NF_CLEAR_PATH != 0 && !self.sv_in_pvs(ent_origin, position) {
                        // Not even potentially clear.
                        self.set_checked_node(node_id, ent_number, CHECKED_FAILED);
                        continue;
                    }
                    // Do we need a clear path?
                    if flags & NF_CLEAR_PATH != 0
                        && self.test_node_path(host, ent, goal_number, position, qtrue) == 0
                    {
                        self.set_checked_node(node_id, ent_number, CHECKED_FAILED);
                        continue;
                    }
                } // otherwise, inside the node so it must be clear (?)
                self.set_checked_node(node_id, ent_number, CHECKED_PASSED);
            }

            if host.cvar_integer("d_altRoutes") != 0 {
                // Calc the paths for this node if they're out of date.
                if self.nodes[node_id as usize].get_flags() & NF_RECALC != 0 {
                    self.calculate_path(node_id);
                }
            }

            for &(node_id2, ndist2) in node_chain2.iter() {
                if host.cvar_integer("d_altRoutes") != 0
                    && self.nodes[node_id2 as usize].get_flags() & NF_RECALC != 0
                {
                    self.calculate_path(node_id2);
                }

                let position2 = self.nodes[node_id2 as usize].get_position();

                // First get the entire path cost, including distance to first
                // node from ents' positions.
                let mut cost = (distance(ent_origin, position) as f64
                    + distance(goal_origin, position2) as f64)
                    .floor() as i32;

                if host.cvar_integer("d_altRoutes") != 0 {
                    let mut path_cost = 0;
                    next_node = self.get_best_node_alt_route(
                        host,
                        node_id,
                        node_id2,
                        &mut path_cost,
                        best_node,
                    );
                    cost += path_cost;
                } else {
                    cost += self.get_path_cost(node_id, node_id2) as i32;
                }

                if cost >= best_cost {
                    continue;
                }

                // Okay, this is the shortest path we've found yet.
                let cn2 = self.checked_node(node_id2, goal_number);
                if cn2 == CHECKED_FAILED {
                    continue;
                }
                if cn2 == CHECKED_PASSED {
                    // Already checked this node against goal and it passed.
                } else {
                    // Haven't checked this node against goal yet.
                    if self.node_failed(goal, node_id2) != qfalse {
                        self.set_checked_node(node_id2, goal_number, CHECKED_FAILED);
                        continue;
                    }

                    let radius2 = self.nodes[node_id2 as usize].get_radius();

                    if ndist2 >= (radius2 * radius2) as u32
                        || (position2[2] - goal_origin[2]).abs() >= MAX_Z_DELTA
                    {
                        if flags & NF_CLEAR_PATH != 0 && !self.sv_in_pvs(goal_origin, position2) {
                            self.set_checked_node(node_id2, goal_number, CHECKED_FAILED);
                            continue;
                        }
                        if flags & NF_CLEAR_PATH != 0
                            && self.test_node_path(host, goal, ent_number, position2, qfalse) == 0
                        {
                            self.set_checked_node(node_id2, goal_number, CHECKED_FAILED);
                            continue;
                        }
                    } // otherwise, inside the node so it must be clear (?)
                    self.set_checked_node(node_id2, goal_number, CHECKED_PASSED);
                }

                best_cost = cost;
                best_node = next_node;
                unsafe { (*ent).waypoint = node_id };
                unsafe { (*goal).waypoint = node_id2 };
            }
        }

        if host.cvar_integer("d_altRoutes") == 0 {
            // bestNode would not have been set by GetBestNodeAltRoute above.
            let ent_wp = unsafe { (*ent).waypoint };
            let goal_wp = unsafe { (*goal).waypoint };
            if ent_wp != NODE_NONE && goal_wp != NODE_NONE {
                // Have 2 valid waypoints which means a valid path.
                let mut bc = best_cost;
                best_node = self.get_best_node_alt_route(host, ent_wp, goal_wp, &mut bc, NODE_NONE);
                best_cost = bc;
                let _ = best_cost;
            }
        }

        best_node
    }

    /// Raven `void CNavigator::CheckBlockedEdges( void )` — trace-dependent
    /// (3c surface); door/breakable callbacks via `host.vm_call`.
    /// `G_NAV_CHECKBLOCKEDEDGES`.
    ///
    /// Source: `oracle/codemp/server/NPCNav/navigator.cpp:1036-1109`
    pub fn check_blocked_edges(&mut self, host: &mut impl EngineHost) {
        let n = self.nodes.len();

        // Go through all edges and test the ones that were blocked.
        for ni in 0..n {
            let num_edges = self.nodes[ni].get_num_edges();
            for edge_num in 0..num_edges {
                let flags = self.nodes[ni].get_edge_flags(edge_num);
                if (flags as i32 & EFLAG_BLOCKED) != 0 {
                    let first = self.nodes[ni].get_id();
                    let second = self.nodes[ni].get_edge(edge_num);
                    let p1 = self.nodes[first as usize].get_position();
                    let p2 = self.nodes[second as usize].get_position();
                    let mut failed = qfalse;

                    let mut trace = zeroed_trace();
                    host.trace(
                        &mut trace,
                        &p1,
                        &WP_MINS,
                        &WP_MAXS,
                        &p2,
                        ENTITYNUM_NONE,
                        MASK_SOLID | CONTENTS_MONSTERCLIP | CONTENTS_BOTCLIP,
                        false,
                        0,
                        10,
                    );

                    let trace_ent = trace.entityNum as i32;
                    if trace_ent < ENTITYNUM_WORLD
                        && (trace.fraction < 1.0 || trace.startsolid != 0 || trace.allsolid != 0)
                    {
                        if GNavCallback_G_EntIsDoor(host, trace_ent) != qfalse {
                            // Door.
                            if GNavCallback_G_EntIsUnlockedDoor(host, trace_ent) == qfalse {
                                // Locked door.
                                failed = qtrue;
                            }
                        } else if GNavCallback_G_EntIsBreakable(host, trace_ent) != qfalse {
                            // Do same for breakable brushes/models/glass?
                            failed = qtrue;
                        } else if GNavCallback_G_EntIsRemovableUsable(host, trace_ent) != qfalse {
                            failed = qtrue;
                        } else if trace.allsolid != 0 || trace.startsolid != 0 {
                            // Raven note: stuck inside an ent or the world?
                        } else {
                            // Raven note: what about func_plats and scripted movers?
                        }
                    }

                    if failed != qfalse {
                        self.add_failed_edge(host, ENTITYNUM_NONE, first, second);
                    }
                }
            }
        }
    }

    // ---- Checked-node memoisation (file-scope `CheckedNodes`, promoted) ----

    /// Raven `void CNavigator::ClearCheckedNodes( void )` — clears
    /// `checked_nodes`. `G_NAV_CLEARCHECKEDNODES`.
    ///
    /// Source: `oracle/codemp/server/NPCNav/navigator.cpp:1687-1719`
    pub fn clear_checked_nodes(&mut self) {
        self.checked_nodes.clear();
    }

    /// Raven `byte CNavigator::CheckedNode( int wayPoint, int ent )` — reads
    /// `checked_nodes[wayPoint*MAX_GENTITIES+ent]`, `CHECKED_NO` on miss.
    /// `G_NAV_CHECKEDNODE`.
    ///
    /// Source: `oracle/codemp/server/NPCNav/navigator.cpp:1687-1719`
    pub fn checked_node(&self, waypoint: i32, ent: i32) -> u8 {
        if waypoint < 0 || waypoint >= MAX_STORED_WAYPOINTS {
            return CHECKED_NO;
        }
        // `assert(ent>=0&&ent<MAX_GENTITIES)` — release-elided (D-6).
        match self
            .checked_nodes
            .get(&(waypoint * MAX_GENTITIES as i32 + ent))
        {
            Some(&v) => v,
            None => CHECKED_NO,
        }
    }

    /// Raven `void CNavigator::SetCheckedNode( int wayPoint, int ent, byte
    /// value )` — writes `checked_nodes[wayPoint*MAX_GENTITIES+ent] = value`.
    /// `G_NAV_SETCHECKEDNODE`.
    ///
    /// Source: `oracle/codemp/server/NPCNav/navigator.cpp:1687-1719`
    pub fn set_checked_node(&mut self, waypoint: i32, ent: i32, value: u8) {
        if waypoint < 0 || waypoint >= MAX_STORED_WAYPOINTS {
            return;
        }
        // `assert(ent...)` / `assert(value==...)` — release-elided (D-6).
        self.checked_nodes
            .insert(waypoint * MAX_GENTITIES as i32 + ent, value);
    }

    /// Raven `void CNavigator::FlagAllNodes( int newFlag )` — `AddFlag`s every
    /// node. `G_NAV_FLAGALLNODES`.
    ///
    /// Source: `oracle/codemp/server/NPCNav/navigator.cpp:496-504`
    pub fn flag_all_nodes(&mut self, new_flag: i32) {
        for node in &mut self.nodes {
            node.add_flag(new_flag);
        }
    }

    // ---- Protected helpers (not on the G_NAV_* surface; internal only) -----

    /// The deferred `SV_inPVS` shim (`server.h:356`) — **not** on the frozen
    /// `EngineHost` trait; the doc defers it to the server-spine work (Seam
    /// note), not to npcnav. Reports "potentially visible" conservatively: the
    /// only sites that gate real control flow on it (`CheckFailedEdge`,
    /// `GetBestPathBetweenEnts`) immediately re-verify with the authoritative
    /// host trace / `TestNodePath`, so over-approximating visibility defers to
    /// that ground truth; in `ShowNodes`/`ShowEdges` it gates only the
    /// stripped renderer draws. Reported under `problems`.
    fn sv_in_pvs(&self, _a: vec3_t, _b: vec3_t) -> bool {
        true
    }

    /// Raven `protected int CNavigator::TestNodePath( sharedEntity_t *ent, int
    /// okToHitEntNum, vec3_t position, qboolean includeEnts )` — trace/host
    /// callback helper; derefs the `*mut sharedEntity_t` (NAV-D3).
    ///
    /// Source: `oracle/codemp/server/NPCNav/navigator.cpp:1150-1237`
    fn test_node_path(
        &mut self,
        host: &mut impl EngineHost,
        ent: *mut sharedEntity_t,
        ok_to_hit_ent_num: i32,
        position: vec3_t,
        include_ents: qboolean,
    ) -> i32 {
        let _ = &self; // self is unused: the check is a pure game callback.
        let mut clipmask = MASK_SOLID; // ent->clipmask;
        if include_ents == qfalse {
            clipmask &= !CONTENTS_BODY;
        }

        let mins = unsafe { (*ent).r.mins };
        let maxs = unsafe { (*ent).r.maxs };

        // Check the path.
        if GNavCallback_NAV_ClearPathToPoint(
            host,
            ent,
            &mins,
            &maxs,
            &position,
            clipmask,
            ok_to_hit_ent_num,
        ) == qfalse
        {
            return false as i32;
        }

        true as i32
    }

    /// Raven `protected int CNavigator::TestNodeLOS( sharedEntity_t *ent,
    /// vec3_t position )` — trace/host callback helper; derefs the `*mut
    /// sharedEntity_t` (NAV-D3). Zero-caller: Raven's only call site is the
    /// commented-out `NF_CLEAR_LOS` block in `GetNearestNode` (:1593-1602), so
    /// this ports the method but retains no live caller (§20).
    ///
    /// Source: `oracle/codemp/server/NPCNav/navigator.cpp:1150-1237`
    #[allow(dead_code)]
    fn test_node_los(
        &mut self,
        host: &mut impl EngineHost,
        ent: *mut sharedEntity_t,
        position: vec3_t,
    ) -> i32 {
        let _ = &self; // self is unused: LOS is a pure game callback.
        GNavCallback_NPC_ClearLOS(host, ent, &position)
    }

    /// Raven `protected int CNavigator::TestBestFirst( sharedEntity_t *ent,
    /// int lastID, int flags )` — trace/host callback helper; derefs the
    /// `*mut sharedEntity_t` (NAV-D3).
    ///
    /// Source: `oracle/codemp/server/NPCNav/navigator.cpp:1150-1237`
    fn test_best_first(
        &mut self,
        host: &mut impl EngineHost,
        ent: *mut sharedEntity_t,
        last_id: i32,
        flags: i32,
    ) -> i32 {
        let _ = flags; // Raven takes `flags` but never reads it here.

        // Must be a valid one to begin with.
        if last_id == NODE_NONE {
            return NODE_NONE;
        }
        if last_id >= self.nodes.len() as i32 {
            return NODE_NONE;
        }

        let ent_origin = unsafe { (*ent).r.currentOrigin };
        let ent_number = unsafe { (*ent).s.number };

        let node_pos0 = self.nodes[last_id as usize].get_position();
        let num_edges = self.nodes[last_id as usize].get_num_edges();

        // Setup our last node as our root, and search for a closer one.
        let mut best_node = if self.test_node_path(host, ent, ENTITYNUM_NONE, node_pos0, qtrue) != 0
        {
            last_id
        } else {
            NODE_NONE
        };
        let mut best_dist = if best_node == NODE_NONE {
            Q3_INFINITE as f32
        } else {
            distance_squared(ent_origin, node_pos0)
        };

        // Test all these edges first.
        for i in 0..num_edges {
            let edge_id = self.nodes[last_id as usize].get_edge(i);
            let test_id = self.nodes[edge_id as usize].get_id();

            if self.node_failed(ent, test_id) != qfalse {
                continue;
            }

            let node_pos = self.nodes[edge_id as usize].get_position();

            let dist = distance_squared(ent_origin, node_pos);

            // Test against current best.
            if dist < best_dist {
                // See if this node is valid.
                if self.checked_node(test_id, ent_number) == CHECKED_PASSED
                    || self.test_node_path(host, ent, ENTITYNUM_NONE, node_pos, qtrue) != 0
                {
                    best_dist = dist;
                    best_node = test_id;
                    self.set_checked_node(test_id, ent_number, CHECKED_PASSED);
                } else {
                    self.set_checked_node(test_id, ent_number, CHECKED_FAILED);
                }
            }
        }

        best_node
    }

    /// Raven `protected int CNavigator::CollectNearestNodes( vec3_t origin,
    /// int radius, int maxCollect, nodeChain_l &nodeChain )` (`#if
    /// __NEWCOLLECT`, on) — host-free pure distance collection; Raven's
    /// `nodeChain_l` (`list<nodeList_t>`, `nodeList_t{int nodeID; unsigned
    /// int distance;}`) becomes an insert-sorted `Vec<(i32, u32)>` of
    /// `(nodeID, distance)` pairs (NAV-D3: "`Vec`/`VecDeque` insert-sorted" —
    /// internal, non-seam shape, a free porter choice per porting-rules §A).
    ///
    /// Source: `oracle/codemp/server/NPCNav/navigator.cpp:1249-1318`
    fn collect_nearest_nodes(
        &self,
        origin: vec3_t,
        radius: i32,
        max_collect: i32,
        node_chain: &mut Vec<(i32, u32)>,
    ) -> i32 {
        let mut collected = 0;

        // Get a distance rating for each node in the system.
        for ni in &self.nodes {
            let position = ni.get_position();
            let dist = distance_squared(position, origin);

            // Must be within our radius range.
            if dist > (radius * radius) as f32 {
                continue;
            }

            // Always add the first node.
            if node_chain.is_empty() {
                node_chain.insert(0, (ni.get_id(), dist as u32));
                continue;
            }

            let mut added = false;

            // Compare it to what we already have.
            let mut idx = 0;
            while idx < node_chain.len() {
                // If we're less than this entry, insert before it (Raven's
                // `dist(float) < (*nci).distance(uint)` promotes to float).
                if dist < node_chain[idx].1 as f32 {
                    node_chain.insert(idx, (ni.get_id(), dist as u32));
                    collected = node_chain.len() as i32;
                    added = true;

                    // If we've hit our collection limit, throw off the oldest.
                    if node_chain.len() > max_collect as usize {
                        node_chain.pop();
                    }

                    break;
                }
                idx += 1;
            }

            // Otherwise, pad out the collection if possible.
            if !added && node_chain.len() < max_collect as usize {
                node_chain.push((ni.get_id(), dist as u32));
            }
        }

        collected
    }

    /// Raven `protected int CNavigator::GetInt( fileHandle_t file )` — a
    /// cursor read, **not** a host call (Seam note): `Load`'s header parse
    /// reads an `int` off the same shared in-memory cursor `load` built
    /// (:531 `FS_Read`); the cursor's concrete type is deliberately unfrozen
    /// (internal, non-seam signature, porting-rules §A).
    ///
    /// Source: `oracle/codemp/server/NPCNav/navigator.cpp:527-534`
    fn get_int(&self, cursor: &mut std::io::Cursor<&[u8]>) -> i32 {
        let mut b = [0u8; 4];
        let _ = cursor.read_exact(&mut b);
        i32::from_le_bytes(b)
    }

    /// Raven `protected long CNavigator::GetLong( fileHandle_t file )` — a
    /// cursor read, **not** a host call (Seam note); the `.nav` NAV header id
    /// this feeds (`long navID = GetLong(file)`, :614) is pinned to exactly 4
    /// bytes (NAV-D1 / RULING 44) — this returns `i32`, **never**
    /// `core::ffi::c_long`.
    ///
    /// Source: `oracle/codemp/server/NPCNav/navigator.cpp:557-564`
    fn get_long(&self, cursor: &mut std::io::Cursor<&[u8]>) -> i32 {
        // NAV-D1: read exactly 4 bytes (retail Win32 ILP32 `long`), never 8.
        let mut b = [0u8; 4];
        let _ = cursor.read_exact(&mut b);
        i32::from_le_bytes(b)
    }

    /// Raven `protected void CNavigator::SetEdgeCost( int ID1, int ID2, int
    /// cost )` — pure `nodes`/`GetPosition`/`Distance`/`AddEdge` work,
    /// host-free; adds the edge bidirectionally to both endpoints
    /// (:778-779). Sole callee of
    /// [`clear_failed_edge`](Self::clear_failed_edge) (NAV-D4).
    ///
    /// Source: `oracle/codemp/server/NPCNav/navigator.cpp:757-779`
    fn set_edge_cost(&mut self, id1: i32, id2: i32, cost: i32) {
        if id1 == -1 || id2 == -1 {
            // Not valid nodes (came from the ClearAllFailedEdges init calls).
            return;
        }

        let mut cost = cost;
        if cost == -1 {
            // They want us to calc it.
            let pos1 = self.nodes[id1 as usize].get_position();
            let pos2 = self.nodes[id2 as usize].get_position();
            cost = distance(pos1, pos2) as i32;
        }

        // Set it (bidirectional, :778-779).
        self.nodes[id1 as usize].add_edge(id2, cost, EFLAG_NONE);
        self.nodes[id2 as usize].add_edge(id1, cost, EFLAG_NONE);
    }

    /// Raven `protected int CNavigator::GetEdgeCost( CNode *first, CNode
    /// *second )` — the trace form the public `GetEdgeCost(int,int)`
    /// (`get_edge_cost`) unconditionally delegates to (:2634);
    /// trace-dependent (3c surface). `first`/`second` are node ids, not
    /// pointers (NAV-D3, §B5: owned arena, node id == index).
    ///
    /// Source: `oracle/codemp/server/NPCNav/navigator.cpp:734-755`
    fn get_edge_cost_trace(
        &mut self,
        host: &mut impl EngineHost,
        first_id: i32,
        second_id: i32,
    ) -> i32 {
        // Setup the player size.
        let mins = [-8.0f32, -8.0, -8.0];
        let maxs = [8.0f32, 8.0, 8.0];

        // Setup the points.
        let start = self.nodes[first_id as usize].get_position();
        let end = self.nodes[second_id as usize].get_position();

        let mut trace = zeroed_trace();
        host.trace(
            &mut trace,
            &start,
            &mins,
            &maxs,
            &end,
            ENTITYNUM_NONE,
            MASK_SOLID,
            false,
            0,
            10,
        );

        if trace.fraction < 1.0 || trace.allsolid != 0 || trace.startsolid != 0 {
            return Q3_INFINITE;
        }

        // Connection successful, return the cost.
        distance(start, end) as i32
    }

    /// Raven `protected void CNavigator::AddNodeEdges( CNode *node, int
    /// addDist, edge_l &edgeList, bool *checkedNodes )` — id-indexed,
    /// host-free; `node` is a node id (NAV-D3), `edge_l` (`list<CEdge>`)
    /// becomes `&mut Vec<Edge>`, `bool *checkedNodes` becomes `&mut [bool]`.
    ///
    /// Source: `oracle/codemp/server/NPCNav/navigator.cpp:780-806`
    /// Zero-caller: Raven declares `AddNodeEdges` but no site calls it (the
    /// flood-fill inlines its own edge push), so it ports without a live
    /// caller (§20).
    #[allow(dead_code)]
    fn add_node_edges(
        &mut self,
        node_id: i32,
        add_dist: i32,
        edge_list: &mut Vec<Edge>,
        checked_nodes: &mut [bool],
    ) {
        let root_id = self.nodes[node_id as usize].get_id();
        let num_edges = self.nodes[node_id as usize].get_num_edges();

        // Add all edges.
        for i in 0..num_edges {
            let edge_i = self.nodes[node_id as usize].get_edge(i);

            // Make sure we don't add an old edge twice.
            if checked_nodes[edge_i as usize] {
                continue;
            }

            // Get the node.
            let next_id = self.nodes[edge_i as usize].get_id();

            // This node has now been checked.
            checked_nodes[next_id as usize] = true;

            // Add it to the list.
            let cost = add_dist + self.nodes[node_id as usize].get_edge_cost(i);
            edge_list.push(Edge::new(next_id, root_id, cost));
        }
    }

    /// Raven `protected void CNavigator::CalculatePath( CNode *node )` — the
    /// host-free inner priority-queue flood fill: seeds the frontier with
    /// each direct edge, then repeatedly pops the min-cost
    /// [`Edge`](super::Edge), assigns `node->AddRank(testNode->GetID(),
    /// curRank++)` in **pop order** (:853 — the tie-break is parity-visible,
    /// NAV-D3 / RULING 26), and pushes each unchecked neighbour at cumulative
    /// cost. `node` is a node id (NAV-D3).
    ///
    /// Source: `oracle/codemp/server/NPCNav/navigator.cpp:814-877`
    fn calculate_path(&mut self, node_id: i32) {
        let mut cur_rank = 0;

        let mut path_list = PriorityQueue::default();

        // Init the completion table (`new BYTE[size]` + `memset 0`).
        let mut checked = vec![false; self.nodes.len()];

        // Mark this node as checked.
        let this_id = self.nodes[node_id as usize].get_id();
        checked[this_id as usize] = true;
        self.nodes[node_id as usize].add_rank(this_id, cur_rank);
        cur_rank += 1;

        // Add all initial nodes.
        let num_edges = self.nodes[node_id as usize].get_num_edges();
        for i in 0..num_edges {
            let next_id = self.nodes[node_id as usize].get_edge(i);
            checked[next_id as usize] = true;
            let cost = self.nodes[node_id as usize].get_edge_cost(i);
            path_list.push(Edge::new(next_id, next_id, cost));
        }

        // Now flood fill all the others.
        while !path_list.empty() {
            let test = path_list.pop();

            let test_first = test.first;
            let test_id = self.nodes[test_first as usize].get_id();

            self.nodes[node_id as usize].add_rank(test_id, cur_rank);
            cur_rank += 1;

            // Add in all the new edges.
            let test_num_edges = self.nodes[test_first as usize].get_num_edges();
            for i in 0..test_num_edges {
                let add_id = self.nodes[test_first as usize].get_edge(i);

                if checked[add_id as usize] {
                    continue;
                }

                let new_dist = test.cost + self.nodes[test_first as usize].get_edge_cost(i);
                path_list.push(Edge::new(add_id, test.second, new_dist));

                checked[add_id as usize] = true;
            }
        }

        self.nodes[node_id as usize].remove_flag(NF_RECALC);
    }
}

/// Raven `STEPSIZE` (`bg_public.h:22`, `18`) — imported from `mp_qshared`
/// (NAV-D3 / RULING 39d). A thin wrapper so the `CheckFailedEdge` bound-box
/// math reads it by value without importing the symbol into the module's
/// const namespace twice.
#[inline]
fn stepsize() -> f32 {
    mp_qshared::common::mp::bg::stepsize::STEPSIZE
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `checked_node`/`set_checked_node` key math (`wayPoint*MAX_GENTITIES+ent`)
    /// and the `MAX_STORED_WAYPOINTS` range guard (navigator.cpp:1693-1719).
    /// Touches no sibling whose body may still be an unfilled stub.
    #[test]
    fn checked_node_key_and_range_guard() {
        let mut nav = Navigator::default();

        // Fresh miss reads CHECKED_NO.
        assert_eq!(nav.checked_node(3, 5), CHECKED_NO);

        // Round-trips through the flat `wayPoint*MAX_GENTITIES+ent` key.
        nav.set_checked_node(3, 5, CHECKED_PASSED);
        assert_eq!(nav.checked_node(3, 5), CHECKED_PASSED);
        // A different (waypoint, ent) with the same product would collide, but
        // distinct pairs below MAX_GENTITIES do not: (3,5) vs (2,5).
        assert_eq!(nav.checked_node(2, 5), CHECKED_NO);

        // Out-of-range waypoint is a no-op write and a CHECKED_NO read.
        nav.set_checked_node(MAX_STORED_WAYPOINTS, 0, CHECKED_FAILED);
        assert_eq!(nav.checked_node(MAX_STORED_WAYPOINTS, 0), CHECKED_NO);
        nav.set_checked_node(-1, 0, CHECKED_FAILED);
        assert_eq!(nav.checked_node(-1, 0), CHECKED_NO);
    }

    /// `edge_failed`'s two-phase `equal_range` first-match scan
    /// (navigator.cpp:1876-1898): `startID` then `endID` direction. Builds the
    /// lookup state by hand — no sibling calls.
    #[test]
    fn edge_failed_equal_range_first_match() {
        let mut nav = Navigator::default();

        // Slot 4 records the failed edge (10 -> 20); slot 7 records (20 -> 99).
        nav.failed_edges[4] = failedEdge_t {
            startID: 10,
            endID: 20,
            checkTime: 0,
            entID: 0,
        };
        nav.failed_edges[7] = failedEdge_t {
            startID: 20,
            endID: 99,
            checkTime: 0,
            entID: 0,
        };
        nav.edge_lookup.entry(10).or_default().push(4);
        nav.edge_lookup.entry(20).or_default().push(7);

        // Forward match on the startID side.
        assert_eq!(nav.edge_failed(10, 20), 4);
        // Reverse match falls through to the endID side (start=20 has endID 99,
        // not 10; but the endID-keyed pass finds slot 4's endID==20 vs start).
        assert_eq!(nav.edge_failed(20, 10), 4);
        // No such edge.
        assert_eq!(nav.edge_failed(10, 99), -1);
    }

    /// `clear_all_failed_edges` leaves every slot at the sentinel state
    /// (navigator.cpp:1867-1874): `set_edge_cost(-1,-1,-1)` early-returns on the
    /// -1 ids (no node access), then each slot is `WAYPOINT_NONE`/`ENTITYNUM_NONE`.
    #[test]
    fn clear_all_failed_edges_sentinels() {
        let mut nav = Navigator::default();
        nav.failed_edges[0] = failedEdge_t {
            startID: 3,
            endID: 4,
            checkTime: 123,
            entID: 5,
        };

        nav.clear_all_failed_edges();

        for e in &nav.failed_edges {
            assert_eq!(e.startID, WAYPOINT_NONE);
            assert_eq!(e.endID, WAYPOINT_NONE);
            assert_eq!(e.entID, ENTITYNUM_NONE);
            assert_eq!(e.checkTime, 0);
        }
    }
}
