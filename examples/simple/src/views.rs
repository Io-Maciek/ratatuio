use crate::event_log;
use crate::navigation::{handle_global_key, Page};
use ratatuio::app::Context;
use ratatuio::crossterm::event::{KeyCode, KeyEvent, MouseButton, MouseEvent, MouseEventKind};
use ratatuio::ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::Line,
    widgets::{Block, Borders, Gauge, List, ListState, Paragraph, Wrap},
    Frame,
};
use ratatuio::view::View;
use std::{io, time::Duration};

// === Paragraph View: on_paste ===
pub struct ParagraphView {
    pasted: Option<String>,
}

impl ParagraphView {
    pub fn new() -> Self {
        Self { pasted: None }
    }
}

impl View for ParagraphView {
    fn render(&mut self, frame: &mut Frame) {
        let block = Block::default().title("Paragraph").borders(Borders::ALL);
        let mut text = vec![
            Line::from("This is a paragraph widget."),
            Line::from("Press A/D or ←/→ to switch views, Q to quit."),
            Line::from(""),
            Line::from("Paste some text into the terminal (on_paste):"),
        ];
        match &self.pasted {
            Some(t) => {
                let shown: String = t.replace(['\n', '\r'], " ⏎ ").chars().take(200).collect();
                text.push(Line::styled(shown, Style::default().fg(Color::Green)));
            }
            None => text.push(Line::styled(
                "(nothing pasted yet)",
                Style::default().fg(Color::DarkGray),
            )),
        }
        frame.render_widget(Paragraph::new(text).block(block).wrap(Wrap {trim:false}), frame.area());
    }

    fn on_appear(&mut self) {
        event_log::push("Paragraph: on_appear");
    }

    fn on_disappear(&mut self) {
        event_log::push("Paragraph: on_disappear");
    }

    fn on_key(&mut self, key: KeyEvent, ctx: &mut Context) -> io::Result<()> {
        handle_global_key(key, Page::Paragraph, ctx);
        Ok(())
    }

    fn on_paste(&mut self, text: &str, _ctx: &mut Context) -> io::Result<()> {
        event_log::push(format!("Paragraph: on_paste ({} chars)", text.chars().count()));
        self.pasted = Some(text.to_string());
        Ok(())
    }
}

// === Block View: on_resize ===
pub struct BlockView {
    resizes: u32,
}

impl BlockView {
    pub fn new() -> Self {
        Self { resizes: 0 }
    }
}

impl View for BlockView {
    fn render(&mut self, frame: &mut Frame) {
        let area = frame.area();
        let block = Block::default()
            .title("Block Widget")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));
        let inner = block.inner(area);
        frame.render_widget(block, area);

        let text = vec![
            Line::from(format!("Current size: {}x{}", area.width, area.height)),
            Line::from(format!("Resize events (on_resize): {}", self.resizes)),
            Line::from("Resize the terminal window to see it change."),
        ];
        frame.render_widget(Paragraph::new(text), inner);
    }

    fn on_appear(&mut self) {
        event_log::push("Block: on_appear");
    }

    fn on_disappear(&mut self) {
        event_log::push("Block: on_disappear");
    }

    fn on_key(&mut self, key: KeyEvent, ctx: &mut Context) -> io::Result<()> {
        handle_global_key(key, Page::Block, ctx);
        Ok(())
    }

    fn on_resize(&mut self, w: u16, h: u16, _ctx: &mut Context) -> io::Result<()> {
        self.resizes += 1;
        event_log::push(format!("Block: on_resize {w}x{h}"));
        Ok(())
    }
}

// === List View: on_key (W/S) + on_mouse (click, scroll wheel) ===
const ITEMS: [&str; 3] = ["Item 1", "Item 2", "Item 3"];

pub struct ListView {
    state: ListState,
    area: Rect, // remembered from render, used to map mouse clicks to rows
}

impl ListView {
    pub fn new() -> Self {
        Self {
            state: ListState::default().with_selected(Some(0)),
            area: Rect::default(),
        }
    }

    fn move_up(&mut self) {
        let i = self.state.selected().unwrap_or(0);
        self.state.select(Some((i + ITEMS.len() - 1) % ITEMS.len()));
    }

    fn move_down(&mut self) {
        let i = self.state.selected().unwrap_or(0);
        self.state.select(Some((i + 1) % ITEMS.len()));
    }
}

