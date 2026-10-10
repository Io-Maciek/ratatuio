use ratatuio::app::{App, Context};
use ratatuio::crossterm::event::{KeyCode, KeyEvent, MouseButton, MouseEvent, MouseEventKind};
use ratatuio::ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols::Marker,
    text::{Line, Span},
    widgets::{
        Axis, Block, Chart, Dataset, Gauge, GraphType, Paragraph, Row, Table, TableState,
    },
    Frame,
};
use ratatuio::view::View;
use std::{cell::RefCell, collections::VecDeque, io, rc::Rc, time::Duration};
use sysinfo::{Components, ProcessesToUpdate, System};

fn main() -> io::Result<()> {
    App::new(DashboardView::new(Stats::new()))
        // The loop ticks often; data is refreshed on its own interval (see `Stats::tick`).
        .tick_rate(Duration::from_millis(100))
        // Needed for `View::on_mouse` (wheel and click in the process table).
        .mouse(true)
        .run()
}

const HISTORY: usize = 120;
const MIN_INTERVAL: Duration = Duration::from_millis(250);
const MAX_INTERVAL: Duration = Duration::from_secs(5);

type Shared = Rc<RefCell<Stats>>;

// ---------------------------------------------------------------------------
// Shared data
// ---------------------------------------------------------------------------

struct Stats {
    sys: System,
    components: Components,
    cpu_history: VecDeque<f64>,
    mem_history: VecDeque<f64>,
    interval: Duration,
    since_refresh: Duration,
    paused: bool,
}

impl Stats {
    fn new() -> Shared {
        let mut sys = System::new();
        sys.refresh_cpu_usage();
        sys.refresh_memory();
        Rc::new(RefCell::new(Self {
            sys,
            components: Components::new_with_refreshed_list(),
            cpu_history: VecDeque::new(),
            mem_history: VecDeque::new(),
            interval: Duration::from_secs(1),
            since_refresh: Duration::ZERO,
            paused: false,
        }))
    }

    /// Called from `View::update` with the time since the previous tick.
    fn tick(&mut self, dt: Duration, with_processes: bool) {
        if self.paused {
            return;
        }
        self.since_refresh += dt;
        if self.since_refresh < self.interval {
            return;
        }
        self.since_refresh = Duration::ZERO;

        self.sys.refresh_cpu_usage();
        self.sys.refresh_memory();
        self.components.refresh(true);
        if with_processes {
            self.refresh_processes();
        }

        push(&mut self.cpu_history, self.sys.global_cpu_usage() as f64);
        let total = self.sys.total_memory().max(1) as f64;
        push(&mut self.mem_history, self.sys.used_memory() as f64 / total * 100.0);
    }

    fn refresh_processes(&mut self) {
        self.sys.refresh_processes(ProcessesToUpdate::All, true);
    }

    fn change_interval(&mut self, faster: bool) {
        let ms = self.interval.as_millis() as u64;
        let ms = if faster { ms / 2 } else { ms * 2 };
        self.interval = Duration::from_millis(ms).clamp(MIN_INTERVAL, MAX_INTERVAL);
    }
}

fn push(history: &mut VecDeque<f64>, value: f64) {
    if history.len() == HISTORY {
        history.pop_front();
    }
    history.push_back(value);
}

fn human(bytes: u64) -> String {
    const MIB: f64 = 1024.0 * 1024.0;
    let b = bytes as f64;
    if b >= 1024.0 * MIB {
        format!("{:.1} GiB", b / (1024.0 * MIB))
    } else {
        format!("{:.0} MiB", b / MIB)
    }
}

fn usage_color(percent: f64) -> Color {
    match percent {
        p if p < 50.0 => Color::Green,
        p if p < 80.0 => Color::Yellow,
        _ => Color::Red,
    }
}

fn temp_color(celsius: f32) -> Color {
    match celsius {
        t if t < 60.0 => Color::Green,
        t if t < 80.0 => Color::Yellow,
        _ => Color::Red,
    }
}

fn ratio(used: u64, total: u64) -> f64 {
    if total == 0 {
        0.0
    } else {
        (used as f64 / total as f64).clamp(0.0, 1.0)
    }
}

