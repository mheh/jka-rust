//! `CPrimitiveTemplate` parsing, one method per `Parse*` call that turns a GP2
//! group into template field values.
//!
//! A valueless pair (`key []`) makes Raven's `GetTopValue()` return null, and
//! Raven passes that null straight into `sscanf`/`stricmp` (a null deref, UB).
//! Every dispatcher here reads `pair.top_value().unwrap_or("")` instead, so a
//! valueless pair yields a plain parse failure (porting-rules §19).
//!
//! Source: `oracle/codemp/client/FxTemplate.cpp:168-2386`

#![allow(non_camel_case_types, non_snake_case)]

use mp_engine_qcommon::gp2::gp_group::GpGroup;
use mp_engine_qcommon::gp2::gp_value::GpValue;
use native_math::vector::vec3_t;

use crate::fx::cprimitive_template::CPrimitiveTemplate;
use crate::fx::emat_impact_effect::EMatImpactEffect;
use crate::fx::fx_flags::{
    FX_ACCEL_IS_ABSOLUTE, FX_AFFECTED_BY_WIND, FX_ALPHA_SHIFT, FX_APPLY_PHYSICS, FX_ATTACHED_MODEL,
    FX_AXIS_FROM_SPHERE, FX_CHEAP_ORG2_CALC, FX_CHEAP_ORG_CALC, FX_CLAMP, FX_DEATH_RUNS_FX,
    FX_DEPTH_HACK, FX_EMIT_FX, FX_EVEN_DISTRIBUTION, FX_EXPENSIVE_PHYSICS, FX_GHOUL2_DECALS,
    FX_GHOUL2_TRACE, FX_IMPACT_RUNS_FX, FX_KILL_ON_IMPACT, FX_LENGTH_SHIFT, FX_LINEAR,
    FX_LOCALIZED_FLASH, FX_NONLINEAR, FX_ORG2_FROM_TRACE, FX_ORG2_IS_OFFSET, FX_ORG_ON_CYLINDER,
    FX_ORG_ON_SPHERE, FX_PAPER_PHYSICS, FX_PLAYER_VIEW, FX_RAND, FX_RAND_ROT_AROUND_FWD,
    FX_RELATIVE, FX_RGB_COMPONENT_INTERP, FX_RGB_SHIFT, FX_SET_SHADER_TIME, FX_SIZE2_SHIFT,
    FX_SIZE_SHIFT, FX_TRACE_IMPACT_FX, FX_USE_ALPHA, FX_USE_BBOX, FX_VEL_IS_ABSOLUTE, FX_WAVE,
};
use crate::fx::fx_host::FxHost;
use crate::fx::fx_scheduler::fx_register_effect;
use crate::fx::fx_system::FxSystem;

/// Scan the longest float prefix of one whitespace-delimited token.
///
/// `str::parse` rejects a token like `12abc`, but Raven's `sscanf("%f")`
/// converts its `12` prefix and stops. This walks the token's own float
/// grammar (sign, digits, `.`, exponent) and parses only the longest prefix
/// that grammar accepts, so it reproduces the C library's behavior.
fn scan_float_prefix(token: &str) -> Option<f32> {
    let bytes = token.as_bytes();
    let mut end = 0usize;
    let mut i = 0usize;

    if i < bytes.len() && (bytes[i] == b'+' || bytes[i] == b'-') {
        i += 1;
    }

    let mut saw_digit = false;
    while i < bytes.len() && bytes[i].is_ascii_digit() {
        i += 1;
        saw_digit = true;
    }
    if saw_digit {
        end = i;
    }

    if i < bytes.len() && bytes[i] == b'.' {
        let dot = i;
        i += 1;
        let mut saw_frac_digit = false;
        while i < bytes.len() && bytes[i].is_ascii_digit() {
            i += 1;
            saw_frac_digit = true;
        }
        if saw_digit || saw_frac_digit {
            end = i;
        } else {
            i = dot;
        }
    }

    if end == 0 {
        return None;
    }

    // An exponent suffix only counts if it has at least one digit after it.
    if i < bytes.len() && (bytes[i] == b'e' || bytes[i] == b'E') {
        let mut j = i + 1;
        if j < bytes.len() && (bytes[j] == b'+' || bytes[j] == b'-') {
            j += 1;
        }
        let exp_digits_start = j;
        while j < bytes.len() && bytes[j].is_ascii_digit() {
            j += 1;
        }
        if j > exp_digits_start {
            end = j;
        }
    }

    token[..end].parse::<f32>().ok()
}

/// Raven `sscanf(val, "%f %f", min, max)`, up to two whitespace-separated floats.
///
/// Zero parsed fields is a failure. One field copies min into max.
///
/// Source: `oracle/codemp/client/FxTemplate.cpp:181-202`
fn ParseFloat(val: &str) -> Option<(f32, f32)> {
    let mut tokens = val.split_whitespace();
    let min = match tokens.next().and_then(scan_float_prefix) {
        Some(v) => v,
        None => return None,
    };
    let max = match tokens.next().and_then(scan_float_prefix) {
        Some(v) => v,
        None => min,
    };
    Some((min, max))
}

/// Raven `sscanf(val, "%f %f %f   %f %f %f", ...)`, up to six floats forming
/// a min vector and a max vector.
///
/// Fewer than 3 fields, exactly 4, or exactly 5 fields is a failure. Exactly
/// 3 copies min into max.
///
/// Source: `oracle/codemp/client/FxTemplate.cpp:218-240`
fn ParseVector(val: &str) -> Option<(vec3_t, vec3_t)> {
    let mut tokens = val.split_whitespace();
    let mut fields = [0.0f32; 6];
    let mut count = 0usize;
    for slot in fields.iter_mut() {
        match tokens.next().and_then(scan_float_prefix) {
            Some(v) => {
                *slot = v;
                count += 1;
            }
            None => break,
        }
    }

    if count < 3 || count == 4 || count == 5 {
        return None;
    }

    let min = [fields[0], fields[1], fields[2]];
    let max = if count == 3 {
        min
    } else {
        [fields[3], fields[4], fields[5]]
    };
    Some((min, max))
}

