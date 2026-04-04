#[cfg(test)]
mod tests {
    use crate::Client;
    use crate::proto::testdata;
    use tokio::io::AsyncWriteExt;

    #[tokio::test]
    async fn playlistid_parses_single_song() {
        let (mut server, client_io) = tokio::io::duplex(2048);
        server.write_all(b"OK MPD 0.23.5\n").await.unwrap();
        server
            .write_all(testdata::PLAYLISTID_RESPONSE.as_bytes())
            .await
            .unwrap();
        server.shutdown().await.unwrap();

        let mut client = Client::new(client_io).await.unwrap();
        let song = client.playlistid(7).await.unwrap();
        assert_eq!(song.file, "queue_item.ogg");
        assert_eq!(song.title.as_deref(), Some("Current slot"));
        assert_eq!(song.pos, Some(2));
        assert_eq!(song.id, Some(7));
    }
}
