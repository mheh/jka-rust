//! `LightStyleTable` — the sim-owned light-style colour table (`R2-D5`).

use mp_engine_qcommon::qfiles::light_style_limits::MAX_LIGHT_STYLES;

/// `RE_SetLightStyle`/`RE_GetLightStyle`'s backing table (A6, extended by
/// A9) — sim-owned, **`RenderAssets`-ADJACENT, not inside its `Arc`**: mutated
/// in place at trap time via ordinary `&mut` access, not `Arc::make_mut`
/// copy-on-write, because it snapshots at scene-render marks
/// (`FrameEvent::RenderScene.light_styles`, A11) rather than publishing per
/// registration event. `[u8; 4]` replaces the oracle's packed `int color`,
/// matching the `*(DWORD *)styleColors[…]` reinterpretation its render-side
/// consumers already do (`oracle/codemp/renderer/tr_shade.cpp:1401`).
///
/// Source: `oracle/codemp/renderer/tr_local.h:1888`;
/// `oracle/codemp/renderer/tr_shade.cpp:26`
pub struct LightStyleTable {
    pub colors: [[u8; 4]; MAX_LIGHT_STYLES],
}