/// Raven `sscanf(val, "%s %s %s %s", flag[0..4])` into `char flag[4][32]`.
///
/// A token past 31 characters overruns Raven's stack buffer (undefined
/// behavior); the port truncates at 31 characters instead (porting-rules §19).
/// A bad flag token keeps `ok` false but does not stop the scan, matching
/// Raven's fall-through loop.
///
/// Source: `oracle/codemp/client/FxTemplate.cpp:255-306`
fn ParseGroupFlags(val: &str) -> (i32, bool) {
    let mut flags = 0i32;
    let mut ok = true;

    for token in val.split_whitespace().take(4) {
        let token = if token.len() > 31 {
            &token[..31]
        } else {
            token
        };

        if token.eq_ignore_ascii_case("linear") {
            flags |= FX_LINEAR;
        } else if token.eq_ignore_ascii_case("nonlinear") {
            flags |= FX_NONLINEAR;
        } else if token.eq_ignore_ascii_case("wave") {
            flags |= FX_WAVE;
        } else if token.eq_ignore_ascii_case("random") {
            flags |= FX_RAND;
        } else if token.eq_ignore_ascii_case("clamp") {
            flags |= FX_CLAMP;
        } else {
            // We have badness going on, but continue in case a later field is valid.
            ok = false;
        }
    }

    (flags, ok)
}

impl CPrimitiveTemplate {
    /// Reads a min bounding box field in vector format.
    ///
    /// Source: `oracle/codemp/client/FxTemplate.cpp:318-332`
    pub fn ParseMin(&mut self, val: &str) -> bool {
        match ParseVector(val) {
            Some((min, _)) => {
                self.mMin = min;
                // A min bound implies physics and a bounding box.
                self.mFlags |= FX_USE_BBOX | FX_APPLY_PHYSICS;
                true
            }
            None => false,
        }
    }

    /// Reads a max bounding box field in vector format.
    ///
    /// Source: `oracle/codemp/client/FxTemplate.cpp:344-358`
    pub fn ParseMax(&mut self, val: &str) -> bool {
        match ParseVector(val) {
            Some((max, _)) => {
                self.mMax = max;
                // A max bound implies physics and a bounding box.
                self.mFlags |= FX_USE_BBOX | FX_APPLY_PHYSICS;
                true
            }
            None => false,
        }
    }

    /// Reads a ranged life value.
    ///
    /// Source: `oracle/codemp/client/FxTemplate.cpp:370-381`
    pub fn ParseLife(&mut self, val: &str) -> bool {
        match ParseFloat(val) {
            Some((min, max)) => {
                self.mLife.SetRange(min, max);
                true
            }
            None => false,
        }
    }

    /// Reads a ranged delay value.
    ///
    /// Source: `oracle/codemp/client/FxTemplate.cpp:393-404`
    pub fn ParseDelay(&mut self, val: &str) -> bool {
        match ParseFloat(val) {
            Some((min, max)) => {
                self.mSpawnDelay.SetRange(min, max);
                true
            }
            None => false,
        }
    }

    /// Reads a ranged count value.
    ///
    /// Source: `oracle/codemp/client/FxTemplate.cpp:416-427`
    pub fn ParseCount(&mut self, val: &str) -> bool {
        match ParseFloat(val) {
            Some((min, max)) => {
                self.mSpawnCount.SetRange(min, max);
                true
            }
            None => false,
        }
    }

    /// Reads a ranged elasticity value.
    ///
    /// Source: `oracle/codemp/client/FxTemplate.cpp:439-454`
    pub fn ParseElasticity(&mut self, val: &str) -> bool {
        match ParseFloat(val) {
            Some((min, max)) => {
                self.mElasticity.SetRange(min, max);
                // Elasticity implies physics, but not a bounding box unless min/max are set.
                self.mFlags |= FX_APPLY_PHYSICS;
                true
            }
            None => false,
        }
    }

    /// Reads an origin field in vector format.
    ///
    /// Source: `oracle/codemp/client/FxTemplate.cpp:466-479`
    pub fn ParseOrigin1(&mut self, val: &str) -> bool {
        match ParseVector(val) {
            Some((min, max)) => {
                self.mOrigin1X.SetRange(min[0], max[0]);
                self.mOrigin1Y.SetRange(min[1], max[1]);
                self.mOrigin1Z.SetRange(min[2], max[2]);
                true
            }
            None => false,
        }
    }

    /// Reads an origin field in vector format.
    ///
    /// Source: `oracle/codemp/client/FxTemplate.cpp:491-504`
    pub fn ParseOrigin2(&mut self, val: &str) -> bool {
        match ParseVector(val) {
            Some((min, max)) => {
                self.mOrigin2X.SetRange(min[0], max[0]);
                self.mOrigin2Y.SetRange(min[1], max[1]);
                self.mOrigin2Z.SetRange(min[2], max[2]);
                true
            }
            None => false,
        }
    }

    /// Reads a ranged radius value.
    ///
    /// Source: `oracle/codemp/client/FxTemplate.cpp:516-527`
    pub fn ParseRadius(&mut self, val: &str) -> bool {
        match ParseFloat(val) {
            Some((min, max)) => {
                self.mRadius.SetRange(min, max);
                true
            }
            None => false,
        }
    }

    /// Reads a ranged height value.
    ///
    /// Source: `oracle/codemp/client/FxTemplate.cpp:539-550`
    pub fn ParseHeight(&mut self, val: &str) -> bool {
        match ParseFloat(val) {
            Some((min, max)) => {
                self.mHeight.SetRange(min, max);
                true
            }
            None => false,
        }
    }

    /// Reads a ranged wind modifier value.
    ///
    /// Source: `oracle/codemp/client/FxTemplate.cpp:562-573`
    pub fn ParseWindModifier(&mut self, val: &str) -> bool {
        match ParseFloat(val) {
            Some((min, max)) => {
                self.mWindModifier.SetRange(min, max);
                true
            }
            None => false,
        }
    }

    /// Reads a ranged rotation value.
    ///
    /// Source: `oracle/codemp/client/FxTemplate.cpp:585-596`
    pub fn ParseRotation(&mut self, val: &str) -> bool {
        match ParseFloat(val) {
            Some((min, max)) => {
                self.mRotation.SetRange(min, max);
                true
            }
            None => false,
        }
    }

