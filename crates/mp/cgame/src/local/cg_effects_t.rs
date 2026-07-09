#![allow(non_camel_case_types, non_snake_case)]

use mp_qshared::shared::fxHandle_t;

/// Raven `cgEffects_t` — cached effect handles for cgame-side weapon/force/misc effects.
///
/// Type definition source: `oracle/codemp/cgame/cg_local.h:1385-1509`
#[repr(C)]
pub struct cgEffects_t {
	//concussion
	pub concussionShotEffect: fxHandle_t,
	pub concussionImpactEffect: fxHandle_t,

	// BRYAR PISTOL
	pub bryarShotEffect: fxHandle_t,
	pub bryarPowerupShotEffect: fxHandle_t,
	pub bryarWallImpactEffect: fxHandle_t,
	pub bryarWallImpactEffect2: fxHandle_t,
	pub bryarWallImpactEffect3: fxHandle_t,
	pub bryarFleshImpactEffect: fxHandle_t,
	pub bryarDroidImpactEffect: fxHandle_t,

	// BLASTER
	pub blasterShotEffect: fxHandle_t,
	pub blasterWallImpactEffect: fxHandle_t,
	pub blasterFleshImpactEffect: fxHandle_t,
	pub blasterDroidImpactEffect: fxHandle_t,

	// DISRUPTOR
	pub disruptorRingsEffect: fxHandle_t,
	pub disruptorProjectileEffect: fxHandle_t,
	pub disruptorWallImpactEffect: fxHandle_t,
	pub disruptorFleshImpactEffect: fxHandle_t,
	pub disruptorAltMissEffect: fxHandle_t,
	pub disruptorAltHitEffect: fxHandle_t,

	// BOWCASTER
	pub bowcasterShotEffect: fxHandle_t,
	pub bowcasterImpactEffect: fxHandle_t,

	// REPEATER
	pub repeaterProjectileEffect: fxHandle_t,
	pub repeaterAltProjectileEffect: fxHandle_t,
	pub repeaterWallImpactEffect: fxHandle_t,
	pub repeaterFleshImpactEffect: fxHandle_t,
	pub repeaterAltWallImpactEffect: fxHandle_t,

	// DEMP2
	pub demp2ProjectileEffect: fxHandle_t,
	pub demp2WallImpactEffect: fxHandle_t,
	pub demp2FleshImpactEffect: fxHandle_t,

	// FLECHETTE
	pub flechetteShotEffect: fxHandle_t,
	pub flechetteAltShotEffect: fxHandle_t,
	pub flechetteWallImpactEffect: fxHandle_t,
	pub flechetteFleshImpactEffect: fxHandle_t,

	// ROCKET
	pub rocketShotEffect: fxHandle_t,
	pub rocketExplosionEffect: fxHandle_t,

	// THERMAL
	pub thermalExplosionEffect: fxHandle_t,
	pub thermalShockwaveEffect: fxHandle_t,

	// TRIPMINE
	pub tripmineLaserFX: fxHandle_t,
	pub tripmineGlowFX: fxHandle_t,

	//FORCE
	pub forceLightning: fxHandle_t,
	pub forceLightningWide: fxHandle_t,

	pub forceDrain: fxHandle_t,
	pub forceDrainWide: fxHandle_t,
	pub forceDrained: fxHandle_t,

	//TURRET
	pub turretShotEffect: fxHandle_t,

	//Whatever
	pub itemCone: fxHandle_t,

