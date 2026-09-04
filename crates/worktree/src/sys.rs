//! The doors out of the sandbox, and the stubs that stand in for them.
//!
//! This plugin uses two of the host's imports and could not reach anything if
//! it wanted to: it has no network, no filesystem, no clock and no thread. The
//! one thing it does need — whether a tab's directory is a worktree — is not
//! something it can go and look at; the host answers it, and only because
//! somebody allowed it to. They are wrapped here rather than called from the
//! logic for one reason: everything else in this crate then builds and runs on
//! an ordinary machine, so what this plugin decides is decided by something
//! `cargo test` can run without a terminal to install a plugin into.

/// How loud a line to the host's log is.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Level {
    /// Something failed.
    Error = 1,
    /// Something is not right but nothing failed.
    Warn = 2,
    /// Worth knowing.
    Info = 3,
}

#[cfg(target_arch = "wasm32")]
mod imports {
    #[link(wasm_import_module = "crook")]
    unsafe extern "C" {
        pub fn contribute(
            slot: *const u8,
            slot_len: usize,
            entry: *const u8,
            entry_len: usize,
            order: i32,
        );
        pub fn log(level: i32, text: *const u8, len: usize);
    }
}

/// Contributes to a slot the host declares.
///
/// `order` is where this goes among everything else in the slot, and lower is
/// earlier. `tab.row.badge` holds one thing, so the order is what decides
/// whether this plugin or another one draws it.
pub fn contribute(slot: &str, entry: &str, order: i32) {
    #[cfg(target_arch = "wasm32")]
    unsafe {
        imports::contribute(
            slot.as_ptr(),
            slot.len(),
            entry.as_ptr(),
            entry.len(),
            order,
        );
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (slot, entry, order);
    }
}

/// Says something in the host's log, which is the only way a plugin can be
/// heard by whoever is running it when it has nothing to draw.
pub fn log(level: Level, text: &str) {
    #[cfg(target_arch = "wasm32")]
    unsafe {
        imports::log(level as i32, text.as_ptr(), text.len());
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (level, text);
    }
}
