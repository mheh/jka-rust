#![allow(non_camel_case_types, non_snake_case)]

/// Raven `CFxRange` — a min/max float range used by the fx scheduler.
///
/// Raven: (none).
/// Type definition source: `oracle/oracle/codemp/client/FxScheduler.h:91-113`
#[repr(C)]
pub struct CFxRange {
    mMin: f32,
    mMax: f32,
}

const _: () = assert!(core::mem::size_of::<CFxRange>() == 8);
const _: () = assert!(core::mem::offset_of!(CFxRange, mMin) == 0);
const _: () = assert!(core::mem::offset_of!(CFxRange, mMax) == 4);
