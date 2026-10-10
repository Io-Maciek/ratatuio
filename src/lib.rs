//! This library expands on [ratatui] by abstracting the main loop and view navigation, which can be
//! helpful in making modular applications with many [`view::View`]s as well as using architectural
//! patterns like MVVM.
//!
//! ## How to run
//! 1. Implement [`view::View`] for your screen (only [`view::View::render()`] is required).
//! 2. Create the application with [`app::App::new()`] and the starting view.
//! 3. Call [`app::App::run()`]. The terminal is restored automatically on exit or error.
//! 4. Inside a view, use [`app::Context`] to switch views ([`app::Context::change_view()`])
//!    or exit ([`app::Context::quit()`]).
//!
//! ## Basic example
//!
//! ```no_run
//! use ratatuio::app::{App, Context};
//! use ratatuio::view::View;
//! use ratatuio::crossterm::event::{KeyCode, KeyEvent};
//! use ratatuio::ratatui::{widgets::Paragraph, Frame};
//! use std::io;
//!
//! struct MainPage;
//!
//! impl View for MainPage {
//!     fn render(&mut self, frame: &mut Frame) {
//!         frame.render_widget(Paragraph::new("Hello World! Press 'q' to quit."), frame.area());
//!     }
//!
//!     fn on_key(&mut self, key: KeyEvent, ctx: &mut Context) -> io::Result<()> {
//!         if matches!(key.code, KeyCode::Char('q') | KeyCode::Char('Q')) {
//!             ctx.quit();
//!         }
//!         Ok(())
//!     }
//! }
//!
//! fn main() -> io::Result<()> {
//!     App::new(MainPage).run()
//! }
//! ```
//!
//! ## Handling input
//!
//! For the typical case override the specialized methods: [`view::View::on_key()`],
//! [`view::View::on_mouse()`], [`view::View::on_resize()`] and [`view::View::on_paste()`].
//! Key events are filtered to `Press` only, so actions do not fire twice on Windows.
//!
//! [`view::View::on_mouse()`] and [`view::View::on_paste()`] receive events only after enabling
//! them with [`app::App::mouse()`] and [`app::App::paste()`], because the terminal does not
//! report them by default.
//!
//! If you need the raw [`crossterm::event::Event`] (e.g. key releases), override
//! [`view::View::handle_event()`] instead. Note that this replaces the default dispatching.
//!
//! ## Switching views
//!
//! Call [`app::Context::change_view()`] from any of the input methods or from
//! [`view::View::update()`]. The old view receives [`view::View::on_disappear()`] and the new one
//! [`view::View::on_appear()`].
//!
//! ```ignore
//! ctx.change_view(SettingsPage::new());
//! ```

pub use crossterm;
pub use ratatui;
pub mod app;
pub mod view;