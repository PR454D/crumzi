use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode};
use image::imageops::FilterType;
use lofty::file::TaggedFileExt;
use lofty::picture::PictureType;
use lofty::read_from_path;
use mpd::{Client, Playlist, Song};
use ratatui::{
    DefaultTerminal,
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    symbols,
    text::{Line, Span},
    widgets::{
        Block, BorderType, Borders, List, ListItem, ListState, StatefulWidget,
        Tabs, Widget,
    },
};
use std::{ops::RangeFull, path::Path, str::FromStr, time::Duration};
use strum::{Display, EnumIter, FromRepr, IntoEnumIterator};

#[derive(Default, Clone)]
pub struct App {
    selected_tab: SelectedTab,
    volume: i8,
    state: AppState,
    queue_state: ListState,
    playlists_state: ListState,
    editor_state: ListState,
    /// Playlist chosen on the Playlists tab (Enter); shown in PlaylistEditor.
    selected_playlist: Option<String>,
}

impl App {
    pub async fn run(mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        let mut state = ClientState {
            client: Client::default(),
            queue: vec![],
            playlists: vec![],
            editor_tracks: vec![],
            elapsed: 0_u64,
            music_dir_cache: None,
            cover_cache_key: None,
            cover_rgba: None,
        };
        while self.is_running() {
            if event::poll(Duration::from_millis(250))?
                && let Event::Key(key_event) = event::read()?
            {
                self.handle_key(key_event.code, &mut state).await;
            }
            terminal.draw(|frame| {
                frame.render_stateful_widget(
                    &mut self,
                    frame.area(),
                    &mut state,
                )
            })?;
        }
        Ok(())
    }

    fn toggle_pause(&mut self, client: &mut Client) {
        let _ = client.toggle_pause();
    }

    fn clear_queue(&mut self, client: &mut Client) {
        let _ = client.clear();
    }

    fn next_song(&mut self, client: &mut Client) {
        let _ = client.next();
    }

    fn prev_song(&mut self, client: &mut Client) {
        let _ = client.prev();
    }

    fn is_running(&self) -> bool {
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
        let _ = client.volume(self.volume);
    }

    fn toggle_repeat(&mut self, client: &mut Client) {
        if let Ok(st) = client.status() {
            let _ = client.repeat(!st.repeat);
        }
    }

    fn toggle_random(&mut self, client: &mut Client) {
        if let Ok(st) = client.status() {
            let _ = client.random(!st.random);
        }
    }

    fn toggle_consume(&mut self, client: &mut Client) {
        if let Ok(st) = client.status() {
            let _ = client.consume(!st.consume);
        }
    }

    fn on_activate(&mut self, state: &mut ClientState) {
        match self.selected_tab {
            SelectedTab::Current => {
                if let Some(i) = self.queue_state.selected() {
                    let _ = state.client.switch(i as u32);
                }
            }
            SelectedTab::Playlists => {
                let Some(i) = self.playlists_state.selected() else {
                    return;
                };
                let Some(playlist) = state.playlists.get(i) else {
                    return;
                };
                let name = playlist.name.clone();
                let _ = state.client.load(&name, RangeFull);
                self.selected_playlist = Some(name);
            }
            SelectedTab::PlaylistEditor => {}
        }
    }

    fn clamp_list(state: &mut ListState, len: usize) {
        if len == 0 {
            state.select(None);
            return;
        }
        match state.selected() {
            None => state.select(Some(0)),
            Some(i) if i >= len => state.select(Some(len - 1)),
            _ => {}
        }
    }

