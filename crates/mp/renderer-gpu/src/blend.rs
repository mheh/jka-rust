//! `blend` — decode Raven's `GLS_*` state bits into a `wgpu::BlendState`.
//!
//! `mp_renderer`'s shader parser stores each stage's blend mode as the same
//! packed `GLS_SRCBLEND_*`/`GLS_DSTBLEND_*` nibble pair the oracle fed to
//! `GL_State`'s `qglBlendFunc` call (`oracle/codemp/renderer/tr_backend.cpp`,
//! `GL_State`). Fixed-function GL had one blend func per draw; wgpu bakes the
//! blend state into the pipeline, so the decode below is what turns a stage's
//! state bits into a pipeline cache key.
//!
//! The GL blend func applied to the colour and alpha channels alike
//! (`glBlendFunc` sets both), so the decode mirrors the colour component into
//! the alpha component. The one exception is `SrcAlphaSaturated`, which WebGPU
//! forbids in the alpha component — it degrades to `One` there.

use wgpu::{BlendComponent, BlendFactor, BlendOperation, BlendState};

use mp_renderer::tr_shader::{
    GLS_DEPTHTEST_DISABLE, GLS_DSTBLEND_BITS, GLS_DSTBLEND_DST_ALPHA, GLS_DSTBLEND_ONE,
    GLS_DSTBLEND_ONE_MINUS_DST_ALPHA, GLS_DSTBLEND_ONE_MINUS_SRC_ALPHA,
    GLS_DSTBLEND_ONE_MINUS_SRC_COLOR, GLS_DSTBLEND_SRC_ALPHA, GLS_DSTBLEND_SRC_COLOR,
    GLS_DSTBLEND_ZERO, GLS_SRCBLEND_ALPHA_SATURATE, GLS_SRCBLEND_BITS, GLS_SRCBLEND_DST_ALPHA,
    GLS_SRCBLEND_DST_COLOR, GLS_SRCBLEND_ONE, GLS_SRCBLEND_ONE_MINUS_DST_ALPHA,
    GLS_SRCBLEND_ONE_MINUS_DST_COLOR, GLS_SRCBLEND_ONE_MINUS_SRC_ALPHA, GLS_SRCBLEND_SRC_ALPHA,
    GLS_SRCBLEND_ZERO,
};

/// `GLS_SRCBLEND_SRC_ALPHA | GLS_DSTBLEND_ONE_MINUS_SRC_ALPHA` decoded — the
/// standard 2D blend every `RB_SetGL2D` frame starts in.
///
/// Source: `oracle/codemp/renderer/tr_backend.cpp:1282-1284`
pub const ALPHA_BLEND: BlendState = BlendState {
    color: BlendComponent {
        src_factor: BlendFactor::SrcAlpha,
        dst_factor: BlendFactor::OneMinusSrcAlpha,
        operation: BlendOperation::Add,
    },
    alpha: BlendComponent {
        src_factor: BlendFactor::SrcAlpha,
        dst_factor: BlendFactor::OneMinusSrcAlpha,
        operation: BlendOperation::Add,
    },
};

/// The state bits `RB_SetGL2D` installs before any 2D command runs: depth test
/// off plus the standard alpha blend. The R4a executor stamps every
/// `DrawStretchPic` with these until per-shader stage state arrives.
///
/// Source: `oracle/codemp/renderer/tr_backend.cpp:1282-1284`
pub const GLS_2D_DEFAULT: u32 =
    (GLS_DEPTHTEST_DISABLE | GLS_SRCBLEND_SRC_ALPHA | GLS_DSTBLEND_ONE_MINUS_SRC_ALPHA) as u32;

