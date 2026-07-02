#![allow(non_camel_case_types, non_snake_case)]

/// Raven `EPrimType` — effect primitive type enumeration.
///
/// Raven: (no comment).
/// Type definition source: `oracle/oracle/codemp/client/FxScheduler.h:120-136`
#[repr(i32)]
pub enum EPrimType {
	None = 0,
	Particle,			// sprite
	Line,
	Tail,				// comet-like tail thing
	Cylinder,
	Emitter,			// emits effects as it moves, can also attach a chunk
	Sound,
	Decal,				// projected onto architecture
	OrientedParticle,
	Electricity,
	FxRunner,
	Light,
	CameraShake,
	ScreenFlash,
}
