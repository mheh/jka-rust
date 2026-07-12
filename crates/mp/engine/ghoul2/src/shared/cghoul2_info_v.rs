#![allow(non_camel_case_types, non_snake_case)]

use crate::ghoul2_system::Ghoul2System;
use crate::shared::cghoul2_info::CGhoul2Info;

/// Raven `CGhoul2Info_v` — handle to a vector of `CGhoul2Info` model instances,
/// indexed into the global `IGhoul2InfoArray`.
///
/// Raven: (none).
/// Type definition source: `oracle/codemp/ghoul2/ghoul2_shared.h:328-457`
#[repr(C)]
pub struct CGhoul2Info_v {
    /// don't be bad and muck with this
    pub mItem: i32,
}

const _: () = assert!(core::mem::size_of::<CGhoul2Info_v>() == 4);
const _: () = assert!(core::mem::offset_of!(CGhoul2Info_v, mItem) == 0);

// G2SV-D10 (ruling 22, §F21): the forwarding/lifecycle impl below colocates
// with the frozen struct rather than living in `info_array.rs`. Every method
// threads `&Ghoul2System`/`&mut Ghoul2System` explicitly (porting-rules §B4,
// state threaded not reached) in place of Raven's private `InfoArray()`
// (`ghoul2_shared.h:330-333`) and `Array()`/`Array() const` (`:350-359`)
// helpers, which reached the ambient `TheGhoul2InfoArray()` singleton — those
// two private helpers fold directly into each method below and get no
// separate stubs of their own.
//
// `kill()` (`:450-456`) is a §20 drop, not stubbed: its only callers are
// client-side FX code (`client/FxScheduler.cpp:1128`,
// `client/FxPrimitives.h:246,317`), entirely outside this server-side crate's
// scope (not merely unreached within it).
//
// Raven's destructor (`~CGhoul2Info_v`, `:370-373`) auto-calls `Free()` on
// scope exit; a Rust `Drop` impl cannot take the `&mut Ghoul2System` that
// `free` needs, so that RAII teardown does not port — callers must call
// `.free(g2)` explicitly at every site that relied on it, matching Raven's own
// comment that this is a backstop ("this had better be taken care of via the
// clean ghoul2 models call") over the real explicit cleanup path. The two
// trivial constructors (`:362-369`, zero-init / raw-handle-init) need no
// method port either — callers build `CGhoul2Info_v { mItem: ... }` directly
// since the field is `pub`.
impl CGhoul2Info_v {
    /// Raven `CGhoul2Info_v::Alloc` — allocates a fresh arena slot for this
    /// handle (`mItem` must be null going in).
    ///
    /// Raven: `assert(!mItem); //already alloced`.
    /// Source: `oracle/codemp/ghoul2/ghoul2_shared.h:335-340`
    pub fn alloc(&mut self, g2: &mut Ghoul2System) {
        // Raven: `assert(!mItem); mItem=InfoArray().New(); assert(!Array().size());`
        // `InfoArray().New()` → `Ghoul2InfoArray::new_handle`; `Array()` (the
        // private `Array()` helper, `:350-354`) folds into a direct
        // `g2.info_array` read (porting-rules §B4).
        debug_assert_eq!(self.mItem, 0, "already alloced");
        self.mItem = g2.info_array.new_handle();
        debug_assert!(g2.info_array.get(self.mItem).is_empty());
    }

    /// Raven `CGhoul2Info_v::Free` — releases the arena slot this handle owns
    /// (`Ghoul2InfoArray::Delete`, moved up to `Ghoul2System::delete`,
    /// `G2SV-D13`(a)) and zeroes `mItem`; a no-op when already null.
    ///
    /// Source: `oracle/codemp/ghoul2/ghoul2_shared.h:341-349`
    pub fn free(&mut self, g2: &mut Ghoul2System) {
        // Raven: `if (mItem) { assert(InfoArray().IsValid(mItem));
        // InfoArray().Delete(mItem); mItem=0; }`. `Delete` moved UP to
        // `Ghoul2System::delete` (`G2SV-D13`(a), ruling 29).
        if self.mItem != 0 {
            debug_assert!(g2.info_array.is_valid(self.mItem));
            g2.delete(self.mItem);
            self.mItem = 0;
        }
    }

