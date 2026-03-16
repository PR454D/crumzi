//! Async MPD client (Tokio).
//!
//! This crate provides an async, MPD-protocol client with an API inspired by the
//! [`mpd`](https://docs.rs/mpd/latest/mpd/) crate, but built on Tokio IO.
//!
//! ## Example
//!
//! ```no_run
//! use crumzi_client::Client;
//!
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let mut client = Client::connect("127.0.0.1:6600").await?;
//! client.clear().await?;
//! client.add("music/foo.mp3").await?;
//! let queue = client.playlistinfo().await?;
//! println!("queue size={}", queue.len());
//! # Ok(())
//! # }
//! ```

use tokio::{
    io::{AsyncBufReadExt, AsyncRead, AsyncWrite, BufStream},
    net::TcpStream,
};

use error::{ProtoError, Result};

mod error;
mod proto;
mod queue;
mod status;
mod types;

pub use types::Song;

pub struct Client<S = TcpStream>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    socket: BufStream<S>,
    server_version: String,
}

impl Client<TcpStream> {
    pub async fn connect(addr: impl ToString) -> Result<Client<TcpStream>> {
        let socket = TcpStream::connect(addr.to_string()).await?;
        Client::new(socket).await
    }
}

impl<S> Client<S>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    pub async fn new(socket: S) -> Result<Client<S>> {
        let mut socket = BufStream::new(socket);
        let mut banner = String::new();
        socket.read_line(&mut banner).await?;

        let banner = banner.trim_end_matches(['\r', '\n']);
        let version = banner
            .strip_prefix("OK MPD ")
            .ok_or_else(|| ProtoError::BadBanner(banner.to_string()))?;
        if version.is_empty() {
            return Err(ProtoError::BadBanner(banner.to_string()).into());
        }

        Ok(Client {
            socket,
            server_version: version.to_string(),
        })
    }

    pub fn server_version(&self) -> &str {
        &self.server_version
    }

    pub(crate) async fn run(&mut self, cmd: proto::command::Command) -> Result<Vec<String>> {
        proto::send(&mut self.socket, &cmd).await?;
        proto::read_response_lines(&mut self.socket).await
    }
}
