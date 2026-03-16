use crate::error::Result;
use crate::proto::command::Command;
use crate::types::song::parse_song_list;
use crate::{Client, Song};
use tokio::io::{AsyncRead, AsyncWrite};

impl<S> Client<S>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    pub async fn add(&mut self, uri: &str) -> Result<()> {
        self.run(Command::new("add").arg(uri)).await?;
        Ok(())
    }

    pub async fn clear(&mut self) -> Result<()> {
        self.run(Command::new("clear")).await?;
        Ok(())
    }

    pub async fn delete(&mut self, pos: u32) -> Result<()> {
        self.run(Command::new("delete").arg(pos.to_string()))
            .await?;
        Ok(())
    }

    pub async fn move_song(&mut self, from: u32, to: u32) -> Result<()> {
        self.run(
            Command::new("move")
                .arg(from.to_string())
                .arg(to.to_string()),
        )
        .await?;
        Ok(())
    }

    pub async fn playlistinfo(&mut self) -> Result<Vec<Song>> {
        let lines = self.run(Command::new("playlistinfo")).await?;
        parse_song_list(&lines)
    }

    pub async fn playlistinfo_range(&mut self, start: u32, end: u32) -> Result<Vec<Song>> {
        let range = format!("{start}:{end}");
        let lines = self.run(Command::new("playlistinfo").arg(range)).await?;
        parse_song_list(&lines)
    }

    pub async fn add_id(&mut self, uri: &str) -> Result<u32> {
        let lines = self.run(Command::new("addid").arg(uri)).await?;
        // response: Id: <number>
        for line in lines {
            if let Some(v) = line.strip_prefix("Id: ") {
                return v
                    .parse()
                    .map_err(|_| crate::error::Error::Parse(format!("bad Id: {v:?}")));
            }
        }
        Err(crate::error::ProtoError::MissingField("Id").into())
    }
}
