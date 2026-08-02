#![allow(non_camel_case_types, non_snake_case)]

use native_math::rng::QRand;

/// Raven `CFxRange` — the min/max float range every primitive-template field uses.
///
/// `get_val` draws from the generator, so every call is a parity-visible RNG
/// consumption. Do not hoist or skip a call.
/// Type definition source: `oracle/codemp/client/FxScheduler.h:91-113`
#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub struct CFxRange {
    mMin: f32,
    mMax: f32,
}

impl CFxRange {
    /// Source: `oracle/codemp/client/FxScheduler.h:102`
    pub fn SetRange(&mut self, min: f32, max: f32) {
        self.mMin = min;
        self.mMax = max;
    }

    /// Source: `oracle/codemp/client/FxScheduler.h:104`
    pub fn GetMax(&self) -> f32 {
        self.mMax
    }

    /// Source: `oracle/codemp/client/FxScheduler.h:105`
    pub fn GetMin(&self) -> f32 {
        self.mMin
    }

    /// Interpolate at a caller-supplied fraction, which draws no random number.
    ///
    /// Source: `oracle/codemp/client/FxScheduler.h:106`
    pub fn GetValFraction(&self, fraction: f32) -> f32 {
        if self.mMin != self.mMax {
            self.mMin + fraction * (self.mMax - self.mMin)
        } else {
            self.mMin
        }
    }

    /// Draw a value in the range. Consumes one `flrand` when the range is not degenerate.
    ///
    /// Source: `oracle/codemp/client/FxScheduler.h:107`
    pub fn GetVal(&self, rng: &mut QRand) -> f32 {
        if self.mMin != self.mMax {
            rng.flrand(self.mMin, self.mMax)
        } else {
            self.mMin
        }
    }

    /// Source: `oracle/codemp/client/FxScheduler.h:109-110`
    pub fn GetRoundedVal(&self, rng: &mut QRand) -> i32 {
        if self.mMin == self.mMax {
            return self.mMin as i32;
        }
        (rng.flrand(self.mMin, self.mMax) + 0.5) as i32
    }
}
