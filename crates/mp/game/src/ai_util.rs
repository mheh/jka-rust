// PORT-COMPLETE: ai_util.c
//! FAITHFUL port of `oracle/codemp/game/ai_util.c`.
//!
//! Filled by the jampgame mega-pass. `BOT_ZMALLOC`/`BOTMEMTRACK`/`DEBUG` are not defined
//! in this build (grep-confirmed no callers define them), so the
//! `#ifdef BOT_ZMALLOC` branches are dead and their bodies are omitted
//! faithfully (porting-rules §C10 — behavior, not shape).
//!
//! All 12 functions in the file are ported here. `BotDoChat`/`ReadChatGroups`/
//! `BotUtilizePersonality` reach `gBotChatBuffer` and `trap::*` engine calls
//! through the `GameContext` passed in; `B_InitAlloc` reaches `gWPArray` the
//! same way.
#![allow(non_snake_case, unused, clippy::all)]

use core::ffi::CStr;

use crate::botai::bot_state_s::MAX_LOVED_ONES;
use crate::prelude::*;
use crate::trap;
use mp_abi::game::syscalls::G_CVAR_VARIABLE_INTEGER_VALUE::GCvarVariableIntegerValue;
use mp_abi::game::syscalls::G_FS_FCLOSE_FILE::GFsFcloseFile;
use mp_abi::game::syscalls::G_FS_FOPEN_FILE::GFsFopenFile;
use mp_abi::game::syscalls::G_FS_READ::GFsRead;
use native_string::atof_bytes;

/// Raven `B_TempAlloc`.
///
/// Source: `oracle/codemp/game/ai_util.c:14-17`
pub fn B_TempAlloc(ctx: &mut GameContext, size: c_int) -> *mut c_void {
    mp_bg::bg_misc::BG_TempAlloc(size, &mut ctx.world.bg_state)
}

/// Raven `B_TempFree`.
///
/// Source: `oracle/codemp/game/ai_util.c:19-22`
pub fn B_TempFree(ctx: &mut GameContext, size: c_int) {
    mp_bg::bg_misc::BG_TempFree(size, &mut ctx.world.bg_state)
}

/// Raven `B_Alloc`.
///
/// Raven: `BOT_ZMALLOC` is not defined in this build, so only the plain
/// `return BG_Alloc(size);` branch (`ai_util.c:77`) is live.
/// Source: `oracle/codemp/game/ai_util.c:25-80`
pub fn B_Alloc(ctx: &mut GameContext, size: c_int) -> *mut c_void {
    mp_bg::bg_misc::BG_Alloc(size, &mut ctx.world.bg_state)
}

/// Raven `B_Free`.
///
/// Raven: the entire body is guarded by `#ifdef BOT_ZMALLOC`
/// (`ai_util.c:84-129`), which is not defined in this build, so this is a
/// faithful no-op.
/// Source: `oracle/codemp/game/ai_util.c:82-131`
pub fn B_Free(ptr: *mut c_void) {}

/// Raven `B_InitAlloc`.
///
/// Zeros out the waypoint arena and allocator tracking lists. The `BOT_ZMALLOC`
/// branch is dead in this build (see module doc).
///
/// Source: `oracle/codemp/game/ai_util.c:133-140`
pub fn B_InitAlloc(ctx: &mut GameContext) {
    // SAFETY: world is valid; globals.gWPArray is an owned field.
    ctx.world.globals.gWPArray = crate::game_globals::WpArray::default();
}

/// Raven `B_CleanupAlloc`.
///
/// Raven: the entire body is guarded by `#ifdef BOT_ZMALLOC`
/// (`ai_util.c:144-157`), which is not defined in this build, so this is a
/// faithful no-op.
/// Source: `oracle/codemp/game/ai_util.c:142-158`
pub fn B_CleanupAlloc() {}

