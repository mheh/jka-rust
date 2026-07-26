//! `Handle<K>` — the generation-counted, per-kind-typed arena index (`R2-D3`).

use core::hash::{Hash, Hasher};
use core::marker::PhantomData;

/// A generation-counted index into one of `RenderAssets`'s four arenas.
/// Per-kind typing (`K`) catches cross-kind handle mixups at compile time
/// (A2) — an `ImageHandle` cannot be passed where a `ShaderHandle` is
/// expected, even though both are `(u32, u32)` underneath.
///
/// `Handle { index: 0, generation: 0 }` on a capped arena IS that registry's
/// live default entry (A12/`R2-D4`), not a null sentinel — `qhandle_t` 0 maps
/// to slot 0 as the identity at the seam.
///
/// No oracle citation: new Rust-side infrastructure implementing ruling 11's
/// generation-counted-handle requirement. Second instance of the
/// `AlignedBytes` justified-exception precedent (`docs/subsystems/tr-model.md`
/// `TRM-D4`/ruling 58) — approved by A7, no `Source:` line.
pub struct Handle<K> {
    index: u32,
    generation: u32,
    _kind: PhantomData<fn() -> K>,
}

impl<K> Handle<K> {
    /// The handle addressing `index` at `generation`.
    pub const fn new(index: u32, generation: u32) -> Handle<K> {
        Handle {
            index,
            generation,
            _kind: PhantomData,
        }
    }

    /// `Handle { index: 0, generation: 0 }` — the capped arenas' pre-populated
    /// default entry and every oracle overflow path's return value (A12).
    pub const fn slot_zero() -> Handle<K> {
        Handle::new(0, 0)
    }

    pub const fn index(self) -> u32 {
        self.index
    }

    pub const fn generation(self) -> u32 {
        self.generation
    }
}

// Hand-written, not `#[derive(...)]` (NB-4): a derive adds a `K: Trait` bound,
// but `K` is only ever a marker — the asset structs it is instantiated with are
// not `Copy`, so a derived `Copy` would make every `*Handle` non-`Copy` and
// break every by-value handle call site (`R2-D3`).
impl<K> Clone for Handle<K> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<K> Copy for Handle<K> {}

impl<K> PartialEq for Handle<K> {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index && self.generation == other.generation
    }
}

impl<K> Eq for Handle<K> {}

impl<K> Hash for Handle<K> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.index.hash(state);
        self.generation.hash(state);
    }
}