// ---------------------------------------------------------------------------
// Keys shared by both views
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq)]
enum Page {
    Dashboard,
    Processes,
}

/// Returns `true` when the key was consumed.
fn global_key(key: KeyEvent, current: Page, stats: &Shared, ctx: &mut Context) -> bool {
    match key.code {
        KeyCode::Char('q' | 'Q') => ctx.quit(),
        KeyCode::Tab | KeyCode::BackTab => match current {
            Page::Dashboard => ctx.change_view(ProcessesView::new(stats.clone())),
            Page::Processes => ctx.change_view(DashboardView::new(stats.clone())),
        },
        KeyCode::Char('1') if current != Page::Dashboard => {
            ctx.change_view(DashboardView::new(stats.clone()))
        }
        KeyCode::Char('2') if current != Page::Processes => {
            ctx.change_view(ProcessesView::new(stats.clone()))
        }
        KeyCode::Char(' ') => {
            let mut s = stats.borrow_mut();
            s.paused = !s.paused;
        }
        KeyCode::Char('+') | KeyCode::Char('=') => stats.borrow_mut().change_interval(true),
        KeyCode::Char('-') => stats.borrow_mut().change_interval(false),
        _ => return false,
    }
    true
}

fn footer(text: &str, stats: &Stats) -> Paragraph<'static> {
    let state = if stats.paused { " [PAUSED]" } else { "" };
    Paragraph::new(Line::from(vec![
        Span::styled(
            format!(" {text} "),
            Style::default().fg(Color::Black).bg(Color::Gray),
        ),
        Span::styled(
            format!(" refresh {} ms{state}", stats.interval.as_millis()),
            Style::default().fg(Color::Gray),
        ),
    ]))
}

// ---------------------------------------------------------------------------
// Dashboard
// ---------------------------------------------------------------------------

struct DashboardView {
    stats: Shared,
}

impl DashboardView {
    fn new(stats: Shared) -> Self {
        Self { stats }
    }
}

impl View for DashboardView {
    fn update(&mut self, _ctx: &mut Context, dt: Duration) {
        self.stats.borrow_mut().tick(dt, false);
    }

    fn on_key(&mut self, key: KeyEvent, ctx: &mut Context) -> io::Result<()> {
        global_key(key, Page::Dashboard, &self.stats, ctx);
        Ok(())
    }