/// Raven `GetValueGroup`.
///
/// Finds the named `group { ... }` block in `buf` (the group name must be
/// preceded by a newline and immediately followed by `{`), and copies its
/// (nesting-aware) contents into `outbuf`. Returns `0` if not found.
///
/// Raven: ported as raw-pointer byte scanning to match the C pointer
/// arithmetic exactly (`place - buf` offset math, no bounds other than the
/// ones Raven itself relies on — same UB envelope, porting-rules §19).
/// Source: `oracle/codemp/game/ai_util.c:160-234`
pub fn GetValueGroup(buf: *mut c_char, group: *mut c_char, outbuf: *mut c_char) -> c_int {
    unsafe {
        let buf_b = buf as *const u8;
        let group_b = group as *const u8;

        // strstr(buf, group)
        let mut place = match c_strstr(buf_b, group_b) {
            Some(p) => p,
            None => return 0,
        };

        let group_len = c_strlen(group_b);
        let mut startpoint = (place as isize - buf_b as isize) as isize + group_len as isize + 1;
        let mut startletter = (place as isize - buf_b as isize) - 1;

        let mut failure = false;

        while *buf_b.offset(startpoint + 1) as u8 != b'{'
            || *buf_b.offset(startletter) as u8 != b'\n'
        {
            match c_strstr(place.add(1), group_b) {
                Some(placesecond) => {
                    let delta = placesecond as isize - place as isize;
                    startpoint += delta;
                    startletter += delta;
                    place = placesecond;
                }
                None => {
                    failure = true;
                    break;
                }
            }
        }

        if failure {
            return 0;
        }

        // we have found the proper group name if we made it here, so find the
        // opening brace and read into the outbuf until hitting the end brace
        while *buf_b.offset(startpoint) as u8 != b'{' {
            startpoint += 1;
        }

        startpoint += 1;

        let outbuf_b = outbuf as *mut u8;
        let mut i: isize = 0;
        let mut subg: c_int = 0;

        while *buf_b.offset(startpoint) as u8 != b'}' || subg != 0 {
            let c = *buf_b.offset(startpoint) as u8;
            if c == b'{' {
                subg += 1;
            } else if c == b'}' {
                subg -= 1;
            }
            *outbuf_b.offset(i) = c;
            i += 1;
            startpoint += 1;
        }
        *outbuf_b.offset(i) = 0;

        1
    }
}

/// Raven `GetPairedValue`.
///
/// Finds `key` in `buf` (word-boundary matched by whitespace/tab/newline/
/// NUL), skips leading whitespace, and copies the rest of that line into
/// `outbuf`. Also mutates `buf` in place, replacing `//`-comment lines with
/// `/` characters up to the newline (Raven's own quirky comment-stripping
/// pass, preserved verbatim).
///
/// Raven: `startletter` can go to `-1` when `key` matches at the very start
/// of `buf`; the guard checks `== 0`, not `== -1`, so the backward
/// `buf[startletter]` read is inherited UB, faithfully preserved (same UB
/// envelope as `GetValueGroup` above, porting-rules §19).
/// Source: `oracle/codemp/game/ai_util.c:236-326`
pub fn GetPairedValue(buf: *mut c_char, key: *mut c_char, outbuf: *mut c_char) -> c_int {
    if buf.is_null() || key.is_null() || outbuf.is_null() {
        return 0;
    }

    unsafe {
        let buf_b = buf as *mut u8;
        let key_b = key as *const u8;

        // comment-stripping pass (mutates buf)
        let mut i: isize = 0;
        while *buf_b.offset(i) != 0 {
            if *buf_b.offset(i) == b'/' {
                if *buf_b.offset(i + 1) != 0 && *buf_b.offset(i + 1) == b'/' {
                    while *buf_b.offset(i) != b'\n' {
                        *buf_b.offset(i) = b'/';
                        i += 1;
                    }
                }
            }
            i += 1;
        }

        let buf_cb = buf_b as *const u8;
        let mut place = match c_strstr(buf_cb, key_b) {
            Some(p) => p,
            None => return 0,
        };

        let key_len = c_strlen(key_b);
        let mut startpoint = (place as isize - buf_cb as isize) + key_len as isize;
        let mut startletter = (place as isize - buf_cb as isize) - 1;

        let mut found = false;

        loop {
            if startletter == 0
                || *buf_cb.offset(startletter) == 0
                || *buf_cb.offset(startletter) == 9
                || *buf_cb.offset(startletter) == b' '
                || *buf_cb.offset(startletter) == b'\n'
            {
                let c = *buf_cb.offset(startpoint);
                if c == 0 || c == 9 || c == b' ' || c == b'\n' {
                    found = true;
                    break;
                }
            }

            match c_strstr(place.add(1), key_b) {
                Some(placesecond) => {
                    let delta = placesecond as isize - place as isize;
                    startpoint += delta;
                    startletter += delta;
                    place = placesecond;
                }
                None => {
                    // place = NULL
                    found = false;
                    break;
                }
            }
        }

        if !found || *buf_cb.offset(startpoint) == 0 {
            return 0;
        }

        while *buf_cb.offset(startpoint) == b' '
            || *buf_cb.offset(startpoint) == 9
            || *buf_cb.offset(startpoint) == b'\n'
        {
            startpoint += 1;
        }

        let outbuf_b = outbuf as *mut u8;
        let mut oi: isize = 0;

        while *buf_cb.offset(startpoint) != 0 && *buf_cb.offset(startpoint) != b'\n' {
            *outbuf_b.offset(oi) = *buf_cb.offset(startpoint);
            oi += 1;
            startpoint += 1;
        }
        *outbuf_b.offset(oi) = 0;

        1
    }
}

