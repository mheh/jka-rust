//! Zeroed heap construction — the Rust mirror of C static zero-initialization
//! for large all-zeroes-valid `#[repr(C)]` types (STATE-D9).

/// THE sanctioned construction idiom for large `#[repr(C)]` all-zeroes-valid
/// types: `alloc_zeroed` the storage and `Box::from_raw` it, so a large array is
/// built directly on the heap and never transits the stack (naive
/// stack-build-then-box risks overflow on constrained-stack targets).
///
/// # Safety
/// The caller guarantees `T` is `#[repr(C)]` and that the all-zero bit pattern is
/// a valid value of `T` (no `NonNull`/enum-niche/reference fields).
///
/// Source: `docs/architecture/state-ownership.md` § `zeroed_box` (STATE-D9).
pub fn zeroed_box<T>() -> Box<T> {
    todo!("Port zeroed_box — alloc_zeroed + Box::from_raw (STATE-D9)")
}
