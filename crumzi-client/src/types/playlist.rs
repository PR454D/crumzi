use crate::error::Result;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Playlist {
    pub name: String,
}

pub fn parse_listplaylists(lines: &[String]) -> Result<Vec<Playlist>> {
    let mut out = Vec::new();
    for line in lines {
        if let Some(name) = line.strip_prefix("playlist: ") {
            out.push(Playlist {
                name: name.to_string(),
            });
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_playlist_lines() {
        let lines = vec![
            "playlist: alpha".to_string(),
            "playlist: beta gamma".to_string(),
        ];
        let pl = parse_listplaylists(&lines).unwrap();
        assert_eq!(pl.len(), 2);
        assert_eq!(pl[0].name, "alpha");
        assert_eq!(pl[1].name, "beta gamma");
    }
}
