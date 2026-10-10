//! This module contains [`App`] (owns the main loop) and [`Context`] (lets views
//! talk back to the application).
//!
//! - [`App::new()`] - creates the application with a starting view.
//! - [`App::tick_rate()`] - optionally changes how often [`View::update()`] runs.
//! - [`App::run()`] - initializes the terminal, runs the loop and always restores the terminal.
//! - [`Context::change_view()`] - requests a view change.
//! - [`Context::quit()`] - requests application exit.

use crate::view::View;
use crossterm::{
    event::{
        self, DisableBracketedPaste, DisableMouseCapture, EnableBracketedPaste,
        EnableMouseCapture,
    },
    execute,
};
use ratatui::DefaultTerminal;
use std::{
    io,
    time::{Duration, Instant},
};

/// Handle passed to a [`View`] so it can control the application.
///
/// Requests are applied by the main loop after the current callback returns.
pub struct Context {
    next_view: Option<Box<dyn View>>,
    quit: bool,
}

impl Context {
    fn new() -> Self {
        Self {
            next_view: None,
            quit: false,
        }
    }

    /// Requests a switch to a new view.
    ///
    /// The current view gets [`View::on_disappear()`], the new one gets [`View::on_appear()`].
    /// If called multiple times before the loop applies it, the last view wins.
    pub fn change_view(&mut self, view: impl View + 'static) {
        self.next_view = Some(Box::new(view));
    }

    /// Requests the application to exit. Takes precedence over a pending [`Context::change_view()`].
    pub fn quit(&mut self) {
        self.quit = true;
    }
}

/// The application. Owns the current view and runs the main loop.
pub struct App {
    view: Box<dyn View>,
    tick_rate: Duration,
    mouse: bool,
    paste: bool,
}

impl App {
    /// Creates a new application starting with `view`.
    pub fn new(view: impl View + 'static) -> Self {
        Self {
            view: Box::new(view),
            tick_rate: Duration::from_millis(16),
            mouse: false,
            paste: false,
        }
    }

    /// Sets how often [`View::update()`] is called (default: 16 ms, about 60 Hz).
    pub fn tick_rate(mut self, tick_rate: Duration) -> Self {
        self.tick_rate = tick_rate;
        self
    }

    /// Enables mouse capture, so [`View::on_mouse()`] receives events (default: off).
    pub fn mouse(mut self, enabled: bool) -> Self {
        self.mouse = enabled;
        self
    }

    /// Enables bracketed paste, so [`View::on_paste()`] receives events (default: off).
    pub fn paste(mut self, enabled: bool) -> Self {
        self.paste = enabled;
        self
    }

    /// Initializes the terminal and runs the main loop until [`Context::quit()`] is called.
    ///
    /// The terminal is always restored, also when the loop returns an error.
    pub fn run(mut self) -> io::Result<()> {
        let mut terminal = ratatui::init();
        let result = self
            .enable_features()
            .and_then(|_| self.main_loop(&mut terminal));
        self.disable_features();
        ratatui::restore();
        result
    }

    fn enable_features(&self) -> io::Result<()> {
        if self.mouse {
            execute!(io::stdout(), EnableMouseCapture)?;
        }
        if self.paste {
            execute!(io::stdout(), EnableBracketedPaste)?;
        }
        Ok(())
    }

    fn disable_features(&self) {
        // Best effort: we are already shutting down.
        if self.mouse {
            let _ = execute!(io::stdout(), DisableMouseCapture);
        }
        if self.paste {
            let _ = execute!(io::stdout(), DisableBracketedPaste);
        }
    }

    fn main_loop(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        let mut ctx = Context::new();
        let mut last_update = Instant::now();

        self.view.on_appear();

        loop {
            terminal.draw(|frame| self.view.render(frame))?;

            // Wait for input, but no longer than until the next tick.
            let timeout = self.tick_rate.saturating_sub(last_update.elapsed());
            if event::poll(timeout)? {
                // Drain everything that is already queued.
                loop {
                    let ev = event::read()?;
                    self.view.handle_event(&ev, &mut ctx)?;
                    if ctx.quit || ctx.next_view.is_some() || !event::poll(Duration::ZERO)? {
                        break;
                    }
                }
            }

            let elapsed = last_update.elapsed();
            if elapsed >= self.tick_rate {
                self.view.update(&mut ctx, elapsed);
                last_update = Instant::now();
            }

            if self.apply(&mut ctx) {
                return Ok(());
            }
        }
    }

