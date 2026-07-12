#![allow(non_camel_case_types, non_snake_case)]

/// Raven `ffFX_e` — force-feedback effect types.
///
/// Type definition source: `oracle/codemp/client/fffx.h:13-57`
#[repr(i32)]
pub enum ffFX_e {
    fffx_RandomNoise = 0,
    fffx_AircraftCarrierTakeOff, // this one is pointless / dumb
    fffx_BasketballDribble,
    fffx_CarEngineIdle,
    fffx_ChainsawIdle,
    fffx_ChainsawInAction,
    fffx_DieselEngineIdle,
    fffx_Jump,
    fffx_Land,
    fffx_MachineGun,
    fffx_Punched,
    fffx_RocketLaunch,
    fffx_SecretDoor,
    fffx_SwitchClick,
    fffx_WindGust,
    fffx_WindShear, // also pretty crap
    fffx_Pistol,
    fffx_Shotgun,
    fffx_Laser1,
    fffx_Laser2,
    fffx_Laser3,
    fffx_Laser4,
    fffx_Laser5,
    fffx_Laser6,
    fffx_OutOfAmmo,
    fffx_LightningGun,
    fffx_Missile,
    fffx_GatlingGun,
    fffx_ShortPlasma,
    fffx_PlasmaCannon1,
    fffx_PlasmaCannon2,
    fffx_Cannon,
    fffx_NUMBEROF,
    fffx_NULL, // special use, ignore during array mallocs etc, use fffx_NUMBEROF instead
}
