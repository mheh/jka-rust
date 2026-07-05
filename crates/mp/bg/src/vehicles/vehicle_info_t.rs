//! MP `vehicleInfo_t` copied from Raven `codemp/game/bg_vehicles.h`.

#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::{c_char, c_float, c_int};

use mp_qshared::shared::{qboolean, vec3_t};

// Fork-7 retired the fn-ptr slots, so `usercmd_t`/`bgEntity_t`/`Vehicle_t` (only
// referenced by those slot signatures) are no longer imported here.
use crate::vehicles::vehicle_s::{MAX_VEHICLE_MUZZLES, MAX_VEHICLE_TURRETS, MAX_VEHICLE_WEAPONS};
use crate::vehicles::{turretStats_t, vehWeaponStats_t, vehicleType_t};

/// Raven `vehicleInfo_t` — static per-vehicle-type data (parsed from `.veh`
/// files) plus the function-pointer "virtual" interface each vehicle
/// implementation fills in.
///
/// Raven: `//*** IMPORTANT!!! *** vehFields table correponds to this
/// structure!` — see the `vehFields` parsing table this layout must stay in
/// sync with.
/// Type definition source: `oracle/oracle/codemp/game/bg_vehicles.h:131-360`
#[repr(C)]
pub struct vehicleInfo_t {
    /// unique name of the vehicle
    pub name: *mut c_char,

    // general data
    /// what kind of vehicle
    pub r#type: vehicleType_t,
    /// if 2 hands, no weapons, if 1 hand, can use 1-handed weapons, if 0 hands, can use 2-handed weapons
    pub numHands: c_int,
    /// How far you can look up and down off the forward of the vehicle
    pub lookPitch: c_float,
    /// How far you can look left and right off the forward of the vehicle
    pub lookYaw: c_float,
    /// how long it is - used for body length traces when turning/moving?
    pub length: c_float,
    /// how wide it is - used for body length traces when turning/moving?
    pub width: c_float,
    /// how tall it is - used for body length traces when turning/moving?
    pub height: c_float,
    /// offset from origin: {forward, right, up} as a modifier on that dimension (-1.0f is all the way back, 1.0f is all the way forward)
    pub centerOfGravity: vec3_t,

    // speed stats
    /// top speed
    pub speedMax: c_float,
    /// turbo speed
    pub turboSpeed: c_float,
    /// if < 0, can go in reverse
    pub speedMin: c_float,
    /// what speed it drifts to when no accel/decel input is given
    pub speedIdle: c_float,
    /// if speedIdle > 0, how quickly it goes up to that speed
    pub accelIdle: c_float,
    /// when pressing on accelerator
    pub acceleration: c_float,
    /// when giving no input, how quickly it drops to speedIdle
    pub decelIdle: c_float,
    /// if true, speed stays at whatever you accel/decel to, unless you turbo or brake
    pub throttleSticks: c_float,
    /// multiplier on current speed for strafing.  If 1.0f, you can strafe at the same speed as you're going forward, 0.5 is half, 0 is no strafing
    pub strafePerc: c_float,

    // handling stats
    /// how quickly it pitches and rolls (not under player control)
    pub bankingSpeed: c_float,
    /// how far it can roll to either side
    pub rollLimit: c_float,
    /// how far it can roll forward or backward
    pub pitchLimit: c_float,
    /// when pressing on decelerator
    pub braking: c_float,
    /// The mouse yaw override.
    pub mouseYaw: c_float,
    /// The mouse pitch override.
    pub mousePitch: c_float,
    /// how quickly you can turn
    pub turningSpeed: c_float,
    /// whether or not you can turn when not moving
    pub turnWhenStopped: qboolean,
    /// how much your command input affects velocity
    pub traction: c_float,
    /// how much velocity is cut on its own
    pub friction: c_float,
    /// the max slope that it can go up with control
    pub maxSlope: c_float,
    /// vehicle turns faster the faster it's going
    pub speedDependantTurning: qboolean,

    // durability stats
    /// for momentum and impact force (player mass is 10)
    pub mass: c_int,
    /// total points of damage it can take
    pub armor: c_int,
    /// energy shield damage points
    pub shields: c_int,
    /// energy shield milliseconds per point recharged
    pub shieldRechargeMS: c_int,
    /// modifies incoming damage, 1.0 is normal, 0.5 is half, etc.  Simulates being made of tougher materials/construction
    pub toughness: c_float,
    /// when armor drops to or below this point, start malfunctioning
    pub malfunctionArmorLevel: c_int,
    /// can parts of this thing be torn off on impact? -rww
    pub surfDestruction: c_int,

    // individual "area" health -rww
    pub health_front: c_int,
    pub health_back: c_int,
    pub health_right: c_int,
    pub health_left: c_int,

