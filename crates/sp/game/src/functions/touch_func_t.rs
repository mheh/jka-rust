#![allow(non_camel_case_types, non_snake_case)]

/// Raven `touchFunc_t` — enumeration of entity touch/collision callback function IDs.
///
/// Type definition source: `oracle/oracle/code/game/g_functions.h:286-308`
#[repr(i32)]
pub enum touchFunc_t {
    touchF_NULL = 0,
    //
    touchF_Touch_Item,
    touchF_teleporter_touch,
    touchF_charge_stick,
    touchF_Touch_DoorTrigger,
    touchF_Touch_PlatCenterTrigger,
    touchF_Touch_Plat,
    touchF_Touch_Button,
    touchF_Touch_Multi,
    touchF_trigger_push_touch,
    touchF_trigger_teleporter_touch,
    touchF_hurt_touch,
    touchF_NPC_Touch,
    touchF_touch_ammo_crystal_tigger,
    touchF_funcBBrushTouch,
    touchF_touchLaserTrap,
    touchF_prox_mine_stick,
    touchF_func_rotating_touch,
    touchF_TouchTieBomb,
}
