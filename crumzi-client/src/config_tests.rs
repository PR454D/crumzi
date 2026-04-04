#[cfg(test)]
mod tests {
    use crate::Client;
    use crate::proto::testdata;
    use tokio::io::AsyncWriteExt;

    #[tokio::test]
    async fn music_directory_from_config() {
        let (mut server, client_io) = tokio::io::duplex(2048);
        server.write_all(b"OK MPD 0.23.5\n").await.unwrap();
        server
            .write_all(testdata::CONFIG_MUSIC_DIRECTORY_RESPONSE.as_bytes())
            .await
            .unwrap();
        server.shutdown().await.unwrap();

        let mut client = Client::new(client_io).await.unwrap();
        let dir = client.music_directory().await.unwrap();
        assert_eq!(dir, "/var/lib/mpd/music");
    }
}