	pub mSparks: fxHandle_t,
	pub mSaberCut: fxHandle_t,
	pub mTurretMuzzleFlash: fxHandle_t,
	pub mSaberBlock: fxHandle_t,
	pub mSaberBloodSparks: fxHandle_t,
	pub mSaberBloodSparksSmall: fxHandle_t,
	pub mSaberBloodSparksMid: fxHandle_t,
	pub mSpawn: fxHandle_t,
	pub mJediSpawn: fxHandle_t,
	pub mBlasterDeflect: fxHandle_t,
	pub mBlasterSmoke: fxHandle_t,
	pub mForceConfustionOld: fxHandle_t,
	pub mDisruptorDeathSmoke: fxHandle_t,
	pub mSparkExplosion: fxHandle_t,
	pub mTurretExplode: fxHandle_t,
	pub mEmplacedExplode: fxHandle_t,
	pub mEmplacedDeadSmoke: fxHandle_t,
	pub mTripmineExplosion: fxHandle_t,
	pub mDetpackExplosion: fxHandle_t,
	pub mFlechetteAltBlow: fxHandle_t,
	pub mStunBatonFleshImpact: fxHandle_t,
	pub mAltDetonate: fxHandle_t,
	pub mSparksExplodeNoSound: fxHandle_t,
	pub mTripMineLaster: fxHandle_t,
	pub mEmplacedMuzzleFlash: fxHandle_t,
	pub mConcussionAltRing: fxHandle_t,
	pub mHyperspaceStars: fxHandle_t,
	pub mBlackSmoke: fxHandle_t,
	pub mShipDestDestroyed: fxHandle_t,
	pub mShipDestBurning: fxHandle_t,
	pub mBobaJet: fxHandle_t,

	//footstep effects
	pub footstepMud: fxHandle_t,
	pub footstepSand: fxHandle_t,
	pub footstepSnow: fxHandle_t,
	pub footstepGravel: fxHandle_t,
	//landing effects
	pub landingMud: fxHandle_t,
	pub landingSand: fxHandle_t,
	pub landingDirt: fxHandle_t,
	pub landingSnow: fxHandle_t,
	pub landingGravel: fxHandle_t,
	//splashes
	pub waterSplash: fxHandle_t,
	pub lavaSplash: fxHandle_t,
	pub acidSplash: fxHandle_t,
}