    /// Reads a ranged rotationDelta value.
    ///
    /// Source: `oracle/codemp/client/FxTemplate.cpp:608-619`
    pub fn ParseRotationDelta(&mut self, val: &str) -> bool {
        match ParseFloat(val) {
            Some((min, max)) => {
                self.mRotationDelta.SetRange(min, max);
                true
            }
            None => false,
        }
    }

    /// Reads an angle field in vector format.
    ///
    /// Source: `oracle/codemp/client/FxTemplate.cpp:631-644`
    pub fn ParseAngle(&mut self, val: &str) -> bool {
        match ParseVector(val) {
            Some((min, max)) => {
                self.mAngle1.SetRange(min[0], max[0]);
                self.mAngle2.SetRange(min[1], max[1]);
                self.mAngle3.SetRange(min[2], max[2]);
                true
            }
            None => false,
        }
    }

    /// Reads an angleDelta field in vector format.
    ///
    /// Source: `oracle/codemp/client/FxTemplate.cpp:656-669`
    pub fn ParseAngleDelta(&mut self, val: &str) -> bool {
        match ParseVector(val) {
            Some((min, max)) => {
                self.mAngle1Delta.SetRange(min[0], max[0]);
                self.mAngle2Delta.SetRange(min[1], max[1]);
                self.mAngle3Delta.SetRange(min[2], max[2]);
                true
            }
            None => false,
        }
    }

    /// Reads a velocity field in vector format.
    ///
    /// Source: `oracle/codemp/client/FxTemplate.cpp:681-694`
    pub fn ParseVelocity(&mut self, val: &str) -> bool {
        match ParseVector(val) {
            Some((min, max)) => {
                self.mVelX.SetRange(min[0], max[0]);
                self.mVelY.SetRange(min[1], max[1]);
                self.mVelZ.SetRange(min[2], max[2]);
                true
            }
            None => false,
        }
    }

    /// Reads the primitive-wide flags, not specific to any single group.
    ///
    /// A bad flag token keeps `ok` false but does not stop the scan.
    ///
    /// Source: `oracle/codemp/client/FxTemplate.cpp:707-799`
    pub fn ParseFlags(&mut self, val: &str) -> bool {
        let mut ok = true;

        for token in val.split_whitespace().take(7) {
            let token = if token.len() > 31 {
                &token[..31]
            } else {
                token
            };

            if token.eq_ignore_ascii_case("useModel") {
                self.mFlags |= FX_ATTACHED_MODEL;
            } else if token.eq_ignore_ascii_case("useBBox") {
                self.mFlags |= FX_USE_BBOX;
            } else if token.eq_ignore_ascii_case("usePhysics") {
                self.mFlags |= FX_APPLY_PHYSICS;
            } else if token.eq_ignore_ascii_case("expensivePhysics") {
                self.mFlags |= FX_EXPENSIVE_PHYSICS;
            } else if token.eq_ignore_ascii_case("ghoul2Collision") {
                self.mFlags |= FX_GHOUL2_TRACE | FX_APPLY_PHYSICS | FX_EXPENSIVE_PHYSICS;
            } else if token.eq_ignore_ascii_case("ghoul2Decals") {
                self.mFlags |= FX_GHOUL2_DECALS;
            } else if token.eq_ignore_ascii_case("impactKills") {
                self.mFlags |= FX_KILL_ON_IMPACT;
            } else if token.eq_ignore_ascii_case("impactFx") {
                self.mFlags |= FX_IMPACT_RUNS_FX;
            } else if token.eq_ignore_ascii_case("deathFx") {
                self.mFlags |= FX_DEATH_RUNS_FX;
            } else if token.eq_ignore_ascii_case("useAlpha") {
                self.mFlags |= FX_USE_ALPHA;
            } else if token.eq_ignore_ascii_case("emitFx") {
                self.mFlags |= FX_EMIT_FX;
            } else if token.eq_ignore_ascii_case("depthHack") {
                self.mFlags |= FX_DEPTH_HACK;
            } else if token.eq_ignore_ascii_case("relative") {
                self.mFlags |= FX_RELATIVE;
            } else if token.eq_ignore_ascii_case("setShaderTime") {
                self.mFlags |= FX_SET_SHADER_TIME;
            } else if token.eq_ignore_ascii_case("paperPhysics") {
                // Shared flag: use with a cylinder only, or expect evilness.
                self.mFlags |= FX_PAPER_PHYSICS;
            } else if token.eq_ignore_ascii_case("localizedFlash") {
                // Shared flag: use with a cylinder only, or expect evilness.
                self.mFlags |= FX_LOCALIZED_FLASH;
            } else if token.eq_ignore_ascii_case("playerView") {
                // Shared flag: use with a cylinder only, or expect evilness.
                self.mFlags |= FX_PLAYER_VIEW;
            } else {
                // We have badness going on, but continue in case a later field is valid.
                ok = false;
            }
        }

        ok
    }