/// Raven `BotDoChat`.
///
/// Selects a random chat line from the named section of the bot's personality
/// file (loaded into `gBotChatBuffer`), substitutes entity names for `%s` and
/// `%a` markers, and schedules it for delivery (`bs->doChat`, `bs->chatTime`).
/// Returns 1 on success, 0 if chat is disabled, unavailable, or frequency-rolled
/// out.
///
/// Source: `oracle/codemp/game/ai_util.c:328-515`
pub fn BotDoChat(
    ctx: &mut GameContext,
    bs: *mut bot_state_t,
    section: *mut c_char,
    always: c_int,
) -> c_int {
    unsafe {
        if bs.is_null() {
            return 0;
        }

        let bs_ref = &mut *bs;

        // Early exit: bot can't chat
        if bs_ref.canChat == 0 {
            return 0;
        }

        // Early exit: already have a chat scheduled
        if bs_ref.doChat != 0 {
            return 0;
        }

        // Early exit: non-English language selected
        let lang_result = trap::Cvar_VariableIntegerValue(ctx.engine, "se_language");
        if lang_result != 0 {
            return 0;
        }

        // Frequency roll: skip chat unless always==true or lucky roll
        if ctx.world.bg_state.rng.Q_irand(1, 10) > bs_ref.chatFrequency && always == 0 {
            return 0;
        }

        bs_ref.chatTeam = 0;

        // Allocate temporary buffer for the chat group
        let chatgroup =
            B_TempAlloc(ctx, crate::game_globals::MAX_CHAT_BUFFER_SIZE as c_int) as *mut c_char;

        // Get the chat group from the personality buffer
        // Raven indexes `gBotChatBuffer[bs->client]` unconditionally; `client`
        // is always a valid slot in practice, but Rust array indexing would
        // panic rather than read OOB, so this guard is a defensive divergence
        // (never taken) applied consistently at every `gBotChatBuffer[client]`
        // site in this file.
        let gBotChatBuffer_base = &ctx.world.globals.gBotChatBuffer.0;
        let client_idx = bs_ref.client as usize;

        let rVal = if client_idx < mp_qshared::shared::MAX_CLIENTS {
            GetValueGroup(
                gBotChatBuffer_base[client_idx].as_ptr() as *mut c_char,
                section,
                chatgroup,
            )
        } else {
            0
        };

        // Early exit: no group defined for this chat event
        if rVal == 0 {
            B_TempFree(ctx, crate::game_globals::MAX_CHAT_BUFFER_SIZE as c_int);
            return 0;
        }

        // Remove CR/tab characters from the group
        // Oracle: `inc_1 = 0; inc_2 = 2;` (ai_util.c:372-373).
        let chatgroup_b = chatgroup as *const u8;
        let mut inc_1 = 0isize;
        let mut inc_2 = 2isize;

        while *chatgroup_b.offset(inc_2) != 0 {
            if *chatgroup_b.offset(inc_2) != 13 && *chatgroup_b.offset(inc_2) != 9 {
                *(chatgroup as *mut u8).offset(inc_1) = *chatgroup_b.offset(inc_2);
                inc_1 += 1;
            }
            inc_2 += 1;
        }
        *(chatgroup as *mut u8).offset(inc_1) = 0;

        // Count newlines
        let mut inc_1 = 0isize;
        let mut lines = 0 as c_int;

        while *chatgroup_b.offset(inc_1) != 0 {
            if *chatgroup_b.offset(inc_1) == b'\n' as u8 {
                lines += 1;
            }
            inc_1 += 1;
        }

        // Early exit: no lines
        if lines == 0 {
            B_TempFree(ctx, crate::game_globals::MAX_CHAT_BUFFER_SIZE as c_int);
            return 0;
        }

        // Pick a random line
        let mut getthisline = ctx.world.bg_state.rng.Q_irand(0, lines + 1);
        if getthisline < 1 {
            getthisline = 1;
        }
        if getthisline > lines {
            getthisline = lines;
        }

        // Find the start of the chosen line
        let mut checkedline = 1 as c_int;
        let mut inc_1 = 0isize;

        while checkedline != getthisline {
            if *chatgroup_b.offset(inc_1) != 0 {
                if *chatgroup_b.offset(inc_1) == b'\n' as u8 {
                    inc_1 += 1;
                    checkedline += 1;
                }
            }

            if checkedline == getthisline {
                break;
            }

            inc_1 += 1;
        }

        // Extract the line into a temp buffer
        let mut inc_2 = 0isize;

        while *chatgroup_b.offset(inc_1) != b'\n' as u8 {
            *(chatgroup as *mut u8).offset(inc_2) = *chatgroup_b.offset(inc_1);
            inc_2 += 1;
            inc_1 += 1;
        }
        *(chatgroup as *mut u8).offset(inc_2) = 0;

        // Check line length
        let line_len = c_strlen(chatgroup_b);
        if line_len > crate::botai::bot_state_s::MAX_CHAT_LINE_SIZE {
            B_TempFree(ctx, crate::game_globals::MAX_CHAT_BUFFER_SIZE as c_int);
            return 0;
        }

        // Process the line: substitute names for %s and %a markers
        let mut inc_1 = 0isize;
        let mut inc_2 = 0isize;
        let currentChat = &mut bs_ref.currentChat;
        let currentChat_b = currentChat.as_mut_ptr();

        while *chatgroup_b.offset(inc_1) != 0 {
            if *chatgroup_b.offset(inc_1) == b'%' as u8
                && *chatgroup_b.offset(inc_1 + 1) != b'%' as u8
            {
                inc_1 += 1;

                // Raven: `%s`/`%a` select chatObject/chatAltObject; a null
                // handle mirrors Raven's null `cobject`, skipping the block.
                let cobject_id: Option<EntityId> = if *chatgroup_b.offset(inc_1) == b's' as u8 {
                    bs_ref.chatObject
                } else if *chatgroup_b.offset(inc_1) == b'a' as u8 {
                    bs_ref.chatAltObject
                } else {
                    None
                };

                // Raven derefs `cobject->client->pers.netname`. chatObject can be
                // an NPC (lastHurt = any attacker), whose client is pool-allocated,
                // NOT level.clients[entnum] — so the netname read must go through
                // the entity's client pointer (gclient deref regime, task #7).
                if let Some(id) = cobject_id {
                    let client = ctx.world.entity(id).client;
                    if !client.is_null() {
                        // `netname` is a `String`; copy its bytes (no trailing
                        // NUL — a C `netname[]` had none in its content either).
                        let nbytes = unsafe { (*client).pers.netname.as_bytes() };
                        let mut inc_n = 0usize;

                        while inc_n < nbytes.len() {
                            *currentChat_b.offset(inc_2) = nbytes[inc_n];
                            inc_2 += 1;
                            inc_n += 1;
                        }
                        inc_2 -= 1; // to make up for the auto-increment below
                    }
                }
            } else {
                *currentChat_b.offset(inc_2) = *chatgroup_b.offset(inc_1);
            }
            inc_2 += 1;
            inc_1 += 1;
        }
        *currentChat_b.offset(inc_2) = 0;

        // Set chat duration
        if c_strcmp(section, "GeneralGreetings\0".as_ptr() as *const c_char) == 0 {
            bs_ref.doChat = 2;
        } else {
            bs_ref.doChat = 1;
        }
        // Oracle uses `strlen(bs->currentChat)` — the post-%-substitution length.
        // C types: `strlen` is size_t, so the (possibly-negative on LP64)
        // `Q_irand` roll promotes to unsigned — a negative roll wraps to a
        // huge value and the float conversion lands on 2^64 (chat scheduled
        // never). Keep the size_t-width arithmetic; i32 math here made the
        // bot chat immediately instead (lockstep frame-364 find, 2026-07-14).
        bs_ref.chatTime_stored = (c_strlen(currentChat_b as *const u8))
            .wrapping_mul(45)
            .wrapping_add(ctx.world.bg_state.rng.Q_irand(1300, 1500) as isize as usize)
            as f32;
        bs_ref.chatTime = (ctx.world.level.time as f32) + bs_ref.chatTime_stored;

        B_TempFree(ctx, crate::game_globals::MAX_CHAT_BUFFER_SIZE as c_int);

        1
    }
}

