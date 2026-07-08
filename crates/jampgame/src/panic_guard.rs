//! Panic capture at the module's C-ABI boundary (ABI panic policy, 2026-07-08).
//!
//! `std::panic::catch_unwind` recovers only a lossy `Box<dyn Any>`: it can pull
//! a `&str`/`String` payload back out, but it can NEVER recover the panic
//! *location*. So `dllEntry` installs a process panic hook that records both the
//! payload AND the `file:line:col` location into a thread-local the instant a
//! panic fires; the `vmMain` choke point reads it back after `catch_unwind`
//! unwinds, so every panic yields a readable `file:line — message` before the
//! game dies through the engine's error path.

use std::cell::RefCell;

thread_local! {
    /// The most recent panic's formatted `file:line:col — message` record,
    /// captured by [`install_hook`]'s hook and consumed once by [`take`] after
    /// `catch_unwind` at `vmMain`.
    static LAST_PANIC: RefCell<Option<String>> = const { RefCell::new(None) };
}

/// Install the process panic hook. Called once from `dllEntry`. The hook
/// records every panic's location + payload so the `vmMain` boundary can report
/// a readable message through the engine instead of losing it to
/// `catch_unwind`'s opaque `Any`.
pub fn install_hook() {
    std::panic::set_hook(Box::new(|info: &std::panic::PanicHookInfo<'_>| {
        let location = info
            .location()
            .map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column()));
        let message = payload_message(info.payload());
        store(format_record(location.as_deref(), &message));
    }));
}

/// Take (and clear) the last captured panic record for the current thread.
pub fn take() -> Option<String> {
    LAST_PANIC.with(|c| c.borrow_mut().take())
}

/// Extract a human-readable message from a panic payload — the common
/// `&str`/`String` cases (`panic!`/`assert!`/`.expect()` all land here);
/// anything else is opaque to `Any` and reported as such.
fn payload_message(payload: &(dyn std::any::Any + Send)) -> String {
    if let Some(s) = payload.downcast_ref::<&str>() {
        (*s).to_string()
    } else if let Some(s) = payload.downcast_ref::<String>() {
        s.clone()
    } else {
        "<non-string panic payload>".to_string()
    }
}

/// Format the captured `location` + `message` into the stored record.
fn format_record(location: Option<&str>, message: &str) -> String {
    match location {
        Some(loc) => format!("{loc} — {message}"),
        None => format!("<unknown location> — {message}"),
    }
}

/// Store a record into the thread-local, overwriting any prior one.
fn store(record: String) {
    LAST_PANIC.with(|c| *c.borrow_mut() = Some(record));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_record_names_file_line_and_message() {
        let r = format_record(Some("g_main.rs:42:9"), "boom");
        assert!(r.contains("g_main.rs:42:9"), "record must name file:line: {r}");
        assert!(r.contains("boom"), "record must carry the payload: {r}");
    }

    #[test]
    fn format_record_tolerates_missing_location() {
        let r = format_record(None, "boom");
        assert!(r.contains("boom"));
        assert!(r.contains("unknown location"));
    }

    #[test]
    fn store_take_roundtrips_and_clears() {
        store("g_utils.rs:7:1 — kaboom".to_string());
        let got = take().expect("a record was stored");
        assert!(got.contains("g_utils.rs:7:1"));
        assert!(got.contains("kaboom"));
        assert!(take().is_none(), "take must clear the slot");
    }

    #[test]
    fn hook_captures_real_panic_payload_and_location() {
        // Install our hook, provoke a real panic through catch_unwind, then
        // restore the harness hook. Proves the wiring end to end: payload AND
        // this file's location both survive into the record.
        let prev = std::panic::take_hook();
        install_hook();
        let _ = take(); // clear any stray record
        let caught = std::panic::catch_unwind(|| panic!("integration boom"));
        std::panic::set_hook(prev);

        assert!(caught.is_err(), "the panic must have been caught");
        let rec = take().expect("the hook recorded the panic");
        assert!(rec.contains("integration boom"), "payload captured: {rec}");
        assert!(
            rec.contains("panic_guard.rs"),
            "location captured (this file): {rec}"
        );
    }
}
