//! The `w_saber.h` saber geometry / event constants that `bg_saber.c`/
//! `bg_saberLoad.c` consume, re-exported from their shared home in
//! `mp_qshared::common::mp::qcommon::saber::w_saber_consts`.

pub use mp_qshared::common::mp::qcommon::saber::w_saber_consts::{
    SABERMAXS_X, SABERMAXS_Y, SABERMAXS_Z, SABERMINS_X, SABERMINS_Y, SABERMINS_Z,
    SABER_MIN_THROW_DIST, SABER_RADIUS_STANDARD, SEF_LOCK_WON,
};