    /// Reads the spawn flags. These steer spawning only and never reach a primitive.
    ///
    /// A bad flag token keeps `ok` false but does not stop the scan.
    ///
    /// Source: `oracle/codemp/client/FxTemplate.cpp:812-890`
    pub fn ParseSpawnFlags(&mut self, val: &str) -> bool {
        let mut ok = true;

        for token in val.split_whitespace().take(7) {
            let token = if token.len() > 31 {
                &token[..31]
            } else {
                token
            };

            if token.eq_ignore_ascii_case("org2fromTrace") {
                self.mSpawnFlags |= FX_ORG2_FROM_TRACE;
            } else if token.eq_ignore_ascii_case("traceImpactFx") {
                self.mSpawnFlags |= FX_TRACE_IMPACT_FX;
            } else if token.eq_ignore_ascii_case("org2isOffset") {
                self.mSpawnFlags |= FX_ORG2_IS_OFFSET;
            } else if token.eq_ignore_ascii_case("cheapOrgCalc") {
                self.mSpawnFlags |= FX_CHEAP_ORG_CALC;
            } else if token.eq_ignore_ascii_case("cheapOrg2Calc") {
                self.mSpawnFlags |= FX_CHEAP_ORG2_CALC;
            } else if token.eq_ignore_ascii_case("absoluteVel") {
                self.mSpawnFlags |= FX_VEL_IS_ABSOLUTE;
            } else if token.eq_ignore_ascii_case("absoluteAccel") {
                self.mSpawnFlags |= FX_ACCEL_IS_ABSOLUTE;
            } else if token.eq_ignore_ascii_case("orgOnSphere") {
                self.mSpawnFlags |= FX_ORG_ON_SPHERE;
            } else if token.eq_ignore_ascii_case("orgOnCylinder") {
                self.mSpawnFlags |= FX_ORG_ON_CYLINDER;
            } else if token.eq_ignore_ascii_case("axisFromSphere") {
                self.mSpawnFlags |= FX_AXIS_FROM_SPHERE;
            } else if token.eq_ignore_ascii_case("randrotaroundfwd") {
                self.mSpawnFlags |= FX_RAND_ROT_AROUND_FWD;
            } else if token.eq_ignore_ascii_case("evenDistribution") {
                self.mSpawnFlags |= FX_EVEN_DISTRIBUTION;
            } else if token.eq_ignore_ascii_case("rgbComponentInterpolation") {
                self.mSpawnFlags |= FX_RGB_COMPONENT_INTERP;
            } else if token.eq_ignore_ascii_case("affectedByWind") {
                self.mSpawnFlags |= FX_AFFECTED_BY_WIND;
            } else {
                // We have badness going on, but continue in case a later field is valid.
                ok = false;
            }
        }

        ok
    }

    /// Reads the material-impact effect key. An unknown value resets to none.
    ///
    /// Source: `oracle/codemp/client/FxTemplate.cpp:894-907`
    pub fn ParseMaterialImpact(&mut self, host: &mut FxHost<'_, '_>, val: &str) -> bool {
        if val.eq_ignore_ascii_case("shellsound") {
            self.mMatImpactFX = EMatImpactEffect::MATIMPACTFX_SHELLSOUND;
            true
        } else {
            self.mMatImpactFX = EMatImpactEffect::MATIMPACTFX_NONE;
            host.Print("CPrimitiveTemplate::ParseMaterialImpact -- unknown materialImpact type!\n");
            false
        }
    }

    /// Reads an acceleration field in vector format.
    ///
    /// Source: `oracle/codemp/client/FxTemplate.cpp:920-933`
    pub fn ParseAcceleration(&mut self, val: &str) -> bool {
        match ParseVector(val) {
            Some((min, max)) => {
                self.mAccelX.SetRange(min[0], max[0]);
                self.mAccelY.SetRange(min[1], max[1]);
                self.mAccelZ.SetRange(min[2], max[2]);
                true
            }
            None => false,
        }
    }

    /// Reads a ranged gravity value.
    ///
    /// Source: `oracle/codemp/client/FxTemplate.cpp:945-956`
    pub fn ParseGravity(&mut self, val: &str) -> bool {
        match ParseFloat(val) {
            Some((min, max)) => {
                self.mGravity.SetRange(min, max);
                true
            }
            None => false,
        }
    }

    /// Reads a ranged density value. Density steers how often an emitter emits.
    ///
    /// Source: `oracle/codemp/client/FxTemplate.cpp:970-981`
    pub fn ParseDensity(&mut self, val: &str) -> bool {
        match ParseFloat(val) {
            Some((min, max)) => {
                self.mDensity.SetRange(min, max);
                true
            }
            None => false,
        }
    }

    /// Reads a ranged variance value. Variance is the slop in an emitter's density calc.
    ///
    /// Source: `oracle/codemp/client/FxTemplate.cpp:996-1007`
    pub fn ParseVariance(&mut self, val: &str) -> bool {
        match ParseFloat(val) {
            Some((min, max)) => {
                self.mVariance.SetRange(min, max);
                true
            }
            None => false,
        }
    }

    /// Reads the rgbStart field in vector format.
    ///
    /// Source: `oracle/codemp/client/FxTemplate.cpp:1019-1032`
    pub fn ParseRGBStart(&mut self, val: &str) -> bool {
        match ParseVector(val) {
            Some((min, max)) => {
                self.mRedStart.SetRange(min[0], max[0]);
                self.mGreenStart.SetRange(min[1], max[1]);
                self.mBlueStart.SetRange(min[2], max[2]);
                true
            }
            None => false,
        }
    }

    /// Reads the rgbEnd field in vector format.
    ///
    /// Source: `oracle/codemp/client/FxTemplate.cpp:1044-1057`
    pub fn ParseRGBEnd(&mut self, val: &str) -> bool {
        match ParseVector(val) {
            Some((min, max)) => {
                self.mRedEnd.SetRange(min[0], max[0]);
                self.mGreenEnd.SetRange(min[1], max[1]);
                self.mBlueEnd.SetRange(min[2], max[2]);
                true
            }
            None => false,
        }
    }

    /// Reads the rgbParm field in float format.
    ///
    /// Source: `oracle/codemp/client/FxTemplate.cpp:1069-1080`
    pub fn ParseRGBParm(&mut self, val: &str) -> bool {
        match ParseFloat(val) {
            Some((min, max)) => {
                self.mRGBParm.SetRange(min, max);
                true
            }
            None => false,
        }
    }

    /// Reads the rgbFlags field and shifts the generic flags into the RGB group range.
    ///
    /// Source: `oracle/codemp/client/FxTemplate.cpp:1092-1104`
    pub fn ParseRGBFlags(&mut self, val: &str) -> bool {
        let (flags, ok) = ParseGroupFlags(val);
        self.mFlags |= flags << FX_RGB_SHIFT;
        ok
    }

    /// Reads the alphaStart field in float format.
    ///
    /// Source: `oracle/codemp/client/FxTemplate.cpp:1116-1127`
    pub fn ParseAlphaStart(&mut self, val: &str) -> bool {
        match ParseFloat(val) {
            Some((min, max)) => {
                self.mAlphaStart.SetRange(min, max);
                true
            }
            None => false,
        }
    }

