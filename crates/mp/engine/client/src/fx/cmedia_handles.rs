#![allow(non_camel_case_types, non_snake_case)]

use native_math::rng::QRand;

/// Raven `CMediaHandles` — one primitive template's shader, sound, model, or effect list.
///
/// `GetHandle` draws one `irand` per call whenever the list is not empty, so the
/// call count is parity surface.
/// Type definition source: `oracle/codemp/client/FxScheduler.h:66-79`
#[derive(Clone, PartialEq, Debug, Default)]
pub struct CMediaHandles {
    mMediaList: Vec<i32>,
}

impl CMediaHandles {
    /// Source: `oracle/codemp/client/FxScheduler.h:74`
    pub fn AddHandle(&mut self, item: i32) {
        self.mMediaList.push(item);
    }

    /// Source: `oracle/codemp/client/FxScheduler.h:75-76`
    pub fn GetHandle(&self, rng: &mut QRand) -> i32 {
        if self.mMediaList.is_empty() {
            0
        } else {
            self.mMediaList[rng.irand(0, self.mMediaList.len() as i32 - 1) as usize]
        }
    }

    /// The stored list, in insertion order. The parity dumper reads it.
    pub fn handles(&self) -> &[i32] {
        &self.mMediaList
    }
}