    fn render(&mut self, frame: &mut Frame) {
        let s = self.stats.borrow();
        let [header, body, bottom] = Layout::vertical([
            Constraint::Length(4),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .areas(frame.area());
        let [left, right] =
            Layout::horizontal([Constraint::Percentage(62), Constraint::Percentage(38)]).areas(body);
        let [cpu_area, mem_area] =
            Layout::vertical([Constraint::Min(8), Constraint::Length(4)]).areas(left);
        let [cores_area, temps_area] =
            Layout::vertical([Constraint::Min(6), Constraint::Percentage(40)]).areas(right);

        render_header(frame, header, &s);
        render_cpu_chart(frame, cpu_area, &s);
        render_memory(frame, mem_area, &s);
        render_cores(frame, cores_area, &s);
        render_temps(frame, temps_area, &s);
        frame.render_widget(footer("Tab: processes  Space: pause  +/-: interval  Q: quit", &s), bottom);
    }
}

fn render_header(frame: &mut Frame, area: Rect, s: &Stats) {
    let load = System::load_average();
    let brand = s.sys.cpus().first().map(|c| c.brand().to_string()).unwrap_or_default();
    let up = System::uptime();
    let lines = vec![
        Line::from(format!(
            "{} · {} {}",
            System::host_name().unwrap_or_else(|| "unknown host".into()),
            System::name().unwrap_or_default(),
            System::os_version().unwrap_or_default(),
        )),
        Line::from(format!("{} ({} cores)", brand, s.sys.cpus().len())),
        Line::from(format!(
            "uptime {}d {:02}:{:02}:{:02}   load {:.2} {:.2} {:.2}",
            up / 86_400,
            up % 86_400 / 3600,
            up % 3600 / 60,
            up % 60,
            load.one,
            load.five,
            load.fifteen
        )),
    ];
    frame.render_widget(
        Paragraph::new(lines).block(Block::bordered().title(" System ")),
        area,
    );
}

fn render_cpu_chart(frame: &mut Frame, area: Rect, s: &Stats) {
    let data: Vec<(f64, f64)> = s
        .cpu_history
        .iter()
        .enumerate()
        .map(|(i, v)| (i as f64, *v))
        .collect();
    let now = s.cpu_history.back().copied().unwrap_or(0.0);

    let dataset = Dataset::default()
        .marker(Marker::Braille)
        .graph_type(GraphType::Line)
        .style(Style::default().fg(usage_color(now)))
        .data(&data);

    let chart = Chart::new(vec![dataset])
        .block(Block::bordered().title(format!(" CPU {now:.1}% ")))
        .x_axis(Axis::default().bounds([0.0, (HISTORY - 1) as f64]))
        .y_axis(
            Axis::default()
                .bounds([0.0, 100.0])
                .labels(["0%", "50%", "100%"]),
        );
    frame.render_widget(chart, area);
}

fn render_memory(frame: &mut Frame, area: Rect, s: &Stats) {
    let block = Block::bordered().title(" Memory ");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let [ram_row, swap_row] =
        Layout::vertical([Constraint::Length(1), Constraint::Length(1)]).areas(inner);

    let (used, total) = (s.sys.used_memory(), s.sys.total_memory());
    let r = ratio(used, total);
    frame.render_widget(
        Gauge::default()
            .gauge_style(Style::default().fg(usage_color(r * 100.0)))
            .ratio(r)
            .label(format!("RAM  {} / {}", human(used), human(total))),
        ram_row,
    );

    let (used, total) = (s.sys.used_swap(), s.sys.total_swap());
    let r = ratio(used, total);
    frame.render_widget(
        Gauge::default()
            .gauge_style(Style::default().fg(usage_color(r * 100.0)))
            .ratio(r)
            .label(format!("Swap {} / {}", human(used), human(total))),
        swap_row,
    );
}

fn render_cores(frame: &mut Frame, area: Rect, s: &Stats) {
    let block = Block::bordered().title(" Cores ");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    // "NN " + " 100.0%" around the bar
    let bar_width = (inner.width as usize).saturating_sub(3 + 7);
    let lines: Vec<Line> = s
        .sys
        .cpus()
        .iter()
        .enumerate()
        .take(inner.height as usize)
        .map(|(i, cpu)| {
            let usage = cpu.cpu_usage() as f64;
            let filled = ((usage / 100.0) * bar_width as f64).round() as usize;
            Line::from(vec![
                Span::raw(format!("{i:>2} ")),
                Span::styled("█".repeat(filled), Style::default().fg(usage_color(usage))),
                Span::styled(
                    "░".repeat(bar_width.saturating_sub(filled)),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::raw(format!(" {usage:>5.1}%")),
            ])
        })
        .collect();
    frame.render_widget(Paragraph::new(lines), inner);
}

fn render_temps(frame: &mut Frame, area: Rect, s: &Stats) {
    let mut lines: Vec<Line> = s
        .components
        .iter()
        .filter_map(|c| {
            let t = c.temperature()?;
            Some(Line::from(vec![
                Span::raw(format!("{:<22}", c.label())),
                Span::styled(format!("{t:>5.1} °C"), Style::default().fg(temp_color(t))),
            ]))
        })
        .collect();
    if lines.is_empty() {
        lines.push(Line::styled(
            "No temperature sensors found",
            Style::default().fg(Color::DarkGray),
        ));
    }
    frame.render_widget(
        Paragraph::new(lines).block(Block::bordered().title(" Temperatures ")),
        area,
    );
}

// ---------------------------------------------------------------------------
// Processes
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq)]
enum SortBy {
    Cpu,
    Memory,
}

struct ProcRow {
    pid: u32,
    name: String,
    cpu: f32,
    mem: u64,
}

struct ProcessesView {
    stats: Shared,
    state: TableState,
    sort: SortBy,
    area: Rect,
}

impl ProcessesView {
    fn new(stats: Shared) -> Self {
        Self {
            stats,
            state: TableState::default().with_selected(Some(0)),
            sort: SortBy::Cpu,
            area: Rect::default(),
        }
    }

