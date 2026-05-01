use crate::{
    common::{
        types::LogEvent,
        utils::{dt_to_slots_idx, get_current_dt},
    },
    config::Config,
};
use ansi_to_tui::IntoText;
use chrono::{Datelike, Local, Timelike};
use colored::{ColoredString, Colorize};
use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers, MouseButton,
        MouseEventKind,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, is_raw_mode_enabled},
};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Text},
    widgets::{Block, Borders, Paragraph, Wrap},
};
use std::{
    fs::OpenOptions,
    io::{self, Write, stdout},
    sync::mpsc::Receiver,
    time::Duration,
};
use tokio::{
    task::JoinHandle,
    time::{Interval, interval},
};

#[cfg(unix)]
fn force_wake_launcher() {
    unsafe {
        libc::raise(libc::SIGINT);
    }
}

#[cfg(windows)]
fn force_wake_launcher() {}

#[derive(PartialEq, Copy, Clone)]
enum SelectedBox {
    Ais = 0,
    System = 1,
    Computer = 2,
    Satcom = 3,
    Gps = 4,
}

pub struct LogsCli {
    system_logs: Vec<Line<'static>>,
    ais_logs: Vec<Line<'static>>,
    gps_logs: Vec<Line<'static>>,
    satcom_logs: Vec<Line<'static>>,
    computer_logs: Vec<Line<'static>>,
    scrolls: [u16; 5],
    auto_scroll: [bool; 5],
    areas: [Rect; 5],
    focused: SelectedBox,
    rx: Receiver<LogEvent>,
    sys_logs_filename: String,
    ais_logs_filename: String,
    gps_logs_filename: String,
    satcom_logs_filename: String,
    computer_logs_filename: String,
}

impl LogsCli {
    pub fn new(
        rx: Receiver<LogEvent>,
        sys_logs_filename: String,
        ais_logs_filename: String,
        gps_logs_filename: String,
        satcom_logs_filename: String,
        computer_logs_filename: String,
    ) -> Self {
        Self {
            system_logs: Vec::new(),
            ais_logs: Vec::new(),
            gps_logs: Vec::new(),
            satcom_logs: Vec::new(),
            computer_logs: Vec::new(),
            scrolls: [0; 5],
            auto_scroll: [true; 5],
            areas: [Rect::default(); 5],
            focused: SelectedBox::Ais,
            rx,
            sys_logs_filename,
            ais_logs_filename,
            gps_logs_filename,
            satcom_logs_filename,
            computer_logs_filename,
        }
    }

    fn get_visual_line_count(logs: &[Line<'static>], width: u16) -> usize {
        if width == 0 {
            return 0;
        }
        let width = width as usize;
        let mut count = 0;
        for line in logs {
            let lw = line.width();
            if lw == 0 {
                count += 1;
            } else {
                count += (lw + width - 1) / width;
            }
        }
        count
    }

    pub fn run(mut self) -> Result<JoinHandle<()>, io::Error> {
        let mut out = stdout();

        let mut refresh_interval: Interval = interval(Duration::from_millis(
            *Config::load().unwrap().cli_refresh_delay(),
        ));

        let raw_was_on = is_raw_mode_enabled().unwrap_or(false);
        if !raw_was_on {
            enable_raw_mode()?;
        }

        execute!(out, EnableMouseCapture)?;

        let mut terminal: Terminal<CrosstermBackend<io::Stdout>> =
            Terminal::new(CrosstermBackend::new(out))?;
        terminal.clear()?;

        Ok(tokio::spawn(async move {
            let mut should_quit = false;

            while !should_quit {
                while let Ok(event) = self.rx.try_recv() {
                    match event {
                        LogEvent::System(m) => self.system_log(m),
                        LogEvent::Ais(m) => self.ais_log(m),
                        LogEvent::Gps(m) => self.gps_log(m),
                        LogEvent::Satcom(m) => self.satcom_log(m),
                        LogEvent::Computer(m) => self.computer_log(m),
                    }
                }

                terminal.draw(|f| self.ui(f)).unwrap();

                if event::poll(Duration::from_millis(16)).unwrap_or(false) {
                    if let Ok(evt) = event::read() {
                        self.process_input(evt, &mut should_quit);

                        while event::poll(Duration::from_millis(0)).unwrap_or(false) {
                            if let Ok(next_evt) = event::read() {
                                self.process_input(next_evt, &mut should_quit);
                            }
                        }
                    }
                }
            }

            let _ = terminal.clear();

            let _ = execute!(stdout(), DisableMouseCapture, crossterm::cursor::Show);

            if !raw_was_on {
                let _ = disable_raw_mode();
            }

            print!("\x1b[6n");
            let _ = stdout().flush();

            refresh_interval.tick().await;

            force_wake_launcher();
        }))
    }

    fn process_input(&mut self, evt: Event, should_quit: &mut bool) {
        match evt {
            Event::Key(key) => match key.code {
                KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    *should_quit = true
                }
                KeyCode::Char('q') | KeyCode::Esc => *should_quit = true,
                KeyCode::Tab => self.next_focus(),
                KeyCode::Up => self.scroll_current(-1),
                KeyCode::Down => self.scroll_current(1),
                KeyCode::PageUp => self.scroll_current(-5),
                KeyCode::PageDown => self.scroll_current(5),
                _ => {}
            },
            Event::Mouse(mouse) => match mouse.kind {
                MouseEventKind::ScrollUp => self.scroll_current(-3),
                MouseEventKind::ScrollDown => self.scroll_current(3),
                MouseEventKind::Down(MouseButton::Left) => {
                    self.handle_click(mouse.column, mouse.row)
                }
                _ => {}
            },
            _ => {}
        }
    }

