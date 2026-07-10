#![allow(non_camel_case_types, non_snake_case)]

/// `MAX_SNAPSHOT_ENTITIES`.
///
/// Source: `oracle/codemp/server/sv_snapshot.cpp:245`
pub const MAX_SNAPSHOT_ENTITIES: usize = 1024;

/// Raven `snapshotEntityNumbers_t` — the working set of entity numbers gathered
/// while building a client snapshot.
///
/// Type definition source: `oracle/codemp/server/sv_snapshot.cpp:246-249`
#[repr(C)]
pub struct snapshotEntityNumbers_t {
	pub numSnapshotEntities: i32,
	pub snapshotEntities: [i32; MAX_SNAPSHOT_ENTITIES],
}

const _: () = assert!(core::mem::size_of::<snapshotEntityNumbers_t>() == 4100);
const _: () = assert!(core::mem::offset_of!(snapshotEntityNumbers_t, numSnapshotEntities) == 0);
const _: () = assert!(core::mem::offset_of!(snapshotEntityNumbers_t, snapshotEntities) == 4);
