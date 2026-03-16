use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncWrite, AsyncWriteExt, BufStream};

use crate::error::{AckError, ProtoError, Result};

pub mod command;
pub mod response;

pub async fn send<S>(socket: &mut BufStream<S>, cmd: &command::Command) -> Result<()>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    let line = cmd.to_line();
    socket.write_all(line.as_bytes()).await?;
    socket.write_all(b"\n").await?;
    socket.flush().await?;
    Ok(())
}

pub async fn read_response_lines<S>(socket: &mut BufStream<S>) -> Result<Vec<String>>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    let mut out = Vec::new();
    loop {
        let mut line = String::new();
        let n = socket.read_line(&mut line).await?;
        if n == 0 {
            return Err(ProtoError::UnexpectedEof.into());
        }

        let line = line.trim_end_matches(['\r', '\n']).to_string();
        if line == "OK" {
            return Ok(out);
        }

        if let Some(ack) = response::parse_ack_line(&line)? {
            return Err(AckError {
                code: ack.code,
                command_idx: ack.command_idx,
                command: ack.command,
                message: ack.message,
            }
            .into());
        }

        out.push(line);
    }
}

