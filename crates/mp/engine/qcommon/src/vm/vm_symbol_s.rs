#![allow(non_camel_case_types, non_snake_case)]

/// Raven `vmSymbol_t` — a single debug/profiling symbol entry in a VM's symbol table.
///
/// Raven: `symName[1]` is variable sized — the real allocation trails extra bytes
/// for the name past this one-byte placeholder.
/// Type definition source: `oracle/codemp/qcommon/vm_local.h:101-106`
#[repr(C)]
pub struct vmSymbol_t {
	pub next: *mut vmSymbol_t,
	pub symValue: i32,
	pub profileCount: i32,
	// variable sized
	pub symName: [core::ffi::c_char; 1],
}

/// Raven C tag `vmSymbol_s` for the same type.
pub type vmSymbol_s = vmSymbol_t;

const _: () = assert!(core::mem::size_of::<vmSymbol_t>() == 24);
const _: () = assert!(core::mem::offset_of!(vmSymbol_t, next) == 0);
const _: () = assert!(core::mem::offset_of!(vmSymbol_t, symValue) == 8);
const _: () = assert!(core::mem::offset_of!(vmSymbol_t, profileCount) == 12);
const _: () = assert!(core::mem::offset_of!(vmSymbol_t, symName) == 16);
