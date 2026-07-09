#![allow(non_camel_case_types, non_snake_case)]

use sp_qshared::shared::fxHandle_t;

/// Raven `cgEffects_t` — cached effect handles for weapon, Force power, and
/// footstep/landing effects.
///
/// Type definition source: `oracle/code/cgame/cg_media.h:311-362`
#[repr(C)]
pub struct cgEffects_t {
    // BRYAR PISTOL
    pub bryarShotEffect: fxHandle_t,
    pub bryarPowerupShotEffect: fxHandle_t,
    pub bryarWallImpactEffect: fxHandle_t,
    pub bryarWallImpactEffect2: fxHandle_t,
    pub bryarWallImpactEffect3: fxHandle_t,
    pub bryarFleshImpactEffect: fxHandle_t,

    // BLASTER
    pub blasterShotEffect: fxHandle_t,
    pub blasterOverchargeEffect: fxHandle_t,
    pub blasterWallImpactEffect: fxHandle_t,
    pub blasterFleshImpactEffect: fxHandle_t,

    // BOWCASTER
    pub bowcasterShotEffect: fxHandle_t,
    pub bowcasterBounceEffect: fxHandle_t,
    pub bowcasterImpactEffect: fxHandle_t,

    // FLECHETTE
    pub flechetteShotEffect: fxHandle_t,
    pub flechetteAltShotEffect: fxHandle_t,
    pub flechetteShotDeathEffect: fxHandle_t,
    pub flechetteFleshImpactEffect: fxHandle_t,
    pub flechetteRicochetEffect: fxHandle_t,

    // FORCE
    pub forceConfusion: fxHandle_t,
    pub forceLightning: fxHandle_t,
    pub forceLightningWide: fxHandle_t,
    // fxHandle_t forceInvincibility;
    pub forceHeal: fxHandle_t,

    // new stuff for Jedi Academy
    pub forceDrain: fxHandle_t,
    pub forceDrainWide: fxHandle_t,
    pub forceDrained: fxHandle_t,

    // footstep effects
    pub footstepMud: fxHandle_t,
    pub footstepSand: fxHandle_t,
    pub footstepSnow: fxHandle_t,
    pub footstepGravel: fxHandle_t,
    // landing effects
    pub landingMud: fxHandle_t,
    pub landingSand: fxHandle_t,
    pub landingDirt: fxHandle_t,
    pub landingSnow: fxHandle_t,
    pub landingGravel: fxHandle_t,
}

const _: () = assert!(core::mem::size_of::<cgEffects_t>() == 136);
const _: () = assert!(core::mem::offset_of!(cgEffects_t, bryarShotEffect) == 0);
const _: () = assert!(core::mem::offset_of!(cgEffects_t, bryarPowerupShotEffect) == 4);
const _: () = assert!(core::mem::offset_of!(cgEffects_t, bryarWallImpactEffect) == 8);
const _: () = assert!(core::mem::offset_of!(cgEffects_t, bryarWallImpactEffect2) == 12);
const _: () = assert!(core::mem::offset_of!(cgEffects_t, bryarWallImpactEffect3) == 16);
const _: () = assert!(core::mem::offset_of!(cgEffects_t, bryarFleshImpactEffect) == 20);
const _: () = assert!(core::mem::offset_of!(cgEffects_t, blasterShotEffect) == 24);
const _: () = assert!(core::mem::offset_of!(cgEffects_t, blasterOverchargeEffect) == 28);
const _: () = assert!(core::mem::offset_of!(cgEffects_t, blasterWallImpactEffect) == 32);
const _: () = assert!(core::mem::offset_of!(cgEffects_t, blasterFleshImpactEffect) == 36);
const _: () = assert!(core::mem::offset_of!(cgEffects_t, bowcasterShotEffect) == 40);
const _: () = assert!(core::mem::offset_of!(cgEffects_t, bowcasterBounceEffect) == 44);
const _: () = assert!(core::mem::offset_of!(cgEffects_t, bowcasterImpactEffect) == 48);
const _: () = assert!(core::mem::offset_of!(cgEffects_t, flechetteShotEffect) == 52);
const _: () = assert!(core::mem::offset_of!(cgEffects_t, flechetteAltShotEffect) == 56);
const _: () = assert!(core::mem::offset_of!(cgEffects_t, flechetteShotDeathEffect) == 60);
const _: () = assert!(core::mem::offset_of!(cgEffects_t, flechetteFleshImpactEffect) == 64);
const _: () = assert!(core::mem::offset_of!(cgEffects_t, flechetteRicochetEffect) == 68);
const _: () = assert!(core::mem::offset_of!(cgEffects_t, forceConfusion) == 72);
const _: () = assert!(core::mem::offset_of!(cgEffects_t, forceLightning) == 76);
const _: () = assert!(core::mem::offset_of!(cgEffects_t, forceLightningWide) == 80);
const _: () = assert!(core::mem::offset_of!(cgEffects_t, forceHeal) == 84);
const _: () = assert!(core::mem::offset_of!(cgEffects_t, forceDrain) == 88);
const _: () = assert!(core::mem::offset_of!(cgEffects_t, forceDrainWide) == 92);
const _: () = assert!(core::mem::offset_of!(cgEffects_t, forceDrained) == 96);
const _: () = assert!(core::mem::offset_of!(cgEffects_t, footstepMud) == 100);
const _: () = assert!(core::mem::offset_of!(cgEffects_t, footstepSand) == 104);
const _: () = assert!(core::mem::offset_of!(cgEffects_t, footstepSnow) == 108);
const _: () = assert!(core::mem::offset_of!(cgEffects_t, footstepGravel) == 112);
const _: () = assert!(core::mem::offset_of!(cgEffects_t, landingMud) == 116);
const _: () = assert!(core::mem::offset_of!(cgEffects_t, landingSand) == 120);
const _: () = assert!(core::mem::offset_of!(cgEffects_t, landingDirt) == 124);
const _: () = assert!(core::mem::offset_of!(cgEffects_t, landingSnow) == 128);
const _: () = assert!(core::mem::offset_of!(cgEffects_t, landingGravel) == 132);
