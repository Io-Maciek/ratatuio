use ratatuio::ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    text::Line,
    widgets::{Block, Borders, Gauge, List, ListItem, Paragraph, Widget},
};
use ratatuio::{app, view::View};
use ratatuio::crossterm::event::{Event, KeyCode};
use std::io;

// === Navigation Helper ===
fn handle_navigation(event: &Event, current: &str) -> Option<Box<dyn View + Send + Sync>> {
    if let Event::Key(key) = event {
        let next = match (current, key.code) {
            // Paragraph ↔ Block
            ("Paragraph", KeyCode::Right) | ("Paragraph", KeyCode::Char('d')) | ("Paragraph", KeyCode::Char('D')) => "Block",
            ("Block", KeyCode::Left) | ("Block", KeyCode::Char('a')) | ("Block", KeyCode::Char('A')) => "Paragraph",

            // Block ↔ List
            ("Block", KeyCode::Right) | ("Block", KeyCode::Char('d')) | ("Block", KeyCode::Char('D')) => "List",
            ("List", KeyCode::Left) | ("List", KeyCode::Char('a')) | ("List", KeyCode::Char('A')) => "Block",

            // List ↔ Gauge
            ("List", KeyCode::Right) | ("List", KeyCode::Char('d')) | ("List", KeyCode::Char('D')) => "Gauge",
            ("Gauge", KeyCode::Left) | ("Gauge", KeyCode::Char('a')) | ("Gauge", KeyCode::Char('A')) => "List",

            // Loop around
            ("Gauge", KeyCode::Right) | ("Gauge", KeyCode::Char('d')) | ("Gauge", KeyCode::Char('D')) => "Paragraph",
            ("Paragraph", KeyCode::Left) | ("Paragraph", KeyCode::Char('a')) | ("Paragraph", KeyCode::Char('A')) => "Gauge",

            _ => return None,
        };
        return Some(view_from_name(next));
    }
    None
}

// === Paragraph View ===
struct ParagraphView;
impl View for ParagraphView {
    fn render_view(&self, area: Rect, buf: &mut Buffer) {
        let block = Block::default().title("Paragraph").borders(Borders::ALL);
        let text = vec![
            Line::from("This is a paragraph widget."),
            Line::from("Press A/D or ←/→ to switch views, Q to quit."),
        ];
        Paragraph::new(text).block(block).render(area, buf);
    }

    fn handle_events(&mut self, event: &Event) -> io::Result<()> {
        if let Event::Key(key) = event {
            if key.code == KeyCode::Char('q') || key.code == KeyCode::Char('Q') {
                app::quit();
            }
        }
        if let Some(next_view) = handle_navigation(event, "Paragraph") {
            app::change_view(next_view);
        }
        Ok(())
    }
}

// === Block View ===
struct BlockView;
impl View for BlockView {
    fn render_view(&self, area: Rect, buf: &mut Buffer) {
        Block::default()
            .title("Block Widget")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan))
            .render(area, buf);
    }

    fn handle_events(&mut self, event: &Event) -> io::Result<()> {
        if let Event::Key(key) = event {
            if key.code == KeyCode::Char('q') || key.code == KeyCode::Char('Q') {
                app::quit();
            }
        }
        if let Some(next_view) = handle_navigation(event, "Block") {
            app::change_view(next_view);
        }
        Ok(())
    }
}

// === List View ===
struct ListView {
    selected: usize,
}
impl ListView {
    fn new() -> Self { Self { selected: 0 } }
    fn move_up(&mut self, len: usize) { self.selected = if self.selected > 0 { self.selected - 1 } else { len - 1 }; }
    fn move_down(&mut self, len: usize) { self.selected = (self.selected + 1) % len; }
}
impl View for ListView {
    fn render_view(&self, area: Rect, buf: &mut Buffer) {
        let items = vec!["Item 1", "Item 2", "Item 3"];
        let list_items: Vec<_> = items.iter().enumerate().map(|(i, text)| {
            let mut item = ListItem::new(*text);
            if i == self.selected { item = item.style(Style::default().fg(Color::Yellow)); }
            item
        }).collect();

        List::new(list_items)
            .block(Block::default().title("List Widget").borders(Borders::ALL))
            .render(area, buf);
    }

    fn handle_events(&mut self, event: &Event) -> io::Result<()> {
        if let Event::Key(key) = event {
            if key.code == KeyCode::Char('q') || key.code == KeyCode::Char('Q') {
                app::quit();
            }
            if let Some(next_view) = handle_navigation(event, "List") {
                app::change_view(next_view);
                return Ok(());
            }
            match key.code {
                KeyCode::Up | KeyCode::Char('w') | KeyCode::Char('W') => self.move_up(3),
                KeyCode::Down | KeyCode::Char('s') | KeyCode::Char('S') => self.move_down(3),
                _ => {}
            }
        }
        Ok(())
    }
}

// === Gauge View with looping progress ===
struct GaugeView {
    progress: f64,       // shared progress
}

impl GaugeView {
    fn new() -> GaugeView {
        GaugeView { progress: 0.0 }
    }
        
}

impl View for GaugeView {
    fn render_view(&self, area: Rect, buf: &mut Buffer) {
        Gauge::default()
            .block(Block::default().title("Gauge Widget").borders(Borders::ALL))
            .gauge_style(Style::default().fg(Color::Magenta))
            .ratio(self.progress)
            .render(area, buf);
    }

    fn update(&mut self) {
        self.progress += 0.01;
        if self.progress > 1.0 {
            self.progress = 0.0; // loop back
        }
    }

    fn on_disappear(&mut self) {
    }

    fn handle_events(&mut self, event: &Event) -> io::Result<()> {
        // increment the gauge every loop

        if let Event::Key(key) = event {
            // Quit
            if key.code == KeyCode::Char('q') || key.code == KeyCode::Char('Q') {
                app::quit();
            }
            // Navigation
            if let Some(next_view) = handle_navigation(event, "Gauge") {
                app::change_view(next_view);
            }
        }
        Ok(())
    }
}

// === View Factory ===
fn view_from_name(name: &str) -> Box<dyn View + Send + Sync> {
    match name {
        "Paragraph" => Box::new(ParagraphView),
        "Block" => Box::new(BlockView),
        "List" => Box::new(ListView::new()),
        "Gauge" => Box::new(GaugeView::new()),
        _ => Box::new(ParagraphView),
    }
}

// === Entry Point ===
fn main() -> io::Result<()> {
    app::init(ParagraphView);
    app::run()
}