    async fn handle_key(&mut self, code: KeyCode, state: &mut ClientState) {
        use crossterm::event::KeyCode::*;
        let client = &mut state.client;

        match code {
            Char('q') | Esc => self.quit(),
            Char('=') | Char('+') => self.modify_vol(5, client),
            Char('-') => self.modify_vol(-5, client),
            Char('p') => self.toggle_pause(client),
            Char('X') => self.clear_queue(client),
            Char('1') => self.selected_tab = SelectedTab::Current,
            Char('2') => self.selected_tab = SelectedTab::Playlists,
            Char('3') => self.selected_tab = SelectedTab::PlaylistEditor,
            Char('r') => self.toggle_repeat(client),
            Char('z') => self.toggle_random(client),
            Char('c') => self.toggle_consume(client),
            Char('>') => self.next_song(client),
            Char('<') => self.prev_song(client),
            Enter | Char(' ') => self.on_activate(state),
            Up | Char('k') => self.on_move_up(state),
            Down | Char('j') => self.on_move_down(state),
            Left | Char('h') => {
                if self.selected_tab == SelectedTab::Current {
                    self.prev_song(client);
                }
            }
            Right | Char('l') => {
                if self.selected_tab == SelectedTab::Current {
                    self.next_song(client);
                }
            }
            _ => (),
        }
    }

    fn on_move_up(&mut self, state: &ClientState) {
        match self.selected_tab {
            SelectedTab::Current => {
                self.queue_state.select_previous();
                Self::clamp_list(&mut self.queue_state, state.queue.len());
            }
            SelectedTab::Playlists => {
                self.playlists_state.select_previous();
                Self::clamp_list(
                    &mut self.playlists_state,
                    state.playlists.len(),
                );
            }
            SelectedTab::PlaylistEditor => {
                self.editor_state.select_previous();
                Self::clamp_list(
                    &mut self.editor_state,
                    state.editor_tracks.len(),
                );
            }
        }
    }

    fn on_move_down(&mut self, state: &ClientState) {
        match self.selected_tab {
            SelectedTab::Current => {
                self.queue_state.select_next();
                Self::clamp_list(&mut self.queue_state, state.queue.len());
            }
            SelectedTab::Playlists => {
                self.playlists_state.select_next();
                Self::clamp_list(
                    &mut self.playlists_state,
                    state.playlists.len(),
                );
            }
            SelectedTab::PlaylistEditor => {
                self.editor_state.select_next();
                Self::clamp_list(
                    &mut self.editor_state,
                    state.editor_tracks.len(),
                );
            }
        }
    }

    fn sync_queue_selection(
        &mut self,
        queue_len: usize,
        current_pos: Option<usize>,
    ) {
        if queue_len == 0 {
            self.queue_state.select(None);
            return;
        }
        match self.queue_state.selected() {
            None => {
                let p = current_pos.unwrap_or(0).min(queue_len - 1);
                self.queue_state.select(Some(p));
            }
            Some(i) if i >= queue_len => {
                self.queue_state
                    .select(Some(current_pos.unwrap_or(0).min(queue_len - 1)));
            }
            _ => {}
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct Theme {
    text: Color,
    background: Color,
    highlight: Color,
    accent: Color,
}

impl StatefulWidget for &mut App {
    type State = ClientState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let Ok(status) = state.client.status() else {
            return;
        };

        self.volume = status.volume;

        let theme = Theme {
            text: Color::from_str("#c0caf5").unwrap(),
            background: Color::from_str("#24283b").unwrap(),
            highlight: Color::from_str("#364A82").unwrap(),
            accent: Color::from_str("#73daca").unwrap(),
        };

        buf.set_style(
            area,
            Style::new()
                .fg(theme.text)
                .bg(theme.background)
                .underline_color(theme.highlight),
        );

        let border_style = Style::new().fg(theme.accent);

        // Progress uses a bordered Block + title; needs ≥3 lines (top, gauge, bottom).
        let outer_layout = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(area);

        Tabs::new(
            SelectedTab::iter()
                .map(|x| Line::from(x.to_string()).fg(theme.text).bold()),
        )
        .divider(symbols::DOT)
        .padding(" ", " ")
        .select(self.selected_tab as usize)
        .style(Style::new().fg(theme.text).bg(theme.background))
        .render(outer_layout[0], buf);

        state.queue = state.client.queue().unwrap_or_default();
        state.playlists = state.client.playlists().unwrap_or_default();

        if self.selected_tab == SelectedTab::PlaylistEditor {
            if let Some(ref name) = self.selected_playlist {
                state.editor_tracks =
                    state.client.playlist(name).unwrap_or_default();
            } else {
                state.editor_tracks.clear();
            }
        } else {
            state.editor_tracks.clear();
        }

        App::clamp_list(&mut self.playlists_state, state.playlists.len());
        App::clamp_list(&mut self.editor_state, state.editor_tracks.len());

        let current_index = status.song.map(|s| s.pos as usize);
        self.sync_queue_selection(state.queue.len(), current_index);

        let current_file_key = current_track_path(
            &mut state.client,
            &state.queue,
            status.song.as_ref(),
        );
        state.sync_cover_art(current_file_key.as_deref());

        let top_block = Block::new()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(border_style)
            .title("Status")
            .title_style(Style::new().fg(theme.text).bold())
            .style(Style::new().bg(theme.background));
        let top_inner = top_block.inner(outer_layout[1]);
        let top_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(22),
                Constraint::Percentage(48),
                Constraint::Percentage(20),
                Constraint::Percentage(10),
            ])
            .split(top_inner);

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

        let title_line = match state.client.currentsong().unwrap_or(None) {
            Some(song) => {
                song.title.unwrap_or_else(|| "Unknown title".to_string())
            }
            None => "No song playing".to_string(),
        };
        Line::from(title_line).render(top_layout[1], buf);

        Line::from(format!("Vol: {}%", self.volume)).render(top_layout[2], buf);

        let mode_line = mode_indicator_line(&status, theme.text);
        mode_line.render(top_layout[3], buf);

        top_block.render(outer_layout[1], buf);

        fn main_shell(border_style: Style, bg: Color) -> Block<'static> {
            Block::new()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(border_style)
                .style(Style::new().bg(bg))
        }