    /// Raven `CGhoul2Info_v::clear` — alias for `Free`.
    ///
    /// Source: `oracle/codemp/ghoul2/ghoul2_shared.h:426-429`
    pub fn clear(&mut self, g2: &mut Ghoul2System) {
        // Raven: `clear() { Free(); }`.
        self.free(g2);
    }

    /// Raven `CGhoul2Info_v::DeepCopy` — frees this handle, then (if `other`
    /// is non-null) allocates a fresh slot, copies `other`'s instance vector,
    /// and zeroes each copied instance's runtime-only fields (`mBoneCache`,
    /// `mTransformedVertsArray`, `mSkelFrameNum`, `mMeshFrameNum`) so no
    /// runtime state aliases across the copy.
    ///
    /// Source: `oracle/codemp/ghoul2/ghoul2_shared.h:382-397`
    pub fn deep_copy(&mut self, g2: &mut Ghoul2System, other: &CGhoul2Info_v) {
        // Raven: `Free(); if (other.mItem) { Alloc(); Array()=other.Array();
        // for (i=0;i<size();i++) { Array()[i].mBoneCache=0;
        // Array()[i].mTransformedVertsArray=0; Array()[i].mSkelFrameNum=0;
        // Array()[i].mMeshFrameNum=0; } }`. The vector copy-assignment
        // (`Array()=other.Array()`, memberwise via `CGhoul2Info`'s copy) becomes
        // a `.to_vec()` clone; the runtime-state zeroing loop nulls the same
        // fields (`mBoneCache` → `bone_cache: None`, `mTransformedVertsArray` →
        // `transformed_verts_array: None`).
        self.free(g2);
        if other.mItem != 0 {
            self.alloc(g2);
            let copy = g2.info_array.get(other.mItem).to_vec();
            let dest = g2.info_array.get_mut(self.mItem);
            *dest = copy;
            for info in dest.iter_mut() {
                info.bone_cache = None;
                info.transformed_verts_array = None;
                info.skel_frame_num = 0;
                info.mesh_frame_num = 0;
            }
        }
    }

    /// Raven `CGhoul2Info_v::operator=(const CGhoul2Info_v &other)` — handle
    /// copy (`mItem = other.mItem`); no arena traffic.
    ///
    /// Source: `oracle/codemp/ghoul2/ghoul2_shared.h:374-377`
    pub fn assign(&mut self, other: &CGhoul2Info_v) {
        // Raven: `operator=(const CGhoul2Info_v &other) { mItem=other.mItem; }`.
        self.mItem = other.mItem;
    }

    /// Raven `CGhoul2Info_v::operator=(const int otherItem)` — raw-handle
    /// assignment "from the VM side item number".
    ///
    /// Source: `oracle/codemp/ghoul2/ghoul2_shared.h:378-381`
    pub fn assign_item(&mut self, other_item: i32) {
        // Raven: `operator=(const int otherItem) { mItem=otherItem; }`.
        self.mItem = other_item;
    }

