//! A tiny global log shared by all views, shown in the "Events" view.
//!
//! `thread_local!` is enough because the whole UI runs on one thread
//! (this is exactly why `View` does not require `Send + Sync`).

use std::cell::RefCell;

thread_local! {
    static LOG: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
}

pub fn push(msg: impl Into<String>) {
    LOG.with(|l| l.borrow_mut().push(msg.into()));
}

/// Returns the last `n` entries.
pub fn tail(n: usize) -> Vec<String> {
    LOG.with(|l| {
        let l = l.borrow();
        l[l.len().saturating_sub(n)..].to_vec()
    })
}

pub fn clear() {
    LOG.with(|l| l.borrow_mut().clear());
}