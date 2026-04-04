use crate::Client;
use crate::error::{Error, Result};
use crate::proto::command::Command;
use crate::proto::response::parse_pair_line;
use tokio::io::{AsyncRead, AsyncWrite};

impl<S> Client<S>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    pub async fn config_value(&mut self, name: &str) -> Result<String> {
        let lines = self.run(Command::new("config").arg(name)).await?;
        parse_config_line(&lines, name)
    }

    pub async fn music_directory(&mut self) -> Result<String> {
        self.config_value("music_directory").await
    }
}

fn parse_config_line(lines: &[String], key: &str) -> Result<String> {
    for line in lines {
        let (k, v) = parse_pair_line(line)?;
        if k == key {
            return Ok(v.to_string());
        }
    }
    Err(Error::Parse(format!("config: missing key {key:?}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_matching_key() {
        let lines = vec!["music_directory: /data/music".to_string()];
        assert_eq!(
            parse_config_line(&lines, "music_directory").unwrap(),
            "/data/music"
        );
    }

    #[test]
    fn errors_when_key_missing() {
        let lines = vec!["other: x".to_string()];
        assert!(parse_config_line(&lines, "music_directory").is_err());
    }
}
