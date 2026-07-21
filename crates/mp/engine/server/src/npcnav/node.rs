//! Raven `CNode` — one graph node in the nav mesh.
//!
//! C++-track idiomatic reimplementation (porting-rules §F, NAV-D3): the Rust
//! name drops the bare `C` prefix (NAV-D3 / RULING 40 — `CNode` → `Node`).
//! Raven's `new`/`delete` node lifecycle (`CNode::Create`/`~CNode`) becomes an
//! owned, `Vec`-held value — no heap allocation, no destructor method (Rust
//! drop glue frees `edges`/`ranks` for free). `m_ranks` (a raw `int*` heap
//! array, `-1`-initialised by `InitRanks`) becomes an owned `ranks: Vec<i32>`.
//! `m_numEdges` (a redundant cache always in lock-step with `m_edges.size()`,
//! navigator.cpp:180 `m_numEdges++` on every real push) is **not** carried as
//! a separate field — `edges.len()` stands in for every `m_numEdges` read
//! (internals are free, porting-rules §A). `GetPosition`'s `VectorCopy` and
//! the sibling vec3 primitives are imported from `mp_qshared`, never
//! redeclared here (NAV-D3 / RULING 39d).
//!
//! `Node::save`/`Node::load` take a **shared in-memory cursor/byte-buffer**,
//! not `&mut impl EngineHost` — Raven's `Save`/`Load` share one open
//! `fileHandle_t` the *caller* (`CNavigator::Load`/`Save`) owns
//! (navigator.cpp:385,426); the single whole-file `fs_read_file`/
//! `fs_write_file` host call lives only at `Navigator::load`/`save`, which
//! loops these methods over that one cursor (Method-transcription table,
//! docs/subsystems/npcnav.md). The cursor's concrete Rust type is
//! deliberately unfrozen by the doc; this file uses a byte slice that
//! advances for reads and a `Vec<u8>` append target for writes.
//!
//! Class definition source: `oracle/codemp/server/NPCNav/navigator.h:70-126`
//! Method source: `oracle/codemp/server/NPCNav/navigator.cpp:104-470`

use mp_qshared::common::mp::game::Q3_INFINITE;
use mp_qshared::shared::{_VectorCopy, qboolean, vec3_t};

use super::NODE_HEADER_ID;

// --- Shared in-memory cursor helpers (internal, non-seam — porting-rules §A:
// internals are free) --------------------------------------------------

/// Reads a little-endian `u32` off the shared read cursor and advances it 4
/// bytes — the `Node::load`/`Node::save` analogue of Raven's shared
/// `fileHandle_t` `FS_Read`/`FS_Write` calls (navigator.cpp:385-470).
fn read_u32(cursor: &mut &[u8]) -> u32 {
    let (bytes, rest) = cursor.split_at(4);
    *cursor = rest;
    u32::from_le_bytes(bytes.try_into().unwrap())
}

/// Reads a little-endian `i32` off the shared read cursor and advances it 4
/// bytes.
pub(crate) fn read_i32(cursor: &mut &[u8]) -> i32 {
    read_u32(cursor) as i32
}

/// Reads a little-endian `f32` off the shared read cursor and advances it 4
/// bytes.
fn read_f32(cursor: &mut &[u8]) -> f32 {
    f32::from_bits(read_u32(cursor))
}

/// Reads a single byte off the shared read cursor and advances it 1 byte.
fn read_u8(cursor: &mut &[u8]) -> u8 {
    let (bytes, rest) = cursor.split_at(1);
    *cursor = rest;
    bytes[0]
}