        match self.selected_tab {
            SelectedTab::Current => {
                let mid = outer_layout[2];
                let split = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([
                        Constraint::Percentage(38),
                        Constraint::Percentage(62),
                    ])
                    .split(mid);

                let cover_block =
                    main_shell(border_style, theme.background).title("Cover");
                let cover_inner = cover_block.inner(split[0]);
                if let Some(ref img) = state.cover_rgba {
                    render_cover_halfblocks(buf, cover_inner, img);
                } else {
                    Line::from("No cover art")
                        .style(Style::new().fg(theme.text).italic())
                        .render(cover_inner, buf);
                }
                cover_block.render(split[0], buf);

                let queue_block =
                    main_shell(border_style, theme.background).title("Queue");
                let inner = queue_block.inner(split[1]);
                let songs: Vec<ListItem> = state
                    .queue
                    .iter()
                    .enumerate()
                    .map(|(idx, song)| {
                        let title =
                            song.title.as_deref().unwrap_or("Unknown title");
                        let prefix = if Some(idx) == current_index {
                            "▶ "
                        } else {
                            "  "
                        };
                        ListItem::new(Line::from(format!("{prefix}{title}")))
                    })
                    .collect();
                let list = List::new(songs)
                    .highlight_symbol("> ")
                    .highlight_style(
                        Style::new()
                            .add_modifier(Modifier::REVERSED)
                            .fg(theme.text),
                    )
                    .style(Style::new().fg(theme.text).bg(theme.background));
                StatefulWidget::render(list, inner, buf, &mut self.queue_state);
                queue_block.render(split[1], buf);
            }
            SelectedTab::Playlists => {
                let pl_block = main_shell(border_style, theme.background)
                    .title("Playlists (Enter: load)");
                let inner = pl_block.inner(outer_layout[2]);
                let items: Vec<ListItem> = state
                    .playlists
                    .iter()
                    .map(|p| ListItem::new(Line::from(p.name.as_str())))
                    .collect();
                let list = List::new(items)
                    .highlight_symbol("> ")
                    .highlight_style(
                        Style::new()
                            .add_modifier(Modifier::REVERSED)
                            .fg(theme.text),
                    )
                    .style(Style::new().fg(theme.text).bg(theme.background));
                StatefulWidget::render(
                    list,
                    inner,
                    buf,
                    &mut self.playlists_state,
                );
                pl_block.render(outer_layout[2], buf);
            }
            SelectedTab::PlaylistEditor => {
                let title = self
                    .selected_playlist
                    .as_deref()
                    .unwrap_or("(no playlist — load one on tab 2)");
                let ed_block = main_shell(border_style, theme.background)
                    .title(format!("Playlist: {title}"));
                let inner = ed_block.inner(outer_layout[2]);
                if state.editor_tracks.is_empty()
                    && self.selected_playlist.is_some()
                {
                    Line::from("Empty playlist")
                        .style(Style::new().fg(theme.text).italic())
                        .render(inner, buf);
                } else if self.selected_playlist.is_none() {
                    Line::from("Press 2, select a playlist, Enter to load — then open this tab.")
                        .style(Style::new().fg(theme.text).italic())
                        .render(inner, buf);
                } else {
                    let items: Vec<ListItem> = state
                        .editor_tracks
                        .iter()
                        .map(|s| {
                            ListItem::new(Line::from(
                                s.title.as_deref().unwrap_or("Unknown title"),
                            ))
                        })
                        .collect();
                    let list = List::new(items)
                        .highlight_symbol("> ")
                        .highlight_style(
                            Style::new()
                                .add_modifier(Modifier::REVERSED)
                                .fg(theme.text),
                        )
                        .style(
                            Style::new().fg(theme.text).bg(theme.background),
                        );
                    StatefulWidget::render(
                        list,
                        inner,
                        buf,
                        &mut self.editor_state,
                    );
                }
                ed_block.render(outer_layout[2], buf);
            }
        }