    // visuals & sounds
    /// what model to use - if make it an NPC's primary model, don't need this?
    pub model: *mut c_char,
    /// what skin to use - if make it an NPC's primary model, don't need this?
    pub skin: *mut c_char,
    /// render radius for the ghoul2 model
    pub g2radius: c_int,
    /// what animation the rider uses
    pub riderAnim: c_int,
    /// what icon to show on radar in MP
    pub radarIconHandle: c_int,
    /// what image to use for the frame of the damage indicator
    pub dmgIndicFrameHandle: c_int,
    /// what image to use for the shield of the damage indicator
    pub dmgIndicShieldHandle: c_int,
    /// what image to use for the background of the damage indicator
    pub dmgIndicBackgroundHandle: c_int,
    /// what image to use for the front of the ship on the damage indicator
    pub iconFrontHandle: c_int,
    /// what image to use for the back of the ship on the damage indicator
    pub iconBackHandle: c_int,
    /// what image to use for the right of the ship on the damage indicator
    pub iconRightHandle: c_int,
    /// what image to use for the left of the ship on the damage indicator
    pub iconLeftHandle: c_int,
    /// what image to use for the left of the ship on the damage indicator
    pub crosshairShaderHandle: c_int,
    /// What shader to use when drawing the shield shell
    pub shieldShaderHandle: c_int,
    /// NPC to attach to *droidunit tag (if it exists in the model)
    pub droidNPC: *mut c_char,

    /// sound to play when get on it
    pub soundOn: c_int,
    /// sound to play when ship takes off
    pub soundTakeOff: c_int,
    /// sound to play when ship's thrusters first activate
    pub soundEngineStart: c_int,
    /// sound to loop while riding it
    pub soundLoop: c_int,
    /// sound to loop while spiraling out of control
    pub soundSpin: c_int,
    /// sound to play when turbo/afterburner kicks in
    pub soundTurbo: c_int,
    /// sound to play when ship lands
    pub soundHyper: c_int,
    /// sound to play when ship lands
    pub soundLand: c_int,
    /// sound to play when get off
    pub soundOff: c_int,
    /// sound to play when they buzz you
    pub soundFlyBy: c_int,
    /// alternate sound to play when they buzz you
    pub soundFlyBy2: c_int,
    /// sound to play when accelerating
    pub soundShift1: c_int,
    /// sound to play when accelerating
    pub soundShift2: c_int,
    /// sound to play when decelerating
    pub soundShift3: c_int,
    /// sound to play when decelerating
    pub soundShift4: c_int,

    /// exhaust effect, played from "*exhaust" bolt(s)
    pub iExhaustFX: c_int,
    /// turbo exhaust effect, played from "*exhaust" bolt(s) when ship is in "turbo" mode
    pub iTurboFX: c_int,
    /// turbo begin effect, played from "*exhaust" bolts when "turbo" mode begins
    pub iTurboStartFX: c_int,
    /// trail effect, played from "*trail" bolt(s)
    pub iTrailFX: c_int,
    /// impact effect, for when it bumps into something
    pub iImpactFX: c_int,
    /// explosion effect, for when it blows up (should have the sound built into explosion effect)
    pub iExplodeFX: c_int,
    /// effect it makes when going across water
    pub iWakeFX: c_int,
    /// effect to play on damage from a weapon or something
    pub iDmgFX: c_int,
    pub iInjureFX: c_int,
    /// effect for nose piece flying away when blown off
    pub iNoseFX: c_int,
    /// effect for left wing piece flying away when blown off
    pub iLWingFX: c_int,
    /// effect for right wing piece flying away when blown off
    pub iRWingFX: c_int,

    // Weapon stats
    pub weapon: [vehWeaponStats_t; MAX_VEHICLE_WEAPONS],

    /// Which weapon a muzzle fires (has to match one of the weapons this vehicle has). So 1 would be weapon 1,
    /// 2 would be weapon 2 and so on.
    pub weapMuzzle: [c_int; MAX_VEHICLE_MUZZLES],

    // turrets (if any) on the vehicle
    pub turret: [turretStats_t; MAX_VEHICLE_TURRETS],

    /// The max height before this ship (?) starts (auto)landing.
    pub landingHeight: c_float,

    // other misc stats
    /// normal is 800
    pub gravity: c_int,
    /// if 0, it's a ground vehicle
    pub hoverHeight: c_float,
    /// how hard it pushes off ground when less than hover height... causes "bounce", like shocks
    pub hoverStrength: c_float,
    /// can drive underwater if it has to
    pub waterProof: qboolean,
    /// when in water, how high it floats (1 is neutral bouyancy)
    pub bouyancy: c_float,
    /// how much fuel it can hold (capacity)
    pub fuelMax: c_int,
    /// how quickly is uses up fuel
    pub fuelRate: c_int,
    /// how long turbo lasts
    pub turboDuration: c_int,
    /// how long turbo takes to recharge
    pub turboRecharge: c_int,
    /// for sight alerts
    pub visibility: c_int,
    /// for sound alerts
    pub loudness: c_int,
    /// range of explosion
    pub explosionRadius: c_float,
    /// damage of explosion
    pub explosionDamage: c_int,