/// Faithful `strcmp` over raw NUL-terminated C strings.
unsafe fn c_strcmp(s1: *const c_char, s2: *const c_char) -> c_int {
    unsafe {
        let mut i = 0isize;
        loop {
            let c1 = *s1.offset(i) as u8;
            let c2 = *s2.offset(i) as u8;
            if c1 != c2 {
                return (c1 as c_int) - (c2 as c_int);
            }
            if c1 == 0 {
                return 0;
            }
            i += 1;
        }
    }
}

/// Raven `ParseEmotionalAttachments`.
///
/// Parses `{ name level }` pairs out of `buf` into `bs->loved[]` until
/// `MAX_LOVED_ONES` is reached or the closing `}` is hit.
///
/// Raven: `tbuf[16]` is a fixed 16-byte scratch buffer with no bounds check
/// on the digit run copied into it (matches the oracle's own UB envelope,
/// porting-rules §19 — same as the unchecked `name`/`tbuf` writes below).
/// Source: `oracle/codemp/game/ai_util.c:517-572`
pub fn ParseEmotionalAttachments(bs: *mut bot_state_t, buf: *mut c_char) {
    unsafe {
        let buf_b = buf as *const u8;
        let mut i: isize = 0;

        while *buf_b.offset(i) != 0 && *buf_b.offset(i) != b'}' {
            while *buf_b.offset(i) == b' '
                || *buf_b.offset(i) == b'{'
                || *buf_b.offset(i) == 9
                || *buf_b.offset(i) == 13
                || *buf_b.offset(i) == b'\n'
            {
                i += 1;
            }

            if *buf_b.offset(i) != 0 && *buf_b.offset(i) != b'}' {
                let lovednum = (*bs).lovednum as usize;
                let mut ic: isize = 0;
                while *buf_b.offset(i) != b'{'
                    && *buf_b.offset(i) != 9
                    && *buf_b.offset(i) != 13
                    && *buf_b.offset(i) != b'\n'
                {
                    (*bs).loved[lovednum].name[ic as usize] = *buf_b.offset(i);
                    ic += 1;
                    i += 1;
                }
                (*bs).loved[lovednum].name[ic as usize] = 0;

                while *buf_b.offset(i) == b' '
                    || *buf_b.offset(i) == b'{'
                    || *buf_b.offset(i) == 9
                    || *buf_b.offset(i) == 13
                    || *buf_b.offset(i) == b'\n'
                {
                    i += 1;
                }

                let mut tbuf = [0u8; 16];
                let mut tc: isize = 0;
                while *buf_b.offset(i) != b'{'
                    && *buf_b.offset(i) != 9
                    && *buf_b.offset(i) != 13
                    && *buf_b.offset(i) != b'\n'
                {
                    tbuf[tc as usize] = *buf_b.offset(i);
                    tc += 1;
                    i += 1;
                }
                tbuf[tc as usize] = 0;

                (*bs).loved[lovednum].level = atoi(tbuf.as_ptr() as *const c_char);

                (*bs).lovednum += 1;
            } else {
                break;
            }

            if (*bs).lovednum as usize >= MAX_LOVED_ONES {
                return;
            }

            i += 1;
        }
    }
}

