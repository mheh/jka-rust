#![allow(non_camel_case_types, non_snake_case)]

/// Raven `saying_t` — voice command acknowledgement/refusal/error responses.
///
/// Type definition source: `oracle/oracle/code/game/say.h:4-28`
#[repr(i32)]
pub enum saying_t {
    /// Acknowledge command
    SAY_ACKCOMM1,
    SAY_ACKCOMM2,
    SAY_ACKCOMM3,
    SAY_ACKCOMM4,
    /// Refuse command
    SAY_REFCOMM1,
    SAY_REFCOMM2,
    SAY_REFCOMM3,
    SAY_REFCOMM4,
    /// Bad command
    SAY_BADCOMM1,
    SAY_BADCOMM2,
    SAY_BADCOMM3,
    SAY_BADCOMM4,
    /// Unfinished hail
    SAY_BADHAIL1,
    SAY_BADHAIL2,
    SAY_BADHAIL3,
    SAY_BADHAIL4,
    NUM_SAYINGS,
}
