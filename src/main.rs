use std::io::{self, Stdout};
use std::time::{Duration, Instant};

use anyhow::Result;
use clap::{ArgAction, Parser};
use crossterm::event::{
    self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, KeyModifiers,
};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::Terminal;
use ratatui::layout::{Alignment, Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Clear, Gauge, Paragraph};

/// 运行参数
#[derive(Debug, Clone, Parser)]
#[command(name = "pomodoros", version, about = "Rust TUI Pomodoro Timer")]
struct CliArgs {
    /// Focus duration in minutes
    #[arg(short = 'f', long = "focus", default_value_t = 25)]
    focus_minutes: u64,

    /// Short break duration in minutes
    #[arg(short = 's', long = "short", default_value_t = 5)]
    short_break_minutes: u64,

    /// Long break duration in minutes
    #[arg(short = 'l', long = "long", default_value_t = 15)]
    long_break_minutes: u64,

    /// Take a long break after every N focus sessions
    #[arg(short = 'e', long = "every", default_value_t = 4)]
    long_every: u32,

    /// Mute terminal bell
    #[arg(long = "mute", default_value_t = false, action = ArgAction::SetTrue)]
    mute: bool,

    /// Tick interval in milliseconds
    #[arg(long = "tick", default_value_t = 200)]
    tick_ms: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Phase {
    Focus,
    ShortBreak,
    LongBreak,
}

impl Phase {
    fn name(self) -> &'static str {
        match self {
            Phase::Focus => "Focus",
            Phase::ShortBreak => "Short Break",
            Phase::LongBreak => "Long Break",
        }
    }

    fn color(self) -> Color {
        match self {
            Phase::Focus => Color::LightGreen,
            Phase::ShortBreak => Color::Cyan,
            Phase::LongBreak => Color::Magenta,
        }
    }
}

#[derive(Debug, Clone)]
struct PomodoroConfig {
    focus: Duration,
    short_break: Duration,
    long_break: Duration,
    long_every: u32,
    mute: bool,
}

#[derive(Debug)]
struct PomodoroApp {
    config: PomodoroConfig,
    phase: Phase,
    total: Duration,
    remaining: Duration,
    running: bool,
    completed_focus: u32,
    last_tick: Instant,
}

impl PomodoroApp {
    fn new(config: PomodoroConfig) -> Self {
        let total = config.focus;
        Self {
            config,
            phase: Phase::Focus,
            total,
            remaining: total,
            running: false,
            completed_focus: 0,
            last_tick: Instant::now(),
        }
    }

    fn reset_current(&mut self) {
        self.total = match self.phase {
            Phase::Focus => self.config.focus,
            Phase::ShortBreak => self.config.short_break,
            Phase::LongBreak => self.config.long_break,
        };
        self.remaining = self.total;
    }

    fn toggle(&mut self) {
        self.running = !self.running;
    }

    fn skip(&mut self) {
        self.to_next_phase();
    }

    fn update(&mut self) {
        if !self.running {
            self.last_tick = Instant::now();
            return;
        }
        let now = Instant::now();
        let delta = now.saturating_duration_since(self.last_tick);
        self.last_tick = now;

        if delta >= self.remaining {
            self.remaining = Duration::ZERO;
            self.on_finish();
        } else {
            self.remaining -= delta;
        }
    }

    fn on_finish(&mut self) {
        if !self.config.mute {
            // 终端响铃
            print!("\x07");
            let _ = io::Write::flush(&mut io::stdout());
        }

        match self.phase {
            Phase::Focus => {
                self.completed_focus += 1;
                let use_long = self.completed_focus % self.config.long_every == 0;
                self.phase = if use_long {
                    Phase::LongBreak
                } else {
                    Phase::ShortBreak
                };
            }
            Phase::ShortBreak | Phase::LongBreak => {
                self.phase = Phase::Focus;
            }
        }
        self.reset_current();
        self.running = true; // 自动开始下一阶段
    }

    fn to_next_phase(&mut self) {
        match self.phase {
            Phase::Focus => {
                self.phase = Phase::ShortBreak;
            }
            Phase::ShortBreak => {
                self.phase = Phase::Focus;
            }
            Phase::LongBreak => {
                self.phase = Phase::Focus;
            }
        }
        self.reset_current();
    }

    fn formatted_remaining(&self) -> String {
        let total_secs = self.remaining.as_secs();
        let minutes = total_secs / 60;
        let seconds = total_secs % 60;
        format!("{:02}:{:02}", minutes, seconds)
    }

