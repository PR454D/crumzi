use color_eyre::Result;

mod app;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    ratatui::run(|term| app::App::default().run(term))

    // let mut client = Client::connect("localhost:6600".to_string()).await?;
    // let _mpd_client = MpdClient::default();
    //
    // client.status().await?;
    //
    // Ok(())
}
