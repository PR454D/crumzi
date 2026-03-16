#[cfg(test)]
mod tests {
    use crate::proto::testdata;
    use crate::{Client, State};
    use tokio::io::AsyncWriteExt;

    #[tokio::test]
    async fn parses_status_via_client() {
        let (mut server, client_io) = tokio::io::duplex(2048);
        server.write_all(b"OK MPD 0.23.5\n").await.unwrap();
        server
            .write_all(testdata::STATUS_RESPONSE.as_bytes())
            .await
            .unwrap();
        server.shutdown().await.unwrap();

        let mut client = Client::new(client_io).await.unwrap();
        let st = client.status().await.unwrap();
        assert_eq!(st.volume, Some(50));
        assert_eq!(st.state, Some(State::Play));
        assert_eq!(st.elapsed, Some(12));
        assert_eq!(st.duration, Some(120));
    }

    #[tokio::test]
    async fn parses_currentsong_via_client() {
        let (mut server, client_io) = tokio::io::duplex(2048);
        server.write_all(b"OK MPD 0.23.5\n").await.unwrap();
        server
            .write_all(testdata::CURRENTSONG_RESPONSE.as_bytes())
            .await
            .unwrap();
        server.shutdown().await.unwrap();

        let mut client = Client::new(client_io).await.unwrap();
        let song = client.currentsong().await.unwrap().unwrap();
        assert_eq!(song.file, "a.mp3");
        assert_eq!(song.title.as_deref(), Some("A"));
        assert_eq!(song.artist.as_deref(), Some("AA"));
        assert_eq!(song.duration, Some(200));
    }
}
