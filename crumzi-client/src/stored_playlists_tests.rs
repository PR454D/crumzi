#[cfg(test)]
mod tests {
    use crate::proto::testdata;
    use crate::{Client, Playlist};
    use tokio::io::AsyncWriteExt;

    #[tokio::test]
    async fn listplaylists_parsed() {
        let (mut server, client_io) = tokio::io::duplex(2048);
        server.write_all(b"OK MPD 0.23.5\n").await.unwrap();
        server
            .write_all(testdata::LISTPLAYLISTS_RESPONSE.as_bytes())
            .await
            .unwrap();
        server.shutdown().await.unwrap();

        let mut client = Client::new(client_io).await.unwrap();
        let pl = client.listplaylists().await.unwrap();
        assert_eq!(
            pl,
            vec![
                Playlist {
                    name: "favorites".into()
                },
                Playlist {
                    name: "work mix".into()
                },
            ]
        );
    }

    #[tokio::test]
    async fn listplaylist_parses_songs_without_pos_id() {
        let (mut server, client_io) = tokio::io::duplex(2048);
        server.write_all(b"OK MPD 0.23.5\n").await.unwrap();
        server
            .write_all(testdata::LISTPLAYLIST_RESPONSE.as_bytes())
            .await
            .unwrap();
        server.shutdown().await.unwrap();

        let mut client = Client::new(client_io).await.unwrap();
        let songs = client.listplaylist("mylist").await.unwrap();
        assert_eq!(songs.len(), 2);
        assert_eq!(songs[0].file, "dir/track1.flac");
        assert_eq!(songs[0].title.as_deref(), Some("One"));
        assert_eq!(songs[0].pos, None);
        assert_eq!(songs[0].id, None);
        assert_eq!(songs[1].file, "dir/track2.flac");
        assert_eq!(songs[1].artist.as_deref(), Some("Two"));
    }

    #[tokio::test]
    async fn load_succeeds_on_ok() {
        let (mut server, client_io) = tokio::io::duplex(2048);
        server.write_all(b"OK MPD 0.23.5\n").await.unwrap();
        server
            .write_all(testdata::LOAD_OK_RESPONSE.as_bytes())
            .await
            .unwrap();
        server.shutdown().await.unwrap();

        let mut client = Client::new(client_io).await.unwrap();
        client.load("favorites", None).await.unwrap();
    }
}
