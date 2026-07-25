//! MP UI local definitions (`ui_local.h`).
//!
//! Everything here is Class C by the frozen-vs-free census — ui registers no
//! shared memory with the engine — so the list/browser records land owned
//! (`String`, `bool`, `Vec`). The one exception is
//! [`post_game_info_s::postGameInfo_t`], whose bytes are `trap_FS_Read`/
//! `trap_FS_Write`d as a raw blob and therefore stay layout-frozen.
//!
//! PORT-NOTE (dropped surface, porting-rules §20 + DEC-36 D7):
//! * `uiStatic_t uis` — declared `extern` in `ui_local.h:882` with no
//!   definition anywhere in either tree (Q3 lineage); dropped.
//! * `lerpFrame_t`, `playerInfo_t` — the `ui_players.c` player-preview types.
//!   `ui_players.c` is absent from `ui.vcproj` and `ui.q3asm`, and
//!   `ui_main.c`'s only `playerInfo_t` use is inside the commented-out
//!   `UI_DrawOpponent` (`ui_main.c:2855-2885`); dropped.
//! * `characterInfo` — the `characterList` field it typed is commented out of
//!   `uiInfo_t` and the type has no other use; dropped.
//! * `awardType_t` — declared beside `UI_LogAwardData`/`UI_GetAwardLevel`,
//!   neither of which is defined in any compiled MP ui file; dropped.
//! * `menuframework_s`, `menucommon_s`, `mfield_t`, `menufield_s`,
//!   `menuslider_s`, `menulist_s`, `menuaction_s`, `menuradiobutton_s`,
//!   `menubitmap_s`, `menutext_s` — the pre-`ui_shared.c` widget set, whose
//!   implementing files (`ui_qmenu.c`, `ui_mfield.c`, `ui_menu.c`, …) do not
//!   exist in `codemp/ui`; dropped.

pub mod alias_info;
pub mod game_type_info;
pub mod map_info;
pub mod mod_info_t;
pub mod pending_server_status_t;
pub mod pending_server_t;
pub mod pinglist_t;
pub mod player_species_info_t;
pub mod post_game_info_s;
pub mod server_filter_s;
pub mod server_status_info_t;
pub mod server_status_s;
pub mod team_info;
pub mod tier_info;
