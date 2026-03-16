use crate::error::{Error, Result};
use crate::proto::response::parse_pair_line;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum State {
    Play,
    Pause,
    Stop,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub struct Status {
    pub volume: Option<i8>,
    pub repeat: bool,
    pub random: bool,
    pub single: bool,
    pub consume: bool,
    pub state: Option<State>,
    pub song: Option<u32>,
    pub songid: Option<u32>,
    pub elapsed: Option<u32>,
    pub duration: Option<u32>,
}

pub fn parse_status(lines: &[String]) -> Result<Status> {
    let mut st = Status::default();
    for line in lines {
        let (k, v) = parse_pair_line(line)?;
        match k {
            "volume" => st.volume = Some(parse_i8(v, "volume")?),
            "repeat" => st.repeat = parse_bool01(v, "repeat")?,
            "random" => st.random = parse_bool01(v, "random")?,
            "single" => st.single = parse_bool01(v, "single")?,
            "consume" => st.consume = parse_bool01(v, "consume")?,
            "state" => {
                st.state = Some(match v {
                    "play" => State::Play,
                    "pause" => State::Pause,
                    "stop" => State::Stop,
                    _ => return Err(Error::Parse(format!("bad state: {v:?}"))),
                })
            }
            "song" => st.song = Some(parse_u32(v, "song")?),
            "songid" => st.songid = Some(parse_u32(v, "songid")?),
            "elapsed" => {
                // elapsed can be float in some MPD versions; parse leniently
                st.elapsed = Some(parse_u32_lenient(v, "elapsed")?)
            }
            "duration" => st.duration = Some(parse_u32_lenient(v, "duration")?),
            _ => {}
        }
    }
    Ok(st)
}

fn parse_bool01(v: &str, field: &'static str) -> Result<bool> {
    match v {
        "0" => Ok(false),
        "1" => Ok(true),
        _ => Err(Error::Parse(format!("bad {field}: {v:?}"))),
    }
}

fn parse_u32(v: &str, field: &'static str) -> Result<u32> {
    v.parse()
        .map_err(|_| Error::Parse(format!("bad {field}: {v:?}")))
}

fn parse_u32_lenient(v: &str, field: &'static str) -> Result<u32> {
    if let Ok(n) = v.parse::<u32>() {
        return Ok(n);
    }
    if let Ok(f) = v.parse::<f64>()
        && f.is_finite()
        && f >= 0.0
    {
        return Ok(f.floor() as u32);
    };
    Err(Error::Parse(format!("bad {field}: {v:?}")))
}

fn parse_i8(v: &str, field: &'static str) -> Result<i8> {
    v.parse()
        .map_err(|_| Error::Parse(format!("bad {field}: {v:?}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_status_minimal() {
        let lines = vec![
            "volume: 50".to_string(),
            "repeat: 1".to_string(),
            "random: 0".to_string(),
            "single: 0".to_string(),
            "consume: 1".to_string(),
            "state: play".to_string(),
            "song: 3".to_string(),
            "songid: 9".to_string(),
            "elapsed: 12.5".to_string(),
            "duration: 120".to_string(),
        ];
        let st = parse_status(&lines).unwrap();
        assert_eq!(st.volume, Some(50));
        assert!(st.repeat);
        assert!(!st.random);
        assert!(!st.single);
        assert!(st.consume);
        assert_eq!(st.state, Some(State::Play));
        assert_eq!(st.song, Some(3));
        assert_eq!(st.songid, Some(9));
        assert_eq!(st.elapsed, Some(12));
        assert_eq!(st.duration, Some(120));
    }
}
