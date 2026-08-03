//! `WorldWalkScratch` — the per-walk marks the BSP traversal used to stamp
//! into the world itself (W2-F4).

use crate::render_state::placeholders::WorldAsset;
use crate::render_state::walk_warnings::WalkWarnings;

/// The runtime marks Raven keeps inside `msurface_t`/`mnode_t`, held beside an
/// immutable world instead of inside it.
///
/// Raven stamps `msurface_t::viewCount`, `msurface_t::data->dlightBits` and
/// `mnode_t::visframe` during the world walk, which makes the loaded BSP
/// mutable per frame. The world crosses to the render thread behind an `Arc`,
/// so those three marks move here and the render thread owns them (user ruling
/// 2026-08-03, W2-F4). Every vector is indexed by the same flat index the
/// world uses: `surf_view_count` and `surf_dlight_bits` by the
/// `WorldAsset::surfaces` subscript, `node_visframe` by the `WorldAsset::nodes`
/// subscript.
///
/// Source: `oracle/codemp/renderer/tr_local.h:872-878` (`msurface_t`),
/// `oracle/codemp/renderer/tr_local.h:917-934` (`mnode_t`)
#[derive(Default)]
pub struct WorldWalkScratch {
    /// `msurface_t::viewCount` — Raven: if == `tr.viewCount`, already added.
    pub surf_view_count: Vec<i32>,
    /// `mnode_t::visframe` — Raven: node needs to be traversed if current.
    pub node_visframe: Vec<i32>,
    /// `srfSurfaceFace_t::dlightBits`, `srfGridMesh_t::dlightBits` and
    /// `srfTriangles_t::dlightBits`, which all three carry one per surface.
    pub surf_dlight_bits: Vec<i32>,
    /// `tr.viewCount` — the generation `surf_view_count` compares against.
    /// W2-F4 homes it beside the array it stamps rather than on the view
    /// state.
    ///
    /// Source: `oracle/codemp/renderer/tr_local.h:1315`
    pub view_count: i32,
    /// `tr.visCount` — Raven: incremented every time a new vis cluster is
    /// entered. The generation `node_visframe` compares against.
    ///
    /// Source: `oracle/codemp/renderer/tr_local.h:1316`
    pub vis_count: i32,

    /// The once-per-process print latches the walk shares. They ride here
    /// because the walk already threads this carrier end to end.
    pub warnings: WalkWarnings,
}

impl WorldWalkScratch {
    /// Sizes the mark arrays for a newly loaded world and clears every mark.
    ///
    /// Raven gets the same zeroed state from the `Hunk_Alloc` block a map load
    /// allocates. Call this whenever the loaded world changes.
    pub fn set_world(&mut self, world: &WorldAsset) {
        self.resize(world.surfaces.len(), world.nodes.len());
    }

    /// Sizes the mark arrays to `num_surfaces` and `num_nodes`, and clears
    /// every mark. The generation counters keep running, so a stale mark from
    /// the last world can never match.
    pub fn resize(&mut self, num_surfaces: usize, num_nodes: usize) {
        self.surf_view_count.clear();
        self.surf_view_count.resize(num_surfaces, 0);
        self.surf_dlight_bits.clear();
        self.surf_dlight_bits.resize(num_surfaces, 0);
        self.node_visframe.clear();
        self.node_visframe.resize(num_nodes, 0);
    }
}