    /// Reads the alphaEnd field in float format.
    ///
    /// Source: `oracle/codemp/client/FxTemplate.cpp:1139-1150`
    pub fn ParseAlphaEnd(&mut self, val: &str) -> bool {
        match ParseFloat(val) {
            Some((min, max)) => {
                self.mAlphaEnd.SetRange(min, max);
                true
            }
            None => false,
        }
    }

    /// Reads the alphaParm field in float format.
    ///
    /// Source: `oracle/codemp/client/FxTemplate.cpp:1162-1173`
    pub fn ParseAlphaParm(&mut self, val: &str) -> bool {
        match ParseFloat(val) {
            Some((min, max)) => {
                self.mAlphaParm.SetRange(min, max);
                true
            }
            None => false,
        }
    }

    /// Reads the alphaFlags field and shifts the generic flags into the alpha group range.
    ///
    /// Source: `oracle/codemp/client/FxTemplate.cpp:1185-1197`
    pub fn ParseAlphaFlags(&mut self, val: &str) -> bool {
        let (flags, ok) = ParseGroupFlags(val);
        self.mFlags |= flags << FX_ALPHA_SHIFT;
        ok
    }

    /// Reads the sizeStart field in float format.
    ///
    /// Source: `oracle/codemp/client/FxTemplate.cpp:1209-1220`
    pub fn ParseSizeStart(&mut self, val: &str) -> bool {
        match ParseFloat(val) {
            Some((min, max)) => {
                self.mSizeStart.SetRange(min, max);
                true
            }
            None => false,
        }
    }

    /// Reads the sizeEnd field in float format.
    ///
    /// Source: `oracle/codemp/client/FxTemplate.cpp:1232-1243`
    pub fn ParseSizeEnd(&mut self, val: &str) -> bool {
        match ParseFloat(val) {
            Some((min, max)) => {
                self.mSizeEnd.SetRange(min, max);
                true
            }
            None => false,
        }
    }

    /// Reads the sizeParm field in float format.
    ///
    /// Source: `oracle/codemp/client/FxTemplate.cpp:1255-1266`
    pub fn ParseSizeParm(&mut self, val: &str) -> bool {
        match ParseFloat(val) {
            Some((min, max)) => {
                self.mSizeParm.SetRange(min, max);
                true
            }
            None => false,
        }
    }

    /// Reads the sizeFlags field and shifts the generic flags into the size group range.
    ///
    /// Source: `oracle/codemp/client/FxTemplate.cpp:1278-1290`
    pub fn ParseSizeFlags(&mut self, val: &str) -> bool {
        let (flags, ok) = ParseGroupFlags(val);
        self.mFlags |= flags << FX_SIZE_SHIFT;
        ok
    }

    /// Reads the size2Start field in float format.
    ///
    /// Source: `oracle/codemp/client/FxTemplate.cpp:1302-1313`
    pub fn ParseSize2Start(&mut self, val: &str) -> bool {
        match ParseFloat(val) {
            Some((min, max)) => {
                self.mSize2Start.SetRange(min, max);
                true
            }
            None => false,
        }
    }

    /// Reads the size2End field in float format.
    ///
    /// Source: `oracle/codemp/client/FxTemplate.cpp:1325-1336`
    pub fn ParseSize2End(&mut self, val: &str) -> bool {
        match ParseFloat(val) {
            Some((min, max)) => {
                self.mSize2End.SetRange(min, max);
                true
            }
            None => false,
        }
    }

    /// Reads the size2Parm field in float format.
    ///
    /// Source: `oracle/codemp/client/FxTemplate.cpp:1348-1359`
    pub fn ParseSize2Parm(&mut self, val: &str) -> bool {
        match ParseFloat(val) {
            Some((min, max)) => {
                self.mSize2Parm.SetRange(min, max);
                true
            }
            None => false,
        }
    }

    /// Reads the size2Flags field and shifts the generic flags into the size2 group range.
    ///
    /// Source: `oracle/codemp/client/FxTemplate.cpp:1371-1383`
    pub fn ParseSize2Flags(&mut self, val: &str) -> bool {
        let (flags, ok) = ParseGroupFlags(val);
        self.mFlags |= flags << FX_SIZE2_SHIFT;
        ok
    }

    /// Reads the lengthStart field in float format.
    ///
    /// Source: `oracle/codemp/client/FxTemplate.cpp:1395-1406`
    pub fn ParseLengthStart(&mut self, val: &str) -> bool {
        match ParseFloat(val) {
            Some((min, max)) => {
                self.mLengthStart.SetRange(min, max);
                true
            }
            None => false,
        }
    }

    /// Reads the lengthEnd field in float format.
    ///
    /// Source: `oracle/codemp/client/FxTemplate.cpp:1418-1429`
    pub fn ParseLengthEnd(&mut self, val: &str) -> bool {
        match ParseFloat(val) {
            Some((min, max)) => {
                self.mLengthEnd.SetRange(min, max);
                true
            }
            None => false,
        }
    }

    /// Reads the lengthParm field in float format.
    ///
    /// Source: `oracle/codemp/client/FxTemplate.cpp:1441-1452`
    pub fn ParseLengthParm(&mut self, val: &str) -> bool {
        match ParseFloat(val) {
            Some((min, max)) => {
                self.mLengthParm.SetRange(min, max);
                true
            }
            None => false,
        }
    }

    /// Reads the lengthFlags field and shifts the generic flags into the length group range.
    ///
    /// Source: `oracle/codemp/client/FxTemplate.cpp:1464-1476`
    pub fn ParseLengthFlags(&mut self, val: &str) -> bool {
        let (flags, ok) = ParseGroupFlags(val);
        self.mFlags |= flags << FX_LENGTH_SHIFT;
        ok
    }

    /// Reads a group of shaders and registers each one.
    ///
    /// An empty value list is a failure. A list value registers every entry.
    ///
    /// Source: `oracle/codemp/client/FxTemplate.cpp:1488-1528`
    pub fn ParseShaders(&mut self, host: &mut FxHost<'_, '_>, grp: &GpValue) -> bool {
        if grp.is_list() {
            for val in grp.values() {
                let handle = host.RegisterShader(val);
                self.mMediaHandles.AddHandle(handle);
            }
        } else {
            match grp.top_value() {
                Some(val) => {
                    let handle = host.RegisterShader(val);
                    self.mMediaHandles.AddHandle(handle);
                }
                None => {
                    host.Print("CPrimitiveTemplate::ParseShaders called with an empty list!\n");
                    return false;
                }
            }
        }

        true
    }

