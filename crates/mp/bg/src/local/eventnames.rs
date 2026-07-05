//! MP `bg_misc.c` debug event-name table.

#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]

/// Raven `eventnames[]` — `entity_event_t` names used by the `showevents`
/// debug print in `BG_AddPredictableEventToPlayerstate`. Raven only ever
/// filled in the first 116 of the 192 `entity_event_t` values (trailing
/// comment: "fixme, added a bunch that aren't here!"); the commented-out
/// `"EV_POWERUP_REGEN"` line is dropped, matching the live C array.
///
/// Source: `oracle/oracle/codemp/game/bg_misc.c:2464-2623`
pub static eventnames: [&str; 116] = [
    "EV_NONE",
    "EV_CLIENTJOIN",
    "EV_FOOTSTEP",
    "EV_FOOTSTEP_METAL",
    "EV_FOOTSPLASH",
    "EV_FOOTWADE",
    "EV_SWIM",
    "EV_STEP_4",
    "EV_STEP_8",
    "EV_STEP_12",
    "EV_STEP_16",
    "EV_FALL",
    "EV_JUMP_PAD", // boing sound at origin, jump sound on player
    "EV_GHOUL2_MARK", // create a projectile impact mark on something with a client-side g2 instance.
    "EV_GLOBAL_DUEL",
    "EV_PRIVATE_DUEL",
    "EV_JUMP",
    "EV_ROLL",
    "EV_WATER_TOUCH", // foot touches
    "EV_WATER_LEAVE", // foot leaves
    "EV_WATER_UNDER", // head touches
    "EV_WATER_CLEAR", // head leaves
    "EV_ITEM_PICKUP",        // normal item pickups are predictable
    "EV_GLOBAL_ITEM_PICKUP", // powerup / team sounds are broadcast to everyone
    "EV_VEH_FIRE",
    "EV_NOAMMO",
    "EV_CHANGE_WEAPON",
    "EV_FIRE_WEAPON",
    "EV_ALT_FIRE",
    "EV_SABER_ATTACK",
    "EV_SABER_HIT",
    "EV_SABER_BLOCK",
    "EV_SABER_CLASHFLARE",
    "EV_SABER_UNHOLSTER",
    "EV_BECOME_JEDIMASTER",
    "EV_DISRUPTOR_MAIN_SHOT",
    "EV_DISRUPTOR_SNIPER_SHOT",
    "EV_DISRUPTOR_SNIPER_MISS",
    "EV_DISRUPTOR_HIT",
    "EV_DISRUPTOR_ZOOMSOUND",
    "EV_PREDEFSOUND",
    "EV_TEAM_POWER",
    "EV_SCREENSHAKE",
    "EV_LOCALTIMER",
    "EV_USE", // +Use key
    "EV_USE_ITEM0",
    "EV_USE_ITEM1",
    "EV_USE_ITEM2",
    "EV_USE_ITEM3",
    "EV_USE_ITEM4",
    "EV_USE_ITEM5",
    "EV_USE_ITEM6",
    "EV_USE_ITEM7",
    "EV_USE_ITEM8",
    "EV_USE_ITEM9",
    "EV_USE_ITEM10",
    "EV_USE_ITEM11",
    "EV_USE_ITEM12",
    "EV_USE_ITEM13",
    "EV_USE_ITEM14",
    "EV_USE_ITEM15",
    "EV_ITEMUSEFAIL",
    "EV_ITEM_RESPAWN",
    "EV_ITEM_POP",
    "EV_PLAYER_TELEPORT_IN",
    "EV_PLAYER_TELEPORT_OUT",
    "EV_GRENADE_BOUNCE", // eventParm will be the soundindex
    "EV_MISSILE_STICK",
    "EV_PLAY_EFFECT",
    "EV_PLAY_EFFECT_ID", // finally gave in and added it..
    "EV_PLAY_PORTAL_EFFECT_ID",
    "EV_PLAYDOORSOUND",
    "EV_PLAYDOORLOOPSOUND",
    "EV_BMODEL_SOUND",
    "EV_MUTE_SOUND",
    "EV_VOICECMD_SOUND",
    "EV_GENERAL_SOUND",
    "EV_GLOBAL_SOUND",      // no attenuation
    "EV_GLOBAL_TEAM_SOUND",
    "EV_ENTITY_SOUND",
    "EV_PLAY_ROFF",
    "EV_GLASS_SHATTER",
    "EV_DEBRIS",
    "EV_MISC_MODEL_EXP",
    "EV_CONC_ALT_IMPACT",
    "EV_MISSILE_HIT",
    "EV_MISSILE_MISS",
    "EV_MISSILE_MISS_METAL",
    "EV_BULLET", // otherEntity is the shooter
    "EV_PAIN",
    "EV_DEATH1",
    "EV_DEATH2",
    "EV_DEATH3",
    "EV_OBITUARY",
    "EV_POWERUP_QUAD",
    "EV_POWERUP_BATTLESUIT",
    // "EV_POWERUP_REGEN", // commented out in Raven source
    "EV_FORCE_DRAINED",
    "EV_GIB_PLAYER", // gib a previously living player
    "EV_SCOREPLUM",  // score plum
    "EV_CTFMESSAGE",
    "EV_BODYFADE",
    "EV_SIEGE_ROUNDOVER",
    "EV_SIEGE_OBJECTIVECOMPLETE",
    "EV_DESTROY_GHOUL2_INSTANCE",
    "EV_DESTROY_WEAPON_MODEL",
    "EV_GIVE_NEW_RANK",
    "EV_SET_FREE_SABER",
    "EV_SET_FORCE_DISABLE",
    "EV_WEAPON_CHARGE",
    "EV_WEAPON_CHARGE_ALT",
    "EV_SHIELD_HIT",
    "EV_DEBUG_LINE",
    "EV_TESTLINE",
    "EV_STOPLOOPINGSOUND",
    "EV_STARTLOOPINGSOUND",
    "EV_TAUNT",
    //fixme, added a bunch that aren't here!
];
