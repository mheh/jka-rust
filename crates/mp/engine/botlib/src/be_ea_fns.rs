#![allow(non_camel_case_types, non_snake_case, unused_variables)]

//! MP botlib `be_ea.cpp` — the elementary-action layer: bots write their
//! per-frame chat/inventory/movement/view intents into `botinputs[client]`
//! here; `EA_GetInput` copies the accumulated frame out for the game to
//! consume.
//!
//! Source: `oracle/codemp/botlib/be_ea.cpp`

use core::ffi::{c_char, c_int, c_ulong};

use mp_qshared::common::mp::botlib::action::{
    ACTION_ALT_ATTACK, ACTION_ATTACK, ACTION_CROUCH, ACTION_DELAYEDJUMP, ACTION_FORCEPOWER,
    ACTION_GESTURE, ACTION_JUMP, ACTION_MOVEBACK, ACTION_MOVEDOWN, ACTION_MOVEFORWARD,
    ACTION_MOVELEFT, ACTION_MOVERIGHT, ACTION_MOVEUP, ACTION_RESPAWN, ACTION_TALK, ACTION_USE,
    ACTION_WALK,
};
use mp_qshared::common::mp::botlib::bot_input_s::bot_input_t;
use mp_qshared::common::mp::botlib::botlib_error::BLERR_NOERROR;
use mp_qshared::shared::q_format::FmtArg;
use mp_qshared::shared::q_string::va;
use mp_qshared::shared::q_math::{_VectorCopy, VectorClear};
use mp_qshared::shared::vec3_t;

use crate::be_ea::ea_consts::{ACTION_JUMPEDLASTFRAME, MAX_USERMOVE};
use crate::l_memory_fns::{FreeMemory, GetClearedHunkMemory};
use crate::BotLib;

use mp_engine_qcommon::common_fns::Com_Memcpy;

/// Raven `EA_Say`.
///
/// Source: `oracle/codemp/botlib/be_ea.cpp:35-38`
pub fn EA_Say(bot: &mut BotLib, client: c_int, str: *mut c_char) {
    unsafe {
        let fmt = c"say %s";
        bot.botimport.BotClientCommand.unwrap()(client, va(fmt.as_ptr(), &[FmtArg::cstr(str)]));
    }
}

/// Raven `EA_SayTeam`.
///
/// Source: `oracle/codemp/botlib/be_ea.cpp:45-48`
pub fn EA_SayTeam(bot: &mut BotLib, client: c_int, str: *mut c_char) {
    unsafe {
        let fmt = c"say_team %s";
        bot.botimport.BotClientCommand.unwrap()(client, va(fmt.as_ptr(), &[FmtArg::cstr(str)]));
    }
}

/// Raven `EA_Tell`.
///
/// Source: `oracle/codemp/botlib/be_ea.cpp:55-58`
pub fn EA_Tell(bot: &mut BotLib, client: c_int, clientto: c_int, str: *mut c_char) {
    unsafe {
        let fmt = c"tell %d, %s";
        bot.botimport.BotClientCommand.unwrap()(
            client,
            va(fmt.as_ptr(), &[FmtArg::Int(clientto), FmtArg::cstr(str)]),
        );
    }
}

/// Raven `EA_UseItem`.
///
/// Source: `oracle/codemp/botlib/be_ea.cpp:65-68`
pub fn EA_UseItem(bot: &mut BotLib, client: c_int, it: *mut c_char) {
    unsafe {
        let fmt = c"use %s";
        bot.botimport.BotClientCommand.unwrap()(client, va(fmt.as_ptr(), &[FmtArg::cstr(it)]));
    }
}

/// Raven `EA_DropItem`.
///
/// Source: `oracle/codemp/botlib/be_ea.cpp:75-78`
pub fn EA_DropItem(bot: &mut BotLib, client: c_int, it: *mut c_char) {
    unsafe {
        let fmt = c"drop %s";
        bot.botimport.BotClientCommand.unwrap()(client, va(fmt.as_ptr(), &[FmtArg::cstr(it)]));
    }
}

