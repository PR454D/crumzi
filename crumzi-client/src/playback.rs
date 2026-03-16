use crate::error::Result;
use crate::proto::command::Command;
use crate::types::status::parse_status;
use crate::{Client, Song, Status};
use tokio::io::{AsyncRead, AsyncWrite};

impl<S> Client<S>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    pub async fn status(&mut self) -> Result<Status> {
        let lines = self.run(Command::new("status")).await?;
        parse_status(&lines)
    }

    pub async fn currentsong(&mut self) -> Result<Option<Song>> {
        let lines = self.run(Command::new("currentsong")).await?;
        if lines.is_empty() {
            return Ok(None);
        }

        let mut file: Option<String> = None;
        for line in &lines {
            if let Some(v) = line.strip_prefix("file: ") {
                file = Some(v.to_string());
                break;
            }
        }

        let Some(file) = file else {
            return Ok(None);
        };

        let mut normalized = Vec::with_capacity(lines.len());
        normalized.push(format!("file: {file}"));
        for line in lines {
            if !line.starts_with("file: ") {
                normalized.push(line);
            }
        }

        let mut songs = crate::types::song::parse_song_list(&normalized)?;
        Ok(songs.pop())
    }

    pub async fn play(&mut self) -> Result<()> {
        self.run(Command::new("play")).await?;
        Ok(())
    }

    pub async fn play_pos(&mut self, pos: u32) -> Result<()> {
        self.run(Command::new("play").arg(pos.to_string())).await?;
        Ok(())
    }

    pub async fn pause(&mut self, pause: bool) -> Result<()> {
        let v = if pause { "1" } else { "0" };
        self.run(Command::new("pause").arg(v)).await?;
        Ok(())
    }

    pub async fn stop(&mut self) -> Result<()> {
        self.run(Command::new("stop")).await?;
        Ok(())
    }

    pub async fn next(&mut self) -> Result<()> {
        self.run(Command::new("next")).await?;
        Ok(())
    }

    pub async fn prev(&mut self) -> Result<()> {
        self.run(Command::new("previous")).await?;
        Ok(())
    }

    pub async fn seek(&mut self, pos: u32, seconds: u32) -> Result<()> {
        self.run(
            Command::new("seek")
                .arg(pos.to_string())
                .arg(seconds.to_string()),
        )
        .await?;
        Ok(())
    }

    pub async fn setvol(&mut self, volume: i8) -> Result<()> {
        self.run(Command::new("setvol").arg(volume.to_string()))
            .await?;
        Ok(())
    }

    pub async fn repeat(&mut self, enabled: bool) -> Result<()> {
        self.run(Command::new("repeat").arg(bool01(enabled)))
            .await?;
        Ok(())
    }

    pub async fn random(&mut self, enabled: bool) -> Result<()> {
        self.run(Command::new("random").arg(bool01(enabled)))
            .await?;
        Ok(())
    }

    pub async fn single(&mut self, enabled: bool) -> Result<()> {
        self.run(Command::new("single").arg(bool01(enabled)))
            .await?;
        Ok(())
    }

    pub async fn consume(&mut self, enabled: bool) -> Result<()> {
        self.run(Command::new("consume").arg(bool01(enabled)))
            .await?;
        Ok(())
    }
}

fn bool01(v: bool) -> &'static str {
    if v { "1" } else { "0" }
}
