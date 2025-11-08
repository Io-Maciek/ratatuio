//! This module contains the primary methods for initializing, starting and configuring the application.
//! 
//! # Methods
//! - [`init()`] - Initializes the application.
//! - [`run()`] - Starts the main loop.
//! - [`change_view()`] - Changes the main view.

use crate::view::{View, ViewWidgetWrapper};
use ratatui::widgets::WidgetRef;
use std::{
    io,
    sync::{Mutex, RwLock, atomic::{AtomicBool, Ordering}}, time::Duration,
};
use lazy_static::lazy_static;

/// A global, thread-safe, mutable application state.
///
/// This static variable holds the application state. It is wrapped in an [`RwLock`] to allow
/// safe concurrent read/write access. The `Option<App>` inside the `RwLock` allows the state
/// to be either `Some(App)` when the application is initialized or `None` if the application
/// has not been initialized yet.
pub static APPLICATION: RwLock<Option<App>> = RwLock::new(None);

/// A global, thread-safe, mutable view state.
///
/// This static variable holds the current view of the application. It is wrapped in an [`RwLock`] 
/// to ensure safe concurrent access and modification.
pub static VIEW: RwLock<Option<Box<dyn View + Sync + Send>>> = RwLock::new(None);


static CHANGE_VIEW: AtomicBool = AtomicBool::new(false);
lazy_static! {
    static ref NEXT_VIEW: Mutex<Option<Box<dyn View + Sync + Send>>> = Mutex::new(None);
}

/// Represents the main application state.
pub struct App {
    pub is_running: bool,
}

/// Initializes the application with the provided view. Must be run before any other application code like [`run()`].
/// 
/// This function:
/// - Initializes [`VIEW`]
/// - Initializes [`APPLICATION`]
pub fn init<T: View + Sync + Send + 'static>(view: T) {
    let mut mainpage = VIEW.write().expect("Failed to lock VIEW");
    if mainpage.is_none() {
        *mainpage = Some(Box::new(view));
    }

    let mut app = APPLICATION.write().expect("Failed to lock APPLICATION");
    if app.is_none() {
        *app = Some(App { is_running: true });
    }
}

/// Starts the main application loop.
/// 
/// While the application is running, this function will:
/// - Draw the current view using [`View::render_view()`].
/// - Dispatch input events to [`View::handle_events()`].
/// - Switch to a new view if [`change_view()`] was called.
pub fn run() -> io::Result<()> {
    let mut terminal = ratatui::init();

    let mut is_running = APPLICATION
        .read()
        .unwrap()
        .as_ref()
        .expect("APPLICATION is None. Did you run app::init()?")
        .is_running;

    while is_running {
        // Handle queued view change

        {
            let mut mainpage = VIEW.write().expect("Failed to lock VIEW during change");

            if CHANGE_VIEW.swap(false, Ordering::SeqCst) {



                if let Some(next_view) = NEXT_VIEW.lock().unwrap().take() {
                    mainpage.as_mut().unwrap().on_disappear();
                    *mainpage = Some(next_view);
                    mainpage.as_mut().unwrap().on_appear();
                }
            }

            mainpage.as_mut().unwrap().update();
        }

        // Draw the current view
        terminal.draw(|frame: &mut ratatui::Frame<'_>| {
            ViewWidgetWrapper(&VIEW.read().unwrap().as_ref().unwrap())
                .render_ref(frame.area(), frame.buffer_mut());
        })?;

        // Handle events
        if crossterm::event::poll(Duration::from_millis(8))? {
            let event = crossterm::event::read()?;
            VIEW
                .write()
                .unwrap()
                .as_mut()
                .unwrap()
                .handle_events(&event)?;
        }

        // Update running state
        is_running = APPLICATION
            .read()
            .unwrap()
            .as_ref()
            .expect("APPLICATION is None. Did you run app::init()?")
            .is_running;
    }

    ratatui::restore();
    Ok(())
}

/// Changes the current view to the new one provided.
///
/// NOTE: This function MUST be called after [`init()`].
///
/// # Parameters
/// - `view`: A struct implementing the [`View`] trait. Represents the new view.
pub fn change_view<V>(view: V)
where
    V: Into<Box<dyn View + Send + Sync>>,
{
    if APPLICATION.read().unwrap().is_none() {
        panic!("APPLICATION is None. Did you run app::init()?");
    }

    let mut next = NEXT_VIEW.lock().unwrap();
    *next = Some(view.into());
    CHANGE_VIEW.store(true, Ordering::SeqCst);
}

/// This function will stop the main application loop on the next tick.
/// 
/// NOTE: This function MUST be called after [`init()`].
pub fn quit(){
    if APPLICATION.read().unwrap().is_none() {
        panic!("APPLICATION is None. Did you run app::init()?");
    }

    APPLICATION
        .write()
        .unwrap()
        .as_mut()
        .unwrap()
        .is_running = false;
}