    /// Reads a group of sounds and registers each one.
    ///
    /// An empty value list is a failure. A list value registers every entry.
    ///
    /// Source: `oracle/codemp/client/FxTemplate.cpp:1540-1580`
    pub fn ParseSounds(&mut self, host: &mut FxHost<'_, '_>, grp: &GpValue) -> bool {
        if grp.is_list() {
            for val in grp.values() {
                let handle = host.RegisterSound(val);
                self.mMediaHandles.AddHandle(handle);
            }
        } else {
            match grp.top_value() {
                Some(val) => {
                    let handle = host.RegisterSound(val);
                    self.mMediaHandles.AddHandle(handle);
                }
                None => {
                    host.Print("CPrimitiveTemplate::ParseSounds called with an empty list!\n");
                    return false;
                }
            }
        }

        true
    }

    /// Reads a group of models and registers each one.
    ///
    /// Also marks the template as an attached-model primitive, even when the
    /// list is empty and the parse fails.
    ///
    /// Source: `oracle/codemp/client/FxTemplate.cpp:1592-1634`
    pub fn ParseModels(&mut self, host: &mut FxHost<'_, '_>, grp: &GpValue) -> bool {
        let mut ok = true;

        if grp.is_list() {
            for val in grp.values() {
                let handle = host.RegisterModel(val);
                self.mMediaHandles.AddHandle(handle);
            }
        } else {
            match grp.top_value() {
                Some(val) => {
                    let handle = host.RegisterModel(val);
                    self.mMediaHandles.AddHandle(handle);
                }
                None => {
                    host.Print("CPrimitiveTemplate::ParseModels called with an empty list!\n");
                    ok = false;
                }
            }
        }

        self.mFlags |= FX_ATTACHED_MODEL;

        ok
    }

    /// Reads a group of effect file names for the impact list and registers each one.
    ///
    /// A missing effect file aborts the whole parse; a found one sets the
    /// impact-runs-fx and apply-physics flags.
    ///
    /// Source: `oracle/codemp/client/FxTemplate.cpp:1646-1705`
    pub fn ParseImpactFxStrings(
        &mut self,
        host: &mut FxHost<'_, '_>,
        fx: &mut FxSystem,
        grp: &GpValue,
    ) -> bool {
        if grp.is_list() {
            for val in grp.values() {
                let handle = fx_register_effect(fx, host, val);
                if handle != 0 {
                    self.mImpactFxHandles.AddHandle(handle);
                } else {
                    host.Print("FxTemplate: Impact effect file not found.\n");
                    return false;
                }
            }
        } else {
            match grp.top_value() {
                Some(val) => {
                    let handle = fx_register_effect(fx, host, val);
                    if handle != 0 {
                        self.mImpactFxHandles.AddHandle(handle);
                    } else {
                        host.Print("FxTemplate: Impact effect file not found.\n");
                        return false;
                    }
                }
                None => {
                    host.Print(
                        "CPrimitiveTemplate::ParseImpactFxStrings called with an empty list!\n",
                    );
                    return false;
                }
            }
        }

        self.mFlags |= FX_IMPACT_RUNS_FX | FX_APPLY_PHYSICS;

        true
    }

    /// Reads a group of effect file names for the death list and registers each one.
    ///
    /// A missing effect file aborts the whole parse.
    ///
    /// Source: `oracle/codemp/client/FxTemplate.cpp:1717-1776`
    pub fn ParseDeathFxStrings(
        &mut self,
        host: &mut FxHost<'_, '_>,
        fx: &mut FxSystem,
        grp: &GpValue,
    ) -> bool {
        if grp.is_list() {
            for val in grp.values() {
                let handle = fx_register_effect(fx, host, val);
                if handle != 0 {
                    self.mDeathFxHandles.AddHandle(handle);
                } else {
                    host.Print("FxTemplate: Death effect file not found.\n");
                    return false;
                }
            }
        } else {
            match grp.top_value() {
                Some(val) => {
                    let handle = fx_register_effect(fx, host, val);
                    if handle != 0 {
                        self.mDeathFxHandles.AddHandle(handle);
                    } else {
                        host.Print("FxTemplate: Death effect file not found.\n");
                        return false;
                    }
                }
                None => {
                    host.Print(
                        "CPrimitiveTemplate::ParseDeathFxStrings called with an empty list!\n",
                    );
                    return false;
                }
            }
        }

        self.mFlags |= FX_DEATH_RUNS_FX;

        true
    }

    /// Reads a group of effect file names for the emitter list and registers each one.
    ///
    /// A missing effect file aborts the whole parse.
    ///
    /// Source: `oracle/codemp/client/FxTemplate.cpp:1788-1847`
    pub fn ParseEmitterFxStrings(
        &mut self,
        host: &mut FxHost<'_, '_>,
        fx: &mut FxSystem,
        grp: &GpValue,
    ) -> bool {
        if grp.is_list() {
            for val in grp.values() {
                let handle = fx_register_effect(fx, host, val);
                if handle != 0 {
                    self.mEmitterFxHandles.AddHandle(handle);
                } else {
                    host.Print("FxTemplate: Emitter effect file not found.\n");
                    return false;
                }
            }
        } else {
            match grp.top_value() {
                Some(val) => {
                    let handle = fx_register_effect(fx, host, val);
                    if handle != 0 {
                        self.mEmitterFxHandles.AddHandle(handle);
                    } else {
                        host.Print("FxTemplate: Emitter effect file not found.\n");
                        return false;
                    }
                }
                None => {
                    host.Print(
                        "CPrimitiveTemplate::ParseEmitterFxStrings called with an empty list!\n",
                    );
                    return false;
                }
            }
        }

        self.mFlags |= FX_EMIT_FX;

        true
    }