    /// The max number of passengers this vehicle may have (Default = 0).
    pub maxPassengers: c_int,
    /// rider (and passengers?) should not be drawn
    pub hideRider: qboolean,
    /// if rider is on vehicle when it dies, they should die
    pub killRiderOnDeath: qboolean,
    /// whether or not the vehicle should catch on fire before it explodes
    pub flammable: qboolean,
    /// how long the vehicle should be on fire/dying before it explodes
    pub explosionDelay: c_int,
    // camera stuff
    /// whether or not to use all of the following 3rd person camera override values
    pub cameraOverride: qboolean,
    /// how far back the camera should be - normal is 80
    pub cameraRange: c_float,
    /// how high over the vehicle origin the camera should be - normal is 16
    pub cameraVertOffset: c_float,
    /// how far to left/right (negative/positive) of of the vehicle origin the camera should be - normal is 0
    pub cameraHorzOffset: c_float,
    /// a modifier on the camera's pitch (up/down angle) to the vehicle - normal is 0
    pub cameraPitchOffset: c_float,
    /// third person camera FOV, default is 80
    pub cameraFOV: c_float,
    /// fade out the vehicle to this alpha (0.1-1.0f) if it's in the way of the crosshair
    pub cameraAlpha: c_float,
    /// use the hacky AT-ST pitch dependant vertical offset
    pub cameraPitchDependantVertOffset: qboolean,

    // NOTE: some info on what vehicle weapon to use?  Like ATST or TIE bomber or TIE fighter or X-Wing...?

    // THE FOLLOWING FIELDS are not in the vehFields table because they are
    // internal variables, not read in from the .veh file
    /// set internally, not until this vehicle is spawned into the level
    pub modelIndex: c_int,

    // Fork-7 (blessed 2026-07-03): the 25 `vehicleInfo_t` function-pointer
    // "virtual" slots (`AnimateVehicle`..`Inhabited`, Raven bg_vehicles.h:291-359)
    // are RETIRED. Raven filled them once at `.veh` load via `G_Set*VehicleFunctions`
    // and never reassigned/address-compared them; per porting-rules §C8/§F17 that
    // closed hierarchy is now `vehicleType_t`-keyed dispatch in
    // `crate::veh_dispatch` (game tier). This struct is bg/game-internal (never
    // crosses the engine ABI seam), so §D12 grants layout latitude: the trailing
    // fn-ptr region and the total-`size_of` static-assert are dropped; the parsed
    // `.veh` DATA fields above keep their exact offsets (the `vehFields` table
    // still indexes them).
}