    /// Raven `CGhoul2Info_v::operator[](int idx)` (mutable overload) —
    /// indexes this handle's instance vector via `Array()[idx]`.
    ///
    /// Source: `oracle/codemp/ghoul2/ghoul2_shared.h:399-404`
    pub fn get_mut<'a>(&self, g2: &'a mut Ghoul2System, idx: i32) -> &'a mut CGhoul2Info {
        // Raven: `assert(mItem); assert(idx>=0&&idx<size()); return Array()[idx];`.
        // `Array()` folds into `g2.info_array.get_mut(mItem)` (porting-rules §B4).
        debug_assert_ne!(self.mItem, 0);
        let arr = g2.info_array.get_mut(self.mItem);
        debug_assert!(idx >= 0 && (idx as usize) < arr.len());
        &mut arr[idx as usize]
    }

    /// Raven `CGhoul2Info_v::operator[](int idx) const` — read-only overload
    /// of the above.
    ///
    /// Source: `oracle/codemp/ghoul2/ghoul2_shared.h:405-410`
    pub fn get<'a>(&self, g2: &'a Ghoul2System, idx: i32) -> &'a CGhoul2Info {
        // Raven: `assert(mItem); assert(idx>=0&&idx<size()); return Array()[idx];`
        // (const overload). `Array()` folds into `g2.info_array.get(mItem)`.
        debug_assert_ne!(self.mItem, 0);
        debug_assert!(idx >= 0 && idx < self.size(g2));
        &g2.info_array.get(self.mItem)[idx as usize]
    }

    /// Raven `CGhoul2Info_v::resize` — allocates a slot first if null and
    /// `num` is non-zero, then resizes the instance vector.
    ///
    /// Source: `oracle/codemp/ghoul2/ghoul2_shared.h:411-425`
    pub fn resize(&mut self, g2: &mut Ghoul2System, num: i32) {
        // Raven: `assert(num>=0); if (num) { if (!mItem) { Alloc(); } }
        // if (mItem||num) { Array().resize(num); }`. `vector::resize(num)` grows
        // with default-constructed `CGhoul2Info`s / truncates.
        debug_assert!(num >= 0);
        if num != 0 && self.mItem == 0 {
            self.alloc(g2);
        }
        if self.mItem != 0 || num != 0 {
            g2.info_array
                .get_mut(self.mItem)
                .resize_with(num as usize, CGhoul2Info::default);
        }
    }

    /// Raven `CGhoul2Info_v::push_back` — allocates a slot first if null,
    /// then appends `model` to the instance vector.
    ///
    /// Source: `oracle/codemp/ghoul2/ghoul2_shared.h:430-437`
    pub fn push_back(&mut self, g2: &mut Ghoul2System, model: CGhoul2Info) {
        // Raven: `if (!mItem) { Alloc(); } Array().push_back(model);`.
        if self.mItem == 0 {
            self.alloc(g2);
        }
        g2.info_array.get_mut(self.mItem).push(model);
    }

    /// Raven `CGhoul2Info_v::size` — `0` when the handle is invalid, else the
    /// instance vector's length.
    ///
    /// Source: `oracle/codemp/ghoul2/ghoul2_shared.h:438-445`
    pub fn size(&self, g2: &Ghoul2System) -> i32 {
        // Raven: `if (!IsValid()) { return 0; } return Array().size();`.
        if !self.is_valid(g2) {
            return 0;
        }
        g2.info_array.get(self.mItem).len() as i32
    }

    /// Raven `CGhoul2Info_v::IsValid` — forwards to
    /// `Ghoul2InfoArray::IsValid(mItem)`.
    ///
    /// NOT named in the doc's method-transcription-table roster for this file
    /// (which lists only the `operator[]/size/resize/push_back` and
    /// `Alloc/Free/clear/DeepCopy/operator=` groups) — ported anyway as a
    /// doc-gap addition (reported to the caller) because it has live
    /// server-side callers this crate must serve: `G2API_CopyGhoul2Instance`
    /// (`G2_API.cpp:2245,2248`) and `G2API_CopySpecificG2Model` (`:2307`),
    /// both within `api_models.rs`'s already-rostered "Copy/Duplicate Ghoul2
    /// models" scope.
    ///
    /// Source: `oracle/codemp/ghoul2/ghoul2_shared.h:446-449`
    pub fn is_valid(&self, g2: &Ghoul2System) -> bool {
        // Raven: `return InfoArray().IsValid(mItem);`.
        g2.info_array.is_valid(self.mItem)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // The handle-copy `operator=` forms move only `mItem` and touch no arena
    // (`operator=(CGhoul2Info_v)` / `operator=(int)`) — the one pair of methods
    // testable without threading a `Ghoul2System` (whose sibling arena bodies are
    // still stubbed): the arena-forwarding methods (`alloc`/`free`/`resize`/…)
    // can't be exercised in isolation without those siblings being live.
    // Source: `oracle/codemp/ghoul2/ghoul2_shared.h:374-381`
    #[test]
    fn handle_copy_assignments_touch_no_arena() {
        let mut v = CGhoul2Info_v { mItem: 0 };
        v.assign_item(7);
        assert_eq!(v.mItem, 7);
        let other = CGhoul2Info_v { mItem: 42 };
        v.assign(&other);
        assert_eq!(v.mItem, 42);
    }
}