/// Raven's `CNode`-nested `edge_t { int ID; int cost; BYTE flags; }` — one
/// node's outbound edge record.
///
/// Not the same type as [`crate::npcnav::Edge`] (`CEdge`), which is the
/// priority queue's generic `(node, root, cost)` triple — this is `CNode`'s
/// own per-edge storage, colocated here per porting-rules §21 (nested member
/// type stays with its owning class' file).
///
/// **On-disk shape (`CNode::Save`/`Load`, navigator.cpp:385-470):** Raven
/// writes/reads the whole `sizeof(edge_t)` in one `FS_Write`/`FS_Read` call
/// per edge (:406,:451) — under natural 4-byte struct alignment that is a
/// **12-byte** record (`ID` 4 + `cost` 4 + `flags` 1 + 3 bytes trailing
/// padding), **not** a packed 9-byte field-by-field encoding. `Node::save`/
/// `load` must emit/consume that same 12-byte whole-record shape (id, cost,
/// flags, 3 pad bytes) per edge to match the goldens `tools/npcnav-oracle`
/// dumps from the identical `sizeof(edge_t)` write.
///
/// Type definition source: `oracle/codemp/server/NPCNav/navigator.h:72-77`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NodeEdge {
    /// Raven `edge_t::ID` — the neighbor node's id.
    pub id: i32,
    /// Raven `edge_t::cost`.
    pub cost: i32,
    /// Raven `edge_t::flags` (`BYTE`) — `EFLAG_*` (`crate::npcnav`).
    pub flags: u8,
}

/// Raven `CNode` — one nav-graph node: position, flags, radius, id, its
/// outbound edges, and its per-target rank table.
///
/// `m_edges` (`vector<edge_t>`) is an owned `Vec<NodeEdge>`; `m_ranks` (a raw
/// `int*` heap array) is an owned `Vec<i32>`. `m_numEdges` is not carried
/// separately — see the module doc note.
///
/// Type definition source: `oracle/codemp/server/NPCNav/navigator.h:70-126`
#[derive(Debug, Clone)]
pub struct Node {
    /// Raven `m_position`.
    pub position: vec3_t,
    /// Raven `m_flags` (`NF_*`, `crate::npcnav`).
    pub flags: i32,
    /// Raven `m_radius`.
    pub radius: i32,
    /// Raven `m_ID`.
    pub id: i32,
    /// Raven `m_edges` (`edge_v`, a `vector<edge_t>`).
    pub edges: Vec<NodeEdge>,
    /// Raven `m_ranks` — one rank per node in the graph, `-1`-initialised by
    /// `InitRanks` (navigator.cpp:351-363).
    pub ranks: Vec<i32>,
}

impl Node {
    /// Raven `CNode::CNode()` (navigator.cpp:104-109) and the parameterless
    /// `CNode::Create(void)` (navigator.cpp:144-146, `return new CNode;`) —
    /// both construct a node with no meaningful field values set beyond the
    /// ctor's `m_numEdges = 0; m_radius = 0; m_ranks = NULL;`; folded into one
    /// Rust constructor per the doc's Method-transcription table (both call
    /// sites — the default-ish ctor and `Create(void)`'s only live caller,
    /// `CNavigator::Load`, navigator.cpp:635 — populate every field via
    /// `Load` immediately after). `~CNode` (navigator.cpp:111-117, `delete []
    /// m_ranks`) has no Rust counterpart — `Vec` drop glue frees `edges`/
    /// `ranks` for free.
    ///
    /// Source: `oracle/codemp/server/NPCNav/navigator.cpp:104-109,144-146`
    pub fn new() -> Self {
        // Raven's ctor only ever sets `m_numEdges = 0; m_radius = 0; m_ranks =
        // NULL;` — position/flags/ID are left at whatever `new CNode` happened
        // to leave them (uninitialised). Every live caller (`Load`, and
        // `Create(vec3_t,...)` immediately below) overwrites those fields
        // right after construction, so zero-initialising them here is
        // behavior-preserving.
        Node {
            position: [0.0, 0.0, 0.0],
            flags: 0,
            radius: 0,
            id: 0,
            edges: Vec::new(),
            ranks: Vec::new(),
        }
    }

