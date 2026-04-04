use crate::error::Result;
use crate::proto::command::Command;
use crate::types::playlist::parse_listplaylists;
use crate::types::song::parse_song_list;
use crate::{Client, Playlist, Song};
use tokio::io::{AsyncRead, AsyncWrite};

impl<S> Client<S>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    pub async fn listplaylists(&mut self) -> Result<Vec<Playlist>> {
        let lines = self.run(Command::new("listplaylists")).await?;
        parse_listplaylists(&lines)
    }

    pub async fn listplaylist(&mut self, name: &str) -> Result<Vec<Song>> {
        let lines = self.run(Command::new("listplaylist").arg(name)).await?;
        parse_song_list(&lines)
    }

    /// Loads a stored playlist into the queue. `range` is MPD’s `start:end` form when set.
    pub async fn load(
        &mut self,
        name: &str,
        range: Option<&str>,
    ) -> Result<()> {
        let mut cmd = Command::new("load").arg(name);
        if let Some(r) = range {
            cmd = cmd.arg(r);
        }
        self.run(cmd).await?;
        Ok(())
    }
}
