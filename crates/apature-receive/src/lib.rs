//! Receive over Wi-Fi: an FTP server inside Apature. Anything that can speak
//! FTP on the same network (a phone running a transfer app, a camera with an
//! FTP client, a WT-7, another computer) drops files into a folder, and Live
//! Ingest takes it from there: verification, naming, the ledger.
//!
//! One account, a password made up for the session and shown in the window,
//! passive mode on a small fixed port range so a firewall rule is simple.
//! Plain FTP on purpose: cameras and most phone apps do not do FTPS or SFTP,
//! and this only ever runs on a private network the photographer controls.

use std::net::{IpAddr, SocketAddr, UdpSocket};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Instant;

use async_trait::async_trait;
use libunftp::ServerBuilder;
use unftp_core::auth::{AuthenticationError, Authenticator, Credentials, Principal};

pub const DEFAULT_PORT: u16 = 2121;
pub const PASSIVE_PORTS: std::ops::RangeInclusive<u16> = 50100..=50140;
pub const USER: &str = "apature";

#[derive(Debug)]
struct OneUser {
    password: String,
    logins: Arc<AtomicUsize>,
}

#[async_trait]
impl Authenticator for OneUser {
    async fn authenticate(
        &self,
        username: &str,
        creds: &Credentials,
    ) -> Result<Principal, AuthenticationError> {
        let ok = username == USER && creds.password.as_deref() == Some(self.password.as_str());
        if ok {
            self.logins.fetch_add(1, Ordering::Relaxed);
            Ok(Principal {
                username: username.to_string(),
            })
        } else {
            Err(AuthenticationError::BadPassword)
        }
    }
}

/// A running server. Dropping it stops the server.
pub struct Receiver {
    runtime: Option<tokio::runtime::Runtime>,
    pub root: PathBuf,
    pub port: u16,
    pub password: String,
    pub since: Instant,
    logins: Arc<AtomicUsize>,
}

impl Receiver {
    /// Start serving `root` on every interface at `port`. The password is
    /// six digits, new each start, so it can be read out loud or typed on a
    /// camera's screen.
    pub fn start(root: &Path, port: u16) -> Result<Receiver, String> {
        if !root.is_dir() {
            return Err(format!("{} is not a folder", root.display()));
        }
        let password = {
            use rand::Rng;
            format!("{:06}", rand::thread_rng().gen_range(0..1_000_000u32))
        };
        let logins = Arc::new(AtomicUsize::new(0));
        let auth = Arc::new(OneUser {
            password: password.clone(),
            logins: logins.clone(),
        });
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()
            .map_err(|e| e.to_string())?;
        let dir = root.to_path_buf();
        unftp_sbe_fs::Filesystem::new(dir.clone()).map_err(|e| e.to_string())?;
        // Bind synchronously first so a port already in use is reported now,
        // not from a background task.
        std::net::TcpListener::bind(("0.0.0.0", port)).map_err(|e| format!("port {port}: {e}"))?;
        let server = ServerBuilder::new(Box::new(move || {
            unftp_sbe_fs::Filesystem::new(dir.clone()).expect("folder checked at start")
        }))
        .greeting("Apature receive")
        .passive_ports(PASSIVE_PORTS)
        .authenticator(auth)
        .build()
        .map_err(|e| e.to_string())?;
        runtime.spawn(async move {
            if let Err(e) = server.listen(format!("0.0.0.0:{port}")).await {
                log::error!("receive server stopped: {e}");
            }
        });
        log::info!(
            "receive: ftp://{}:{port} into {}",
            local_ip()
                .map(|i| i.to_string())
                .unwrap_or_else(|| "?".into()),
            root.display()
        );
        Ok(Receiver {
            runtime: Some(runtime),
            root: root.to_path_buf(),
            port,
            password,
            since: Instant::now(),
            logins,
        })
    }

    /// Successful logins since start (a phone or camera connecting).
    pub fn logins(&self) -> usize {
        self.logins.load(Ordering::Relaxed)
    }

    /// `ftp://<ip>:<port>` for the window and the log; the IP is the one
    /// this machine uses to reach the network it is on (a hotspot, a venue's
    /// Wi-Fi), or `?` when there is none.
    pub fn url(&self) -> String {
        format!(
            "ftp://{}:{}",
            local_ip()
                .map(|i| i.to_string())
                .unwrap_or_else(|| "?".into()),
            self.port
        )
    }
}

impl Drop for Receiver {
    fn drop(&mut self) {
        if let Some(rt) = self.runtime.take() {
            rt.shutdown_background();
        }
        log::info!("receive: stopped");
    }
}

/// The address other devices on the local network reach this machine at.
/// A UDP socket "connected" to a public address picks the right interface
/// without sending anything.
pub fn local_ip() -> Option<IpAddr> {
    let s = UdpSocket::bind("0.0.0.0:0").ok()?;
    s.connect(SocketAddr::from(([1, 1, 1, 1], 53))).ok()?;
    let ip = s.local_addr().ok()?.ip();
    if ip.is_unspecified() || ip.is_loopback() {
        None
    } else {
        Some(ip)
    }
}
