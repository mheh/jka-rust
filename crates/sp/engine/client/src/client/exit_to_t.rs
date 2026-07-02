#![allow(non_camel_case_types, non_snake_case)]

/// Raven `exitTo_t` — exit destination on game shutdown.
///
/// Type definition source: `oracle/oracle/code/client/client.h:160-165`
#[repr(i32)]
pub enum exitTo_t {
    EXIT_CONSOLE = 0,
    EXIT_ARENAS = 1,
    EXIT_SERVERS = 2,
    /// Quit all the way out of the game on disconnect.
    EXIT_LAUNCH = 3,
}
