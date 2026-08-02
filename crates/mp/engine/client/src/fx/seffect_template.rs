#![allow(non_camel_case_types, non_snake_case)]

use std::cell::RefCell;
use std::rc::Rc;

use crate::fx::cprimitive_template::CPrimitiveTemplate;

/// `FX_MAX_EFFECT_COMPONENTS` — how many primitives an effect can hold, this should be plenty.
///
/// Source: `oracle/codemp/client/FxScheduler.h:28`
pub const FX_MAX_EFFECT_COMPONENTS: usize = 24;

/// A primitive template shared between an effect template and the schedule.
///
/// Raven hands out raw `CPrimitiveTemplate*`, and a scheduled effect keeps
/// reading the same object after `PlayEffect` writes the sound volume into it.
/// The shared handle keeps that aliasing.
pub type PrimitiveRef = Rc<RefCell<CPrimitiveTemplate>>;

/// Raven `SEffectTemplate` — one `.efx` file as a set of primitive templates.
///
/// A slot with `mInUse` false is free. `mCopy` marks a run-time clone that
/// `PlayEffect` releases as soon as it has spawned everything.
/// Type definition source: `oracle/codemp/client/FxScheduler.h:346-360`
#[derive(Clone, Debug, Default)]
pub struct SEffectTemplate {
    pub mInUse: bool,
    pub mCopy: bool,
    pub mEffectName: String,
    pub mPrimitiveCount: i32,
    pub mRepeatDelay: i32,
    pub mPrimitives: Vec<PrimitiveRef>,
}

impl SEffectTemplate {
    /// Raven `SEffectTemplate::operator==(const char *name)`.
    ///
    /// Source: `oracle/codemp/client/FxScheduler.h:355-358`
    pub fn matches(&self, name: &str) -> bool {
        self.mEffectName.eq_ignore_ascii_case(name)
    }

    /// Raven `SEffectTemplate::operator=` — a deep copy that marks every primitive as a copy.
    ///
    /// Source: `oracle/codemp/client/FxScheduler.cpp:147-162`
    pub fn copy_from(&mut self, that: &SEffectTemplate) {
        self.mCopy = true;

        self.mEffectName = that.mEffectName.clone();

        self.mPrimitiveCount = that.mPrimitiveCount;

        self.mPrimitives.clear();
        for i in 0..self.mPrimitiveCount as usize {
            let mut prim = that.mPrimitives[i].borrow().clone();
            // Mark use as a copy so that we know that we should be chucked when used up
            prim.mCopy = true;
            self.mPrimitives.push(Rc::new(RefCell::new(prim)));
        }
    }
}