    /// Raven `static CNode *CNode::Create(vec3_t position, int flags, int
    /// radius, int ID)` — the live-build factory (`CNavigator::AddRawPoint`,
    /// navigator.cpp:712).
    ///
    /// Source: `oracle/codemp/server/NPCNav/navigator.cpp:125-136`
    pub fn create(position: vec3_t, flags: i32, radius: i32, id: i32) -> Self {
        let mut node = Self::new();
        _VectorCopy(position, &mut node.position);
        node.flags = flags;
        node.id = id;
        node.radius = radius;
        node
    }

    /// Raven `CNode::AddEdge(int ID, int cost, int flags = EFLAG_NONE)` —
    /// dedups by neighbor id (updates `cost`/`flags` in place on a match,
    /// :163-167) or pushes a new `edge_t` (:172-180). Raven's default
    /// parameter (`flags = EFLAG_NONE`) has no Rust equivalent; callers pass
    /// `crate::npcnav::EFLAG_NONE` explicitly. `assert( m_numEdges < 9 )`
    /// (:182, release-elided under `NDEBUG`) ports as `debug_assert!` (D-6).
    ///
    /// Source: `oracle/codemp/server/NPCNav/navigator.cpp:155-183`
    pub fn add_edge(&mut self, id: i32, cost: i32, flags: i32) {
        // Raven only scans `m_edges` when `m_numEdges` is already nonzero
        // (:159) — an empty-vec scan below is the same no-op for the
        // first-edge case.
        for edge in self.edges.iter_mut() {
            if edge.id == id {
                // Found it — update in place (:165-167).
                edge.cost = cost;
                edge.flags = flags as u8;
                return;
            }
        }

        self.edges.push(NodeEdge {
            id,
            cost,
            flags: flags as u8,
        });

        // `assert( m_numEdges < 9 )` (:182, release-elided under `NDEBUG`) —
        // D-6.
        debug_assert!(self.edges.len() < 9);
    }

    /// Raven `int CNode::GetEdgeNumToNode(int ID)` — linear scan of `m_edges`
    /// for the edge whose `ID` matches, returning its index or `-1`.
    ///
    /// Source: `oracle/codemp/server/NPCNav/navigator.cpp:191-206`
    pub fn get_edge_num_to_node(&self, id: i32) -> i32 {
        for (count, edge) in self.edges.iter().enumerate() {
            if edge.id == id {
                return count as i32;
            }
        }
        -1
    }

    /// Raven `void CNode::AddRank(int ID, int rank)` — `m_ranks[ID] = rank`
    /// (`assert( m_ranks )`, release-elided → `debug_assert!`, D-6). Called
    /// from `CNavigator::CalculatePath`'s pop-order rank assignment
    /// (navigator.cpp:853).
    ///
    /// Source: `oracle/codemp/server/NPCNav/navigator.cpp:214-219`
    pub fn add_rank(&mut self, id: i32, rank: i32) {
        // `assert( m_ranks )` (release-elided, D-6) — the Rust analogue of a
        // non-NULL rank table is a non-empty `ranks` (allocated by
        // `init_ranks`).
        debug_assert!(!self.ranks.is_empty());
        self.ranks[id as usize] = rank;
    }

    /// Raven `void CNode::Draw(qboolean showRadius)` — the renderer calls
    /// (`CG_DrawNode`/`CG_DrawRadius`) are commented out in the shipped
    /// source (:229-235); the method's real, final behavior is an empty
    /// no-op (renderer stripped, §20). Ported as a callable no-op, not
    /// dropped — it still has a Rust symbol for any caller.
    ///
    /// Source: `oracle/codemp/server/NPCNav/navigator.cpp:227-236`
    pub fn draw(&self, show_radius: qboolean) {
        // Raven's body is entirely commented out (`CG_DrawNode`/`CG_DrawRadius`,
        // :230-234) — the renderer is stripped (§20); the real, final
        // behavior is an empty no-op.
        let _ = show_radius;
    }

