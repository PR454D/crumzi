#[allow(dead_code)]
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct Status {
    pub volume: i8,
    pub repeat: bool,
    pub consume: bool,
    pub single: bool,
    pub random: bool,
}