    fn log(msg: ColoredString, log_filename: &str, logs_vec: &mut Vec<Line<'static>>) {
        let current_dt: chrono::DateTime<Local> = get_current_dt();
        let slots: [u16; 2] = dt_to_slots_idx(Some(current_dt));

        let clean_msg: String = msg
            .to_string()
            .chars()
            .filter(|c: &char| {
                !c.is_control() || *c == '\x1b' || *c == '\n' || *c == '\r' || *c == '\t'
            })
            .collect();

        if let Ok(mut file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_filename)
        {
            let file_log_msg: String = format!(
                "({}, {}), {}/{}/{} {}h:{}mn:{}s:\n{}\n",
                slots[0],
                slots[1],
                current_dt.day(),
                current_dt.month(),
                current_dt.year(),
                current_dt.hour(),
                current_dt.minute(),
                current_dt.second(),
                msg.clone().clear()
            );
            let _ = writeln!(file, "{}", file_log_msg);
        }

        let log_str: String = format!(
            "({}, {}), {}/{}/{} {}h:{}mn:{}s:\n{}",
            slots[0],
            slots[1],
            current_dt.day(),
            current_dt.month(),
            current_dt.year(),
            current_dt.hour(),
            current_dt.minute(),
            current_dt.second(),
            clean_msg
        );

        if let Ok(tui_text) = log_str.into_text() {
            logs_vec.extend(tui_text.lines);
            logs_vec.push(Line::default()); // Saut de ligne
        }

        let max_logs_history_length: usize = *Config::load().unwrap().max_cli_logs_history_length();

        if logs_vec.len() > max_logs_history_length {
            let excess: usize = logs_vec.len() - max_logs_history_length;
            logs_vec.drain(0..excess);
        }
    }

    pub fn system_log(&mut self, msg: ColoredString) {
        LogsCli::log(msg, &self.sys_logs_filename, &mut self.system_logs);
    }
    pub fn ais_log(&mut self, msg: ColoredString) {
        LogsCli::log(msg, &self.ais_logs_filename, &mut self.ais_logs);
    }
    pub fn gps_log(&mut self, msg: ColoredString) {
        LogsCli::log(msg, &self.gps_logs_filename, &mut self.gps_logs);
    }
    pub fn satcom_log(&mut self, msg: ColoredString) {
        LogsCli::log(msg, &self.satcom_logs_filename, &mut self.satcom_logs);
    }
    pub fn computer_log(&mut self, msg: ColoredString) {
        LogsCli::log(msg, &self.computer_logs_filename, &mut self.computer_logs);
    }

