//! A mark on every tab whose directory is a linked git worktree.
//!
//! A worktree is a second checkout of one repository, on a branch of its own,
//! in a directory of its own. Crook opens one beside the tab that asked for it
//! and folds the two into a group — but a group says "these two belong
//! together", not "this one is the copy", and a person coming back to a window
//! of eight tabs has no way to tell which of them are worktrees and which are
//! the checkouts they were cut from. This says it: a small mark on the corner
//! of the tab's own mark, on the worktrees and nowhere else.
//!
//! It is deliberately *not* about how the tab was opened. A worktree somebody
//! made at a shell three years ago gets the mark, and so does the one Crook
//! made this morning, because what the mark means is "this directory is a
//! worktree" and that is a fact about the directory.
//!
//! # What it is allowed to do
//!
//! One thing: **see which project each tab is in**. That is
//! `Capability::ReadWorkingDirectory`, it is the only way to answer the
//! question this plugin exists to answer, and until somebody allows it on the
//! Plugins page the plugin draws nothing at all — the host does not refuse it,
//! it simply hands over a row with the directory left out, and a row this
//! plugin cannot see the directory of is a row it has nothing to say about.
//!
//! It cannot read the files in that directory, run git, reach the network, or
//! find out what any of your agents are doing. It has no way to: a sandboxed
//! plugin asks the host for everything and this one asks for nothing at all —
//! its whole side of the boundary is one boolean the host puts in front of it.
//!
//! # What this file is
//!
//! The ABI, and nothing that thinks. Every export the host calls is here, each
//! of them is three lines, and each hands over to [`mark`] — which is a plain
//! Rust module `cargo test` runs on an ordinary machine. See [`sys`] for the
//! other half of that trick.

use std::cell::UnsafeCell;

use crook_plugin_api::{ABI_VERSION, Capability, Manifest, Render, from_bytes, to_bytes};

pub mod mark;
pub mod sys;

use mark::badge;

/// The slot this plugin takes: the small mark on the corner of a row's own.
const SLOT: &str = "tab.row.badge";

/// What this contribution is called, within this plugin.
const ENTRY: &str = "worktree";

/// Where it goes among everything else in that slot.
///
/// Zero, which is the same claim every plugin that wants the badge makes: the
/// slot holds one thing, the lowest order wins it, and a plugin somebody
/// installs later and means more by can still take it back.
const ORDER: i32 = 0;

/// A `static` that is only ever touched by one thread, which on wasm32 is
/// every thread there is.
struct Single<T>(UnsafeCell<T>);

// SAFETY: wasm32 has one thread. Off wasm this crate is a library under test,
// where nothing reaches these statics at all.
unsafe impl<T> Sync for Single<T> {}

impl<T> Single<T> {
    /// SAFETY: the caller must not be inside another borrow. Every export
    /// below takes one, does its work and returns, and the host does not call
    /// in while it is already inside.
    #[allow(clippy::mut_from_ref)]
    unsafe fn get(&self) -> &mut T {
        unsafe { &mut *self.0.get() }
    }
}

/// The last thing handed back to the host, kept alive until the next one.
///
/// A tree is answered as an offset and a length into this memory, so the bytes
/// have to outlive the call that returned them. Kept rather than leaked
/// because a render happens once per row per frame.
static ANSWER: Single<Vec<u8>> = Single(UnsafeCell::new(Vec::new()));

/// Whether the line about not being allowed to see anything has been said.
///
/// Once per build, and a build is what answering on the Plugins page causes —
/// so allowing it stops the line and forbidding it again says it once more. A
/// warning per row per frame would be sixty lines a second about a plugin
/// working exactly as it was told to.
static SAID: Single<bool> = Single(UnsafeCell::new(false));

/// Packs an answer as the host reads it: `(pointer << 32) | length`.
fn hand_back(bytes: Vec<u8>) -> i64 {
    // SAFETY: see `Single::get`.
    let answer = unsafe { ANSWER.get() };
    *answer = bytes;
    ((answer.as_ptr() as u64) << 32 | answer.len() as u64) as i64
}

/// Which version of the vocabulary this was built against.
///
/// Called before anything else, and a mismatch is a refusal by number rather
/// than a plugin that decodes a shape which means something else now.
#[unsafe(no_mangle)]
pub extern "C" fn crook_abi_version() -> i32 {
    ABI_VERSION as i32
}

/// Somewhere for the host to put the bytes it is handing over.
///
/// Exact rather than `Vec::with_capacity`, because [`take`] frees it with the
/// same layout and a capacity the allocator rounded up would be a free of the
/// wrong size.
#[unsafe(no_mangle)]
pub extern "C" fn crook_alloc(length: i32) -> i32 {
    let Ok(layout) = std::alloc::Layout::from_size_align(length.max(1) as usize, 1) else {
        return 0;
    };
    // SAFETY: a non-zero size, and a layout built for it.
    unsafe { std::alloc::alloc(layout) as i32 }
}

