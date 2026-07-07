//! Zeroed heap construction — the Rust mirror of C static zero-initialization
//! for large all-zeroes-valid `#[repr(C)]` types (STATE-D9; form settled by the
//! round-5 STATE-Q10 resolution: safe fn bounded by an unsafe marker trait).

/// Marker trait: the all-zero bit pattern is a valid value of `Self`.
///
/// Hand-rolled `bytemuck::Zeroable` style (round-5 STATE-Q10 resolution — no
/// new dependency): implementing it is the `unsafe` step, one line per type,
/// colocated with that type's layout static-asserts; every `zeroed_box` call
/// site stays safe (porting-rules §D11 confinement).
///
/// # Safety
/// Implementors guarantee `Self` is `#[repr(C)]` and that all-zero bytes are a
/// valid `Self` (no `NonNull`/enum-niche/reference fields).
pub unsafe trait ZeroValid {}

// Arrays of zero-valid elements are zero-valid (the bytemuck-Zeroable array
// rule; the GameWorld `[gentity_t; MAX_GENTITIES]` boxes build through this).
unsafe impl<T: ZeroValid, const N: usize> ZeroValid for [T; N] {}

// Primitive integer byte-patterns: all-zero is a valid value. `u8` backs
// `GameWorld`'s raw scratch `memoryPool` byte array; `i8` backs `c_char` on
// the targets where it is signed (x86_64), so `[c_char; N]` heap buffers
// (e.g. `gBotChatBuffer`) can build through `zeroed_box` on every platform.
unsafe impl ZeroValid for u8 {}
unsafe impl ZeroValid for i8 {}

/// THE sanctioned construction idiom for large `#[repr(C)]` all-zeroes-valid
/// types: `alloc_zeroed` the storage and `Box::from_raw` it, so a large array is
/// built directly on the heap and never transits the stack (naive
/// stack-build-then-box risks overflow on constrained-stack targets). Safe: the
/// all-zero-valid precondition is carried by the `ZeroValid` bound (STATE-Q10,
/// round-5 resolution).
///
/// Source: `docs/architecture/state-ownership.md` § `zeroed_box` (STATE-D9).
pub fn zeroed_box<T: ZeroValid>() -> Box<T> {
    use std::alloc::{alloc_zeroed, handle_alloc_error, Layout};
    let layout = Layout::new::<T>();
    if layout.size() == 0 {
        // ZST: all-zero is trivially the (only) value; no allocation.
        return unsafe { Box::from_raw(core::ptr::NonNull::<T>::dangling().as_ptr()) };
    }
    // SAFETY: the ZeroValid bound carries the all-zero-validity contract; the
    // allocation is exactly Layout::new::<T>() and ownership passes to the Box.
    unsafe {
        let p = alloc_zeroed(layout) as *mut T;
        if p.is_null() {
            handle_alloc_error(layout);
        }
        Box::from_raw(p)
    }
}
