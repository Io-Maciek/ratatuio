//! See [`View`].

use crate::app::Context;
use crossterm::event::{Event, KeyEvent, KeyEventKind, MouseEvent};
use ratatui::Frame;
use std::{io, time::Duration};

/// Trait representing a single screen of the application.
///
/// Only [`View::render()`] is required. Everything else has a default no-op implementation.
///
/// Views are owned by [`crate::app::App`] and run on a single thread, so they do not need
/// to be `Send` or `Sync` and can freely use `Rc<RefCell<_>>` and similar types.
pub trait View {
    /// Called for every input event. Use `ctx` to switch views or quit the application.
    /*fn handle_event(&mut self, _event: &Event, _ctx: &mut Context) -> io::Result<()> {
        Ok(())
    }*/
    fn handle_event(&mut self, event: &Event, ctx: &mut Context) -> io::Result<()> {
        match event {
            Event::Key(k) if k.kind == KeyEventKind::Press => self.on_key(*k, ctx),
            Event::Mouse(m) => self.on_mouse(*m, ctx),
            Event::Resize(w, h) => self.on_resize(*w, *h, ctx),
            Event::Paste(s) => self.on_paste(s, ctx),
            _ => Ok(()),
        }
    }

    /// Called at the app's tick rate. `dt` is the time since the previous call.
    fn update(&mut self, _ctx: &mut Context, _dt: Duration) {}

    /// Draws the view. Gives full access to the ratatui [`Frame`]
    /// (stateful widgets, cursor position, etc.).
    fn render(&mut self, frame: &mut Frame);

    /// Called once, right before the view is drawn for the first time.
    fn on_appear(&mut self) {}

    /// Called once, right before the view is replaced or the application exits.
    fn on_disappear(&mut self) {}

    fn on_key(&mut self, _key: KeyEvent, _ctx: &mut Context) -> io::Result<()> { Ok(()) }
    fn on_mouse(&mut self, _m: MouseEvent, _ctx: &mut Context) -> io::Result<()> { Ok(()) }
    fn on_resize(&mut self, _w: u16, _h: u16, _ctx: &mut Context) -> io::Result<()> { Ok(()) }
    fn on_paste(&mut self, _text: &str, _ctx: &mut Context) -> io::Result<()> { Ok(()) }
}