    fn rows(&self) -> Vec<ProcRow> {
        let s = self.stats.borrow();
        let mut rows: Vec<ProcRow> = s
            .sys
            .processes()
            .values()
            .map(|p| ProcRow {
                pid: p.pid().as_u32(),
                name: p.name().to_string_lossy().into_owned(),
                cpu: p.cpu_usage(),
                mem: p.memory(),
            })
            .collect();
        match self.sort {
            SortBy::Cpu => rows.sort_by(|a, b| b.cpu.total_cmp(&a.cpu)),
            SortBy::Memory => rows.sort_by(|a, b| b.mem.cmp(&a.mem)),
        }
        rows
    }

    fn move_selection(&mut self, delta: isize) {
        let len = self.stats.borrow().sys.processes().len();
        if len == 0 {
            return;
        }
        let current = self.state.selected().unwrap_or(0) as isize;
        let next = (current + delta).clamp(0, len as isize - 1);
        self.state.select(Some(next as usize));
    }
}

impl View for ProcessesView {
    fn on_appear(&mut self) {
        self.stats.borrow_mut().refresh_processes();
    }

    fn update(&mut self, _ctx: &mut Context, dt: Duration) {
        self.stats.borrow_mut().tick(dt, true);
    }

    fn on_key(&mut self, key: KeyEvent, ctx: &mut Context) -> io::Result<()> {
        if global_key(key, Page::Processes, &self.stats, ctx) {
            return Ok(());
        }
        let page = self.area.height.saturating_sub(3).max(1) as isize;
        match key.code {
            KeyCode::Up => self.move_selection(-1),
            KeyCode::Down => self.move_selection(1),
            KeyCode::PageUp => self.move_selection(-page),
            KeyCode::PageDown => self.move_selection(page),
            KeyCode::Home => self.state.select(Some(0)),
            KeyCode::End => self.move_selection(isize::MAX / 2),
            KeyCode::Char('c' | 'C') => self.sort = SortBy::Cpu,
            KeyCode::Char('m' | 'M') => self.sort = SortBy::Memory,
            _ => {}
        }
        Ok(())
    }

    fn on_mouse(&mut self, m: MouseEvent, _ctx: &mut Context) -> io::Result<()> {
        match m.kind {
            MouseEventKind::ScrollUp => self.move_selection(-3),
            MouseEventKind::ScrollDown => self.move_selection(3),
            MouseEventKind::Down(MouseButton::Left) => {
                let first_row = self.area.y + 2;
                if m.row >= first_row && m.row < self.area.bottom().saturating_sub(1) {
                    let index = self.state.offset() + (m.row - first_row) as usize;
                    if index < self.stats.borrow().sys.processes().len() {
                        self.state.select(Some(index));
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn render(&mut self, frame: &mut Frame) {
        let [table_area, bottom] =
            Layout::vertical([Constraint::Min(0), Constraint::Length(1)]).areas(frame.area());
        self.area = table_area;

        let rows = self.rows();
        let sort_mark = |s: SortBy, label: &str| {
            if self.sort == s {
                format!("{label} ▼")
            } else {
                label.to_string()
            }
        };
        let header = Row::new([
            "PID".to_string(),
            "Name".to_string(),
            sort_mark(SortBy::Cpu, "CPU %"),
            sort_mark(SortBy::Memory, "Memory"),
        ])
        .style(Style::default().add_modifier(Modifier::BOLD).fg(Color::Cyan));

        let body: Vec<Row> = rows
            .iter()
            .map(|r| {
                Row::new([
                    r.pid.to_string(),
                    r.name.clone(),
                    format!("{:.1}", r.cpu),
                    human(r.mem),
                ])
            })
            .collect();

        let table = Table::new(
            body,
            [
                Constraint::Length(8),
                Constraint::Min(20),
                Constraint::Length(10),
                Constraint::Length(12),
            ],
        )
        .header(header)
        .block(Block::bordered().title(format!(" Processes ({}) ", rows.len())))
        .row_highlight_style(Style::default().bg(Color::DarkGray));

        frame.render_stateful_widget(table, table_area, &mut self.state);
        frame.render_widget(
            footer(
                "Tab: dashboard  ↑↓/wheel: scroll  C/M: sort  Space: pause  Q: quit",
                &self.stats.borrow(),
            ),
            bottom,
        );
    }
}