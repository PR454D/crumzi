use std::io::Error;

use tokio::net::TcpSocket;

pub struct Client {
    pub socket: TcpSocket,
}

impl Default for Client {
    fn default() -> Self {
        let addr = "127.0.0.1:6600".parse().unwrap();
        let socket = TcpSocket::new_v4().unwrap();
        let _ = socket.bind(addr);
        Self { socket }
    }
}

impl Client {
    pub async fn new(addr: String) -> Result<Self, Error> {
        let addr = addr.parse().unwrap();
        let socket = TcpSocket::new_v4()?;
        socket.bind(addr)?;
        Ok(Self { socket })
    }
}
