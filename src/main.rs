extern crate mpd;

use color_eyre::Result;

mod app;

fn main() -> Result<()> {
    // let mut conn = Client::connect("localhost:6600").unwrap();
    // conn.volume(40).unwrap();
    // conn.load("pixeltee", ..).unwrap();
    // conn.play().unwrap();
    // println!("Status: {:?}", conn.status());

    color_eyre::install()?;
    ratatui::run(|term| app::App::default().run(term))
}
