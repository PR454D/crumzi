pub const PLAYLISTINFO_RESPONSE: &str = r#"file: a.mp3
Title: A
Pos: 0
Id: 42
file: b.mp3
Artist: B
Time: 120
OK
"#;

pub const STATUS_RESPONSE: &str = r#"volume: 50
repeat: 1
random: 0
single: 0
consume: 1
state: play
song: 3
songid: 9
elapsed: 12.5
duration: 120
OK
"#;

pub const CURRENTSONG_RESPONSE: &str = r#"file: a.mp3
Title: A
Artist: AA
Album: AB
Time: 200
Pos: 0
Id: 42
OK
"#;
