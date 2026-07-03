//! `SlotId` — index into the `ModuleRegistry` slots (LOAD-D8/D12f).

/// Index into `slots[0..MAX_VM]`; `pub(crate)` per LOAD-D12f.
///
/// Source: `docs/architecture/module-loading.md` § Module registry (LOAD-D8).
pub struct SlotId(pub(crate) u32);
