#![allow(non_camel_case_types, non_snake_case)]

/// Raven `dynamicMusic_t` — dynamic music state enumeration.
///
/// Type definition source: `oracle/code/game/dmstates.h:5-13`
#[repr(i32)]
pub enum dynamicMusic_t {
	/// Let the game determine the dynamic music as normal.
	DM_AUTO,
	/// Stop the music.
	DM_SILENCE,
	/// Force the exploration music to play.
	DM_EXPLORE,
	/// Force the action music to play.
	DM_ACTION,
	/// Force the boss battle music to play (if there is any).
	DM_BOSS,
	/// Force the "player dead" music to play.
	DM_DEATH,
}
