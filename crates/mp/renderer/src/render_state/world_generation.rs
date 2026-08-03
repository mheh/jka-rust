//! `WorldGeneration` — one loaded world on its way to the render thread
//! (W2-F7).

use std::sync::Arc;

use crate::render_state::bmodel_table::BModelTable;
use crate::render_state::placeholders::WorldAsset;

/// What the render thread needs the moment the loaded world changes.
///
/// The sim puts one of these on the frame package when it loads a map, drops a
/// map, or restarts the video, and puts nothing there on every other frame. The
/// render thread reacts by uploading the geometry, resizing the walk marks and
/// re-seeding the view cluster.
///
/// The world and the brush-submodel rows travel together because one map load
/// produces both: `R_LoadSubmodels` registers the inline models while the BSP
/// loads, so a table built beside a different world would address the wrong
/// rows.
///
/// New construct, no Raven counterpart. The oracle's `tr.world` is a global
/// the whole renderer reads in place.
pub struct WorldGeneration {
    /// The loaded world, or `None` when the map dropped. The same `Arc` the
    /// published registry holds, so this costs a refcount, not a copy.
    pub world: Option<Arc<WorldAsset>>,
    /// The brush-submodel rows this world's load registered (W2-F8).
    pub bmodels: BModelTable,
}