    /// Reads a group of effect file names for the play list and registers each one.
    ///
    /// A missing effect file aborts the whole parse.
    ///
    /// Source: `oracle/codemp/client/FxTemplate.cpp:1859-1916`
    pub fn ParsePlayFxStrings(
        &mut self,
        host: &mut FxHost<'_, '_>,
        fx: &mut FxSystem,
        grp: &GpValue,
    ) -> bool {
        if grp.is_list() {
            for val in grp.values() {
                let handle = fx_register_effect(fx, host, val);
                if handle != 0 {
                    self.mPlayFxHandles.AddHandle(handle);
                } else {
                    host.Print("FxTemplate: Effect file not found.\n");
                    return false;
                }
            }
        } else {
            match grp.top_value() {
                Some(val) => {
                    let handle = fx_register_effect(fx, host, val);
                    if handle != 0 {
                        self.mPlayFxHandles.AddHandle(handle);
                    } else {
                        host.Print("FxTemplate: Effect file not found.\n");
                        return false;
                    }
                }
                None => {
                    host.Print(
                        "CPrimitiveTemplate::ParsePlayFxStrings called with an empty list!\n",
                    );
                    return false;
                }
            }
        }

        true
    }

    /// Reads the pairs in an rgb group.
    ///
    /// An unknown key just prints and moves on. This always returns `true`,
    /// matching Raven.
    ///
    /// The mandated signature (see module header) carries no `FxHost`, so the
    /// unknown-key print Raven makes here has no receiver and is dropped
    /// (porting-rules §19: the one defined behavior under this constraint).
    ///
    /// Source: `oracle/codemp/client/FxTemplate.cpp:1929-1970`
    pub fn ParseRGB(&mut self, grp: &GpGroup<'_>) -> bool {
        for pair in grp.pairs() {
            let key = pair.name();
            let val = pair.top_value().unwrap_or("");

            if key.eq_ignore_ascii_case("start") {
                self.ParseRGBStart(val);
            } else if key.eq_ignore_ascii_case("end") {
                self.ParseRGBEnd(val);
            } else if key.eq_ignore_ascii_case("parm") || key.eq_ignore_ascii_case("parms") {
                self.ParseRGBParm(val);
            } else if key.eq_ignore_ascii_case("flags") || key.eq_ignore_ascii_case("flag") {
                self.ParseRGBFlags(val);
            }
        }

        true
    }

    /// Reads the pairs in an alpha group.
    ///
    /// An unknown key just prints and moves on. This always returns `true`,
    /// matching Raven.
    ///
    /// The mandated signature (see module header) carries no `FxHost`, so the
    /// unknown-key print Raven makes here has no receiver and is dropped
    /// (porting-rules §19: the one defined behavior under this constraint).
    ///
    /// Source: `oracle/codemp/client/FxTemplate.cpp:1983-2024`
    pub fn ParseAlpha(&mut self, grp: &GpGroup<'_>) -> bool {
        for pair in grp.pairs() {
            let key = pair.name();
            let val = pair.top_value().unwrap_or("");

            if key.eq_ignore_ascii_case("start") {
                self.ParseAlphaStart(val);
            } else if key.eq_ignore_ascii_case("end") {
                self.ParseAlphaEnd(val);
            } else if key.eq_ignore_ascii_case("parm") || key.eq_ignore_ascii_case("parms") {
                self.ParseAlphaParm(val);
            } else if key.eq_ignore_ascii_case("flags") || key.eq_ignore_ascii_case("flag") {
                self.ParseAlphaFlags(val);
            }
        }

        true
    }

    /// Reads the pairs in a size group.
    ///
    /// An unknown key just prints and moves on. This always returns `true`,
    /// matching Raven.
    ///
    /// The mandated signature (see module header) carries no `FxHost`, so the
    /// unknown-key print Raven makes here has no receiver and is dropped
    /// (porting-rules §19: the one defined behavior under this constraint).
    ///
    /// Source: `oracle/codemp/client/FxTemplate.cpp:2037-2078`
    pub fn ParseSize(&mut self, grp: &GpGroup<'_>) -> bool {
        for pair in grp.pairs() {
            let key = pair.name();
            let val = pair.top_value().unwrap_or("");

            if key.eq_ignore_ascii_case("start") {
                self.ParseSizeStart(val);
            } else if key.eq_ignore_ascii_case("end") {
                self.ParseSizeEnd(val);
            } else if key.eq_ignore_ascii_case("parm") || key.eq_ignore_ascii_case("parms") {
                self.ParseSizeParm(val);
            } else if key.eq_ignore_ascii_case("flags") || key.eq_ignore_ascii_case("flag") {
                self.ParseSizeFlags(val);
            }
        }

        true
    }

    /// Reads the pairs in a size2 group.
    ///
    /// An unknown key just prints and moves on. This always returns `true`,
    /// matching Raven.
    ///
    /// The mandated signature (see module header) carries no `FxHost`, so the
    /// unknown-key print Raven makes here has no receiver and is dropped
    /// (porting-rules §19: the one defined behavior under this constraint).
    ///
    /// Source: `oracle/codemp/client/FxTemplate.cpp:2091-2132`
    pub fn ParseSize2(&mut self, grp: &GpGroup<'_>) -> bool {
        for pair in grp.pairs() {
            let key = pair.name();
            let val = pair.top_value().unwrap_or("");

            if key.eq_ignore_ascii_case("start") {
                self.ParseSize2Start(val);
            } else if key.eq_ignore_ascii_case("end") {
                self.ParseSize2End(val);
            } else if key.eq_ignore_ascii_case("parm") || key.eq_ignore_ascii_case("parms") {
                self.ParseSize2Parm(val);
            } else if key.eq_ignore_ascii_case("flags") || key.eq_ignore_ascii_case("flag") {
                self.ParseSize2Flags(val);
            }
        }

        true
    }