/// Raven `EA_UseInv`.
///
/// Source: `oracle/codemp/botlib/be_ea.cpp:85-88`
pub fn EA_UseInv(bot: &mut BotLib, client: c_int, inv: *mut c_char) {
    unsafe {
        let fmt = c"invuse %s";
        bot.botimport.BotClientCommand.unwrap()(client, va(fmt.as_ptr(), &[FmtArg::cstr(inv)]));
    }
}

/// Raven `EA_DropInv`.
///
/// Source: `oracle/codemp/botlib/be_ea.cpp:95-98`
pub fn EA_DropInv(bot: &mut BotLib, client: c_int, inv: *mut c_char) {
    unsafe {
        let fmt = c"invdrop %s";
        bot.botimport.BotClientCommand.unwrap()(client, va(fmt.as_ptr(), &[FmtArg::cstr(inv)]));
    }
}

/// Raven `EA_Gesture`.
///
/// Source: `oracle/codemp/botlib/be_ea.cpp:105-112`
pub fn EA_Gesture(bot: &mut BotLib, client: c_int) {
    unsafe {
        let bi = &mut *bot.botinputs.add(client as usize);
        bi.actionflags |= ACTION_GESTURE;
    }
}

/// Raven `EA_Command`.
///
/// Source: `oracle/codemp/botlib/be_ea.cpp:119-122`
pub fn EA_Command(bot: &mut BotLib, client: c_int, command: *mut c_char) {
    unsafe {
        bot.botimport.BotClientCommand.unwrap()(client, command);
    }
}

/// Raven `EA_SelectWeapon`.
///
/// Source: `oracle/codemp/botlib/be_ea.cpp:129-136`
pub fn EA_SelectWeapon(bot: &mut BotLib, client: c_int, weapon: c_int) {
    unsafe {
        let bi = &mut *bot.botinputs.add(client as usize);
        bi.weapon = weapon;
    }
}

/// Raven `EA_Attack`.
///
/// Source: `oracle/codemp/botlib/be_ea.cpp:143-150`
pub fn EA_Attack(bot: &mut BotLib, client: c_int) {
    unsafe {
        let bi = &mut *bot.botinputs.add(client as usize);
        bi.actionflags |= ACTION_ATTACK;
    }
}

/// Raven `EA_Alt_Attack`.
///
/// Source: `oracle/codemp/botlib/be_ea.cpp:157-164`
pub fn EA_Alt_Attack(bot: &mut BotLib, client: c_int) {
    unsafe {
        let bi = &mut *bot.botinputs.add(client as usize);
        bi.actionflags |= ACTION_ALT_ATTACK;
    }
}

/// Raven `EA_ForcePower`.
///
/// Source: `oracle/codemp/botlib/be_ea.cpp:171-178`
pub fn EA_ForcePower(bot: &mut BotLib, client: c_int) {
    unsafe {
        let bi = &mut *bot.botinputs.add(client as usize);
        bi.actionflags |= ACTION_FORCEPOWER;
    }
}

/// Raven `EA_Talk`.
///
/// Source: `oracle/codemp/botlib/be_ea.cpp:185-192`
pub fn EA_Talk(bot: &mut BotLib, client: c_int) {
    unsafe {
        let bi = &mut *bot.botinputs.add(client as usize);
        bi.actionflags |= ACTION_TALK;
    }
}

/// Raven `EA_Use`.
///
/// Source: `oracle/codemp/botlib/be_ea.cpp:199-206`
pub fn EA_Use(bot: &mut BotLib, client: c_int) {
    unsafe {
        let bi = &mut *bot.botinputs.add(client as usize);
        bi.actionflags |= ACTION_USE;
    }
}

/// Raven `EA_Respawn`.
///
/// Source: `oracle/codemp/botlib/be_ea.cpp:213-220`
pub fn EA_Respawn(bot: &mut BotLib, client: c_int) {
    unsafe {
        let bi = &mut *bot.botinputs.add(client as usize);
        bi.actionflags |= ACTION_RESPAWN;
    }
}

