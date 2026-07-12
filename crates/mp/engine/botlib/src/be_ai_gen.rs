//! `be_ai_gen.cpp` — the genetic-algorithm bot rank/parent/child selection
//! helpers.
//!
//! Source: `oracle/codemp/botlib/be_ai_gen.cpp`

use std::os::raw::c_int;

use mp_engine_qcommon::common::Common;
use mp_qshared::common::mp::botlib::print_type::PRT_WARNING;

use crate::BotLib;

/// Raven `GeneticSelection`.
///
/// Source: `oracle/codemp/botlib/be_ai_gen.cpp:35-66`
pub fn GeneticSelection(common: &mut Common, numranks: c_int, rankings: *mut f32) -> c_int {
    let mut sum: f32 = 0.0;
    let mut i: c_int = 0;
    while i < numranks {
        unsafe {
            if *rankings.offset(i as isize) >= 0.0 {
                sum += *rankings.offset(i as isize);
            }
        }
        i += 1;
    }
    if sum > 0.0 {
        // select a bot where the ones with the higest rankings have
        // the highest chance of being selected
        let select = common.qrand.flrand(0.0, 1.0) * sum;
        let mut sum = select;
        let mut i: c_int = 0;
        while i < numranks {
            unsafe {
                let r = *rankings.offset(i as isize);
                if r >= 0.0 {
                    sum -= r;
                    if sum <= 0.0 {
                        return i;
                    }
                }
            }
            i += 1;
        }
    }
    // select a bot randomly
    let mut index = (common.qrand.flrand(0.0, 1.0) * numranks as f32) as c_int;
    let mut i: c_int = 0;
    while i < numranks {
        unsafe {
            if *rankings.offset(index as isize) >= 0.0 {
                return index;
            }
        }
        index = (index + 1) % numranks;
        i += 1;
    }
    0
}

/// Raven `GeneticParentsAndChildSelection`.
///
/// Source: `oracle/codemp/botlib/be_ai_gen.cpp:73-117`
pub fn GeneticParentsAndChildSelection(
    common: &mut Common,
    bot: &mut BotLib,
    numranks: c_int,
    ranks: *mut f32,
    parent1: *mut c_int,
    parent2: *mut c_int,
    child: *mut c_int,
) -> c_int {
    // §19: Raven's `float rankings[256]` local is read only after being
    // populated by `Com_Memcpy` below; zero-init here to satisfy Rust.
    let mut rankings: [f32; 256] = [0.0; 256];

    if numranks > 256 {
        unsafe {
            bot.botimport.Print.unwrap()(
                PRT_WARNING,
                c"GeneticParentsAndChildSelection: too many bots\n".as_ptr()
                    as *mut std::os::raw::c_char,
            );
            *parent1 = 0;
            *parent2 = 0;
            *child = 0;
        }
        return 0; // qfalse
    }

    let mut max: f32 = 0.0;
    let mut i: c_int = 0;
    while i < numranks {
        unsafe {
            if *ranks.offset(i as isize) >= 0.0 {
                max += 1.0;
            }
        }
        i += 1;
    }
    if max < 3.0 {
        unsafe {
            bot.botimport.Print.unwrap()(
                PRT_WARNING,
                c"GeneticParentsAndChildSelection: too few valid bots\n".as_ptr()
                    as *mut std::os::raw::c_char,
            );
            *parent1 = 0;
            *parent2 = 0;
            *child = 0;
        }
        return 0; // qfalse
    }

    mp_engine_qcommon::common_fns::Com_Memcpy(
        rankings.as_mut_ptr() as *mut (),
        ranks as *const (),
        std::mem::size_of::<f32>() * numranks as usize,
    );

    // select first parent
    unsafe {
        *parent1 = GeneticSelection(common, numranks, rankings.as_mut_ptr());
        rankings[*parent1 as usize] = -1.0;

        // select second parent
        *parent2 = GeneticSelection(common, numranks, rankings.as_mut_ptr());
        rankings[*parent2 as usize] = -1.0;
    }

    // reverse the rankings
    let mut max: f32 = 0.0;
    let mut i: c_int = 0;
    while i < numranks {
        let r = rankings[i as usize];
        if r >= 0.0 && r > max {
            max = r;
        }
        i += 1;
    }
    let mut i: c_int = 0;
    while i < numranks {
        let r = rankings[i as usize];
        if r >= 0.0 {
            rankings[i as usize] = max - r;
        }
        i += 1;
    }

    // select child
    unsafe {
        *child = GeneticSelection(common, numranks, rankings.as_mut_ptr());
    }

    1 // qtrue
}