    /// Reads the pairs in a length group.
    ///
    /// An unknown key just prints and moves on. This always returns `true`,
    /// matching Raven.
    ///
    /// The mandated signature (see module header) carries no `FxHost`, so the
    /// unknown-key print Raven makes here has no receiver and is dropped
    /// (porting-rules §19: the one defined behavior under this constraint).
    ///
    /// Source: `oracle/codemp/client/FxTemplate.cpp:2145-2186`
    pub fn ParseLength(&mut self, grp: &GpGroup<'_>) -> bool {
        for pair in grp.pairs() {
            let key = pair.name();
            let val = pair.top_value().unwrap_or("");

            if key.eq_ignore_ascii_case("start") {
                self.ParseLengthStart(val);
            } else if key.eq_ignore_ascii_case("end") {
                self.ParseLengthEnd(val);
            } else if key.eq_ignore_ascii_case("parm") || key.eq_ignore_ascii_case("parms") {
                self.ParseLengthParm(val);
            } else if key.eq_ignore_ascii_case("flags") || key.eq_ignore_ascii_case("flag") {
                self.ParseLengthFlags(val);
            }
        }

        true
    }

    /// Parses a primitive: applies base-level key pairs first, then walks the
    /// subgroups (`rgb`, `alpha`, `size`, `size2`, `length`).
    ///
    /// An unknown key or subgroup name just prints and moves on. This always
    /// returns `true`, matching Raven.
    ///
    /// Source: `oracle/codemp/client/FxTemplate.cpp:2192-2386`
    pub fn ParsePrimitive(
        &mut self,
        host: &mut FxHost<'_, '_>,
        fx: &mut FxSystem,
        grp: &GpGroup<'_>,
    ) -> bool {
        for pair in grp.pairs() {
            let key = pair.name();
            let val = pair.top_value().unwrap_or("");

            if key.eq_ignore_ascii_case("count") {
                self.ParseCount(val);
            } else if key.eq_ignore_ascii_case("shaders") || key.eq_ignore_ascii_case("shader") {
                self.ParseShaders(host, pair);
            } else if key.eq_ignore_ascii_case("models") || key.eq_ignore_ascii_case("model") {
                self.ParseModels(host, pair);
            } else if key.eq_ignore_ascii_case("sounds") || key.eq_ignore_ascii_case("sound") {
                self.ParseSounds(host, pair);
            } else if key.eq_ignore_ascii_case("impactfx") {
                self.ParseImpactFxStrings(host, fx, pair);
            } else if key.eq_ignore_ascii_case("deathfx") {
                self.ParseDeathFxStrings(host, fx, pair);
            } else if key.eq_ignore_ascii_case("emitfx") {
                self.ParseEmitterFxStrings(host, fx, pair);
            } else if key.eq_ignore_ascii_case("playfx") {
                self.ParsePlayFxStrings(host, fx, pair);
            } else if key.eq_ignore_ascii_case("life") {
                self.ParseLife(val);
            } else if key.eq_ignore_ascii_case("delay") {
                self.ParseDelay(val);
            } else if key.eq_ignore_ascii_case("cullrange") {
                // Raven leaves this key's body commented out; mCullRange stays untouched.
            } else if key.eq_ignore_ascii_case("bounce") || key.eq_ignore_ascii_case("intensity") {
                // Reuses elasticity for these two keys, as Raven's own comment admits.
                self.ParseElasticity(val);
            } else if key.eq_ignore_ascii_case("min") {
                self.ParseMin(val);
            } else if key.eq_ignore_ascii_case("max") {
                self.ParseMax(val);
            } else if key.eq_ignore_ascii_case("angle") || key.eq_ignore_ascii_case("angles") {
                self.ParseAngle(val);
            } else if key.eq_ignore_ascii_case("angleDelta") {
                self.ParseAngleDelta(val);
            } else if key.eq_ignore_ascii_case("velocity") || key.eq_ignore_ascii_case("vel") {
                self.ParseVelocity(val);
            } else if key.eq_ignore_ascii_case("acceleration") || key.eq_ignore_ascii_case("accel")
            {
                self.ParseAcceleration(val);
            } else if key.eq_ignore_ascii_case("gravity") {
                self.ParseGravity(val);
            } else if key.eq_ignore_ascii_case("density") {
                self.ParseDensity(val);
            } else if key.eq_ignore_ascii_case("variance") {
                self.ParseVariance(val);
            } else if key.eq_ignore_ascii_case("origin") {
                self.ParseOrigin1(val);
            } else if key.eq_ignore_ascii_case("origin2") {
                self.ParseOrigin2(val);
            } else if key.eq_ignore_ascii_case("radius") {
                self.ParseRadius(val);
            } else if key.eq_ignore_ascii_case("height") {
                self.ParseHeight(val);
            } else if key.eq_ignore_ascii_case("wind") {
                self.ParseWindModifier(val);
            } else if key.eq_ignore_ascii_case("rotation") {
                self.ParseRotation(val);
            } else if key.eq_ignore_ascii_case("rotationDelta") {
                self.ParseRotationDelta(val);
            } else if key.eq_ignore_ascii_case("flags") || key.eq_ignore_ascii_case("flag") {
                // These flags pass on to the spawned primitive.
                self.ParseFlags(val);
            } else if key.eq_ignore_ascii_case("spawnFlags")
                || key.eq_ignore_ascii_case("spawnFlag")
            {
                // These flags steer spawning only and never reach a primitive.
                self.ParseSpawnFlags(val);
            } else if key.eq_ignore_ascii_case("name") {
                if let Some(val) = pair.top_value() {
                    self.set_name(val);
                }
            } else if key.eq_ignore_ascii_case("materialImpact") {
                self.ParseMaterialImpact(host, val);
            } else {
                host.Print(&format!("Unknown key parsing an effect primitive: {key}\n"));
            }
        }

        for sub_grp in grp.subgroups() {
            let key = sub_grp.name();

            if key.eq_ignore_ascii_case("rgb") {
                self.ParseRGB(&sub_grp);
            } else if key.eq_ignore_ascii_case("alpha") {
                self.ParseAlpha(&sub_grp);
            } else if key.eq_ignore_ascii_case("size") || key.eq_ignore_ascii_case("width") {
                self.ParseSize(&sub_grp);
            } else if key.eq_ignore_ascii_case("size2") || key.eq_ignore_ascii_case("width2") {
                self.ParseSize2(&sub_grp);
            } else if key.eq_ignore_ascii_case("length") || key.eq_ignore_ascii_case("height") {
                self.ParseLength(&sub_grp);
            } else {
                host.Print(&format!("Unknown group key parsing a particle: {key}\n"));
            }
        }

        true
    }
}
