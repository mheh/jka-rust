//! `RenderAssetsSim` — the sim-thread owner of the published `RenderAssets`
//! and the light-style table (`R2-D9`/`R2-D5`).

use std::sync::Arc;

use mp_engine_qcommon::common::com_error;
use mp_engine_qcommon::qfiles::light_style_limits::MAX_LIGHT_STYLES;
use mp_qshared::shared::error_parm::errorParm_t;

use crate::render_state::light_style_table::LightStyleTable;
use crate::render_state::render_assets::RenderAssets;

/// Sim-thread-owned. `published` IS the master — there is no separate
/// mutable-then-copied staging struct (NB-1). Registration calls
/// `Arc::make_mut(&mut self.published)`, which mutates the existing allocation
/// in place when the render thread holds no other reference and clones once
/// when it does — ordinary copy-on-write, no locks; the result becomes visible
/// to the render thread (`RenderWorld::assets`) at the next frame boundary
/// (A9). `LightStyleTable` sits adjacent, not behind the `Arc` (A6/A9).
///
/// New construct, no single Raven counterpart: the oracle's `tr` registries
/// are globals mutated in place with no publish step (ruling 1).
pub struct RenderAssetsSim {
    pub published: Arc<RenderAssets>,
    pub light_styles: LightStyleTable,
}

impl RenderAssetsSim {
    /// Raven `RE_SetLightStyle` — mutates `self.light_styles.colors[style]` in
    /// place, **not** via `Arc::make_mut` (A6/A9). Out-param `int color` →
    /// typed `[u8; 4]` per §C7; `style: usize` closes the oracle's missing
    /// `style < 0` check by construction (§19), and the upper bound diverges
    /// through `com_error(ERR_FATAL, …)` exactly as the oracle does
    /// (`R2-D11`).
    ///
    /// Source: `oracle/codemp/renderer/tr_init.cpp:1438-1450`
    pub fn set_light_style(&mut self, style: usize, color: [u8; 4]) {
        if style >= MAX_LIGHT_STYLES {
            com_error(
                errorParm_t::ERR_FATAL,
                format!("RE_SetLightStyle: {} is out of range", style),
            );
        }
        self.light_styles.colors[style] = color;
    }

    /// Raven `RE_GetLightStyle` — reads `self.light_styles.colors[style]`;
    /// same bounds contract and typed-colour shape as `set_light_style`.
    ///
    /// Source: `oracle/codemp/renderer/tr_init.cpp:1427-1436`
    pub fn get_light_style(&self, style: usize) -> [u8; 4] {
        if style >= MAX_LIGHT_STYLES {
            com_error(
                errorParm_t::ERR_FATAL,
                format!("RE_GetLightStyle: {} is out of range", style),
            );
        }
        self.light_styles.colors[style]
    }
}
