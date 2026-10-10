use crate::views::{BlockView, EventsView, GaugeView, ListView, ParagraphView};
use ratatuio::app::Context;
use ratatuio::crossterm::event::{KeyCode, KeyEvent};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Page {
    Paragraph,
    Block,
    List,
    Gauge,
    Events,
}

impl Page {
    const ALL: [Page; 5] = [
        Page::Paragraph,
        Page::Block,
        Page::List,
        Page::Gauge,
        Page::Events,
    ];

    fn index(self) -> usize {
        Self::ALL.iter().position(|p| *p == self).unwrap()
    }

    pub fn next(self) -> Page {
        Self::ALL[(self.index() + 1) % Self::ALL.len()]
    }

    pub fn prev(self) -> Page {
        Self::ALL[(self.index() + Self::ALL.len() - 1) % Self::ALL.len()]
    }

    /// Requests a switch to the view that belongs to this page.
    pub fn open(self, ctx: &mut Context) {
        match self {
            Page::Paragraph => ctx.change_view(ParagraphView::new()),
            Page::Block => ctx.change_view(BlockView::new()),
            Page::List => ctx.change_view(ListView::new()),
            Page::Gauge => ctx.change_view(GaugeView::new()),
            Page::Events => ctx.change_view(EventsView),
        }
    }
}

/// Keys shared by every view: quit and previous/next page.
/// Returns `true` when the key was consumed.
pub fn handle_global_key(key: KeyEvent, current: Page, ctx: &mut Context) -> bool {
    match key.code {
        KeyCode::Char('q' | 'Q') => ctx.quit(),
        KeyCode::Right | KeyCode::Char('d' | 'D') => current.next().open(ctx),
        KeyCode::Left | KeyCode::Char('a' | 'A') => current.prev().open(ctx),
        _ => return false,
    }
    true
}