#![allow(non_camel_case_types, non_snake_case)]

/// Raven `netsrc_t` — network source: client or server.
///
/// Type definition source: `oracle/oracle/codemp/qcommon/qcommon.h:118-121`
#[repr(i32)]
pub enum netsrc_t {
    NS_CLIENT = 0,
    NS_SERVER = 1,
}