impl View for ListView {
    fn render(&mut self, frame: &mut Frame) {
        self.area = frame.area();
        let list = List::new(ITEMS)
            .block(
                Block::default()
                    .title("List Widget (W/S, click, scroll)")
                    .borders(Borders::ALL),
            )
            .highlight_style(Style::default().fg(Color::Yellow));
        // `render` takes `&mut Frame`, so a stateful widget works out of the box.
        frame.render_stateful_widget(list, self.area, &mut self.state);
    }

    fn on_appear(&mut self) {
        event_log::push("List: on_appear");
    }

    fn on_disappear(&mut self) {
        event_log::push("List: on_disappear");
    }

    fn on_key(&mut self, key: KeyEvent, ctx: &mut Context) -> io::Result<()> {
        if handle_global_key(key, Page::List, ctx) {
            return Ok(());
        }
        match key.code {
            KeyCode::Up | KeyCode::Char('w' | 'W') => self.move_up(),
            KeyCode::Down | KeyCode::Char('s' | 'S') => self.move_down(),
            _ => {}
        }
        Ok(())
    }

    fn on_mouse(&mut self, m: MouseEvent, _ctx: &mut Context) -> io::Result<()> {
        match m.kind {
            MouseEventKind::ScrollUp => self.move_up(),
            MouseEventKind::ScrollDown => self.move_down(),
            MouseEventKind::Down(MouseButton::Left) => {
                let first_row = self.area.y + 1; // skip the top border
                if m.row >= first_row {
                    let i = (m.row - first_row) as usize;
                    if i < ITEMS.len() {
                        self.state.select(Some(i));
                        event_log::push(format!("List: clicked item {}", i + 1));
                    }
                }
            }
            _ => {} // ignore mouse movement, it would flood everything
        }
        Ok(())
    }
}

// === Gauge View: update(dt) + on_appear reset, Space pauses ===
pub struct GaugeView {
    progress: f64,
    paused: bool,
}

impl GaugeView {
    pub fn new() -> Self {
        Self {
            progress: 0.0,
            paused: false,
        }
    }
}

impl View for GaugeView {
    fn render(&mut self, frame: &mut Frame) {
        let title = if self.paused {
            "Gauge Widget (paused, Space to resume)"
        } else {
            "Gauge Widget (Space to pause)"
        };
        let gauge = Gauge::default()
            .block(Block::default().title(title).borders(Borders::ALL))
            .gauge_style(Style::default().fg(Color::Magenta))
            .ratio(self.progress);
        frame.render_widget(gauge, frame.area());
    }

    fn update(&mut self, _ctx: &mut Context, dt: Duration) {
        if self.paused {
            return;
        }
        // One full loop every 2 seconds, regardless of the tick rate.
        self.progress = (self.progress + dt.as_secs_f64() * 0.5) % 1.0;
    }

    fn on_appear(&mut self) {
        self.progress = 0.0;
        self.paused = false;
        event_log::push("Gauge: on_appear (progress reset)");
    }

    fn on_disappear(&mut self) {
        event_log::push(format!(
            "Gauge: on_disappear (progress was {:.0}%)",
            self.progress * 100.0
        ));
    }

    fn on_key(&mut self, key: KeyEvent, ctx: &mut Context) -> io::Result<()> {
        if handle_global_key(key, Page::Gauge, ctx) {
            return Ok(());
        }
        if key.code == KeyCode::Char(' ') {
            self.paused = !self.paused;
        }
        Ok(())
    }
}

// === Events View: shows everything the other views logged ===
pub struct EventsView;

impl View for EventsView {
    fn render(&mut self, frame: &mut Frame) {
        let area = frame.area();
        let height = area.height.saturating_sub(2) as usize;
        let lines: Vec<Line> = event_log::tail(height).into_iter().map(Line::from).collect();
        let block = Block::default()
            .title("Events (C to clear)")
            .borders(Borders::ALL);
        frame.render_widget(Paragraph::new(lines).block(block), area);
    }

    fn on_appear(&mut self) {
        event_log::push("Events: on_appear");
    }

    fn on_disappear(&mut self) {
        event_log::push("Events: on_disappear");
    }

    fn on_key(&mut self, key: KeyEvent, ctx: &mut Context) -> io::Result<()> {
        if handle_global_key(key, Page::Events, ctx) {
            return Ok(());
        }
        if matches!(key.code, KeyCode::Char('c' | 'C')) {
            event_log::clear();
        }
        Ok(())
    }
}