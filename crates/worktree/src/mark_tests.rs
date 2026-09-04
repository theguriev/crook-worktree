//! What a row comes to, proven against every shape the host can send.

use super::*;
use crook_plugin_api::{Place, TabFacts};

/// The slot this plugin took, which every request below names.
const SLOT: &str = "tab.row.badge";

/// A request about a row whose directory the plugin may see.
fn row(worktree: bool) -> Option<Render> {
    Some(Render {
        slot: String::from(SLOT),
        subject: Some(Subject::Tab(TabFacts {
            key: 12,
            tab: None,
            place: Some(Place {
                directory: String::from("/home/somebody/work/crook"),
                branch: Some(String::from("main")),
                worktree,
            }),
        })),
    })
}

/// The same row, for a plugin nobody has allowed anything.
fn redacted() -> Option<Render> {
    Some(Render {
        slot: String::from(SLOT),
        subject: Some(Subject::Tab(TabFacts {
            key: 12,
            tab: None,
            place: None,
        })),
    })
}

#[test]
fn a_worktree_is_marked_and_a_checkout_is_not() {
    assert_eq!(
        badge(row(true), SLOT),
        Node::Icon {
            name: String::from(ICON),
            tone: TONE,
        }
    );
    assert_eq!(badge(row(false), SLOT), Node::Empty);
}

#[test]
fn a_row_it_may_not_see_is_a_row_it_says_nothing_about() {
    // Not a failure and not a refusal: the host redacted the directory
    // because nobody has allowed this plugin to see it, and a plugin that
    // cannot tell a worktree from a checkout must not guess at one.
    assert_eq!(badge(redacted(), SLOT), Node::Empty);
    assert!(is_blind(redacted().as_ref()));
    assert!(!is_blind(row(true).as_ref()));
}

#[test]
fn anything_it_was_not_asked_draws_nothing() {
    assert_eq!(badge(None, SLOT), Node::Empty);
    assert!(!is_blind(None));
    assert_eq!(
        badge(
            Some(Render {
                slot: String::from("header.right"),
                subject: None,
            }),
            SLOT
        ),
        Node::Empty
    );
    assert_eq!(
        badge(
            Some(Render {
                slot: String::from(SLOT),
                subject: None,
            }),
            SLOT
        ),
        Node::Empty
    );
}
