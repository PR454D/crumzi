use color_eyre::Result;

mod app;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    ratatui::run(|term| app::App::default().run(term))
}
