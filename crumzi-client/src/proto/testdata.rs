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

pub const LISTPLAYLISTS_RESPONSE: &str = r#"playlist: favorites
playlist: work mix
OK
"#;

/// Stored-playlist listing: same song keys as queue; Pos/Id often omitted.
pub const LISTPLAYLIST_RESPONSE: &str = r#"file: dir/track1.flac
Title: One
file: dir/track2.flac
Artist: Two
OK
"#;

pub const PLAYLISTID_RESPONSE: &str = r#"file: queue_item.ogg
Title: Current slot
Pos: 2
Id: 7
OK
"#;

pub const CONFIG_MUSIC_DIRECTORY_RESPONSE: &str = r#"music_directory: /var/lib/mpd/music
OK
"#;

pub const LOAD_OK_RESPONSE: &str = "OK\n";
