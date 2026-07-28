//! Port of `oracle/codemp/cgame/cg_playerstate.c` — local player-state transitions and reward/damage feedback. Functions land via the C5
//! transcription waves.

#![allow(non_snake_case)]

use core::ffi::c_int;

use mp_bg::public::stat_index::statIndex_t::STAT_HEALTH;
use mp_qshared::shared::q_math::{
    _DotProduct, _VectorSubtract, vec3_origin, AngleVectors, VectorLength, PITCH, ROLL, YAW,
};
use mp_qshared::shared::{qtrue, vec3_t};

use crate::cg_view::DAMAGE_TIME;
use crate::world::cg_world::CgWorld;

/// Raven `CG_CheckAmmo` — the low-ammo warning HUD/sound feature.
///
/// The whole body is wrapped in `#if 0 ... #endif`; the retail build shipped
/// with this switched off, so nothing here is reachable — only Raven's own
/// trailing comment survives.
///
/// Source: `oracle/codemp/cgame/cg_playerstate.c:17-78`
pub fn CG_CheckAmmo() {
    //disabled silly ammo warning stuff for now
}

/// Raven `CG_DamageFeedback` — turns a directional damage event into the
/// screen-flash/view-kick feedback (`cg.damageX`/`damageY`/`v_dmg_*`).
///
/// Source: `oracle/codemp/cgame/cg_playerstate.c:85-187`
pub fn CG_DamageFeedback(world: &mut CgWorld, yawByte: c_int, pitchByte: c_int, damage: c_int) {
    // show the attacking player's head and name in corner
    world.cg.attackerTime = world.cg.time;

    // §F19: Raven reads `cg.snap->ps` with no null check - before the first
    // snapshot that's a null deref, so the port takes the neutral early-out and
    // feeds nothing back. `cg.attackerTime` above is already set, as in Raven.
    let Some(snap) = world.cg.snap_ref() else {
        return;
    };
    // the lower on health you are, the greater the view kick will be
    let health = snap.ps.stats[STAT_HEALTH as usize];
    let snapServerTime = snap.serverTime;

    let scale: f32 = if health < 40 {
        1.0
    } else {
        // Raven's `40.0` is a double literal, so the divide happens in double
        (40.0f64 / f64::from(health)) as f32
    };
    let mut kick = damage as f32 * scale;

    if kick < 5.0 {
        kick = 5.0;
    }
    if kick > 10.0 {
        kick = 10.0;
    }

    // if yaw and pitch are both 255, make the damage always centered (falling, etc)
    if yawByte == 255 && pitchByte == 255 {
        world.cg.damageX = 0.0;
        world.cg.damageY = 0.0;
        world.cg.v_dmg_roll = 0.0;
        world.cg.v_dmg_pitch = -kick;
    } else {
        // positional
        let pitch = (f64::from(pitchByte) / 255.0 * 360.0) as f32;
        let yaw = (f64::from(yawByte) / 255.0 * 360.0) as f32;

        let mut angles: vec3_t = [0.0; 3];
        angles[PITCH] = pitch;
        angles[YAW] = yaw;
        angles[ROLL] = 0.0;

        let mut dir: vec3_t = [0.0; 3];
        AngleVectors(angles, Some(&mut dir), None, None);
        let toUs = dir;
        _VectorSubtract(vec3_origin, toUs, &mut dir);

        let mut front = _DotProduct(dir, world.cg.refdef.viewaxis[0]);
        let left = _DotProduct(dir, world.cg.refdef.viewaxis[1]);
        let up = _DotProduct(dir, world.cg.refdef.viewaxis[2]);

        dir[0] = front;
        dir[1] = left;
        dir[2] = 0.0;
        let mut dist = VectorLength(dir);
        if dist < 0.1 {
            dist = 0.1;
        }

        world.cg.v_dmg_roll = kick * left;

        world.cg.v_dmg_pitch = -kick * front;

        if front <= 0.1 {
            front = 0.1;
        }
        world.cg.damageX = -left / front;
        world.cg.damageY = up / dist;
    }

    // clamp the position
    if world.cg.damageX > 1.0 {
        world.cg.damageX = 1.0;
    }
    if world.cg.damageX < -1.0 {
        world.cg.damageX = -1.0;
    }

    if world.cg.damageY > 1.0 {
        world.cg.damageY = 1.0;
    }
    if world.cg.damageY < -1.0 {
        world.cg.damageY = -1.0;
    }

    // don't let the screen flashes vary as much
    if kick > 10.0 {
        kick = 10.0;
    }
    world.cg.damageValue = kick;
    world.cg.v_dmg_time = (world.cg.time + DAMAGE_TIME) as f32;
    world.cg.damageTime = snapServerTime as f32;

    // Raven's `//JLFRUMBLE` rumble tail is `#ifdef _XBOX`, so the PC build ends here.
}

/// Raven `CG_Respawn` — called on every local-player respawn: turns off error
/// decay for this frame's movement, opens the weapon-select HUD, and re-syncs
/// the selected weapon to whatever the server says we're carrying.
///
/// Source: `oracle/codemp/cgame/cg_playerstate.c:199-208`
pub fn CG_Respawn(world: &mut CgWorld) {
    // no error decay on player movement
    world.cg.thisFrameTeleport = qtrue;

    // display weapons available
    world.cg.weaponSelectTime = world.cg.time;

    // select the weapon the server says we are using
    // §F19: Raven derefs `cg.snap` here with no guard - with no snapshot yet the
    // selection keeps its prior value rather than reading through a null.
    let Some(weapon) = world.cg.snap_ref().map(|snap| snap.ps.weapon) else {
        return;
    };
    world.cg.weaponSelect = weapon;
}
