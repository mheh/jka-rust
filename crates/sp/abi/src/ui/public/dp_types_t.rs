#![allow(non_camel_case_types, non_snake_case)]

/// Raven `dpTypes_t` — display panel type enumeration.
///
/// Type definition source: `oracle/code/ui/ui_public.h:143-149`
#[repr(i32)]
pub enum dpTypes_t {
	DP_HUD = 0,
	DP_OBJECTIVES,
	DP_WEAPONS,
	DP_INVENTORY,
	DP_FORCEPOWERS,
}
