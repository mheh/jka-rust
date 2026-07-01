//! MP `qtime_t` copied from Raven `codemp/game/q_shared.h`.
//!
//! Source: `oracle/oracle/codemp/game/q_shared.h:3009-3021`

#![allow(non_camel_case_types)]

use core::ffi::c_int;

/// Raven `qtime_t`.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct qtime_t {
    /// Raven `tm_sec`: seconds after the minute - [0,59]
    pub tm_sec: c_int,
    /// Raven `tm_min`: minutes after the hour - [0,59]
    pub tm_min: c_int,
    /// Raven `tm_hour`: hours since midnight - [0,23]
    pub tm_hour: c_int,
    /// Raven `tm_mday`: day of the month - [1,31]
    pub tm_mday: c_int,
    /// Raven `tm_mon`: months since January - [0,11]
    pub tm_mon: c_int,
    /// Raven `tm_year`: years since 1900
    pub tm_year: c_int,
    /// Raven `tm_wday`: days since Sunday - [0,6]
    pub tm_wday: c_int,
    /// Raven `tm_yday`: days since January 1 - [0,365]
    pub tm_yday: c_int,
    /// Raven `tm_isdst`: daylight savings time flag
    pub tm_isdst: c_int,
}
