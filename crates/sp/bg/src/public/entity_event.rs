//! SP `bg_public.h` entity event enumeration.
//!
//! Type definition source: `oracle/code/game/bg_public.h:283-465`

#![allow(non_camel_case_types)]

/// Raven `entity_event_t`.
///
/// Type definition source: `oracle/code/game/bg_public.h:283-465`
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum entity_event_t {
    EV_NONE = 0,

    EV_FOOTSTEP = 1,
    EV_FOOTSTEP_METAL = 2,
    EV_FOOTSPLASH = 3,
    EV_FOOTWADE = 4,
    EV_SWIM = 5,

    EV_STEP_4 = 6,
    EV_STEP_8 = 7,
    EV_STEP_12 = 8,
    EV_STEP_16 = 9,

    EV_FALL_SHORT = 10,
    EV_FALL_MEDIUM = 11,
    EV_FALL_FAR = 12,

    EV_JUMP = 13,
    EV_ROLL = 14,
    EV_WATER_TOUCH = 15,   // foot touches
    EV_WATER_LEAVE = 16,   // foot leaves
    EV_WATER_UNDER = 17,   // head touches
    EV_WATER_CLEAR = 18,   // head leaves
    EV_WATER_GURP1 = 19,   // need air 1
    EV_WATER_GURP2 = 20,   // need air 2
    EV_WATER_DROWN = 21,   // drowned
    EV_LAVA_TOUCH = 22,    // foot touches
    EV_LAVA_LEAVE = 23,    // foot leaves
    EV_LAVA_UNDER = 24,    // head touches

    EV_ITEM_PICKUP = 25,

    EV_NOAMMO = 26,
    EV_CHANGE_WEAPON = 27,
    EV_FIRE_WEAPON = 28,
    EV_ALT_FIRE = 29,
    EV_POWERUP_SEEKER_FIRE = 30,
    EV_POWERUP_BATTLESUIT = 31,
    EV_USE = 32,

    EV_REPLICATOR = 33,

    EV_BATTERIES_CHARGED = 34,

    EV_GRENADE_BOUNCE = 35,        // eventParm will be the soundindex
    EV_MISSILE_STICK = 36,         // eventParm will be the soundindex

    EV_BMODEL_SOUND = 37,
    EV_GENERAL_SOUND = 38,
    EV_GLOBAL_SOUND = 39,          // no attenuation

    //#ifdef _IMMERSION
    //	EV_ENTITY_FORCE,
    //	EV_AREA_FORCE,
    //	EV_GLOBAL_FORCE,
    //	EV_FORCE_STOP,
    //#endif // _IMMERSION
    EV_PLAY_EFFECT = 40,
    EV_PLAY_MUZZLE_EFFECT = 41,
    EV_STOP_EFFECT = 42,

    EV_TARGET_BEAM_DRAW = 43,

    EV_DISRUPTOR_MAIN_SHOT = 44,
    EV_DISRUPTOR_SNIPER_SHOT = 45,
    EV_DISRUPTOR_SNIPER_MISS = 46,

    EV_DEMP2_ALT_IMPACT = 47,
    //NEW for JKA weapons:
    EV_CONC_ALT_SHOT = 48,
    EV_CONC_ALT_MISS = 49,
    //END JKA weapons
    EV_PAIN = 50,
    EV_DEATH1 = 51,
    EV_DEATH2 = 52,
    EV_DEATH3 = 53,

    EV_MISSILE_HIT = 54,
    EV_MISSILE_MISS = 55,

    EV_DISINTEGRATION = 56,

    EV_ANGER1 = 57,         //Say when acquire an enemy when didn't have one before
    EV_ANGER2 = 58,
    EV_ANGER3 = 59,

    EV_VICTORY1 = 60,       //Say when killed an enemy
    EV_VICTORY2 = 61,
    EV_VICTORY3 = 62,

    EV_CONFUSE1 = 63,       //Say when confused
    EV_CONFUSE2 = 64,
    EV_CONFUSE3 = 65,

    EV_PUSHED1 = 66,        //Say when pushed
    EV_PUSHED2 = 67,
    EV_PUSHED3 = 68,

    EV_CHOKE1 = 69,         //Say when choking
    EV_CHOKE2 = 70,
    EV_CHOKE3 = 71,

    EV_FFWARN = 72,         //ffire founds
    EV_FFTURN = 73,
    //extra sounds for ST
    EV_CHASE1 = 74,
    EV_CHASE2 = 75,
    EV_CHASE3 = 76,
    EV_COVER1 = 77,
    EV_COVER2 = 78,
    EV_COVER3 = 79,
    EV_COVER4 = 80,
    EV_COVER5 = 81,
    EV_DETECTED1 = 82,
    EV_DETECTED2 = 83,
    EV_DETECTED3 = 84,
    EV_DETECTED4 = 85,
    EV_DETECTED5 = 86,
    EV_LOST1 = 87,
    EV_OUTFLANK1 = 88,
    EV_OUTFLANK2 = 89,
    EV_ESCAPING1 = 90,
    EV_ESCAPING2 = 91,
    EV_ESCAPING3 = 92,
    EV_GIVEUP1 = 93,
    EV_GIVEUP2 = 94,
    EV_GIVEUP3 = 95,
    EV_GIVEUP4 = 96,
    EV_LOOK1 = 97,
    EV_LOOK2 = 98,
    EV_SIGHT1 = 99,
    EV_SIGHT2 = 100,
    EV_SIGHT3 = 101,
    EV_SOUND1 = 102,
    EV_SOUND2 = 103,
    EV_SOUND3 = 104,
    EV_SUSPICIOUS1 = 105,
    EV_SUSPICIOUS2 = 106,
    EV_SUSPICIOUS3 = 107,
    EV_SUSPICIOUS4 = 108,
    EV_SUSPICIOUS5 = 109,
    //extra sounds for Jedi
    EV_COMBAT1 = 110,
    EV_COMBAT2 = 111,
    EV_COMBAT3 = 112,
    EV_JDETECTED1 = 113,
    EV_JDETECTED2 = 114,
    EV_JDETECTED3 = 115,
    EV_TAUNT1 = 116,
    EV_TAUNT2 = 117,
    EV_TAUNT3 = 118,
    EV_JCHASE1 = 119,
    EV_JCHASE2 = 120,
    EV_JCHASE3 = 121,
    EV_JLOST1 = 122,
    EV_JLOST2 = 123,
    EV_JLOST3 = 124,
    EV_DEFLECT1 = 125,
    EV_DEFLECT2 = 126,
    EV_DEFLECT3 = 127,
    EV_GLOAT1 = 128,
    EV_GLOAT2 = 129,
    EV_GLOAT3 = 130,
    EV_PUSHFAIL = 131,

    EV_USE_ITEM = 132,

    EV_USE_INV_BINOCULARS = 133,
    EV_USE_INV_BACTA = 134,
    EV_USE_INV_SEEKER = 135,
    EV_USE_INV_LIGHTAMP_GOGGLES = 136,
    EV_USE_INV_SENTRY = 137,

    EV_USE_FORCE = 138,

    EV_DRUGGED = 139,       // hit by an interrogator

    EV_DEBUG_LINE = 140,
    EV_KOTHOS_BEAM = 141,
}