#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(vehicleInfo_t, name) == 0);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(vehicleInfo_t, r#type) == 8);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(vehicleInfo_t, numHands) == 12);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(vehicleInfo_t, lookPitch) == 16);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(vehicleInfo_t, lookYaw) == 20);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(vehicleInfo_t, length) == 24);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(vehicleInfo_t, width) == 28);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(vehicleInfo_t, height) == 32);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(vehicleInfo_t, centerOfGravity) == 36);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(vehicleInfo_t, speedMax) == 48);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(vehicleInfo_t, turboSpeed) == 52);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(vehicleInfo_t, speedMin) == 56);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(vehicleInfo_t, speedIdle) == 60);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(vehicleInfo_t, accelIdle) == 64);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(vehicleInfo_t, acceleration) == 68);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(vehicleInfo_t, decelIdle) == 72);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(vehicleInfo_t, throttleSticks) == 76);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(vehicleInfo_t, strafePerc) == 80);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(vehicleInfo_t, bankingSpeed) == 84);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(vehicleInfo_t, rollLimit) == 88);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(vehicleInfo_t, pitchLimit) == 92);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(vehicleInfo_t, braking) == 96);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(vehicleInfo_t, mouseYaw) == 100);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(vehicleInfo_t, mousePitch) == 104);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(vehicleInfo_t, turningSpeed) == 108);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(vehicleInfo_t, turnWhenStopped) == 112);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(vehicleInfo_t, traction) == 116);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(vehicleInfo_t, friction) == 120);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(vehicleInfo_t, maxSlope) == 124);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(vehicleInfo_t, speedDependantTurning) == 128);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(vehicleInfo_t, mass) == 132);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(vehicleInfo_t, armor) == 136);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(vehicleInfo_t, shields) == 140);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(vehicleInfo_t, shieldRechargeMS) == 144);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(vehicleInfo_t, toughness) == 148);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(vehicleInfo_t, malfunctionArmorLevel) == 152);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(vehicleInfo_t, surfDestruction) == 156);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(vehicleInfo_t, health_front) == 160);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(vehicleInfo_t, health_back) == 164);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(vehicleInfo_t, health_right) == 168);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(vehicleInfo_t, health_left) == 172);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(vehicleInfo_t, model) == 176);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(vehicleInfo_t, skin) == 184);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(vehicleInfo_t, g2radius) == 192);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(vehicleInfo_t, riderAnim) == 196);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(vehicleInfo_t, radarIconHandle) == 200);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(vehicleInfo_t, dmgIndicFrameHandle) == 204);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(vehicleInfo_t, dmgIndicShieldHandle) == 208);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(vehicleInfo_t, dmgIndicBackgroundHandle) == 212);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(vehicleInfo_t, iconFrontHandle) == 216);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(vehicleInfo_t, iconBackHandle) == 220);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(vehicleInfo_t, iconRightHandle) == 224);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(vehicleInfo_t, iconLeftHandle) == 228);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(vehicleInfo_t, crosshairShaderHandle) == 232);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(vehicleInfo_t, shieldShaderHandle) == 236);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(vehicleInfo_t, droidNPC) == 240);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(vehicleInfo_t, soundOn) == 248);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(vehicleInfo_t, soundTakeOff) == 252);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(vehicleInfo_t, soundEngineStart) == 256);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(vehicleInfo_t, soundLoop) == 260);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(vehicleInfo_t, soundSpin) == 264);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(vehicleInfo_t, soundTurbo) == 268);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(vehicleInfo_t, soundHyper) == 272);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(vehicleInfo_t, soundLand) == 276);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(vehicleInfo_t, soundOff) == 280);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(vehicleInfo_t, soundFlyBy) == 284);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(vehicleInfo_t, soundFlyBy2) == 288);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(vehicleInfo_t, soundShift1) == 292);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(vehicleInfo_t, soundShift2) == 296);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(vehicleInfo_t, soundShift3) == 300);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(vehicleInfo_t, soundShift4) == 304);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(vehicleInfo_t, iExhaustFX) == 308);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(vehicleInfo_t, iTurboFX) == 312);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(vehicleInfo_t, iTurboStartFX) == 316);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(vehicleInfo_t, iTrailFX) == 320);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(vehicleInfo_t, iImpactFX) == 324);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(vehicleInfo_t, iExplodeFX) == 328);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(vehicleInfo_t, iWakeFX) == 332);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(vehicleInfo_t, iDmgFX) == 336);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(vehicleInfo_t, iInjureFX) == 340);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(vehicleInfo_t, iNoseFX) == 344);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(vehicleInfo_t, iLWingFX) == 348);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(vehicleInfo_t, iRWingFX) == 352);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(vehicleInfo_t, weapon) == 356);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(vehicleInfo_t, weapMuzzle) == 412);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(vehicleInfo_t, turret) == 464);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(vehicleInfo_t, landingHeight) == 656);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(vehicleInfo_t, gravity) == 660);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(vehicleInfo_t, hoverHeight) == 664);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(vehicleInfo_t, hoverStrength) == 668);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(vehicleInfo_t, waterProof) == 672);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(vehicleInfo_t, bouyancy) == 676);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(vehicleInfo_t, fuelMax) == 680);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(vehicleInfo_t, fuelRate) == 684);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(vehicleInfo_t, turboDuration) == 688);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(vehicleInfo_t, turboRecharge) == 692);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(vehicleInfo_t, visibility) == 696);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(vehicleInfo_t, loudness) == 700);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(vehicleInfo_t, explosionRadius) == 704);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(vehicleInfo_t, explosionDamage) == 708);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(vehicleInfo_t, maxPassengers) == 712);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(vehicleInfo_t, hideRider) == 716);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(vehicleInfo_t, killRiderOnDeath) == 720);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(vehicleInfo_t, flammable) == 724);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(vehicleInfo_t, explosionDelay) == 728);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(vehicleInfo_t, cameraOverride) == 732);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(vehicleInfo_t, cameraRange) == 736);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(vehicleInfo_t, cameraVertOffset) == 740);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(vehicleInfo_t, cameraHorzOffset) == 744);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(vehicleInfo_t, cameraPitchOffset) == 748);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(vehicleInfo_t, cameraFOV) == 752);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(vehicleInfo_t, cameraAlpha) == 756);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(vehicleInfo_t, cameraPitchDependantVertOffset) == 760);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(vehicleInfo_t, modelIndex) == 764);