const _: () = assert!(core::mem::size_of::<cgEffects_t>() == 356);
const _: () = assert!(core::mem::offset_of!(cgEffects_t, concussionShotEffect) == 0);
const _: () = assert!(core::mem::offset_of!(cgEffects_t, concussionImpactEffect) == 4);
const _: () = assert!(core::mem::offset_of!(cgEffects_t, bryarShotEffect) == 8);
const _: () = assert!(core::mem::offset_of!(cgEffects_t, bryarPowerupShotEffect) == 12);
const _: () = assert!(core::mem::offset_of!(cgEffects_t, bryarWallImpactEffect) == 16);
const _: () = assert!(core::mem::offset_of!(cgEffects_t, bryarWallImpactEffect2) == 20);
const _: () = assert!(core::mem::offset_of!(cgEffects_t, bryarWallImpactEffect3) == 24);
const _: () = assert!(core::mem::offset_of!(cgEffects_t, bryarFleshImpactEffect) == 28);
const _: () = assert!(core::mem::offset_of!(cgEffects_t, bryarDroidImpactEffect) == 32);
const _: () = assert!(core::mem::offset_of!(cgEffects_t, blasterShotEffect) == 36);
const _: () = assert!(core::mem::offset_of!(cgEffects_t, blasterWallImpactEffect) == 40);
const _: () = assert!(core::mem::offset_of!(cgEffects_t, blasterFleshImpactEffect) == 44);
const _: () = assert!(core::mem::offset_of!(cgEffects_t, blasterDroidImpactEffect) == 48);
const _: () = assert!(core::mem::offset_of!(cgEffects_t, disruptorRingsEffect) == 52);
const _: () = assert!(core::mem::offset_of!(cgEffects_t, disruptorProjectileEffect) == 56);
const _: () = assert!(core::mem::offset_of!(cgEffects_t, disruptorWallImpactEffect) == 60);
const _: () = assert!(core::mem::offset_of!(cgEffects_t, disruptorFleshImpactEffect) == 64);
const _: () = assert!(core::mem::offset_of!(cgEffects_t, disruptorAltMissEffect) == 68);
const _: () = assert!(core::mem::offset_of!(cgEffects_t, disruptorAltHitEffect) == 72);
const _: () = assert!(core::mem::offset_of!(cgEffects_t, bowcasterShotEffect) == 76);
const _: () = assert!(core::mem::offset_of!(cgEffects_t, bowcasterImpactEffect) == 80);
const _: () = assert!(core::mem::offset_of!(cgEffects_t, repeaterProjectileEffect) == 84);
const _: () = assert!(core::mem::offset_of!(cgEffects_t, repeaterAltProjectileEffect) == 88);
const _: () = assert!(core::mem::offset_of!(cgEffects_t, repeaterWallImpactEffect) == 92);
const _: () = assert!(core::mem::offset_of!(cgEffects_t, repeaterFleshImpactEffect) == 96);
const _: () = assert!(core::mem::offset_of!(cgEffects_t, repeaterAltWallImpactEffect) == 100);
const _: () = assert!(core::mem::offset_of!(cgEffects_t, demp2ProjectileEffect) == 104);
const _: () = assert!(core::mem::offset_of!(cgEffects_t, demp2WallImpactEffect) == 108);
const _: () = assert!(core::mem::offset_of!(cgEffects_t, demp2FleshImpactEffect) == 112);
const _: () = assert!(core::mem::offset_of!(cgEffects_t, flechetteShotEffect) == 116);
const _: () = assert!(core::mem::offset_of!(cgEffects_t, flechetteAltShotEffect) == 120);
const _: () = assert!(core::mem::offset_of!(cgEffects_t, flechetteWallImpactEffect) == 124);
const _: () = assert!(core::mem::offset_of!(cgEffects_t, flechetteFleshImpactEffect) == 128);
const _: () = assert!(core::mem::offset_of!(cgEffects_t, rocketShotEffect) == 132);
const _: () = assert!(core::mem::offset_of!(cgEffects_t, rocketExplosionEffect) == 136);
const _: () = assert!(core::mem::offset_of!(cgEffects_t, thermalExplosionEffect) == 140);
const _: () = assert!(core::mem::offset_of!(cgEffects_t, thermalShockwaveEffect) == 144);
const _: () = assert!(core::mem::offset_of!(cgEffects_t, tripmineLaserFX) == 148);
const _: () = assert!(core::mem::offset_of!(cgEffects_t, tripmineGlowFX) == 152);
const _: () = assert!(core::mem::offset_of!(cgEffects_t, forceLightning) == 156);
const _: () = assert!(core::mem::offset_of!(cgEffects_t, forceLightningWide) == 160);
const _: () = assert!(core::mem::offset_of!(cgEffects_t, forceDrain) == 164);
const _: () = assert!(core::mem::offset_of!(cgEffects_t, forceDrainWide) == 168);
const _: () = assert!(core::mem::offset_of!(cgEffects_t, forceDrained) == 172);
const _: () = assert!(core::mem::offset_of!(cgEffects_t, turretShotEffect) == 176);
const _: () = assert!(core::mem::offset_of!(cgEffects_t, itemCone) == 180);
const _: () = assert!(core::mem::offset_of!(cgEffects_t, mSparks) == 184);
const _: () = assert!(core::mem::offset_of!(cgEffects_t, mSaberCut) == 188);
const _: () = assert!(core::mem::offset_of!(cgEffects_t, mTurretMuzzleFlash) == 192);
const _: () = assert!(core::mem::offset_of!(cgEffects_t, mSaberBlock) == 196);
const _: () = assert!(core::mem::offset_of!(cgEffects_t, mSaberBloodSparks) == 200);
const _: () = assert!(core::mem::offset_of!(cgEffects_t, mSaberBloodSparksSmall) == 204);
const _: () = assert!(core::mem::offset_of!(cgEffects_t, mSaberBloodSparksMid) == 208);
const _: () = assert!(core::mem::offset_of!(cgEffects_t, mSpawn) == 212);
const _: () = assert!(core::mem::offset_of!(cgEffects_t, mJediSpawn) == 216);
const _: () = assert!(core::mem::offset_of!(cgEffects_t, mBlasterDeflect) == 220);
const _: () = assert!(core::mem::offset_of!(cgEffects_t, mBlasterSmoke) == 224);
const _: () = assert!(core::mem::offset_of!(cgEffects_t, mForceConfustionOld) == 228);
const _: () = assert!(core::mem::offset_of!(cgEffects_t, mDisruptorDeathSmoke) == 232);
const _: () = assert!(core::mem::offset_of!(cgEffects_t, mSparkExplosion) == 236);
const _: () = assert!(core::mem::offset_of!(cgEffects_t, mTurretExplode) == 240);
const _: () = assert!(core::mem::offset_of!(cgEffects_t, mEmplacedExplode) == 244);
const _: () = assert!(core::mem::offset_of!(cgEffects_t, mEmplacedDeadSmoke) == 248);
const _: () = assert!(core::mem::offset_of!(cgEffects_t, mTripmineExplosion) == 252);
const _: () = assert!(core::mem::offset_of!(cgEffects_t, mDetpackExplosion) == 256);
const _: () = assert!(core::mem::offset_of!(cgEffects_t, mFlechetteAltBlow) == 260);
const _: () = assert!(core::mem::offset_of!(cgEffects_t, mStunBatonFleshImpact) == 264);
const _: () = assert!(core::mem::offset_of!(cgEffects_t, mAltDetonate) == 268);
const _: () = assert!(core::mem::offset_of!(cgEffects_t, mSparksExplodeNoSound) == 272);
const _: () = assert!(core::mem::offset_of!(cgEffects_t, mTripMineLaster) == 276);
const _: () = assert!(core::mem::offset_of!(cgEffects_t, mEmplacedMuzzleFlash) == 280);
const _: () = assert!(core::mem::offset_of!(cgEffects_t, mConcussionAltRing) == 284);
const _: () = assert!(core::mem::offset_of!(cgEffects_t, mHyperspaceStars) == 288);
const _: () = assert!(core::mem::offset_of!(cgEffects_t, mBlackSmoke) == 292);
const _: () = assert!(core::mem::offset_of!(cgEffects_t, mShipDestDestroyed) == 296);
const _: () = assert!(core::mem::offset_of!(cgEffects_t, mShipDestBurning) == 300);
const _: () = assert!(core::mem::offset_of!(cgEffects_t, mBobaJet) == 304);
const _: () = assert!(core::mem::offset_of!(cgEffects_t, footstepMud) == 308);
const _: () = assert!(core::mem::offset_of!(cgEffects_t, footstepSand) == 312);
const _: () = assert!(core::mem::offset_of!(cgEffects_t, footstepSnow) == 316);
const _: () = assert!(core::mem::offset_of!(cgEffects_t, footstepGravel) == 320);
const _: () = assert!(core::mem::offset_of!(cgEffects_t, landingMud) == 324);
const _: () = assert!(core::mem::offset_of!(cgEffects_t, landingSand) == 328);
const _: () = assert!(core::mem::offset_of!(cgEffects_t, landingDirt) == 332);
const _: () = assert!(core::mem::offset_of!(cgEffects_t, landingSnow) == 336);
const _: () = assert!(core::mem::offset_of!(cgEffects_t, landingGravel) == 340);
const _: () = assert!(core::mem::offset_of!(cgEffects_t, waterSplash) == 344);
const _: () = assert!(core::mem::offset_of!(cgEffects_t, lavaSplash) == 348);
const _: () = assert!(core::mem::offset_of!(cgEffects_t, acidSplash) == 352);
