//! The module-visible Ghoul2 token (DEC-65 ruling 3).
//!
//! Every ghoul2 reference that leaves the engine crosses as `Ghoul2Handle + 1`, cast to pointer width.
//! Null round-trips to `None`, because handle `0` is the always-invalid arena id (`info_array.rs:132-137`).
//! One scheme serves the module `void*` slots and `refEntity_t.ghoul2`.
//! That is what lets cgame copy its slot value straight into the render entity, and the renderer decode it.
//! Raw ghoul2 pointers never leave the engine.
//!
//! The scheme is live at every seam as of 2026-08-03, which closes the split `tr_scene.rs` used to flag.
//! The render trap decoded tokens while `sv_game.rs` handed out `Box<CGhoul2Info_v>` pointers in the same `void*` slot.
//! All 115 ghoul2 trap arms in `sv_game.rs`, `cl_cgame.rs`, and `cl_ui.rs`, plus the two `sv_world.rs` slot readers, now decode this token.

use core::ffi::c_void;

use crate::info_array::Ghoul2Handle;

/// Decodes a module-visible ghoul2 token into a [`Ghoul2Handle`].
/// A null token reads as no instance.
pub fn ghoul2_token_decode(token: *mut c_void) -> Option<Ghoul2Handle> {
    if token.is_null() {
        None
    } else {
        Some(Ghoul2Handle(token as i32 - 1))
    }
}

/// Encodes a [`Ghoul2Handle`] back into the module-visible token.
/// The inverse of [`ghoul2_token_decode`], and `None` encodes as null.
pub fn ghoul2_token_encode(handle: Option<Ghoul2Handle>) -> *mut c_void {
    match handle {
        Some(h) => (h.0 + 1) as *mut c_void,
        None => core::ptr::null_mut(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn null_token_round_trips_to_none() {
        assert!(ghoul2_token_decode(ghoul2_token_encode(None)).is_none());
        assert!(ghoul2_token_encode(None).is_null());
    }

    #[test]
    fn handle_round_trips_through_the_token() {
        let handle = Ghoul2Handle(1024);
        let token = ghoul2_token_encode(Some(handle));
        assert!(!token.is_null());
        assert_eq!(ghoul2_token_decode(token), Some(handle));
    }

    /// Handle `0` is a real value on the arena side, and it must not collide with the null token.
    #[test]
    fn zero_handle_is_not_the_null_token() {
        let token = ghoul2_token_encode(Some(Ghoul2Handle(0)));
        assert!(!token.is_null());
        assert_eq!(ghoul2_token_decode(token), Some(Ghoul2Handle(0)));
    }
}