    /// Raven `int CNode::GetEdge(int edgeNum)` — the neighbor node id at
    /// position `edgeNum` in `m_edges`, or `-1` on out-of-range/miss. Bound
    /// check is `edgeNum > m_numEdges`, not `>=` — preserved verbatim (D-2).
    ///
    /// Source: `oracle/codemp/server/NPCNav/navigator.cpp:244-263`
    pub fn get_edge(&self, edge_num: i32) -> i32 {
        // Bound check `edgeNum > m_numEdges`, not `>=` — preserved (D-2).
        if edge_num > self.edges.len() as i32 {
            return -1;
        }

        for (count, edge) in self.edges.iter().enumerate() {
            if count as i32 == edge_num {
                return edge.id;
            }
        }

        -1
    }

    /// Raven `int CNode::GetEdgeCost(int edgeNum)` — the edge's cost, or
    /// `Q3_INFINITE` (`mp_qshared`, NAV-D3 / RULING 39d) on out-of-range/miss
    /// (the comment `// return -1;` at :274,:289 marks Raven's own dead
    /// original value). Bound check `edgeNum > m_numEdges`, preserved (D-2).
    ///
    /// Source: `oracle/codemp/server/NPCNav/navigator.cpp:271-290`
    pub fn get_edge_cost(&self, edge_num: i32) -> i32 {
        // Bound check `edgeNum > m_numEdges`, preserved (D-2). `Q3_INFINITE`
        // is Raven's live fallback; the `// return -1;` comment at :274,:289
        // marks Raven's own dead original value.
        if edge_num > self.edges.len() as i32 {
            return Q3_INFINITE;
        }

        for (count, edge) in self.edges.iter().enumerate() {
            if count as i32 == edge_num {
                return edge.cost;
            }
        }

        Q3_INFINITE
    }

    /// Raven `BYTE CNode::GetEdgeFlags(int edgeNum)` — the edge's `flags`
    /// (`EFLAG_*`), or `0` on out-of-range/miss. Bound check `edgeNum >
    /// m_numEdges`, preserved (D-2).
    ///
    /// Source: `oracle/codemp/server/NPCNav/navigator.cpp:298-317`
    pub fn get_edge_flags(&self, edge_num: i32) -> u8 {
        // Bound check `edgeNum > m_numEdges`, preserved (D-2).
        if edge_num > self.edges.len() as i32 {
            return 0;
        }

        for (count, edge) in self.edges.iter().enumerate() {
            if count as i32 == edge_num {
                return edge.flags;
            }
        }

        0
    }

    /// Raven `void CNode::SetEdgeFlags(int edgeNum, int newFlags)` —
    /// overwrites the `edgeNum`-th edge's `flags`; no-op out-of-range. Bound
    /// check `edgeNum > m_numEdges`, preserved (D-2).
    ///
    /// Source: `oracle/codemp/server/NPCNav/navigator.cpp:325-344`
    pub fn set_edge_flags(&mut self, edge_num: i32, new_flags: i32) {
        // Bound check `edgeNum > m_numEdges`, preserved (D-2).
        if edge_num > self.edges.len() as i32 {
            return;
        }

        for (count, edge) in self.edges.iter_mut().enumerate() {
            if count as i32 == edge_num {
                edge.flags = new_flags as u8;
                return;
            }
        }
    }

    /// Raven `void CNode::InitRanks(int size)` — (re)allocates `m_ranks` to
    /// `size` entries, `-1`-filled (`memset( m_ranks, -1, sizeof(int)*size
    /// )` — a byte-pattern fill that happens to equal all-bits-set `-1` for
    /// `int`, so a `vec![-1; size]` reproduces it exactly).
    ///
    /// Source: `oracle/codemp/server/NPCNav/navigator.cpp:351-363`
    pub fn init_ranks(&mut self, size: i32) {
        // Raven `delete []`s the old array first if allocated, then
        // `new int[size]` + `memset(..., -1, ...)`. Reassigning `ranks`
        // reproduces both the free-then-reallocate lifecycle and the
        // `-1`-fill in one step (Rust drop glue frees the old `Vec`).
        self.ranks = vec![-1; size as usize];
    }