    /// Applies requests collected in `ctx`. Returns `true` when the application should exit.
    ///
    /// Kept separate from the loop so it can be tested without a real terminal.
    fn apply(&mut self, ctx: &mut Context) -> bool {
        if ctx.quit {
            self.view.on_disappear();
            return true;
        }

        if let Some(next) = ctx.next_view.take() {
            self.view.on_disappear();
            self.view = next;
            self.view.on_appear();
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
    use ratatui::{backend::TestBackend, widgets::Paragraph, Frame, Terminal};
    use std::{cell::RefCell, rc::Rc};

    type Log = Rc<RefCell<Vec<String>>>;

    /// Test view: records lifecycle hooks, 'n' switches to view "B", 'q' quits.
    /// Uses `Rc<RefCell<_>>`, which works because `View` does not require `Send + Sync`.
    struct Probe {
        name: &'static str,
        log: Log,
    }

    impl Probe {
        fn new(name: &'static str, log: &Log) -> Self {
            Self { name, log: log.clone() }
        }
    }

    impl View for Probe {
        fn render(&mut self, frame: &mut Frame) {
            frame.render_widget(Paragraph::new(self.name), frame.area());
        }
        fn on_appear(&mut self) {
            self.log.borrow_mut().push(format!("{}:appear", self.name));
        }
        fn on_disappear(&mut self) {
            self.log.borrow_mut().push(format!("{}:disappear", self.name));
        }
        fn on_key(&mut self, key: KeyEvent, ctx: &mut Context) -> io::Result<()> {
            match key.code {
                KeyCode::Char('n') => ctx.change_view(Probe::new("B", &self.log)),
                KeyCode::Char('q') => ctx.quit(),
                _ => {}
            }
            Ok(())
        }
    }

    fn key(c: char) -> Event {
        Event::Key(KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE))
    }

    fn screen(terminal: &Terminal<TestBackend>) -> String {
        let buf = terminal.backend().buffer();
        buf.content()
            .chunks(buf.area.width as usize)
            .map(|row| row.iter().map(|c| c.symbol()).collect::<String>())
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn setup() -> (App, Context, Log) {
        let log: Log = Rc::default();
        let mut app = App::new(Probe::new("A", &log));
        app.view.on_appear();
        (app, Context::new(), log)
    }

    #[test]
    fn view_change_runs_hooks_in_order() {
        let (mut app, mut ctx, log) = setup();

        app.view.handle_event(&key('n'), &mut ctx).unwrap();
        assert!(!app.apply(&mut ctx));

        assert_eq!(*log.borrow(), ["A:appear", "A:disappear", "B:appear"]);
    }

    #[test]
    fn quit_calls_on_disappear_and_wins_over_change_view() {
        let (mut app, mut ctx, log) = setup();

        app.view.handle_event(&key('n'), &mut ctx).unwrap();
        app.view.handle_event(&key('q'), &mut ctx).unwrap();
        assert!(app.apply(&mut ctx));

        assert_eq!(*log.borrow(), ["A:appear", "A:disappear"]);
    }

    #[test]
    fn last_change_view_wins() {
        let (mut app, mut ctx, log) = setup();

        ctx.change_view(Probe::new("B", &log));
        ctx.change_view(Probe::new("C", &log));
        app.apply(&mut ctx);

        assert_eq!(*log.borrow(), ["A:appear", "A:disappear", "C:appear"]);
    }

    #[test]
    fn key_release_is_ignored() {
        let (mut app, mut ctx, _log) = setup();

        let release = Event::Key(KeyEvent::new_with_kind(
            KeyCode::Char('q'),
            KeyModifiers::NONE,
            KeyEventKind::Release,
        ));
        app.view.handle_event(&release, &mut ctx).unwrap();

        assert!(!ctx.quit);
    }

    #[test]
    fn renders_current_view_to_test_backend() {
        let (mut app, mut ctx, _log) = setup();
        let mut terminal = Terminal::new(TestBackend::new(10, 1)).unwrap();

        terminal.draw(|f| app.view.render(f)).unwrap();
        assert!(screen(&terminal).starts_with('A'));

        app.view.handle_event(&key('n'), &mut ctx).unwrap();
        app.apply(&mut ctx);
        terminal.draw(|f| app.view.render(f)).unwrap();
        assert!(screen(&terminal).starts_with('B'));
    }
}