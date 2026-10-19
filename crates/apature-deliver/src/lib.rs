//! Delivery: push finished JPEGs to a desk over FTP, FTPS (explicit TLS) or
//! SFTP. One `Target` describes where; the password or key passphrase is
//! handed in at send time and never stored here (the app keeps it in the
//! macOS Keychain). Every file is written to a temporary name and renamed
//! when complete, so a desk's watch folder never picks up a half file.

use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::Instant;

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Kind {
    Ftp,
    /// FTP with explicit TLS (AUTH TLS) on the usual port 21.
    Ftps,
    Sftp,
}

/// Where photos go. `dir` is the remote folder, absolute or relative to the
/// login folder; empty means the login folder.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Target {
    pub name: String,
    pub kind: Kind,
    pub host: String,
    pub port: u16,
    pub user: String,
    pub dir: String,
    /// SFTP only: a private key file; empty = password login.
    #[serde(default)]
    pub key_file: Option<PathBuf>,
}

impl Target {
    pub fn default_port(kind: Kind) -> u16 {
        match kind {
            Kind::Ftp | Kind::Ftps => 21,
            Kind::Sftp => 22,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum DeliverError {
    #[error("connect to {0}: {1}")]
    Connect(String, String),
    #[error("login as {0} failed: {1}")]
    Login(String, String),
    #[error("remote folder {0}: {1}")]
    Folder(String, String),
    #[error("{0}: {1}")]
    File(PathBuf, String),
}

#[derive(Clone, Debug)]
pub enum Event {
    Started { index: usize },
    Sent { index: usize, bytes: u64 },
    Failed { index: usize, error: String },
    Finished,
}

#[derive(Clone, Debug, Default)]
pub struct Report {
    pub sent: usize,
    pub failed: Vec<(PathBuf, String)>,
    pub bytes: u64,
    pub elapsed_secs: f64,
}

/// The remote path for a local file: `dir` + the file name, with `/` joins.
pub fn remote_path(dir: &str, local: &Path) -> String {
    let name = local
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let dir = dir.trim().trim_end_matches('/');
    if dir.is_empty() {
        name
    } else {
        format!("{dir}/{name}")
    }
}

trait Remote {
    fn put(&mut self, local: &Path, remote: &str) -> Result<u64, String>;
    fn rename(&mut self, from: &str, to: &str) -> Result<(), String>;
}

struct Ftp(suppaftp::NativeTlsFtpStream);

impl Remote for Ftp {
    fn put(&mut self, local: &Path, remote: &str) -> Result<u64, String> {
        let mut f = std::fs::File::open(local).map_err(|e| e.to_string())?;
        let n = self.0.put_file(remote, &mut f).map_err(|e| e.to_string())?;
        Ok(n)
    }
    fn rename(&mut self, from: &str, to: &str) -> Result<(), String> {
        self.0.rename(from, to).map_err(|e| e.to_string())
    }
}

struct Sftp {
    _session: ssh2::Session,
    sftp: ssh2::Sftp,
}

impl Remote for Sftp {
    fn put(&mut self, local: &Path, remote: &str) -> Result<u64, String> {
        let mut src = std::fs::File::open(local).map_err(|e| e.to_string())?;
        let mut dst = self
            .sftp
            .create(Path::new(remote))
            .map_err(|e| e.to_string())?;
        let mut buf = vec![0u8; 1 << 16];
        let mut total = 0u64;
        loop {
            let n = src.read(&mut buf).map_err(|e| e.to_string())?;
            if n == 0 {
                break;
            }
            dst.write_all(&buf[..n]).map_err(|e| e.to_string())?;
            total += n as u64;
        }
        Ok(total)
    }
    fn rename(&mut self, from: &str, to: &str) -> Result<(), String> {
        self.sftp
            .rename(
                Path::new(from),
                Path::new(to),
                Some(ssh2::RenameFlags::OVERWRITE),
            )
            .map_err(|e| e.to_string())
    }
}

fn connect(target: &Target, secret: &str) -> Result<Box<dyn Remote>, DeliverError> {
    let addr = format!("{}:{}", target.host, target.port);
    match target.kind {
        Kind::Ftp | Kind::Ftps => {
            let mut s = suppaftp::NativeTlsFtpStream::connect(&addr)
                .map_err(|e| DeliverError::Connect(addr.clone(), e.to_string()))?;
            if target.kind == Kind::Ftps {
                let tls = suppaftp::native_tls::TlsConnector::new()
                    .map_err(|e| DeliverError::Connect(addr.clone(), e.to_string()))?;
                s = s
                    .into_secure(suppaftp::NativeTlsConnector::from(tls), &target.host)
                    .map_err(|e| DeliverError::Connect(addr.clone(), e.to_string()))?;
            }
            s.login(&target.user, &secret.to_string())
                .map_err(|e| DeliverError::Login(target.user.clone(), e.to_string()))?;
            let _ = s.transfer_type(suppaftp::types::FileType::Binary);
            if !target.dir.trim().is_empty() {
                // Create the folder if the desk lets us; then it must exist.
                let _ = s.mkdir(target.dir.trim());
                s.cwd(target.dir.trim())
                    .map_err(|e| DeliverError::Folder(target.dir.clone(), e.to_string()))?;
            }
            Ok(Box::new(Ftp(s)))
        }
        Kind::Sftp => {
            let tcp = std::net::TcpStream::connect(&addr)
                .map_err(|e| DeliverError::Connect(addr.clone(), e.to_string()))?;
            let mut sess = ssh2::Session::new()
                .map_err(|e| DeliverError::Connect(addr.clone(), e.to_string()))?;
            sess.set_tcp_stream(tcp);
            sess.handshake()
                .map_err(|e| DeliverError::Connect(addr.clone(), e.to_string()))?;
            match &target.key_file {
                Some(key) => sess
                    .userauth_pubkey_file(
                        &target.user,
                        None,
                        key,
                        Some(secret).filter(|s| !s.is_empty()),
                    )
                    .map_err(|e| DeliverError::Login(target.user.clone(), e.to_string()))?,
                None => sess
                    .userauth_password(&target.user, secret)
                    .map_err(|e| DeliverError::Login(target.user.clone(), e.to_string()))?,
            }
            let sftp = sess
                .sftp()
                .map_err(|e| DeliverError::Connect(addr.clone(), e.to_string()))?;
            if !target.dir.trim().is_empty() {
                let d = Path::new(target.dir.trim());
                if sftp.stat(d).is_err() {
                    sftp.mkdir(d, 0o755)
                        .map_err(|e| DeliverError::Folder(target.dir.clone(), e.to_string()))?;
                }
            }
            Ok(Box::new(Sftp {
                _session: sess,
                sftp,
            }))
        }
    }
}

/// Connect, log in and open the folder; nothing is sent.
pub fn test_connection(target: &Target, secret: &str) -> Result<(), DeliverError> {
    connect(target, secret).map(|_| ())
}

/// Send every file; each lands as `.name.part` and is renamed on completion.
pub fn send(
    target: &Target,
    secret: &str,
    files: &[PathBuf],
    mut on_event: impl FnMut(Event),
) -> Result<Report, DeliverError> {
    let t0 = Instant::now();
    let mut remote = connect(target, secret)?;
    // After cwd on FTP the folder is current, so names are relative there;
    // SFTP has no cwd, so names carry the folder.
    let base = match target.kind {
        Kind::Sftp => target.dir.clone(),
        _ => String::new(),
    };
    let mut report = Report::default();
    for (index, f) in files.iter().enumerate() {
        on_event(Event::Started { index });
        let final_name = remote_path(&base, f);
        let part = {
            let name = f
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default();
            remote_path(&base, Path::new(&format!(".{name}.part")))
        };
        let result = remote
            .put(f, &part)
            .and_then(|n| remote.rename(&part, &final_name).map(|_| n));
        match result {
            Ok(n) => {
                report.sent += 1;
                report.bytes += n;
                log::info!("delivered {} → {}:{}", f.display(), target.host, final_name);
                on_event(Event::Sent { index, bytes: n });
            }
            Err(e) => {
                log::warn!("delivery failed {} → {}: {e}", f.display(), target.host);
                report.failed.push((f.clone(), e.clone()));
                on_event(Event::Failed { index, error: e });
            }
        }
    }
    report.elapsed_secs = t0.elapsed().as_secs_f64();
    on_event(Event::Finished);
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn remote_paths_join_with_slashes() {
        assert_eq!(remote_path("", Path::new("/a/b/X.JPG")), "X.JPG");
        assert_eq!(
            remote_path("/incoming/", Path::new("/a/X.JPG")),
            "/incoming/X.JPG"
        );
        assert_eq!(remote_path("photos", Path::new("X.JPG")), "photos/X.JPG");
    }

    /// Runs only with a local FTP server: `APATURE_FTP_TEST=host:port:user:pass:dir`.
    #[test]
    fn ftp_round_trip_when_a_server_is_given() {
        let Ok(spec) = std::env::var("APATURE_FTP_TEST") else {
            return;
        };
        let p: Vec<&str> = spec.splitn(5, ':').collect();
        let target = Target {
            name: "test".into(),
            kind: Kind::Ftp,
            host: p[0].into(),
            port: p[1].parse().unwrap(),
            user: p[2].into(),
            dir: p[4].into(),
            key_file: None,
        };
        let dir = tempfile::tempdir().unwrap();
        let f = dir.path().join("TEST-001.JPG");
        std::fs::write(&f, vec![7u8; 100_000]).unwrap();
        let mut events = 0;
        let rep = send(&target, p[3], &[f], |_| events += 1).unwrap();
        assert_eq!(rep.sent, 1);
        assert_eq!(rep.bytes, 100_000);
        assert!(events >= 3);
    }
}