    fn scroll_current(&mut self, delta: i16) {
        let (scroll_idx, logs) = match self.focused {
            SelectedBox::Ais => (0, &self.ais_logs),
            SelectedBox::System => (1, &self.system_logs),
            SelectedBox::Computer => (2, &self.computer_logs),
            SelectedBox::Satcom => (3, &self.satcom_logs),
            SelectedBox::Gps => (4, &self.gps_logs),
        };

        let area_width = self.areas[scroll_idx].width.saturating_sub(2);
        let line_count = Self::get_visual_line_count(logs, area_width);

        let area_height: usize = self.areas[scroll_idx].height.saturating_sub(2) as usize;
        let max_scroll: i16 = (line_count as i16)
            .saturating_sub(area_height as i16)
            .max(0);

        let new_scroll: i16 = (self.scrolls[scroll_idx] as i16 + delta).clamp(0, max_scroll);
        self.scrolls[scroll_idx] = new_scroll as u16;

        self.auto_scroll[scroll_idx] = new_scroll == max_scroll;
    }

    fn next_focus(&mut self) {
        self.focused = match self.focused {
            SelectedBox::Ais => SelectedBox::System,
            SelectedBox::System => SelectedBox::Computer,
            SelectedBox::Computer => SelectedBox::Satcom,
            SelectedBox::Satcom => SelectedBox::Gps,
            SelectedBox::Gps => SelectedBox::Ais,
        };
    }

    fn handle_click(&mut self, x: u16, y: u16) {
        for (i, area) in self.areas.iter().enumerate() {
            if x >= area.x && x < area.x + area.width && y >= area.y && y < area.y + area.height {
                self.focused = match i {
                    0 => SelectedBox::Ais,
                    1 => SelectedBox::System,
                    2 => SelectedBox::Computer,
                    3 => SelectedBox::Satcom,
                    _ => SelectedBox::Gps,
                };
                break;
            }
        }
    }

    fn ui(&mut self, f: &mut ratatui::Frame) {
        let main: std::rc::Rc<[Rect]> = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(f.area());

        let right: std::rc::Rc<[Rect]> = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Ratio(1, 4); 4])
            .split(main[1]);

        self.areas[0] = main[0];
        self.areas[1] = right[0];
        self.areas[2] = right[1];
        self.areas[3] = right[2];
        self.areas[4] = right[3];

        let logs_lens = [
            Self::get_visual_line_count(&self.ais_logs, self.areas[0].width.saturating_sub(2)),
            Self::get_visual_line_count(&self.system_logs, self.areas[1].width.saturating_sub(2)),
            Self::get_visual_line_count(&self.computer_logs, self.areas[2].width.saturating_sub(2)),
            Self::get_visual_line_count(&self.satcom_logs, self.areas[3].width.saturating_sub(2)),
            Self::get_visual_line_count(&self.gps_logs, self.areas[4].width.saturating_sub(2)),
        ];

        for i in 0..5 {
            if self.auto_scroll[i] {
                let area_height = self.areas[i].height.saturating_sub(2) as usize;
                self.scrolls[i] = logs_lens[i].saturating_sub(area_height) as u16;
            }
        }

        self.draw_box(
            f,
            self.areas[0],
            " AIS ",
            &self.ais_logs,
            self.scrolls[0],
            Color::Cyan,
            SelectedBox::Ais,
        );
        self.draw_box(
            f,
            self.areas[1],
            " Système ",
            &self.system_logs,
            self.scrolls[1],
            Color::Red,
            SelectedBox::System,
        );
        self.draw_box(
            f,
            self.areas[2],
            " Ordinateur ",
            &self.computer_logs,
            self.scrolls[2],
            Color::Magenta,
            SelectedBox::Computer,
        );
        self.draw_box(
            f,
            self.areas[3],
            " SATCOM ",
            &self.satcom_logs,
            self.scrolls[3],
            Color::Yellow,
            SelectedBox::Satcom,
        );
        self.draw_box(
            f,
            self.areas[4],
            " GPS ",
            &self.gps_logs,
            self.scrolls[4],
            Color::Green,
            SelectedBox::Gps,
        );
    }

    fn draw_box(
        &self,
        f: &mut ratatui::Frame,
        area: Rect,
        title: &str,
        content: &[Line<'static>],
        scroll: u16,
        color: Color,
        id: SelectedBox,
    ) {
        let is_focused: bool = self.focused == id;

        let title: String = if is_focused {
            format!("{} [SCROLL]", title)
        } else {
            title.to_string()
        };
        let border: Style = if is_focused {
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(color)
        };

        let p: Paragraph<'_> = Paragraph::new(Text::from(content.to_vec()))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(title)
                    .border_style(border),
            )
            .wrap(Wrap { trim: false })
            .scroll((scroll, 0));

        f.render_widget(p, area);
    }
}