    /// Raven `int CNode::GetRank(int ID)` — `m_ranks[ID]` (`assert( m_ranks
    /// )`, release-elided → `debug_assert!`, D-6).
    ///
    /// Source: `oracle/codemp/server/NPCNav/navigator.cpp:371-376`
    pub fn get_rank(&self, id: i32) -> i32 {
        // `assert( m_ranks )` (release-elided, D-6).
        debug_assert!(!self.ranks.is_empty());
        self.ranks[id as usize]
    }

    /// Raven `int CNode::GetID( void ) const` (inline, `{ return m_ID; }`).
    ///
    /// Source: `oracle/codemp/server/NPCNav/navigator.h:94`
    pub fn get_id(&self) -> i32 {
        self.id
    }

    /// Raven `void CNode::GetPosition( vec3_t position ) const` (inline —
    /// `if ( position ) VectorCopy( m_position, position );`). The out-param
    /// + null-check becomes a return value (porting-rules §7); the null
    /// guard has no Rust counterpart (a `&self` return is never absent).
    /// Uses the `mp_qshared`-homed `_VectorCopy` at the implementation site
    /// (NAV-D3 / RULING 39d).
    ///
    /// Source: `oracle/codemp/server/NPCNav/navigator.h:95`
    pub fn get_position(&self) -> vec3_t {
        let mut out = [0.0, 0.0, 0.0];
        _VectorCopy(self.position, &mut out);
        out
    }

    /// Raven `int CNode::GetNumEdges( void ) const` (inline, `{ return
    /// m_numEdges; }`) — `edges.len()` stands in for `m_numEdges` (module
    /// doc note).
    ///
    /// Source: `oracle/codemp/server/NPCNav/navigator.h:97`
    pub fn get_num_edges(&self) -> i32 {
        self.edges.len() as i32
    }

    /// Raven `int CNode::GetRadius( void ) const` (inline, `{ return
    /// m_radius; }`).
    ///
    /// Source: `oracle/codemp/server/NPCNav/navigator.h:103`
    pub fn get_radius(&self) -> i32 {
        self.radius
    }

    /// Raven `int CNode::GetFlags( void ) const` (inline, `{ return m_flags;
    /// }`).
    ///
    /// Source: `oracle/codemp/server/NPCNav/navigator.h:108`
    pub fn get_flags(&self) -> i32 {
        self.flags
    }

    /// Raven `void CNode::AddFlag( int newFlag )` (inline, `{ m_flags |=
    /// newFlag; }`).
    ///
    /// Source: `oracle/codemp/server/NPCNav/navigator.h:109`
    pub fn add_flag(&mut self, new_flag: i32) {
        self.flags |= new_flag;
    }

    /// Raven `void CNode::RemoveFlag( int oldFlag )` (inline, `{ m_flags &=
    /// ~oldFlag; }`).
    ///
    /// Source: `oracle/codemp/server/NPCNav/navigator.h:110`
    pub fn remove_flag(&mut self, old_flag: i32) {
        self.flags &= !old_flag;
    }