/// Raven `ReadChatGroups`.
///
/// Finds the `BEGIN_CHAT_GROUPS` marker in `buf` and copies everything after
/// the next newline into `gBotChatBuffer[bs->client]` for later retrieval by
/// `BotDoChat`. Returns 1 on success, 0 if the marker is not found or the
/// section exceeds the buffer size.
///
/// Source: `oracle/codemp/game/ai_util.c:574-612`
pub fn ReadChatGroups(ctx: &mut GameContext, bs: *mut bot_state_t, buf: *mut c_char) -> c_int {
    unsafe {
        if bs.is_null() || buf.is_null() {
            return 0;
        }

        let bs_ref = &*bs;
        let buf_b = buf as *const u8;

        // Find the BEGIN_CHAT_GROUPS marker
        let marker = "BEGIN_CHAT_GROUPS\0".as_ptr() as *const u8;
        let cgroupbegin = match c_strstr(buf_b, marker) {
            Some(p) => p,
            None => return 0,
        };

        // Check size
        let strlen_result = c_strlen(cgroupbegin);
        if strlen_result >= crate::game_globals::MAX_CHAT_BUFFER_SIZE {
            crate::g_main::G_Printf(
                ctx,
                "^1Error: Personality chat section exceeds max size\n",
            );
            return 0;
        }

        // Calculate offset from buf start
        let mut cgbplace = (cgroupbegin as usize - buf_b as usize) as isize + 1;

        // Skip to end of line
        while *buf_b.offset(cgbplace) != b'\n' as u8 {
            cgbplace += 1;
        }

        // Copy to gBotChatBuffer[bs->client]
        let client_idx = bs_ref.client as usize;

        if client_idx >= mp_qshared::shared::MAX_CLIENTS {
            return 0;
        }

        let mut i = 0usize;
        while *buf_b.offset(cgbplace) != 0 && i < crate::game_globals::MAX_CHAT_BUFFER_SIZE {
            ctx.world.globals.gBotChatBuffer.0[client_idx][i] = *buf_b.offset(cgbplace) as c_char;
            i += 1;
            cgbplace += 1;
        }

        if i < crate::game_globals::MAX_CHAT_BUFFER_SIZE {
            ctx.world.globals.gBotChatBuffer.0[client_idx][i] = 0;
        }

        1
    }
}

