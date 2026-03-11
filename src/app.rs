use color_eyre::Result;
use crossterm::event::{self, KeyCode};
use mpd::Client;
use ratatui::{
    DefaultTerminal,
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    symbols,
    text::Line,
    widgets::{Block, Gauge, Tabs, Widget},
};
use std::str::FromStr;
use strum::{Display, EnumIter, FromRepr, IntoEnumIterator};

use crate::ui::utils;

#[derive(Default, Clone, Copy)]
pub struct App {
    selected_tab: SelectedTab,
    playing: bool,
    volume: i8,
    state: AppState,
}

impl App {
    pub fn run(mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        let mut client = Client::default();
        client.load("pixeltee", ..).unwrap();
        client.play().unwrap();
        while self.is_running() {
            terminal.draw(|frame| frame.render_widget(self, frame.area()))?;
            self.handle_events(&mut client)?;
        }
        Ok(())
    }

    fn toggle_play(&mut self, client: &mut Client) {
        self.playing = !self.playing;
        client.toggle_pause().unwrap();
    }

    fn is_running(self) -> bool {
        self.state == AppState::Running
    }

    fn quit(&mut self, client: &mut Client) {
        client.clear().unwrap();
        self.state = AppState::Quit;
    }

    fn modify_vol(&mut self, vol: i8, client: &mut Client) {
        if (self.volume >= 100 && vol > 0) || (self.volume == 0 && vol < 0) {
            return;
        }
        self.volume += vol;
        client.volume(self.volume).unwrap();
    }

    fn handle_events(&mut self, client: &mut Client) -> Result<()> {
        if let Some(key) = event::read()?.as_key_press_event() {
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => self.quit(client),
                KeyCode::Char('=') | KeyCode::Char('+') => self.modify_vol(5, client),
                KeyCode::Char('-') => self.modify_vol(-5, client),
                KeyCode::Char('p') => self.toggle_play(client),
                KeyCode::Char('1') => self.selected_tab = SelectedTab::Current,
                KeyCode::Char('2') => self.selected_tab = SelectedTab::Playlist,
                _ => (),
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
struct Theme {
    text: Color,
    background: Color,
    hightlight: Color,
    shadow: Color,
}

impl Widget for App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let tokyo_night: Theme = Theme {
            text: Color::from_str("#c0caf5").unwrap(),
            background: Color::from_str("#24283b").unwrap(),
            hightlight: Color::from_str("#364A82").unwrap(),
            shadow: Color::from_str("#73daca").unwrap(),
        };
        // Set overall style for the area
        buf.set_style(
            area,
            Style::default()
                .fg(tokyo_night.text)
                .bg(tokyo_night.background),
        );

        let outer_layout = Layout::vertical(vec![
            Constraint::Percentage(5),
            Constraint::Percentage(93),
            Constraint::Percentage(2),
        ])
        .split(area);

        let top_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(30), // tab layout
                Constraint::Percentage(50),
                Constraint::Percentage(20), // volume bar
            ])
            .split(outer_layout[0]);

        Tabs::new(
            SelectedTab::iter().map(|x| Line::from(x.to_string()).fg(tokyo_night.text).bold()),
        )
        .divider(symbols::DOT)
        .padding(" ", " ")
        .select(self.selected_tab as usize)
        .block(Block::default())
        .render(top_layout[0], buf);

        // Render the volume gauge in the top section
        Gauge::default()
            .label(format!("Volume {}%", self.volume))
            .percent(self.volume as u16)
            .render(top_layout[2], buf);

        utils::center(
            top_layout[2],
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        );

        // Block::bordered()
        //     .title("Playlist: ".to_span().into_centered_line())
        //     .render(area, buf);
    }
}

/// Tabs for the different examples
///
/// The order of the variants is the order in which they are displayed.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, Display, FromRepr, EnumIter)]
enum SelectedTab {
    #[default]
    Current,
    Playlist,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
enum AppState {
    #[default]
    Running,
    Quit,
}
