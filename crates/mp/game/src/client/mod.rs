//! MP server-side player client (`g_local.h`). Ported fresh from oracle.

pub mod client_connected;
pub mod client_persistant;
pub mod client_session;
pub mod gclient;
pub mod player_team_state;
pub mod render_info;
pub mod spectator_state;

pub use client_connected::{clientConnected_t, CON_CONNECTED, CON_CONNECTING, CON_DISCONNECTED};
pub use client_persistant::{
    clientPersistant_t, MAX_NETNAME, MAX_VOTE_COUNT, PSG_TEAMVOTED, PSG_VOTED,
};
pub use client_session::{clientSession_t, FOLLOW_ACTIVE1, FOLLOW_ACTIVE2};
pub use gclient::{gclient_s, gclient_t};
pub use render_info::renderInfo_t;
// `spectatorState_t` / `playerTeamStateState_t` are named enums (per oracle), so
// their members are `T::VARIANT`, not free consts.
pub use player_team_state::{playerTeamStateState_t, playerTeamState_t};
pub use spectator_state::spectatorState_t;