        if let Some(total) = duration {
            let gauge_block = Block::new()
                .borders(Borders::ALL)
                .title("Progress")
                .border_type(BorderType::Rounded)
                .border_style(border_style)
                .style(Style::new().bg(theme.background));
            let gauge_inner = gauge_block.inner(outer_layout[3]);
            let ratio = (elapsed as f64 / total as f64).clamp(0.0, 1.0);
            render_horizontal_bar(
                buf,
                gauge_inner,
                ratio,
                Style::default().fg(theme.accent),
                Style::default().fg(theme.highlight),
            );
            gauge_block.render(outer_layout[3], buf);
        } else {
            let gauge_block = Block::new()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(border_style)
                .style(Style::new().bg(theme.background));
            gauge_block.render(outer_layout[3], buf);
        }
    }
}

/// Thin horizontal bar only (no percentage label).
fn render_horizontal_bar(
    buf: &mut Buffer,
    area: Rect,
    ratio: f64,
    filled_style: Style,
    unfilled_style: Style,
) {
    if area.width == 0 || area.height == 0 {
        return;
    }
    let row = area.top() + area.height.saturating_sub(1) / 2;
    let left = area.left();
    let right = area.right();
    let bar_w = u32::from(right - left);
    let filled = ((bar_w as f64) * ratio.clamp(0.0, 1.0)).floor() as u32;
    let filled = filled.min(bar_w);
    let sym = symbols::line::HORIZONTAL;
    for i in 0..filled {
        let col = left + i as u16;
        buf[(col, row)].set_symbol(sym).set_style(filled_style);
    }
    for i in filled..bar_w {
        let col = left + i as u16;
        buf[(col, row)].set_symbol(sym).set_style(unfilled_style);
    }
}

fn mode_indicator_line(status: &mpd::Status, base: Color) -> Line<'static> {
    let on = Style::new().fg(base).bold();
    let off = Style::new().fg(base).dim();
    Line::from(vec![
        Span::styled("R", if status.repeat { on } else { off }),
        Span::styled("Z", if status.random { on } else { off }),
        Span::styled("C", if status.consume { on } else { off }),
    ])
}

