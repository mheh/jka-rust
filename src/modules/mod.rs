//! Runtime module implementations split by Raven source line and module surface.
//!
//! Raven keeps `game`, `cgame`, and `ui` as peer source/build products under both
//! SP and MP trees. Engine-side systems such as `qcommon`, `renderer`, `server`,
//! `client`, `ghoul2`, `icarus`, and `botlib` live outside this axis.

pub mod mp;
pub mod sp;
