//! Raven `CEdge` — a generic `{first, second, cost}` triple used by the
//! `PriorityQueue`'s owned `Vec<Edge>` heap (bare `C` prefix dropped per
//! NAV-D3 / RULING 40: `CEdge` -> `Edge`). Also constructed directly in
//! `Navigator::calculate_path`/`calculate_paths` when flooding edges into the
//! path list (navigator.cpp:804,838,865).
//!
//! Type definition source: `oracle/codemp/server/NPCNav/navigator.h:44-61`

/// Raven `CEdge`.
///
/// Type definition source: `oracle/codemp/server/NPCNav/navigator.h:50-61`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Edge {
    pub first: i32,
    pub second: i32,
    pub cost: i32,
}

impl Edge {
    /// Raven `CEdge::CEdge(int first, int second, int cost)` — the only ctor
    /// with a live caller (queue nodes `new CEdge(a,b,c)`; the priority-queue
    /// flood in `CalculatePath`/`CalculatePaths`).
    ///
    /// Source: `oracle/codemp/server/NPCNav/navigator.cpp:87-92`
    pub fn new(first: i32, second: i32, cost: i32) -> Self {
        Edge {
            first,
            second,
            cost,
        }
    }
}

// Raven `CEdge::CEdge(void)` (navigator.cpp:82-85) constructs a *discarded
// temporary* `CEdge(-1,-1,-1)` and never assigns `this->m_first/second/cost`
// — UB, and dead: no live caller (queue nodes use the 3-arg ctor; graph edges
// use the inner `edge_t`, ported as `NodeEdge` in `node.rs`). Per the frozen
// npcnav doc's Divergence D-1, `Edge` is ported without a meaning-bearing
// `Default` impl — not stubbed here; `Edge::new(-1, -1, -1)` is the author's
// evident (never-realised) intent if one is ever required.
// Source: `oracle/codemp/server/NPCNav/navigator.cpp:82-85`

// Raven `CEdge::~CEdge(void)` (navigator.cpp:94-96) is an empty destructor —
// it folds away under Rust's automatic `Drop`; not ported as a method.
// Source: `oracle/codemp/server/NPCNav/navigator.cpp:94-96`
