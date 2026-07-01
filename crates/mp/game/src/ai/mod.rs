//! MP group-AI types (`ai.h`).

pub mod consts;
pub mod group_info;
pub mod group_member;

pub use consts::{MAX_FRAME_GROUPS, MAX_GROUP_MEMBERS, NUM_SQUAD_STATES};
pub use group_info::AIGroupInfo_t;
pub use group_member::AIGroupMember_t;
