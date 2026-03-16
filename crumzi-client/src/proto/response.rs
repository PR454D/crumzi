use crate::error::{Error, ProtoError, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AckLine {
    pub code: u32,
    pub command_idx: u32,
    pub command: String,
    pub message: String,
}

pub fn parse_ack_line(line: &str) -> Result<Option<AckLine>> {
    // Format: ACK [<code>@<command_idx>] {<command>} <message>
    if !line.starts_with("ACK ") {
        return Ok(None);
    }

    let rest = &line["ACK ".len()..];
    let rest = rest
        .strip_prefix('[')
        .ok_or_else(|| Error::Parse(format!("bad ACK: {line:?}")))?;
    let (code_idx, rest) = rest
        .split_once(']')
        .ok_or_else(|| Error::Parse(format!("bad ACK: {line:?}")))?;
    let (code, idx) = code_idx
        .split_once('@')
        .ok_or_else(|| Error::Parse(format!("bad ACK code/index: {line:?}")))?;
    let code: u32 = code
        .parse()
        .map_err(|_| Error::Parse(format!("bad ACK code: {line:?}")))?;
    let command_idx: u32 = idx
        .parse()
        .map_err(|_| Error::Parse(format!("bad ACK index: {line:?}")))?;

    let rest = rest.trim_start();
    let rest = rest
        .strip_prefix('{')
        .ok_or_else(|| Error::Parse(format!("bad ACK: {line:?}")))?;
    let (command, rest) = rest
        .split_once('}')
        .ok_or_else(|| Error::Parse(format!("bad ACK: {line:?}")))?;

    let message = rest.trim_start().to_string();
    Ok(Some(AckLine {
        code,
        command_idx,
        command: command.to_string(),
        message,
    }))
}

pub fn parse_pair_line(line: &str) -> Result<(&str, &str)> {
    let (k, v) = line
        .split_once(": ")
        .ok_or_else(|| ProtoError::NotPair(line.to_string()))?;
    Ok((k, v))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_ack() {
        let ack = parse_ack_line("ACK [50@0] {play} No current song").unwrap();
        let ack = ack.unwrap();
        assert_eq!(ack.code, 50);
        assert_eq!(ack.command_idx, 0);
        assert_eq!(ack.command, "play");
        assert_eq!(ack.message, "No current song");
    }

    #[test]
    fn non_ack_is_none() {
        assert_eq!(parse_ack_line("foo").unwrap(), None);
    }

    #[test]
    fn parses_pair_line() {
        assert_eq!(parse_pair_line("file: x.mp3").unwrap(), ("file", "x.mp3"));
    }
}
