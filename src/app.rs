//! This module containts the primary methods for initializing, starting and configuring application.
//! 
//! # Methods
//! - [`init()`] - Iinitializes application.
//! - [`run()`] - Starts running loop.
//! - [`change_view()`] - Changes main view.

use crate::view::{View, ViewWidgetWrapper};
use crossterm::event::{self};
use ratatui::widgets::WidgetRef;
use std::{io, sync::RwLock};

/// A global, thread-safe, mutable application state.
///
/// This static variable holds the application state. It is wrapped in an [`RwLock`] to allow
/// safe concurrent read/write access. The `Option<App>` inside the `RwLock` allows the state
/// to be either `Some(App)` when the application is initialized or `None` if the application
/// has not been initialized yet.
///
/// # Usage
/// 
/// Before interacting with the application, you must call [`init()`] to initialize it. After that,
/// the state can be accessed or modified safely using this global variable.
///
/// This variable is used to track the running status of the application and is shared across the
/// application runtime.
pub static APPLICATION: RwLock<Option<App>> = RwLock::new(None);

/// A global, thread-safe, mutable view state.
///
/// This static variable holds the current view of the application. It is wrapped in an [`RwLock`] 
/// to ensure safe concurrent access and modification. The `Option<Box<dyn View + Sync + Send>>` 
/// allows the application to store the current view as a dynamic trait object that implements the 
/// [`View`] trait.
///
/// # Usage
/// 
/// This variable is used to store the current view of the application, which is rendered to the
/// terminal. The [`init()`] function sets the initial view, and later the view can be changed using
/// the [`change_view()`] function.
///
/// Since it is wrapped in an `RwLock`, it allows for multiple readers, but only one writer at a time.
/// Accessing or modifying the view requires acquiring the lock.
pub static VIEW: RwLock<Option<Box<dyn View + Sync + Send>>> = RwLock::new(None);

static mut _CHANGE_VIEW: bool = false;
static mut _NEXT_VIEW: Option<Box<dyn View + Sync + Send>> = None;

pub struct App {
    pub is_running: bool,
}

/// Initializes the application with the provided view. Must be run before any other application code like [`run()`].
/// 
/// This function:
/// - Initializes [`VIEW`]
/// - Initializes [`APPLICATION`]
/// 
/// # Parameters:
/// - `view`: A struct implementing the [`View`] trait. Represents the initial view of the application.
pub fn init<T: View + Sync + Send + 'static>(view: T) {
    let mut mainpage = VIEW.write().expect("create custom error here");
    if mainpage.is_none() {
        *mainpage = Some(Box::new(view));
    }

    let mut app = APPLICATION.write().expect("create custom error here");
    if app.is_none() {
        *app = Some(App { is_running: true });
    }
}


/// Start running the application loop. 
/// 
/// As long as the application is running this function will:
/// - Refresh view and draw on terminal based on [`View::render_view()`]
/// - Send current events to optional method [`View::handle_events()`]
/// 
/// NOTE: This function MUST be run after [`init()`].
/// 
/// # Returns:
/// - `Ok(())` if application exits.
/// - An `io::Error` if any error occurs while locking the shared resources or while handling the events.
pub fn run() -> io::Result<()> {
    let mut terminal = ratatui::init();

    let mut is_running = APPLICATION
        .read()
        .unwrap()
        .as_ref()
        .expect("APPLICATION is None. Did you run app::init()?")
        .is_running;

    while is_running {
        unsafe {
            if _CHANGE_VIEW {
                _CHANGE_VIEW = false;
                let mut mainpage = VIEW.write().expect("create custom error here");
                *mainpage = Some(_NEXT_VIEW.take().unwrap());
                _NEXT_VIEW = None;
            }
        }

        terminal.draw(|frame: &mut ratatui::Frame<'_>| {
            ViewWidgetWrapper(&VIEW.read().unwrap().as_ref().unwrap())
                .render_ref(frame.area(), frame.buffer_mut());
        })?;

        VIEW.write()
            .unwrap()
            .as_mut()
            .unwrap()
            .handle_events(&event::read()?)?;

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

/// Changes current view to the new provided at the beggining of the next application running loop.
/// 
/// NOTE: This function MUST be run after [`init()`].
/// 
/// # Parameters:
/// - `view`: A struct implementing the [`View`] trait. Represents application view that will override current [`VIEW`].
pub fn change_view<T: View + Sync + Send + 'static>(view: T) {
    if APPLICATION.read().unwrap().is_none(){
        panic!("APPLICATION is None. Did you run app::init()?")
    }

    unsafe {
        if _CHANGE_VIEW {
            return;
        }
        _CHANGE_VIEW = true;
        _NEXT_VIEW = Some(Box::new(view));
    }
}