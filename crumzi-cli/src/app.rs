use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode};
// use crumzi_client::Client as Crumzi_Client;
use mpd::{Client, Song};
use ratatui::{
    DefaultTerminal,
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    symbols,
    text::Line,
    widgets::{Block, LineGauge, List, ListItem, StatefulWidget, Tabs, Widget},
};
use std::{str::FromStr, time::Duration};
use strum::{Display, EnumIter, FromRepr, IntoEnumIterator};

#[derive(Default, Clone, Copy)]
pub struct App {
    selected_tab: SelectedTab,
    playing: bool,
    volume: i8,
    state: AppState,
}

impl App {
    pub fn run(mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        let mut state = ClientState {
            client: Client::default(),
            queue: vec![],
            elapsed: 0_u64,
        };
        while self.is_running() {
            if event::poll(Duration::from_millis(250))?
                && let Event::Key(key_event) = event::read()?
            {
                self.handle_key(key_event.code, &mut state.client);
            }
            terminal
                .draw(|frame| frame.render_stateful_widget(&mut self, frame.area(), &mut state))?;
        }
        Ok(())
    }

    fn toggle_pause(&mut self, client: &mut Client) {
        self.playing = !self.playing;
        client.toggle_pause().unwrap();
    }

    fn play(&mut self, client: &mut Client) {
        self.playing = !self.playing;
        client.play().unwrap();
    }

    fn clear_queue(&mut self, client: &mut Client) {
        self.playing = !self.playing;
        client.clear().unwrap();
    }

    fn next_song(&mut self, client: &mut Client) {
        self.playing = !self.playing;
        client.next().unwrap();
    }

    fn prev_song(&mut self, client: &mut Client) {
        self.playing = !self.playing;
        client.prev().unwrap();
    }

    fn is_running(self) -> bool {
        self.state == AppState::Running
    }

    const fn quit(&mut self) {
        self.state = AppState::Quit;
    }

    fn modify_vol(&mut self, vol: i8, client: &mut Client) {
        if (self.volume >= 100 && vol > 0) || (self.volume == 0 && vol < 0) {
            return;
        }
        self.volume += vol;
        client.volume(self.volume).unwrap();
    }

    fn handle_key(&mut self, code: KeyCode, client: &mut Client) {
        use crossterm::event::KeyCode::*;
        match code {
            Char('q') | KeyCode::Esc => self.quit(),
            Char('=') | KeyCode::Char('+') => self.modify_vol(5, client),
            Char('-') => self.modify_vol(-5, client),
            Char('p') => self.toggle_pause(client),
            Char('c') => self.clear_queue(client),
            Char('1') => self.selected_tab = SelectedTab::Current,
            Char('2') => self.selected_tab = SelectedTab::Playlists,
            Char('3') => self.selected_tab = SelectedTab::PlaylistEditor,
            Char('>') => self.next_song(client),
            Char('<') => self.prev_song(client),
            Up | Down | Left | Right | Char('h') | Char('j') | Char('k') | Char('l') => {
                self.navigate_queue(client, code);
                self.play(client);
            }
            _ => (),
        }
    }

    fn navigate_queue(&mut self, client: &mut Client, code: KeyCode) {
        use crossterm::event::KeyCode::*;
        match code {
            Left | Char('h') => {
                if !self.playing {
                    self.play(client);
                }
                self.prev_song(client)
            }
            Right | Char('l') => {
                if !self.playing {
                    self.play(client);
                }
                self.next_song(client)
            }
            Up | Char('k') => {}
            Down | Char('j') => {}
            _ => {}
        }
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

impl StatefulWidget for &mut App {
    type State = ClientState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let status = &mut state.client.status().unwrap();

        self.volume = status.volume;

        let tokyo_night: Theme = Theme {
            text: Color::from_str("#c0caf5").unwrap(),
            background: Color::from_str("#24283b").unwrap(),
            hightlight: Color::from_str("#364A82").unwrap(),
            shadow: Color::from_str("#73daca").unwrap(),
        };
        // Set overall style for the area
        buf.set_style(
            area,
            Style::new()
                .underline_color(tokyo_night.shadow)
                .fg(tokyo_night.text)
                .bg(tokyo_night.background)
                .underline_color(tokyo_night.hightlight),
        );

        let outer_layout = Layout::vertical(vec![
            Constraint::Percentage(5),  // tab name
            Constraint::Percentage(5),  // top_layout
            Constraint::Percentage(97), // queue list
            Constraint::Percentage(2),  // playback gauge
        ])
        .split(area);
        let top_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(20), // timer
                Constraint::Percentage(50), // song name
                Constraint::Percentage(25), // volume bar
                Constraint::Percentage(5),  // playback mode (e.g. random)
            ])
            .split(outer_layout[1]);

        Tabs::new(
            SelectedTab::iter().map(|x| Line::from(x.to_string()).fg(tokyo_night.text).bold()),
        )
        .divider(symbols::DOT)
        .padding(" ", " ")
        .select(self.selected_tab as usize)
        .block(Block::default())
        .render(outer_layout[0], buf);

        let binding = state.client.queue().unwrap();
        state.queue = binding.clone();
        let current_index = status.song.map(|s| s.pos as usize);
        let songs: Vec<ListItem> = binding
            .iter()
            .enumerate()
            .map(|(idx, song)| {
                let title = song.title.as_deref().unwrap_or("Unknown title");
                let line = Line::from(title.to_string());
                let item = ListItem::new(line);
                if Some(idx) == current_index {
                    item.style(Style::new().add_modifier(Modifier::REVERSED))
                } else {
                    item.style(Style::new().white())
                }
            })
            .collect();
        Widget::render(
            List::new(songs)
                .highlight_symbol(">")
                .highlight_style(Modifier::REVERSED),
            outer_layout[2],
            buf,
        );

        let elapsed = status.elapsed.map(|d| d.as_secs()).unwrap_or(0);
        state.elapsed = elapsed;
        let duration = status.duration.map(|d| d.as_secs());
        let timer = if let Some(total) = duration {
            let (em, es) = (elapsed / 60, elapsed % 60);
            let (tm, ts) = (total / 60, total % 60);
            format!("{:02}:{:02}/{:02}:{:02}", em, es, tm, ts)
        } else {
            let (em, es) = (elapsed / 60, elapsed % 60);
            format!("{:02}:{:02}", em, es)
        };
        Line::from(timer).render(top_layout[0], buf);

        let current_playing = state.client.currentsong().unwrap();
        if let Some(song) = current_playing {
            let title = match song.title {
                Some(title) => title,
                None => "No Song playing".to_string(),
            };
            Line::from(title).render(top_layout[1], buf);
        }

        Line::from(format!("Vol: {}%", self.volume)).render(top_layout[2], buf);

        let random = if status.random { "✅" } else { "❌" };
        Line::from(random).render(top_layout[3], buf);

        if let Some(total) = duration {
            let line_gauge = LineGauge::default()
                .block(Block::new().title("LineGauge:"))
                .filled_style(Style::default().fg(Color::Magenta))
                .ratio(elapsed as f64 / total as f64);
            Widget::render(line_gauge, outer_layout[3], buf);
        }
    }
}

pub struct ClientState {
    client: Client,
    queue: Vec<Song>,
    elapsed: u64,
}

/// Tabs for the different examples
///
/// The order of the variants is the order in which they are displayed.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, Display, FromRepr, EnumIter)]
enum SelectedTab {
    #[default]
    Current,
    Playlists,
    PlaylistEditor,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
enum AppState {
    #[default]
    Running,
    Quit,
}