/// Raven `BotUtilizePersonality`.
///
/// Loads a personality file (referenced in `bs->settings.personalityfile`),
/// parses skill settings, weapon weights, emotional attachments, and chat
/// groups, then populates the bot state accordingly. Falls back to defaults
/// if the file is missing or malformed.
///
/// Source: `oracle/codemp/game/ai_util.c:614-867`
pub fn BotUtilizePersonality(ctx: &mut GameContext, bs: *mut bot_state_t) {
    use core::ffi::c_int;

    unsafe {
        if bs.is_null() {
            return;
        }

        let bs_ref = &mut *bs;

        let buf = B_TempAlloc(ctx, 131072) as *mut c_char;

        // Open the personality file
        let mut f: fileHandle_t = 0;
        // Pass the raw filename bytes (no lossy UTF-8 round-trip) to FS_FOpenFile,
        // per the Cmd_TeamTask_f precedent (g_cmds.rs).
        let path = cstr_from_chars(&bs_ref.settings.personalityfile)
            .to_str()
            .unwrap_or("");
        let len = trap::FS_FOpenFile(ctx.engine, path, &mut f, FS_READ);

        let mut failed = 0 as c_int;

        if f == 0 {
            crate::g_main::G_Printf(
                ctx,
                "^1Error: Specified personality not found\n",
            );
            B_TempFree(ctx, 131072);
            return;
        }

        if len >= 131072 {
            crate::g_main::G_Printf(
                ctx,
                "^1Personality file exceeds maximum length\n",
            );
            B_TempFree(ctx, 131072);
            return;
        }

        // Read the file
        trap::FS_Read(
            ctx.engine,
            std::slice::from_raw_parts_mut(buf as *mut u8, len as usize),
            f,
        );

        let mut rlen = len;

        // Null-terminate everything after the file length
        let mut i = len;
        while i < 131072 {
            *(buf as *mut u8).offset(i as isize) = 0;
            i += 1;
        }

        rlen = len;

        let readbuf = B_TempAlloc(ctx, 1024) as *mut c_char;
        let group = B_TempAlloc(ctx, 65536) as *mut c_char;

        // Parse GeneralBotInfo group
        if GetValueGroup(buf, "GeneralBotInfo\0".as_ptr() as *mut c_char, group) == 0 {
            trap::Printf(
                ctx.engine,
                "^1Personality file contains no GeneralBotInfo group\n",
            );
            failed = 1;
        }

        // Parse reflex (default: 100)
        if failed == 0 && GetPairedValue(group, "reflex\0".as_ptr() as *mut c_char, readbuf) != 0 {
            bs_ref.skills.reflex = atoi(readbuf);
        } else {
            bs_ref.skills.reflex = 100;
        }

        // Parse accuracy (default: 10.0)
        if failed == 0 && GetPairedValue(group, "accuracy\0".as_ptr() as *mut c_char, readbuf) != 0
        {
            bs_ref.skills.accuracy = atof_bytes(CStr::from_ptr(readbuf).to_bytes()) as f32;
        } else {
            bs_ref.skills.accuracy = 10.0;
        }

        // Parse turnspeed (default: 0.01)
        if failed == 0 && GetPairedValue(group, "turnspeed\0".as_ptr() as *mut c_char, readbuf) != 0
        {
            bs_ref.skills.turnspeed = atof_bytes(CStr::from_ptr(readbuf).to_bytes()) as f32;
        } else {
            bs_ref.skills.turnspeed = 0.01;
        }

        // Parse turnspeed_combat (default: 0.05)
        if failed == 0
            && GetPairedValue(group, "turnspeed_combat\0".as_ptr() as *mut c_char, readbuf) != 0
        {
            bs_ref.skills.turnspeed_combat = atof_bytes(CStr::from_ptr(readbuf).to_bytes()) as f32;
        } else {
            bs_ref.skills.turnspeed_combat = 0.05;
        }

        // Parse maxturn (default: 360.0)
        if failed == 0 && GetPairedValue(group, "maxturn\0".as_ptr() as *mut c_char, readbuf) != 0 {
            bs_ref.skills.maxturn = atof_bytes(CStr::from_ptr(readbuf).to_bytes()) as f32;
        } else {
            bs_ref.skills.maxturn = 360.0;
        }

        // Parse perfectaim (default: 0)
        if failed == 0
            && GetPairedValue(group, "perfectaim\0".as_ptr() as *mut c_char, readbuf) != 0
        {
            bs_ref.skills.perfectaim = atoi(readbuf);
        } else {
            bs_ref.skills.perfectaim = 0;
        }

        // Parse chatability (default: 0)
        if failed == 0
            && GetPairedValue(group, "chatability\0".as_ptr() as *mut c_char, readbuf) != 0
        {
            bs_ref.canChat = atoi(readbuf);
        } else {
            bs_ref.canChat = 0;
        }

        // Parse chatfrequency (default: 5)
        if failed == 0
            && GetPairedValue(group, "chatfrequency\0".as_ptr() as *mut c_char, readbuf) != 0
        {
            bs_ref.chatFrequency = atoi(readbuf);
        } else {
            bs_ref.chatFrequency = 5;
        }

        // Parse hatelevel (default: 3)
        if failed == 0 && GetPairedValue(group, "hatelevel\0".as_ptr() as *mut c_char, readbuf) != 0
        {
            bs_ref.loved_death_thresh = atoi(readbuf);
        } else {
            bs_ref.loved_death_thresh = 3;
        }

        // Parse camper (default: 0)
        if failed == 0 && GetPairedValue(group, "camper\0".as_ptr() as *mut c_char, readbuf) != 0 {
            bs_ref.isCamper = atoi(readbuf);
        } else {
            bs_ref.isCamper = 0;
        }

        // Parse saberspecialist (default: 0)
        if failed == 0
            && GetPairedValue(group, "saberspecialist\0".as_ptr() as *mut c_char, readbuf) != 0
        {
            bs_ref.saberSpecialist = atoi(readbuf);
        } else {
            bs_ref.saberSpecialist = 0;
        }

        // Parse forceinfo (default: "5-1-000000000000000000")
        if failed == 0 && GetPairedValue(group, "forceinfo\0".as_ptr() as *mut c_char, readbuf) != 0
        {
            let len = c_strlen(readbuf as *const u8);
            let dest_len = bs_ref.forceinfo.len();
            let copy_len = if len < dest_len - 1 {
                len
            } else {
                dest_len - 1
            };
            core::ptr::copy_nonoverlapping(
                readbuf as *const u8,
                bs_ref.forceinfo.as_mut_ptr(),
                copy_len,
            );
            bs_ref.forceinfo[copy_len] = 0;
        } else {
            let default_forces = "5-1-000000000000000000\0".as_ptr() as *const c_char;
            let len = c_strlen(default_forces as *const u8);
            let dest_len = bs_ref.forceinfo.len();
            let copy_len = if len < dest_len - 1 {
                len
            } else {
                dest_len - 1
            };
            core::ptr::copy_nonoverlapping(
                default_forces as *const u8,
                bs_ref.forceinfo.as_mut_ptr(),
                copy_len,
            );
            bs_ref.forceinfo[copy_len] = 0;
        }

        // Clear the chat buffer for this bot. Raven indexes unconditionally;
        // guarded here for consistency with the other `gBotChatBuffer[client]`
        // sites in this file (see BotDoChat above) — never taken in practice.
        let client_idx = bs_ref.client as usize;
        if client_idx < mp_qshared::shared::MAX_CLIENTS {
            let mut i = 0usize;
            while i < crate::game_globals::MAX_CHAT_BUFFER_SIZE {
                ctx.world.globals.gBotChatBuffer.0[client_idx][i] = 0;
                i += 1;
            }
        }

        // Read chat groups if chat is enabled
        if bs_ref.canChat != 0 {
            if ReadChatGroups(ctx, bs, buf) == 0 {
                bs_ref.canChat = 0;
            }
        }

        // Parse weapon weights
        if GetValueGroup(buf, "BotWeaponWeights\0".as_ptr() as *mut c_char, group) != 0 {
            if GetPairedValue(group, "WP_STUN_BATON\0".as_ptr() as *mut c_char, readbuf) != 0 {
                bs_ref.botWeaponWeights[mp_bg::weapons::weapon_t::WP_STUN_BATON as usize] =
                    atoi(readbuf) as f32;
                bs_ref.botWeaponWeights[mp_bg::weapons::weapon_t::WP_MELEE as usize] =
                    bs_ref.botWeaponWeights[mp_bg::weapons::weapon_t::WP_STUN_BATON as usize];
            }

            if GetPairedValue(group, "WP_SABER\0".as_ptr() as *mut c_char, readbuf) != 0 {
                bs_ref.botWeaponWeights[mp_bg::weapons::weapon_t::WP_SABER as usize] =
                    atoi(readbuf) as f32;
            }

            if GetPairedValue(group, "WP_BRYAR_PISTOL\0".as_ptr() as *mut c_char, readbuf) != 0 {
                bs_ref.botWeaponWeights[mp_bg::weapons::weapon_t::WP_BRYAR_PISTOL as usize] =
                    atoi(readbuf) as f32;
            }

            if GetPairedValue(group, "WP_BLASTER\0".as_ptr() as *mut c_char, readbuf) != 0 {
                bs_ref.botWeaponWeights[mp_bg::weapons::weapon_t::WP_BLASTER as usize] =
                    atoi(readbuf) as f32;
            }

            if GetPairedValue(group, "WP_DISRUPTOR\0".as_ptr() as *mut c_char, readbuf) != 0 {
                bs_ref.botWeaponWeights[mp_bg::weapons::weapon_t::WP_DISRUPTOR as usize] =
                    atoi(readbuf) as f32;
            }

            if GetPairedValue(group, "WP_BOWCASTER\0".as_ptr() as *mut c_char, readbuf) != 0 {
                bs_ref.botWeaponWeights[mp_bg::weapons::weapon_t::WP_BOWCASTER as usize] =
                    atoi(readbuf) as f32;
            }

            if GetPairedValue(group, "WP_REPEATER\0".as_ptr() as *mut c_char, readbuf) != 0 {
                bs_ref.botWeaponWeights[mp_bg::weapons::weapon_t::WP_REPEATER as usize] =
                    atoi(readbuf) as f32;
            }

            if GetPairedValue(group, "WP_DEMP2\0".as_ptr() as *mut c_char, readbuf) != 0 {
                bs_ref.botWeaponWeights[mp_bg::weapons::weapon_t::WP_DEMP2 as usize] =
                    atoi(readbuf) as f32;
            }

            if GetPairedValue(group, "WP_FLECHETTE\0".as_ptr() as *mut c_char, readbuf) != 0 {
                bs_ref.botWeaponWeights[mp_bg::weapons::weapon_t::WP_FLECHETTE as usize] =
                    atoi(readbuf) as f32;
            }

            if GetPairedValue(
                group,
                "WP_ROCKET_LAUNCHER\0".as_ptr() as *mut c_char,
                readbuf,
            ) != 0
            {
                bs_ref.botWeaponWeights[mp_bg::weapons::weapon_t::WP_ROCKET_LAUNCHER as usize] =
                    atoi(readbuf) as f32;
            }

            if GetPairedValue(group, "WP_THERMAL\0".as_ptr() as *mut c_char, readbuf) != 0 {
                bs_ref.botWeaponWeights[mp_bg::weapons::weapon_t::WP_THERMAL as usize] =
                    atoi(readbuf) as f32;
            }

            if GetPairedValue(group, "WP_TRIP_MINE\0".as_ptr() as *mut c_char, readbuf) != 0 {
                bs_ref.botWeaponWeights[mp_bg::weapons::weapon_t::WP_TRIP_MINE as usize] =
                    atoi(readbuf) as f32;
            }

            if GetPairedValue(group, "WP_DET_PACK\0".as_ptr() as *mut c_char, readbuf) != 0 {
                bs_ref.botWeaponWeights[mp_bg::weapons::weapon_t::WP_DET_PACK as usize] =
                    atoi(readbuf) as f32;
            }
        }

        bs_ref.lovednum = 0;

        // Parse emotional attachments
        if GetValueGroup(buf, "EmotionalAttachments\0".as_ptr() as *mut c_char, group) != 0 {
            ParseEmotionalAttachments(bs, group);
        }

        // Free temporary buffers
        B_TempFree(ctx, 131072);
        B_TempFree(ctx, 1024);
        B_TempFree(ctx, 65536);

        // Close the file
        trap::FS_FCloseFile(ctx.engine, f);
    }
}

// ---- local raw-C-string helpers (no libc dependency in this crate) ----

/// Faithful `strlen` over a raw NUL-terminated byte pointer.
unsafe fn c_strlen(s: *const u8) -> usize {
    let mut n = 0usize;
    unsafe {
        while *s.add(n) != 0 {
            n += 1;
        }
    }
    n
}

/// Faithful `strstr` over raw NUL-terminated byte pointers.
unsafe fn c_strstr(haystack: *const u8, needle: *const u8) -> Option<*const u8> {
    unsafe {
        let needle_len = c_strlen(needle);
        if needle_len == 0 {
            return Some(haystack);
        }
        let mut h = haystack;
        while *h != 0 {
            let mut k = 0usize;
            while k < needle_len && *h.add(k) == *needle.add(k) && *h.add(k) != 0 {
                k += 1;
            }
            if k == needle_len {
                return Some(h);
            }
            h = h.add(1);
        }
        None
    }
}