    fn progress_ratio(&self) -> f64 {
        if self.total.is_zero() {
            return 0.0;
        }
        let elapsed = self.total.saturating_sub(self.remaining);
        elapsed.as_secs_f64() / self.total.as_secs_f64()
    }
}

fn ui(frame: &mut ratatui::Frame, app: &PomodoroApp) {
    let size = frame.size();

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(
            [
                Constraint::Length(5), // header
                Constraint::Length(4), // gauge
                Constraint::Min(7),    // big timer
                Constraint::Length(3), // help
            ]
            .as_ref(),
        )
        .split(size);

    // Header
    let accent = app.phase.color();
    let title = Line::from(vec![
        Span::styled("● ", Style::default().fg(accent)),
        Span::styled(
            app.phase.name(),
            Style::default().fg(accent).add_modifier(Modifier::BOLD),
        ),
        Span::raw("  ·  Completed "),
        Span::styled(
            format!("{}", app.completed_focus),
            Style::default()
                .fg(Color::Gray)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(""),
    ]);
    let header = Paragraph::new(title)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title("Status")
                .title_alignment(Alignment::Center),
        )
        .alignment(Alignment::Center);
    frame.render_widget(header, layout[0]);

    // Gauge
    let percent = (app.progress_ratio() * 100.0) as u16;
    let gauge = Gauge::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title("Progress")
                .title_alignment(Alignment::Center),
        )
        .gauge_style(
            Style::default()
                .fg(accent)
                .bg(Color::Black)
                .add_modifier(Modifier::BOLD),
        )
        .label(Span::styled(
            format!("{}  ·  {}%", app.formatted_remaining(), percent),
            Style::default().fg(Color::White),
        ))
        .percent(percent);
    frame.render_widget(gauge, layout[1]);

    // Big timer text
    let time_text = if app.running {
        "⏱ Running"
    } else {
        "⏸ Paused"
    };
    let timer_lines = vec![
        Line::from(Span::styled(
            app.formatted_remaining(),
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(time_text, Style::default().fg(Color::Gray))),
    ];
    let timer = Paragraph::new(timer_lines)
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title("Timer")
                .title_alignment(Alignment::Center),
        );
    frame.render_widget(timer, layout[2]);

    // Help footer
    let help = Paragraph::new(Line::from(vec![
        Span::raw("␣ Space: Start/Pause  ·  "),
        Span::raw("⏭ n: Skip  ·  "),
        Span::raw("⟲ r: Reset  ·  "),
        Span::raw("q: Quit"),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title("Shortcuts")
            .title_alignment(Alignment::Center),
    )
    .alignment(Alignment::Center);
    frame.render_widget(Clear, layout[3]);
    frame.render_widget(help, layout[3]);
}

fn setup_terminal() -> Result<Terminal<ratatui::backend::CrosstermBackend<Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = ratatui::backend::CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

fn restore_terminal(
    mut terminal: Terminal<ratatui::backend::CrosstermBackend<Stdout>>,
) -> Result<()> {
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(())
}

fn main() -> Result<()> {
    let args = CliArgs::parse();
    let config = PomodoroConfig {
        focus: Duration::from_secs(args.focus_minutes * 60),
        short_break: Duration::from_secs(args.short_break_minutes * 60),
        long_break: Duration::from_secs(args.long_break_minutes * 60),
        long_every: args.long_every,
        mute: args.mute,
    };
    let tick = Duration::from_millis(args.tick_ms);

    let mut terminal = setup_terminal()?;
    let mut app = PomodoroApp::new(config);

    let mut last_redraw = Instant::now();
    loop {
        // 处理输入事件
        if event::poll(tick)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc | KeyCode::Char('c')
                            if key.modifiers.contains(KeyModifiers::CONTROL) =>
                        {
                            break;
                        }
                        KeyCode::Char(' ') => app.toggle(),
                        KeyCode::Char('n') | KeyCode::Right => app.skip(),
                        KeyCode::Char('r') => app.reset_current(),
                        KeyCode::Char('q') => break,
                        _ => {}
                    }
                }
            }
        }

        // 更新状态
        app.update();

        // 绘制
        if last_redraw.elapsed() >= Duration::from_millis(16) {
            // ~60FPS 上限
            terminal.draw(|f| ui(f, &app))?;
            last_redraw = Instant::now();
        }
    }

    restore_terminal(terminal)?;
    Ok(())
}