    /// Raven `int CNode::Save(int numNodes, fileHandle_t file)` — writes the
    /// `NODE_HEADER_ID` header (as a 4-byte `u32`, NAV-D1 / RULING 44),
    /// position/flags/id/radius, the edge count and each `NodeEdge` as its
    /// full 12-byte on-disk record (see the `NodeEdge` doc), then `numNodes`
    /// and one rank per node. Takes the **shared** append buffer
    /// `CNavigator::Save` builds (Method-transcription table) — not
    /// `&mut impl EngineHost`; the single host `fs_write_file` call lives
    /// only at `Navigator::save`.
    ///
    /// Source: `oracle/codemp/server/NPCNav/navigator.cpp:385-418`
    pub fn save(&self, num_nodes: i32, out: &mut Vec<u8>) -> bool {
        // Header — a 4-byte `u32` (NAV-D1 / RULING 44), never `c_ulong`.
        out.extend_from_slice(&(NODE_HEADER_ID as u32).to_le_bytes());

        // Position/flags/id/radius.
        for component in self.position {
            out.extend_from_slice(&component.to_le_bytes());
        }
        out.extend_from_slice(&self.flags.to_le_bytes());
        out.extend_from_slice(&self.id.to_le_bytes());
        out.extend_from_slice(&self.radius.to_le_bytes());

        // Edges — `m_numEdges` (edges.len() stands in, module doc), then
        // each edge as the whole 12-byte `sizeof(edge_t)` record (`ID`,
        // `cost`, `flags`, 3 trailing pad bytes — see `NodeEdge` doc).
        out.extend_from_slice(&(self.edges.len() as i32).to_le_bytes());
        for edge in &self.edges {
            out.extend_from_slice(&edge.id.to_le_bytes());
            out.extend_from_slice(&edge.cost.to_le_bytes());
            out.push(edge.flags);
            out.extend_from_slice(&[0u8; 3]);
        }

        // Ranks — `numNodes` then one rank per node.
        out.extend_from_slice(&num_nodes.to_le_bytes());
        for i in 0..num_nodes {
            out.extend_from_slice(&self.ranks[i as usize].to_le_bytes());
        }

        true
    }

    /// Raven `int CNode::Load(int numNodes, fileHandle_t file)` — reads and
    /// validates the `NODE_HEADER_ID` header (as a 4-byte `u32`, NAV-D1 /
    /// RULING 44; `false` on mismatch), then position/flags/id/radius, the
    /// edge count and each `NodeEdge`'s full 12-byte on-disk record, then
    /// `numRanks` and calls `init_ranks` before filling `ranks`. Takes the
    /// **shared** read cursor `CNavigator::Load` advances (Method-
    /// transcription table) — not `&mut impl EngineHost`; the single host
    /// `fs_read_file` call lives only at `Navigator::load`.
    ///
    /// Source: `oracle/codemp/server/NPCNav/navigator.cpp:426-470`
    pub fn load(&mut self, num_nodes: i32, cursor: &mut &[u8]) -> bool {
        // Raven's `numNodes` parameter is unused by `CNode::Load` itself
        // (only `Save` loops it for the rank write) — the rank count here
        // comes from the file's own `numRanks` read below, exactly as Raven
        // does.
        let _ = num_nodes;

        // Header — a 4-byte `u32` (NAV-D1 / RULING 44), never `c_ulong`;
        // `false` on mismatch.
        let header = read_u32(cursor);
        if header != NODE_HEADER_ID as u32 {
            return false;
        }

        // Position/flags/id/radius.
        self.position = [read_f32(cursor), read_f32(cursor), read_f32(cursor)];
        self.flags = read_i32(cursor);
        self.id = read_i32(cursor);
        self.radius = read_i32(cursor);

        // Edges — `m_numEdges`, then that many whole 12-byte `edge_t`
        // records (`ID`, `cost`, `flags`, 3 trailing pad bytes — `NodeEdge`
        // doc).
        let num_edges = read_i32(cursor);
        self.edges = Vec::with_capacity(num_edges.max(0) as usize);
        for _ in 0..num_edges {
            let id = read_i32(cursor);
            let cost = read_i32(cursor);
            let flags = read_u8(cursor);
            let _pad = [read_u8(cursor), read_u8(cursor), read_u8(cursor)];
            self.edges.push(NodeEdge { id, cost, flags });
        }

        // Ranks — `numRanks`, then `init_ranks` allocates before filling.
        let num_ranks = read_i32(cursor);
        self.init_ranks(num_ranks);
        for i in 0..num_ranks {
            self.ranks[i as usize] = read_i32(cursor);
        }

        true
    }
}
