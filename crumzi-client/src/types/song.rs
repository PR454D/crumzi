use crate::error::{Error, ProtoError, Result};
use crate::proto::response::parse_pair_line;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Song {
    pub file: String,
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub duration: Option<u32>,
    pub pos: Option<u32>,
    pub id: Option<u32>,
}

impl Song {
    fn new(file: String) -> Self {
        Self {
            file,
            title: None,
            artist: None,
            album: None,
            duration: None,
            pos: None,
            id: None,
        }
    }
}

pub fn parse_song_list(lines: &[String]) -> Result<Vec<Song>> {
    let mut out: Vec<Song> = Vec::new();
    let mut current: Option<Song> = None;

    for line in lines {
        let (k, v) = parse_pair_line(line)?;
        match k {
            "file" => {
                if let Some(s) = current.take() {
                    out.push(s);
                }
                current = Some(Song::new(v.to_string()));
            }
            _ => {
                let s = current.as_mut().ok_or(ProtoError::MissingField("file"))?;
                match k {
                    "Title" => s.title = Some(v.to_string()),
                    "Artist" => s.artist = Some(v.to_string()),
                    "Album" => s.album = Some(v.to_string()),
                    "Time" => {
                        s.duration = Some(
                            v.parse()
                                .map_err(|_| Error::Parse(format!("bad Time: {v:?}")))?,
                        )
                    }
                    "Pos" => {
                        s.pos = Some(
                            v.parse()
                                .map_err(|_| Error::Parse(format!("bad Pos: {v:?}")))?,
                        )
                    }
                    "Id" => {
                        s.id = Some(
                            v.parse()
                                .map_err(|_| Error::Parse(format!("bad Id: {v:?}")))?,
                        )
                    }
                    _ => {}
                }
            }
        }
    }

    if let Some(s) = current.take() {
        out.push(s);
    }

    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_songs_from_playlistinfo_style_lines() {
        let lines = vec![
            "file: a.mp3".to_string(),
            "Title: A".to_string(),
            "Pos: 0".to_string(),
            "Id: 42".to_string(),
            "file: b.mp3".to_string(),
            "Artist: B".to_string(),
            "Time: 120".to_string(),
        ];
        let songs = parse_song_list(&lines).unwrap();
        assert_eq!(songs.len(), 2);
        assert_eq!(songs[0].file, "a.mp3");
        assert_eq!(songs[0].title.as_deref(), Some("A"));
        assert_eq!(songs[0].pos, Some(0));
        assert_eq!(songs[0].id, Some(42));
        assert_eq!(songs[1].file, "b.mp3");
        assert_eq!(songs[1].artist.as_deref(), Some("B"));
        assert_eq!(songs[1].duration, Some(120));
    }
}
