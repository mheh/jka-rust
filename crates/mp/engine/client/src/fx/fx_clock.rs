//! The FX clock, the one part of `SFxHelper` that carried state.
//!
//! Source: `oracle/codemp/client/FxSystem.h:52-70`,
//! `oracle/codemp/client/FxSystem.cpp:21-80`

#![allow(non_camel_case_types, non_snake_case)]

/// The FX system's own view of time, driven by `CG_FX_ADJUST_TIME` once a frame.
///
/// `mTime` is the absolute cgame time, not an accumulator. A frame time of zero
/// or less freezes every primitive, which is how a paused game stops the FX.
#[derive(Clone, Copy, Debug, Default)]
pub struct FxClock {
    pub mTime: i32,
    pub mOldTime: i32,
    pub mFrameTime: i32,
    pub mTimeFrozen: bool,
    pub mRealTime: f32,
}

impl FxClock {
    /// Raven `SFxHelper::ReInit`.
    ///
    /// Source: `oracle/codemp/client/FxSystem.cpp:30-37`
    pub fn ReInit(&mut self) {
        self.mTime = 0;
        self.mOldTime = 0;
        self.mFrameTime = 0;
        self.mTimeFrozen = false;
    }

    /// Raven `SFxHelper::AdjustTime`.
    ///
    /// The parameter is the new absolute time, in spite of Raven naming it
    /// `frametime`. A non-positive value stops time instead of moving it.
    ///
    /// Source: `oracle/codemp/client/FxSystem.cpp:53-80`
    pub fn AdjustTime(&mut self, frametime: i32) {
        if frametime <= 0 {
            // Allow no time progression when we are paused.
            self.mFrameTime = 0;
            self.mRealTime = 0.0;
        } else {
            self.mOldTime = self.mTime;
            self.mTime = frametime;
            self.mFrameTime = self.mTime - self.mOldTime;

            self.mRealTime = self.mFrameTime as f32 * 0.001;
        }
    }
}