/// Raven `EA_Jump`.
///
/// Source: `oracle/codemp/botlib/be_ea.cpp:227-241`
pub fn EA_Jump(bot: &mut BotLib, client: c_int) {
    unsafe {
        let bi = &mut *bot.botinputs.add(client as usize);
        if bi.actionflags & ACTION_JUMPEDLASTFRAME != 0 {
            bi.actionflags &= !ACTION_JUMP;
        } else {
            bi.actionflags |= ACTION_JUMP;
        }
    }
}

/// Raven `EA_DelayedJump`.
///
/// Source: `oracle/codemp/botlib/be_ea.cpp:248-262`
pub fn EA_DelayedJump(bot: &mut BotLib, client: c_int) {
    unsafe {
        let bi = &mut *bot.botinputs.add(client as usize);
        if bi.actionflags & ACTION_JUMPEDLASTFRAME != 0 {
            bi.actionflags &= !ACTION_DELAYEDJUMP;
        } else {
            bi.actionflags |= ACTION_DELAYEDJUMP;
        }
    }
}

/// Raven `EA_Crouch`.
///
/// Source: `oracle/codemp/botlib/be_ea.cpp:269-276`
pub fn EA_Crouch(bot: &mut BotLib, client: c_int) {
    unsafe {
        let bi = &mut *bot.botinputs.add(client as usize);
        bi.actionflags |= ACTION_CROUCH;
    }
}

/// Raven `EA_Walk`.
///
/// Source: `oracle/codemp/botlib/be_ea.cpp:283-290`
pub fn EA_Walk(bot: &mut BotLib, client: c_int) {
    unsafe {
        let bi = &mut *bot.botinputs.add(client as usize);
        bi.actionflags |= ACTION_WALK;
    }
}

/// Raven `EA_Action`.
///
/// Source: `oracle/codemp/botlib/be_ea.cpp:297-304`
pub fn EA_Action(bot: &mut BotLib, client: c_int, action: c_int) {
    unsafe {
        let bi = &mut *bot.botinputs.add(client as usize);
        bi.actionflags |= action;
    }
}

/// Raven `EA_MoveUp`.
///
/// Source: `oracle/codemp/botlib/be_ea.cpp:311-318`
pub fn EA_MoveUp(bot: &mut BotLib, client: c_int) {
    unsafe {
        let bi = &mut *bot.botinputs.add(client as usize);
        bi.actionflags |= ACTION_MOVEUP;
    }
}

/// Raven `EA_MoveDown`.
///
/// Source: `oracle/codemp/botlib/be_ea.cpp:325-332`
pub fn EA_MoveDown(bot: &mut BotLib, client: c_int) {
    unsafe {
        let bi = &mut *bot.botinputs.add(client as usize);
        bi.actionflags |= ACTION_MOVEDOWN;
    }
}

/// Raven `EA_MoveForward`.
///
/// Source: `oracle/codemp/botlib/be_ea.cpp:339-346`
pub fn EA_MoveForward(bot: &mut BotLib, client: c_int) {
    unsafe {
        let bi = &mut *bot.botinputs.add(client as usize);
        bi.actionflags |= ACTION_MOVEFORWARD;
    }
}

/// Raven `EA_MoveBack`.
///
/// Source: `oracle/codemp/botlib/be_ea.cpp:353-360`
pub fn EA_MoveBack(bot: &mut BotLib, client: c_int) {
    unsafe {
        let bi = &mut *bot.botinputs.add(client as usize);
        bi.actionflags |= ACTION_MOVEBACK;
    }
}

/// Raven `EA_MoveLeft`.
///
/// Source: `oracle/codemp/botlib/be_ea.cpp:367-374`
pub fn EA_MoveLeft(bot: &mut BotLib, client: c_int) {
    unsafe {
        let bi = &mut *bot.botinputs.add(client as usize);
        bi.actionflags |= ACTION_MOVELEFT;
    }
}

