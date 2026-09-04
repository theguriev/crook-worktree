//! What a row comes to: a mark, or nothing.
//!
//! The whole of this plugin's thinking, and it is one boolean. Kept away from
//! [`crate::sys`] so that `cargo test` runs it on an ordinary machine: what a
//! plugin *decides* should not need a terminal to install it into before
//! anybody can find out whether it is right.

use crook_plugin_api::{Node, Render, Subject, Tone};

/// The mark, by the name Crook knows it as.
///
/// Named rather than drawn, which is the rule this tier is built on: a plugin
/// that shipped its own vector art would be a plugin whose mark is the wrong
/// weight beside everything else in the window, and one that survives a theme
/// written years from now cannot have chosen a colour either. So this asks for
/// the icon the panel already uses for a branch, in the tone the theme
/// reserves for the thing being pointed at.
const ICON: &str = "git-branch";

/// Which of the theme's tones it takes.
///
/// Accent, because a worktree is not a warning. The mark says "this checkout
/// is the second one", which is a fact worth noticing and nothing to be
/// alarmed by — and a badge in the danger tone on half a person's tabs is a
/// panel that looks like something has gone wrong with it.
const TONE: Tone = Tone::Accent;

/// What to draw on one row.
///
/// Nothing at all unless three things are true: the host is asking about the
/// slot this plugin took, the subject is a tab, and that tab's directory is a
/// worktree. Every other shape is a row this plugin has nothing to say about,
/// and drawing nothing on one is not a hole — the host puts back whatever it
/// would have drawn there without a plugin, which for the badge is nothing.
///
/// A row whose `place` is `None` is the one worth naming: it is not a row
/// without a directory, it is a row this plugin was not allowed to see the
/// directory of. Both come to the same drawing, and only one of them is worth
/// saying out loud — see [`is_blind`].
pub fn badge(render: Option<Render>, slot: &str) -> Node {
    let Some(render) = render else {
        return Node::Empty;
    };
    let (true, Some(Subject::Tab(facts))) = (render.slot == slot, render.subject) else {
        return Node::Empty;
    };

    match facts.place {
        Some(place) if place.worktree => Node::Icon {
            name: String::from(ICON),
            tone: TONE,
        },
        _ => Node::Empty,
    }
}

/// Whether this render is one the plugin was not allowed to answer.
///
/// A tab arrives with its `place` left out when
/// `Capability::ReadWorkingDirectory` was not granted — the host redacts
/// rather than refuses, so a plugin nobody has answered for yet is a plugin
/// that draws nothing rather than one that fails. Which is correct behaviour
/// and completely indistinguishable, from the outside, from a plugin that is
/// broken. So this is what the log line is decided by.
///
/// A tab that simply has no directory yet would be reported the same way, and
/// that is why the line says "nothing will be marked" rather than "you have
/// not allowed this": a session that has not said where it is working is a
/// real state, briefly, on every tab that has just been opened.
pub fn is_blind(render: Option<&Render>) -> bool {
    matches!(
        render.and_then(|render| render.subject.as_ref()),
        Some(Subject::Tab(facts)) if facts.place.is_none()
    )
}

#[cfg(test)]
#[path = "mark_tests.rs"]
mod tests;
