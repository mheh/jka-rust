#![allow(non_camel_case_types, non_snake_case)]

/// Raven `saying_t` — voice command acknowledgement/refusal/error responses.
///
/// Type definition source: `oracle/codemp/game/say.h:4-28`
/// Type definition source: `oracle/code/game/say.h:4-28`
#[repr(i32)]
pub enum saying_t {
    /// Acknowledge command
    SAY_ACKCOMM1 = 0,
    SAY_ACKCOMM2 = 1,
    SAY_ACKCOMM3 = 2,
    SAY_ACKCOMM4 = 3,
    /// Refuse command
    SAY_REFCOMM1 = 4,
    SAY_REFCOMM2 = 5,
    SAY_REFCOMM3 = 6,
    SAY_REFCOMM4 = 7,
    /// Bad command
    SAY_BADCOMM1 = 8,
    SAY_BADCOMM2 = 9,
    SAY_BADCOMM3 = 10,
    SAY_BADCOMM4 = 11,
    /// Unfinished hail
    SAY_BADHAIL1 = 12,
    SAY_BADHAIL2 = 13,
    SAY_BADHAIL3 = 14,
    SAY_BADHAIL4 = 15,
    NUM_SAYINGS = 16,
}
