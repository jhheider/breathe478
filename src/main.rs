use std::io;
use std::time::{Duration, Instant};

const VERSION: &str = env!("CARGO_PKG_VERSION");

use crossterm::{
    ExecutableCommand,
    event::{self, Event, KeyCode},
    terminal::{self, EnterAlternateScreen},
};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout},
    style::{Color, Style},
    widgets::{Block, Gauge, Paragraph},
};

#[derive(Clone, Copy)]
enum Phase {
    Inhale,
    Hold,
    Exhale,
}

impl Phase {
    fn duration_secs(self) -> u64 {
        match self {
            Phase::Inhale => 4,
            Phase::Hold => 7,
            Phase::Exhale => 8,
        }
    }

    fn name(self) -> &'static str {
        match self {
            Phase::Inhale => "I N H A L E",
            Phase::Hold => "H O L D",
            Phase::Exhale => "E X H A L E",
        }
    }

    fn emoji(self) -> &'static str {
        match self {
            Phase::Inhale => "🌬️",
            Phase::Hold => "✋",
            Phase::Exhale => "💨",
        }
    }

    fn color(self) -> Color {
        match self {
            Phase::Inhale => Color::Cyan,
            Phase::Hold => Color::Yellow,
            Phase::Exhale => Color::Magenta,
        }
    }

    fn next(self) -> Phase {
        match self {
            Phase::Inhale => Phase::Hold,
            Phase::Hold => Phase::Exhale,
            Phase::Exhale => Phase::Inhale,
        }
    }
}

struct App {
    phase: Phase,
    phase_start: Instant,
    app_start: Instant,
    cycles_completed: u32,
}

impl App {
    fn new() -> Self {
        let now = Instant::now();
        Self {
            phase: Phase::Inhale,
            phase_start: now,
            app_start: now,
            cycles_completed: 0,
        }
    }

    fn tick(&mut self, now: Instant) {
        let elapsed = now.duration_since(self.phase_start);
        let duration = Duration::from_secs(self.phase.duration_secs());
        if elapsed >= duration {
            let next = self.phase.next();
            if matches!(next, Phase::Inhale) {
                self.cycles_completed += 1;
            }
            self.phase = next;
            self.phase_start = now;
        }
    }

    fn remaining_secs(&self) -> u64 {
        let elapsed = self.phase_start.elapsed().as_millis() as u64;
        let total = self.phase.duration_secs() * 1000;
        total.saturating_sub(elapsed).div_ceil(1000)
    }

    fn progress_ratio(&self) -> f64 {
        let elapsed = self.phase_start.elapsed().as_secs_f64();
        let total = self.phase.duration_secs() as f64;
        (elapsed / total).min(1.0)
    }

    fn elapsed_display(&self) -> String {
        let secs = self.app_start.elapsed().as_secs();
        let m = secs / 60;
        let s = secs % 60;
        format!("{m}:{s:02}")
    }
}

fn ui(frame: &mut Frame, app: &App) {
    let color = app.phase.color();
    let fg = Color::Black;

    // Fill entire background with phase color
    let area = frame.area();
    frame.render_widget(Block::default().style(Style::new().bg(color)), area);

    let chunks = Layout::vertical([
        Constraint::Percentage(30),
        Constraint::Length(1),
        Constraint::Length(2),
        Constraint::Length(1),
        Constraint::Length(3),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Min(0),
    ])
    .split(area);

    // Emoji
    let emoji = Paragraph::new(app.phase.emoji())
        .alignment(Alignment::Center)
        .style(Style::new().bg(color));
    frame.render_widget(emoji, chunks[1]);

    // Phase name
    let phase_name = Paragraph::new(app.phase.name())
        .alignment(Alignment::Center)
        .style(Style::new().fg(fg).bg(color).bold());
    frame.render_widget(phase_name, chunks[2]);

    // Countdown
    let countdown = Paragraph::new(format!("{}", app.remaining_secs()))
        .alignment(Alignment::Center)
        .style(Style::new().fg(fg).bg(color).bold());
    frame.render_widget(countdown, chunks[3]);

    // Progress gauge
    let gauge = Gauge::default()
        .block(Block::default().style(Style::new().bg(color)))
        .gauge_style(Style::new().fg(fg).bg(color))
        .ratio(app.progress_ratio());
    frame.render_widget(gauge, chunks[4]);

    // Cycle count + elapsed time
    let info = Paragraph::new(format!(
        "Cycle {}  ·  {}",
        app.cycles_completed + 1,
        app.elapsed_display()
    ))
    .alignment(Alignment::Center)
    .style(Style::new().fg(fg).bg(color));
    frame.render_widget(info, chunks[5]);

    // Quit hint
    let hint = Paragraph::new("Press q to quit")
        .alignment(Alignment::Center)
        .style(Style::new().fg(fg).bg(color));
    frame.render_widget(hint, chunks[6]);
}

fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "--version" || a == "-V") {
        println!("breathe478 {VERSION}");
        return Ok(());
    }

    terminal::enable_raw_mode()?;
    io::stdout().execute(EnterAlternateScreen)?;

    let mut terminal = ratatui::init();
    let mut app = App::new();

    loop {
        terminal.draw(|f| ui(f, &app))?;

        if event::poll(Duration::from_millis(50))?
            && let Event::Key(key) = event::read()?
            && (key.code == KeyCode::Char('q') || key.code == KeyCode::Esc)
        {
            break;
        }

        app.tick(Instant::now());
    }

    ratatui::restore();
    Ok(())
}