/// Raven `EA_MoveRight`.
///
/// Source: `oracle/codemp/botlib/be_ea.cpp:381-388`
pub fn EA_MoveRight(bot: &mut BotLib, client: c_int) {
    unsafe {
        let bi = &mut *bot.botinputs.add(client as usize);
        bi.actionflags |= ACTION_MOVERIGHT;
    }
}

/// Raven `EA_Move`.
///
/// Source: `oracle/codemp/botlib/be_ea.cpp:395-406`
pub fn EA_Move(bot: &mut BotLib, client: c_int, dir: vec3_t, mut speed: f32) {
    unsafe {
        let bi = &mut *bot.botinputs.add(client as usize);
        _VectorCopy(dir, &mut bi.dir);
        // cap speed
        if speed > MAX_USERMOVE as f32 {
            speed = MAX_USERMOVE as f32;
        } else if speed < -(MAX_USERMOVE as f32) {
            speed = -(MAX_USERMOVE as f32);
        }
        bi.speed = speed;
    }
}

/// Raven `EA_View`.
///
/// Source: `oracle/codemp/botlib/be_ea.cpp:413-420`
pub fn EA_View(bot: &mut BotLib, client: c_int, viewangles: vec3_t) {
    unsafe {
        let bi = &mut *bot.botinputs.add(client as usize);
        _VectorCopy(viewangles, &mut bi.viewangles);
    }
}

/// Raven `EA_EndRegular`.
///
/// Raven: the entire body is commented out (`/* ... */`) in the oracle —
/// this function is a compiled no-op in retail. Transcribed as a faithful
/// empty body, no receivers touched.
/// Source: `oracle/codemp/botlib/be_ea.cpp:427-447`
pub fn EA_EndRegular(client: c_int, thinktime: f32) {
    let _ = (client, thinktime);
}

/// Raven `EA_GetInput`.
///
/// Source: `oracle/codemp/botlib/be_ea.cpp:454-474`
pub fn EA_GetInput(bot: &mut BotLib, client: c_int, thinktime: f32, input: *mut bot_input_t) {
    unsafe {
        let bi = &mut *bot.botinputs.add(client as usize);
        bi.thinktime = thinktime;
        Com_Memcpy(
            input as *mut (),
            bi as *const bot_input_t as *const (),
            core::mem::size_of::<bot_input_t>(),
        );
    }
}

/// Raven `EA_ResetInput`.
///
/// Source: `oracle/codemp/botlib/be_ea.cpp:481-495`
pub fn EA_ResetInput(bot: &mut BotLib, client: c_int) {
    unsafe {
        let bi = &mut *bot.botinputs.add(client as usize);
        bi.actionflags &= !ACTION_JUMPEDLASTFRAME;

        bi.thinktime = 0.0;
        VectorClear(&mut bi.dir);
        bi.speed = 0.0;
        let jumped = bi.actionflags & ACTION_JUMP;
        bi.actionflags = 0;
        if jumped != 0 {
            bi.actionflags |= ACTION_JUMPEDLASTFRAME;
        }
    }
}

/// Raven `EA_Setup`.
///
/// Source: `oracle/codemp/botlib/be_ea.cpp:502-508`
pub fn EA_Setup(bot: &mut BotLib) -> c_int {
    // initialize the bot inputs
    bot.botinputs = GetClearedHunkMemory(
        bot,
        (bot.botlibglobals.maxclients as usize * core::mem::size_of::<bot_input_t>()) as c_ulong,
    ) as *mut bot_input_t;
    BLERR_NOERROR
}

/// Raven `EA_Shutdown`.
///
/// Source: `oracle/codemp/botlib/be_ea.cpp:515-519`
pub fn EA_Shutdown(bot: &mut BotLib) {
    FreeMemory(bot, bot.botinputs as *mut ());
    bot.botinputs = core::ptr::null_mut();
}