/// Decodes a `GL_State` bit field's blend nibbles into the pipeline blend
/// state.
///
/// Zero in *both* nibbles is the oracle's "blending disabled" case
/// (`GL_State` calls `qglDisable(GL_BLEND)`), which wgpu spells as
/// `One`/`Zero` — an opaque overwrite. An unrecognised nibble (Raven never
/// emits one; a corrupt/extended shader could) falls back to [`ALPHA_BLEND`]
/// with a debug line rather than panicking, so a bad shader script degrades
/// visually instead of taking the frame loop down.
pub fn blend_state_from_gls(state_bits: u32) -> BlendState {
    let src_bits = (state_bits & GLS_SRCBLEND_BITS as u32) as i32;
    let dst_bits = (state_bits & GLS_DSTBLEND_BITS as u32) as i32;

    if src_bits == 0 && dst_bits == 0 {
        return OPAQUE;
    }

    let (Some(src_factor), Some(dst_factor)) = (src_factor(src_bits), dst_factor(dst_bits)) else {
        eprintln!(
            "mp_renderer_gpu: unrecognised GLS blend bits {state_bits:#010x} \
             (src {src_bits:#x} / dst {dst_bits:#x}); falling back to alpha blend"
        );
        return ALPHA_BLEND;
    };

    let color = BlendComponent {
        src_factor,
        dst_factor,
        operation: BlendOperation::Add,
    };
    BlendState {
        color,
        alpha: BlendComponent {
            // WebGPU rejects `SrcAlphaSaturated` in the alpha component; GL
            // treated it as 1 there, which is exactly `One`.
            src_factor: match src_factor {
                BlendFactor::SrcAlphaSaturated => BlendFactor::One,
                other => other,
            },
            dst_factor,
            operation: BlendOperation::Add,
        },
    }
}

/// Blending disabled — `GL_State`'s `qglDisable(GL_BLEND)` path. The PBR
/// backend reads this to route an opaque stage into the lit path and pass a
/// blended stage through unchanged.
pub const OPAQUE: BlendState = BlendState {
    color: BlendComponent {
        src_factor: BlendFactor::One,
        dst_factor: BlendFactor::Zero,
        operation: BlendOperation::Add,
    },
    alpha: BlendComponent {
        src_factor: BlendFactor::One,
        dst_factor: BlendFactor::Zero,
        operation: BlendOperation::Add,
    },
};

/// `GLS_SRCBLEND_*` -> `BlendFactor`; `None` for a nibble Raven never emits.
fn src_factor(src_bits: i32) -> Option<BlendFactor> {
    let factor = match src_bits {
        GLS_SRCBLEND_ZERO => BlendFactor::Zero,
        GLS_SRCBLEND_ONE => BlendFactor::One,
        GLS_SRCBLEND_DST_COLOR => BlendFactor::Dst,
        GLS_SRCBLEND_ONE_MINUS_DST_COLOR => BlendFactor::OneMinusDst,
        GLS_SRCBLEND_SRC_ALPHA => BlendFactor::SrcAlpha,
        GLS_SRCBLEND_ONE_MINUS_SRC_ALPHA => BlendFactor::OneMinusSrcAlpha,
        GLS_SRCBLEND_DST_ALPHA => BlendFactor::DstAlpha,
        GLS_SRCBLEND_ONE_MINUS_DST_ALPHA => BlendFactor::OneMinusDstAlpha,
        GLS_SRCBLEND_ALPHA_SATURATE => BlendFactor::SrcAlphaSaturated,
        _ => return None,
    };
    Some(factor)
}

/// `GLS_DSTBLEND_*` -> `BlendFactor`; `None` for a nibble Raven never emits.
fn dst_factor(dst_bits: i32) -> Option<BlendFactor> {
    let factor = match dst_bits {
        GLS_DSTBLEND_ZERO => BlendFactor::Zero,
        GLS_DSTBLEND_ONE => BlendFactor::One,
        GLS_DSTBLEND_SRC_COLOR => BlendFactor::Src,
        GLS_DSTBLEND_ONE_MINUS_SRC_COLOR => BlendFactor::OneMinusSrc,
        GLS_DSTBLEND_SRC_ALPHA => BlendFactor::SrcAlpha,
        GLS_DSTBLEND_ONE_MINUS_SRC_ALPHA => BlendFactor::OneMinusSrcAlpha,
        GLS_DSTBLEND_DST_ALPHA => BlendFactor::DstAlpha,
        GLS_DSTBLEND_ONE_MINUS_DST_ALPHA => BlendFactor::OneMinusDstAlpha,
        _ => return None,
    };
    Some(factor)
}

#[cfg(test)]
mod tests {
    use super::*;
    use mp_renderer::tr_shader::{GLS_DEPTHMASK_TRUE, GLS_DEPTHTEST_DISABLE};

    #[test]
    fn standard_2d_bits_decode_to_alpha_blend() {
        assert_eq!(blend_state_from_gls(GLS_2D_DEFAULT), ALPHA_BLEND);
    }

    #[test]
    fn non_blend_bits_are_ignored() {
        // `RB_SetGL2D` sets the depth-test bit alongside the blend nibbles; it
        // must not perturb the decode.
        let bits = GLS_2D_DEFAULT | GLS_DEPTHTEST_DISABLE as u32 | GLS_DEPTHMASK_TRUE as u32;
        assert_eq!(blend_state_from_gls(bits), ALPHA_BLEND);
    }

