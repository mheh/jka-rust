//! `EffectHandle` — the generation-counted slot reference the cgame effect
//! pools hand out (DEC-46.3).

/// A slot in an [`EffectPool`](super::effect_pool::EffectPool), paired with the
/// generation that slot carried when the handle was issued.
///
/// Raven passed `localEntity_t *` / `markPoly_t *` around and relied on the
/// intrusive `prev`/`next` chain to tell live from free. The pool steals the
/// oldest entry when it runs dry, so a caller can be holding a reference to a
/// slot that has since been recycled — the generation is what catches that
/// (`get`/`get_mut` return `None` once the slot has moved on) instead of
/// silently reading somebody else's effect.
///
/// DEC-46.3 (`docs/decisions.md`): effect pools are a gen-counted slab plus an
/// explicit LRU queue; the intrusive `prev`/`next` links dissolve.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EffectHandle {
    /// Index into the pool's slab.
    pub index: u32,
    /// The slab generation this handle was issued against.
    pub generation: u32,
}

impl EffectHandle {
    /// Handy for the `usize` the slab indexes with.
    #[inline]
    pub fn index(self) -> usize {
        self.index as usize
    }
}
