//! This library expands on [ratatui] by abstracting main methods to different modules, which can be helpful in making modular applications with many [`view::View`]s as well as using architectural patterns like MVVM.
//!
//! ## How to run
//! To run application use methods from module [`app`]:
//! 1. Initialize the app with [`app::init()`] and provide [`view::View`].
//! 2. Run the application loop [`app::run()`].
//! 3. Done! After compilation you should see your app running in the console.
//!
//! ## Basic example
//!
//! Add required imports.
//!
//! ```
//! use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};
//! use ratatuio::{app, view::View};
//! use std::io;
//! ```
//!
//! Create a struct that will store the state of current view.
//!
//! ```
//! struct MainPage;
//! ```
//!
//! Implement [`view::View`] trait and provide the method [`view::View::render_view()`].
//!
//! ```
//! impl View for MainPage{
//!     fn render_view(&self, area: Rect, buf: &mut Buffer){
//!         "Hello World!".render(area, buf);
//!     }
//! }
//! ```
//!
//! And finally initialize and run the app with created MainPage view in the main method.
//!
//! ```
//! fn main() -> io::Result<()> {
//!     app::init(MainPage);
//!     app::run()
//! }
//! ```
//!
//! ### Adding event handler
//!
//! Above example will run in console however it will be impossible to close. To implement closing the application we first need to handle key press event.
//!
//! In trait [`view::View`] there is an optional method to implement [`view::View::handle_events()`] with parameter [`crossterm::event::Event`].
//! We can implement this method and using Rusts' match statement catch when user is pressing key 'q' or 'Q':
//!
//! ```
//! impl View for MainPage{
//!     //...
//!
//!     fn handle_events(&mut self, event: &Event) -> io::Result<()> {
//!         match event {
//!             Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
//!                 match key_event.code {
//!                     KeyCode::Char('q') | KeyCode::Char('Q') => {
//!                         app::APPLICATION
//!                             .write()
//!                             .unwrap()
//!                             .as_mut()
//!                             .unwrap()
//!                             .is_running = false
//!                     },
//!                     _ => {}
//!                 }
//!             }
//!             _ => {}
//!         }
//!         Ok(())
//!     }
//! ```
//! 
//! Inside `handle_events` we are checking when user is pressing key 'q' or 'Q' after which we are accessing application state [`app::APPLICATION`] and editing its 'is_running' value, which will exit out of the program loop on the next iteration.

pub mod app;
pub mod view;
