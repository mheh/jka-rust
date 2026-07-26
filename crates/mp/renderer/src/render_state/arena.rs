//! `Arena<T>` — the generation-counted store behind one `RenderAssets`
//! registry (`R2-D3`/`R2-D4`).

use crate::render_state::handle::Handle;

/// A generic generation-counted arena backing one `RenderAssets` registry.
/// `#[derive(Clone)]` — required by `Arc::make_mut` on `RenderAssets`
/// (A9/NB-1); adds the ordinary `T: Clone` bound.
///
/// Shader/skin/model arenas soft-cap at their oracle `MAX_*` constant; the
/// image arena stays unbounded, matching its real oracle backing store (A5).
///
/// **Slot 0 reservation (A12).** Every capped arena is constructed with slot 0
/// pre-populated with the registry's oracle default entry — models index 0 is
/// `MOD_BAD` (`R_ModelInit`, `oracle/codemp/renderer/tr_model.cpp:1665-1680`),
/// skins index 0 is `"<default skin>"` (`R_InitSkins`,
/// `oracle/codemp/renderer/tr_image.cpp:3324-3332`), shaders index 0 is
/// `tr.defaultShader` (`CreateInternalShaders`,
/// `oracle/codemp/renderer/tr_shader.cpp:4137-4155`). `Handle { index: 0,
/// generation: 0 }` IS that live default, not a null/invalid sentinel — the
/// image arena has no reserved slot (uncapped, A5; a failed lookup returns
/// `None` from `image_names`, never a handle).
///
/// No oracle citation: new Rust-side infrastructure, the `AlignedBytes`
/// justified-exception precedent approved by A7 (`R2-D3`).
#[derive(Clone)]
pub struct Arena<T> {
    slots: Vec<Option<(u32 /* generation */, T)>>,
    /// Vacated slots, each paired with the generation its next occupant gets.
    /// The R2 sketch spells this `Vec<u32>`; it is widened to carry the
    /// vacated slot's next generation because `slots`' `Option` drops that
    /// generation on removal, and a reused slot restarting at generation 0
    /// would defeat ruling 11's stale-handle detection.
    free_list: Vec<(u32 /* index */, u32 /* next generation */)>,
    /// `None` for the unbounded image arena; `Some(MAX_*)` for
    /// shader/skin/model (A5). `Some` also implies slot 0 is reserved (A12)
    /// and never enters `free_list`.
    soft_cap: Option<u32>,
}

impl<T> Arena<T> {
    /// The unbounded arena (images, A5) — no soft cap, no reserved slot 0.
    pub fn new_unbounded() -> Arena<T> {
        Arena {
            slots: Vec::new(),
            free_list: Vec::new(),
            soft_cap: None,
        }
    }

    /// A capped arena with slot 0 pre-populated with the registry's oracle
    /// default entry (A12). `default_entry` is the caller's — the registration
    /// wave that owns the registry builds it (`R_ModelInit`/`R_InitSkins`/
    /// `CreateInternalShaders`), not this constructor.
    pub fn new_with_slot0(soft_cap: u32, default_entry: T) -> Arena<T> {
        Arena {
            slots: vec![Some((0, default_entry))],
            free_list: Vec::new(),
            soft_cap: Some(soft_cap),
        }
    }

    /// On overflow returns `Handle { index: 0, generation: 0 }` — the
    /// pre-populated default entry (A12), matching every oracle overflow path:
    /// shaders' `return tr.defaultShader` and skins'/models' `return 0` all
    /// resolve to the same live slot-0 object. Never `Result`.
    ///
    /// The per-registry overflow *warning* (retail's `Com_Printf` for
    /// shaders/skins, this port's marked warning for retail-silent models —
    /// A5 amendment) belongs to the `RenderAssetsSim` registration mutator,
    /// which has the engine receiver a print needs; this signature has none.
    pub fn insert(&mut self, value: T) -> Handle<T> {
        if let Some(cap) = self.soft_cap {
            let occupied = self.slots.len() - self.free_list.len();
            if occupied as u32 >= cap {
                return Handle::slot_zero();
            }
        }
        if let Some((index, generation)) = self.free_list.pop() {
            self.slots[index as usize] = Some((generation, value));
            return Handle::new(index, generation);
        }
        let index = self.slots.len() as u32;
        self.slots.push(Some((0, value)));
        Handle::new(index, 0)
    }

    pub fn get(&self, handle: Handle<T>) -> Option<&T> {
        match self.slots.get(handle.index() as usize) {
            Some(Some((generation, value))) if *generation == handle.generation() => Some(value),
            _ => None,
        }
    }

    pub fn get_mut(&mut self, handle: Handle<T>) -> Option<&mut T> {
        match self.slots.get_mut(handle.index() as usize) {
            Some(Some((generation, value))) if *generation == handle.generation() => Some(value),
            _ => None,
        }
    }

    pub fn remove(&mut self, handle: Handle<T>) -> Option<T> {
        let slot = self.slots.get_mut(handle.index() as usize)?;
        let occupied_generation = match slot {
            Some((generation, _)) if *generation == handle.generation() => *generation,
            _ => return None,
        };
        let (_, value) = slot.take()?;
        // Slot 0 of a capped arena is the reserved default entry — it never
        // enters the free list (A12).
        if self.soft_cap.is_none() || handle.index() != 0 {
            self.free_list
                .push((handle.index(), occupied_generation.wrapping_add(1)));
        }
        Some(value)
    }

    pub fn iter(&self) -> impl Iterator<Item = (Handle<T>, &T)> + '_ {
        self.slots
            .iter()
            .enumerate()
            .filter_map(|(index, slot)| match slot {
                Some((generation, value)) => Some((Handle::new(index as u32, *generation), value)),
                None => None,
            })
    }
}