    #[test]
    fn additive_bits_decode_to_one_one() {
        let bits = (GLS_SRCBLEND_ONE | GLS_DSTBLEND_ONE) as u32;
        let decoded = blend_state_from_gls(bits);
        assert_eq!(decoded.color.src_factor, BlendFactor::One);
        assert_eq!(decoded.color.dst_factor, BlendFactor::One);
        assert_eq!(decoded.alpha.src_factor, BlendFactor::One);
        assert_eq!(decoded.alpha.dst_factor, BlendFactor::One);
        assert_eq!(decoded.color.operation, BlendOperation::Add);
    }

    #[test]
    fn empty_blend_nibbles_decode_to_opaque() {
        assert_eq!(blend_state_from_gls(0), OPAQUE);
        // `GLS_DEFAULT` is depth-mask only — still an opaque overwrite.
        assert_eq!(blend_state_from_gls(GLS_DEPTHMASK_TRUE as u32), OPAQUE);
    }

    #[test]
    fn explicit_one_zero_is_opaque_too() {
        let bits = (GLS_SRCBLEND_ONE | GLS_DSTBLEND_ZERO) as u32;
        assert_eq!(blend_state_from_gls(bits), OPAQUE);
    }

    #[test]
    fn filter_blend_decodes_to_dst_color_zero() {
        // The `GL_DST_COLOR`/`GL_ZERO` "filter" mode lightmap stages use.
        let bits = (GLS_SRCBLEND_DST_COLOR | GLS_DSTBLEND_ZERO) as u32;
        let decoded = blend_state_from_gls(bits);
        assert_eq!(decoded.color.src_factor, BlendFactor::Dst);
        assert_eq!(decoded.color.dst_factor, BlendFactor::Zero);
    }

    #[test]
    fn alpha_saturate_degrades_to_one_in_the_alpha_component() {
        let bits = (GLS_SRCBLEND_ALPHA_SATURATE | GLS_DSTBLEND_ONE) as u32;
        let decoded = blend_state_from_gls(bits);
        assert_eq!(decoded.color.src_factor, BlendFactor::SrcAlphaSaturated);
        assert_eq!(decoded.alpha.src_factor, BlendFactor::One);
    }

    #[test]
    fn every_raven_nibble_pair_decodes() {
        let srcs = [
            GLS_SRCBLEND_ZERO,
            GLS_SRCBLEND_ONE,
            GLS_SRCBLEND_DST_COLOR,
            GLS_SRCBLEND_ONE_MINUS_DST_COLOR,
            GLS_SRCBLEND_SRC_ALPHA,
            GLS_SRCBLEND_ONE_MINUS_SRC_ALPHA,
            GLS_SRCBLEND_DST_ALPHA,
            GLS_SRCBLEND_ONE_MINUS_DST_ALPHA,
            GLS_SRCBLEND_ALPHA_SATURATE,
        ];
        let dsts = [
            GLS_DSTBLEND_ZERO,
            GLS_DSTBLEND_ONE,
            GLS_DSTBLEND_SRC_COLOR,
            GLS_DSTBLEND_ONE_MINUS_SRC_COLOR,
            GLS_DSTBLEND_SRC_ALPHA,
            GLS_DSTBLEND_ONE_MINUS_SRC_ALPHA,
            GLS_DSTBLEND_DST_ALPHA,
            GLS_DSTBLEND_ONE_MINUS_DST_ALPHA,
        ];
        for src in srcs {
            for dst in dsts {
                let bits = (src | dst) as u32;
                assert!(src_factor(src).is_some(), "src nibble {src:#x}");
                assert!(dst_factor(dst).is_some(), "dst nibble {dst:#x}");
                // Only the ONE/ZERO pair may land on the opaque state.
                let decoded = blend_state_from_gls(bits);
                if src != GLS_SRCBLEND_ONE || dst != GLS_DSTBLEND_ZERO {
                    assert_ne!(decoded, OPAQUE, "{src:#x}/{dst:#x}");
                }
            }
        }
    }

    #[test]
    fn unknown_nibbles_fall_back_to_alpha_blend() {
        // 0xa..0xf are unused in both fields.
        assert_eq!(blend_state_from_gls(0x0000_000a), ALPHA_BLEND);
        assert_eq!(blend_state_from_gls(0x0000_00f2), ALPHA_BLEND);
        assert!(src_factor(0x0000_000f).is_none());
        assert!(dst_factor(0x0000_00a0).is_none());
    }
}
