//! See [`View`].

use crossterm::event::Event;
use ratatui::{buffer::Buffer, layout::Rect, widgets::WidgetRef};
use std::io;

/// Trait representing a view of application.
///
/// This trait can be used to define the rendering [`View::render_view()`] of its properties and how it should handle events [`View::handle_events()`].
pub trait View {
    fn handle_events(&mut self, _event: &Event) -> io::Result<()> {
        Ok(())
    }

    fn render_view(&self, area: Rect, buf: &mut Buffer);
}

pub(crate) struct ViewWidgetWrapper<'a>(pub(crate) &'a Box<dyn View + Send + Sync>);

impl<'a> WidgetRef for ViewWidgetWrapper<'a> {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        self.0.render_view(area, buf);
    }
}