/// Copies out what the host wrote there, and gives the memory back.
///
/// SAFETY: `pointer` and `length` must be exactly what a previous
/// [`crook_alloc`] answered and what the host wrote into.
unsafe fn take(pointer: i32, length: i32) -> Vec<u8> {
    if pointer <= 0 || length < 0 {
        return Vec::new();
    }
    // SAFETY: the host wrote `length` bytes at `pointer` before calling in.
    let bytes =
        unsafe { std::slice::from_raw_parts(pointer as *const u8, length as usize) }.to_vec();
    // SAFETY: the same layout `crook_alloc` used.
    unsafe {
        std::alloc::dealloc(
            pointer as *mut u8,
            std::alloc::Layout::from_size_align_unchecked(length.max(1) as usize, 1),
        );
    }
    bytes
}

/// What this plugin is and what it needs to be allowed to do.
///
/// Read before any of it runs, which is what lets a person see what it wants
/// and refuse it without running a line of it.
#[unsafe(no_mangle)]
pub extern "C" fn crook_manifest() -> i64 {
    hand_back(to_bytes(&manifest()).unwrap_or_default())
}

/// The manifest, as a value, so that a test can read it.
pub fn manifest() -> Manifest {
    Manifest {
        abi: ABI_VERSION,
        id: String::from("theguriev/worktree"),
        name: String::from("Worktree"),
        description: String::from("A mark on every tab whose directory is a git worktree."),
        version: String::from(env!("CARGO_PKG_VERSION")),
        capabilities: vec![Capability::ReadWorkingDirectory],
    }
}

/// Takes the badge on a row's mark, and registers nothing else.
///
/// The flag that says whether the "allow me" line has been said goes back to
/// false here, because a build is what a person answering a permission causes:
/// a plugin that kept it would say nothing the next time it was forbidden.
#[unsafe(no_mangle)]
pub extern "C" fn crook_build() -> i32 {
    // SAFETY: see `Single::get`.
    *unsafe { SAID.get() } = false;
    sys::contribute(SLOT, ENTRY, ORDER);
    0
}

/// What to draw on one row.
#[unsafe(no_mangle)]
pub extern "C" fn crook_render(pointer: i32, length: i32) -> i64 {
    // SAFETY: the host allocated and wrote this before calling in.
    let bytes = unsafe { take(pointer, length) };
    let render = from_bytes::<Render>(&bytes).ok();

    if mark::is_blind(render.as_ref()) {
        say_once();
    }

    hand_back(to_bytes(&badge(render, SLOT)).unwrap_or_default())
}

/// Says, once, that the plugin has been installed and not allowed.
///
/// The badge is eleven pixels across and there is nothing legible to draw
/// there instead — a mark saying "ask somebody about me" on every row would be
/// worse than the silence. So it goes in the log, where whoever installed the
/// plugin and is wondering why nothing happened can find it.
fn say_once() {
    // SAFETY: see `Single::get`.
    let said = unsafe { SAID.get() };
    if *said {
        return;
    }
    *said = true;
    sys::log(
        sys::Level::Warn,
        "nothing will be marked until this plugin is allowed to see which \
         project each tab is in, on Settings \u{2192} Plugins",
    );
}

/// Runs one of the actions registered while building, of which there are none.
///
/// Exported anyway. The host is entitled to call it, and a module missing an
/// export the ABI names is a plugin that fails at the boundary rather than one
/// that politely does nothing.
#[unsafe(no_mangle)]
pub extern "C" fn crook_run(pointer: i32, length: i32) -> i32 {
    // SAFETY: as above. Taken and dropped, so the host's allocation is freed
    // rather than leaked once per call.
    let name = unsafe { take(pointer, length) };
    sys::log(
        sys::Level::Warn,
        &format!(
            "asked to run {:?}, which this plugin never registered",
            String::from_utf8_lossy(&name)
        ),
    );
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_asks_for_the_one_thing_it_needs_and_nothing_else() {
        let capabilities = manifest().capabilities;

        assert_eq!(capabilities, vec![Capability::ReadWorkingDirectory]);
        assert_eq!(
            capabilities[0].sentence(),
            "See which project each tab is in",
            "the sentence a person is asked to agree to has changed"
        );
        assert_eq!(manifest().abi, ABI_VERSION);
        assert_eq!(manifest().id, "theguriev/worktree");
    }

    #[test]
    fn the_manifest_crosses_the_wire_as_itself() {
        let bytes = to_bytes(&manifest()).expect("a manifest should encode");

        assert_eq!(
            from_bytes::<Manifest>(&bytes).expect("and decode"),
            manifest()
        );
    }
}