/// `file` URI for the playing track: prefer `playlistid` (matches MPD’s `songid`), then `currentsong`, then queue.
fn current_track_path(
    client: &mut Client,
    queue: &[Song],
    playing: Option<&mpd::song::QueuePlace>,
) -> Option<String> {
    let place = playing?;
    if let Ok(Some(s)) = client.playlistid(place.id)
        && !s.file.is_empty()
    {
        return Some(s.file);
    }
    if let Ok(Some(s)) = client.currentsong()
        && !s.file.is_empty()
    {
        return Some(s.file);
    }
    queue
        .iter()
        .find(|s| s.place.as_ref().is_some_and(|p| p.id == place.id))
        .map(|s| s.file.clone())
        .filter(|f| !f.is_empty())
        .or_else(|| {
            queue
                .get(place.pos as usize)
                .map(|s| s.file.clone())
                .filter(|f| !f.is_empty())
        })
}

pub struct ClientState {
    pub client: Client,
    pub queue: Vec<Song>,
    pub playlists: Vec<Playlist>,
    pub editor_tracks: Vec<Song>,
    pub elapsed: u64,
    music_dir_cache: Option<String>,
    cover_cache_key: Option<String>,
    cover_rgba: Option<image::RgbaImage>,
}

impl ClientState {
    fn sync_cover_art(&mut self, song_file: Option<&str>) {
        if self.cover_cache_key.as_deref() == song_file {
            return;
        }
        self.cover_cache_key = song_file.map(str::to_owned);
        self.cover_rgba = song_file.and_then(|f| {
            load_cover_rgba(&mut self.client, &mut self.music_dir_cache, f)
        });
    }
}

fn load_cover_rgba(
    client: &mut Client,
    music_dir: &mut Option<String>,
    song_file: &str,
) -> Option<image::RgbaImage> {
    if music_dir.is_none()
        && let Ok(d) = client.music_directory()
    {
        *music_dir = Some(d);
    }
    if let Some(dir) = music_dir.as_deref() {
        let full = Path::new(dir).join(song_file);
        if let Some(img) = embedded_cover_from_path(&full) {
            return Some(img);
        }
    }

    let path_stub = Song {
        file: song_file.to_string(),
        ..Default::default()
    };
    let bytes = client.albumart(&path_stub).ok()?;
    let img = image::load_from_memory(&bytes).ok()?;
    Some(img.to_rgba8())
}

fn embedded_cover_from_path(path: &Path) -> Option<image::RgbaImage> {
    let tagged = read_from_path(path).ok()?;
    let pic = tagged
        .tags()
        .iter()
        .flat_map(|t| t.pictures())
        .find(|p| p.pic_type() == PictureType::CoverFront)
        .or_else(|| tagged.tags().iter().flat_map(|t| t.pictures()).next())?;
    let img = image::load_from_memory(pic.data()).ok()?;
    Some(img.to_rgba8())
}

/// Two terminal rows per cell using `▀` (upper half block): fg = top pixel, bg = bottom pixel.
fn render_cover_halfblocks(
    buf: &mut Buffer,
    area: Rect,
    src: &image::RgbaImage,
) {
    if area.width == 0 || area.height == 0 {
        return;
    }
    let tw = u32::from(area.width);
    let th = u32::from(area.height).saturating_mul(2);
    let resized = image::imageops::resize(src, tw, th, FilterType::Triangle);
    let sym = "▀";
    for cy in 0..area.height {
        let y0 = u32::from(cy) * 2;
        let y1 = y0 + 1;
        for cx in 0..area.width {
            let x = u32::from(cx);
            let top = rgb_at(&resized, x, y0);
            let bottom = rgb_at(&resized, x, y1);
            let cell_x = area.x + cx;
            let cell_y = area.y + cy;
            buf[(cell_x, cell_y)]
                .set_symbol(sym)
                .set_fg(Color::Rgb(top[0], top[1], top[2]))
                .set_bg(Color::Rgb(bottom[0], bottom[1], bottom[2]));
        }
    }
}

fn rgb_at(img: &image::RgbaImage, x: u32, y: u32) -> [u8; 3] {
    if x < img.width() && y < img.height() {
        let p = img.get_pixel(x, y);
        [p[0], p[1], p[2]]
    } else {
        [0, 0, 0]
    }
}

#[derive(
    Default, Debug, Copy, Clone, PartialEq, Eq, Display, FromRepr, EnumIter,
)]
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
