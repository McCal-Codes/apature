//! A camera over Wi-Fi, no transmitter accessory and no vendor utility.
//!
//! Three pieces, each small:
//! - **Discovery**: the camera advertises `_ptp._tcp` over Bonjour; the TXT
//!   record says whether it is in its pairing wizard (`apps=PAIR`) or in
//!   transfer mode.
//! - **Identity and pairing**: PTP/IP sessions start with a 16-byte host
//!   identity. A camera in its wizard offers a code, shows it on its screen,
//!   and only a host whose person types the matching code is confirmed.
//!   The identity is then kept in the settings folder for later sessions.
//! - **Receiving**: in transfer mode the camera announces each frame with
//!   the standard ObjectAdded event; the host fetches it into a folder,
//!   where Live Ingest takes over.
//!
//! Tested on a Nikon Z 6 II, firmware 1.62, 2026-10-23. Nothing on the
//! camera is deleted or changed.

use std::io::{Read, Write};
use std::net::{IpAddr, Ipv4Addr, TcpStream};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

pub const PORT: u16 = 15740;

/// What the camera advertised.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Found {
    pub ip: IpAddr,
    pub name: String,
    /// `PAIR` in the connection wizard, otherwise a transfer mode such as `WT3T`.
    pub mode: String,
}

/// Run a `dns-sd` command that never exits on its own, for `wait`, and
/// return what it printed.
fn dns_sd(args: &[&str], wait: Duration) -> String {
    use std::process::{Command, Stdio};
    let Ok(mut child) = Command::new("dns-sd")
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
    else {
        return String::new();
    };
    let mut out = child.stdout.take();
    let (tx, rx) = std::sync::mpsc::channel::<String>();
    std::thread::spawn(move || {
        let mut buf = String::new();
        if let Some(o) = out.as_mut() {
            let _ = o.read_to_string(&mut buf);
        }
        let _ = tx.send(buf);
    });
    std::thread::sleep(wait);
    let _ = child.kill();
    let _ = child.wait();
    rx.recv_timeout(Duration::from_secs(2)).unwrap_or_default()
}

/// Look for a camera for about `wait`. Returns the first one that resolves.
pub fn discover(wait: Duration) -> Option<Found> {
    let browse = dns_sd(&["-B", "_ptp._tcp", "local"], wait);
    let name = browse
        .lines()
        .filter(|l| l.contains("_ptp._tcp.") && l.contains(" Add "))
        .filter_map(|l| l.split_whitespace().last())
        .next()?
        .to_string();
    let look = dns_sd(&["-L", &name, "_ptp._tcp", "local"], Duration::from_secs(3));
    let host = look
        .lines()
        .find_map(|l| l.split("can be reached at ").nth(1))
        .and_then(|rest| rest.split(':').next())
        .map(|h| h.trim().to_string())?;
    let mode = look
        .lines()
        .find_map(|l| l.split("apps=").nth(1))
        .map(|r| r.split_whitespace().next().unwrap_or("").to_string())
        .unwrap_or_else(|| "?".into());
    let ip = ip_from_hostname(&host).or_else(|| {
        use std::net::ToSocketAddrs;
        format!("{}:{PORT}", host.trim_end_matches('.'))
            .to_socket_addrs()
            .ok()?
            .find(|a| a.is_ipv4())
            .map(|a| a.ip())
    })?;
    Some(Found { ip, name, mode })
}

fn ip_from_hostname(host: &str) -> Option<IpAddr> {
    let stem = host.split('.').next()?;
    let parts: Vec<&str> = stem.split('-').collect();
    if parts.len() < 5 {
        return None;
    }
    let octets: Vec<u8> = parts[parts.len() - 4..]
        .iter()
        .filter_map(|p| p.parse::<u16>().ok())
        .filter(|v| *v < 256)
        .map(|v| v as u8)
        .collect();
    (octets.len() == 4)
        .then(|| IpAddr::V4(Ipv4Addr::new(octets[0], octets[1], octets[2], octets[3])))
}

/* -- identity --------------------------------------------------------------- */

/// The 16-byte host identity the camera was paired with.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Identity(pub [u8; 16]);

impl Identity {
    pub fn fresh() -> Identity {
        use rand::RngCore;
        let mut b = [0u8; 16];
        rand::thread_rng().fill_bytes(&mut b);
        Identity(b)
    }

    /// Read a saved identity from `path`, else the vendor utility's
    /// system-wide one if that utility paired this machine before.
    pub fn load(path: &Path) -> Option<(Identity, String)> {
        for (p, label) in [
            (path.to_path_buf(), "Apature"),
            (PathBuf::from("/Library/ptpitcp.guid"), "Nikon's utility"),
        ] {
            if let Ok(b) = std::fs::read(&p) {
                if b.len() >= 16 {
                    let mut id = [0u8; 16];
                    id.copy_from_slice(&b[..16]);
                    return Some((Identity(id), label.to_string()));
                }
            }
        }
        None
    }

    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        if let Some(d) = path.parent() {
            std::fs::create_dir_all(d)?;
        }
        std::fs::write(path, self.0)
    }
}

/* -- PTP/IP ----------------------------------------------------------------- */

const OP_GET_DEVICE_INFO: u16 = 0x1001;
const OP_OPEN_SESSION: u16 = 0x1002;
const OP_CLOSE_SESSION: u16 = 0x1003;
const OP_GET_OBJECT_INFO: u16 = 0x1008;
const OP_GET_PARTIAL_OBJECT: u16 = 0x101b;
const RC_OK: u16 = 0x2001;

fn pkt(t: u32, payload: &[u8]) -> Vec<u8> {
    let mut v = Vec::with_capacity(8 + payload.len());
    v.extend_from_slice(&((8 + payload.len()) as u32).to_le_bytes());
    v.extend_from_slice(&t.to_le_bytes());
    v.extend_from_slice(payload);
    v
}

fn read_pkt(s: &mut TcpStream) -> std::io::Result<(u32, Vec<u8>)> {
    let mut hdr = [0u8; 8];
    s.read_exact(&mut hdr)?;
    let len = u32::from_le_bytes(hdr[..4].try_into().unwrap()) as usize;
    let t = u32::from_le_bytes(hdr[4..].try_into().unwrap());
    let mut body = vec![0u8; len.saturating_sub(8)];
    s.read_exact(&mut body)?;
    Ok((t, body))
}

fn utf16z(s: &str) -> Vec<u8> {
    let mut v: Vec<u8> = s.encode_utf16().flat_map(|u| u.to_le_bytes()).collect();
    v.extend_from_slice(&[0, 0]);
    v
}

fn ptp_string(b: &[u8], off: usize) -> (String, usize) {
    let n = b.get(off).copied().unwrap_or(0) as usize;
    let end = (off + 1 + n * 2).min(b.len());
    let units: Vec<u16> = b[off + 1..end]
        .chunks_exact(2)
        .map(|c| u16::from_le_bytes([c[0], c[1]]))
        .collect();
    (
        String::from_utf16_lossy(&units)
            .trim_end_matches('\0')
            .to_string(),
        off + 1 + n * 2,
    )
}

#[derive(Clone, Debug, Default)]
pub struct DeviceInfo {
    pub manufacturer: String,
    pub model: String,
    pub firmware: String,
    pub operations: Vec<u16>,
}

fn parse_device_info(b: &[u8]) -> DeviceInfo {
    let mut off = 8;
    let (_, o) = ptp_string(b, off);
    off = o + 2;
    let mut arrays: Vec<Vec<u16>> = Vec::new();
    for _ in 0..5 {
        if off + 4 > b.len() {
            break;
        }
        let n = u32::from_le_bytes(b[off..off + 4].try_into().unwrap()) as usize;
        let items = b[off + 4..(off + 4 + 2 * n).min(b.len())]
            .chunks_exact(2)
            .map(|c| u16::from_le_bytes([c[0], c[1]]))
            .collect();
        arrays.push(items);
        off += 4 + 2 * n;
    }
    let (manufacturer, o) = ptp_string(b, off);
    let (model, o) = ptp_string(b, o);
    let (firmware, _) = ptp_string(b, o);
    DeviceInfo {
        manufacturer,
        model,
        firmware,
        operations: arrays.into_iter().next().unwrap_or_default(),
    }
}

/// One PTP/IP session: a command channel and an event channel.
pub struct Session {
    cmd: TcpStream,
    evt: TcpStream,
    tid: u32,
    pub camera_name: String,
}

impl Session {
    pub fn connect(ip: IpAddr, id: Identity, host_name: &str) -> Result<Session, String> {
        let addr = std::net::SocketAddr::new(ip, PORT);
        let mut cmd = TcpStream::connect_timeout(&addr, Duration::from_secs(10))
            .map_err(|e| e.to_string())?;
        cmd.set_read_timeout(Some(Duration::from_secs(30))).ok();
        let mut hello = Vec::new();
        hello.extend_from_slice(&id.0);
        hello.extend_from_slice(&utf16z(host_name));
        hello.extend_from_slice(&0x0001_0000u32.to_le_bytes());
        cmd.write_all(&pkt(1, &hello)).map_err(|e| e.to_string())?;
        let (t, b) = read_pkt(&mut cmd).map_err(|e| e.to_string())?;
        if t != 2 {
            return Err(if t == 5 {
                "the camera does not know this computer; pair it first".into()
            } else {
                format!("unexpected answer {t} to the hello")
            });
        }
        let conn = u32::from_le_bytes(b[..4].try_into().unwrap());
        let (camera_name, _) = {
            let units: Vec<u16> = b[20..]
                .chunks_exact(2)
                .map(|c| u16::from_le_bytes([c[0], c[1]]))
                .take_while(|u| *u != 0)
                .collect();
            (String::from_utf16_lossy(&units), 0)
        };
        let mut evt = TcpStream::connect_timeout(&addr, Duration::from_secs(10))
            .map_err(|e| e.to_string())?;
        evt.write_all(&pkt(3, &conn.to_le_bytes()))
            .map_err(|e| e.to_string())?;
        evt.set_read_timeout(Some(Duration::from_secs(10))).ok();
        let (t, _) = read_pkt(&mut evt).map_err(|e| e.to_string())?;
        if t != 4 {
            return Err("the camera refused the event channel".into());
        }
        evt.set_read_timeout(Some(Duration::from_millis(500))).ok();
        Ok(Session {
            cmd,
            evt,
            tid: 0,
            camera_name,
        })
    }

    /// Run one operation; the data phase comes back as bytes.
    pub fn op(&mut self, code: u16, params: &[u32]) -> Result<Vec<u8>, String> {
        self.tid += 1;
        let mut body = Vec::new();
        body.extend_from_slice(&1u32.to_le_bytes());
        body.extend_from_slice(&code.to_le_bytes());
        body.extend_from_slice(&self.tid.to_le_bytes());
        for p in params {
            body.extend_from_slice(&p.to_le_bytes());
        }
        self.cmd
            .write_all(&pkt(6, &body))
            .map_err(|e| e.to_string())?;
        let mut data = Vec::new();
        loop {
            let (t, b) = read_pkt(&mut self.cmd).map_err(|e| e.to_string())?;
            match t {
                10 | 12 => data.extend_from_slice(&b[4.min(b.len())..]),
                7 => {
                    let rc = u16::from_le_bytes(b[..2].try_into().unwrap());
                    return if rc == RC_OK {
                        Ok(data)
                    } else {
                        Err(format!("operation {code:#06x} answered {rc:#06x}"))
                    };
                }
                _ => {}
            }
        }
    }

    pub fn device_info(&mut self) -> Result<DeviceInfo, String> {
        Ok(parse_device_info(&self.op(OP_GET_DEVICE_INFO, &[])?))
    }

    pub fn open(&mut self) -> Result<(), String> {
        self.op(OP_OPEN_SESSION, &[2]).map(|_| ())
    }

    /// Ask the camera in its wizard for the pairing code it is showing.
    pub fn pairing_code(&mut self) -> Result<String, String> {
        let r = self.op(0x952b, &[])?;
        if r.len() < 4 {
            return Err("no pairing code offered".into());
        }
        let n = u32::from_le_bytes(r[..4].try_into().unwrap()) as usize;
        let digits: String = r[4..(4 + n).min(r.len())]
            .iter()
            .map(|d| char::from(b'0' + (d % 10)))
            .collect();
        if digits.is_empty() {
            Err("no pairing code offered".into())
        } else {
            Ok(digits)
        }
    }

    /// Confirm the pairing. Only call this after the person typed the code
    /// the camera shows and it matched `pairing_code`.
    pub fn confirm_pairing(&mut self) -> Result<(), String> {
        self.op(0x935a, &[0x2001]).map(|_| ())
    }

    /// Wait up to `wait` for the next event; `None` on a quiet timeout.
    pub fn next_event(&mut self, wait: Duration) -> Result<Option<(u16, Vec<u32>)>, String> {
        self.evt.set_read_timeout(Some(wait)).ok();
        match read_pkt(&mut self.evt) {
            Ok((13, _)) => {
                let _ = self.evt.write_all(&pkt(14, &[]));
                Ok(None)
            }
            Ok((8, b)) if b.len() >= 6 => {
                let code = u16::from_le_bytes(b[..2].try_into().unwrap());
                let params = b[6..]
                    .chunks_exact(4)
                    .map(|c| u32::from_le_bytes([c[0], c[1], c[2], c[3]]))
                    .collect();
                Ok(Some((code, params)))
            }
            Ok(_) => Ok(None),
            Err(e)
                if e.kind() == std::io::ErrorKind::WouldBlock
                    || e.kind() == std::io::ErrorKind::TimedOut =>
            {
                Ok(None)
            }
            Err(e) => Err(e.to_string()),
        }
    }

    /// Fetch an announced frame into `out`. The vendor utility reads the
    /// announced handle with GetPartialObject in 64 KB pieces; GetObjectInfo
    /// may be refused in transfer mode, so the file is named from its
    /// content when the camera does not give a name.
    pub fn fetch(
        &mut self,
        handle: u32,
        out: &Path,
        log: &mut dyn FnMut(String),
    ) -> Result<PathBuf, String> {
        let mut say = |m: String| {
            log::info!("camera: {m}");
            log(m);
        };
        say(format!("frame announced, handle {handle:#010x}"));
        let (mut name, mut size) = (String::new(), 0usize);
        match self.op(OP_GET_OBJECT_INFO, &[handle]) {
            Ok(info) if info.len() > 52 => {
                size = u32::from_le_bytes(info[8..12].try_into().unwrap()) as usize;
                name = ptp_string(&info, 52).0;
            }
            Ok(_) => {}
            Err(e) => say(format!("object info refused ({e}); reading in parts")),
        }
        let t0 = Instant::now();
        let mut data: Vec<u8> = Vec::with_capacity(size.max(1 << 20));
        const PIECE: u32 = 1 << 16;
        loop {
            let chunk = self.op(OP_GET_PARTIAL_OBJECT, &[handle, data.len() as u32, PIECE])?;
            let n = chunk.len();
            data.extend_from_slice(&chunk);
            if n < PIECE as usize || (size > 0 && data.len() >= size) {
                break;
            }
        }
        if name.is_empty() {
            let ext = if data.starts_with(&[0xFF, 0xD8]) {
                "JPG"
            } else if data.starts_with(b"MM\0*") || data.starts_with(b"II*\0") {
                "NEF"
            } else {
                "bin"
            };
            name = format!("WIFI_{:08X}.{ext}", handle & 0xFFFF_FFFF);
        }
        std::fs::create_dir_all(out).map_err(|e| e.to_string())?;
        let path = out.join(&name);
        let tmp = out.join(format!("{name}.part"));
        std::fs::write(&tmp, &data).map_err(|e| e.to_string())?;
        std::fs::rename(&tmp, &path).map_err(|e| e.to_string())?;
        let secs = t0.elapsed().as_secs_f64().max(0.001);
        say(format!(
            "{name}: {:.1} MB in {secs:.1} s ({:.1} MB/s)",
            data.len() as f64 / 1e6,
            data.len() as f64 / 1e6 / secs
        ));
        Ok(path)
    }

    pub fn close(mut self) {
        let _ = self.op(OP_CLOSE_SESSION, &[]);
    }
}

/* -- the background worker the app talks to ---------------------------------- */

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum State {
    Searching,
    /// The camera is in its connection wizard and shows a code; the person
    /// must type it to pair.
    NeedsPairing {
        code: String,
        camera: String,
    },
    Connecting,
    Connected {
        camera: String,
    },
    Error(String),
    Stopped,
}

#[derive(Clone, Debug)]
pub struct Status {
    pub state: State,
    pub frames: usize,
    pub log: Vec<String>,
}

enum Msg {
    Code(String),
    Stop,
}

/// The receiver as a thread: discovers, pairs when told the code, receives.
pub struct Receiver {
    status: Arc<Mutex<Status>>,
    tx: std::sync::mpsc::Sender<Msg>,
}

impl Receiver {
    pub fn start(identity_path: PathBuf, out: PathBuf, host_name: String) -> Receiver {
        let status = Arc::new(Mutex::new(Status {
            state: State::Searching,
            frames: 0,
            log: Vec::new(),
        }));
        let (tx, rx) = std::sync::mpsc::channel::<Msg>();
        let st = status.clone();
        std::thread::Builder::new()
            .name("camera-wifi".into())
            .spawn(move || worker(st, rx, identity_path, out, host_name))
            .expect("thread");
        Receiver { status, tx }
    }

    pub fn status(&self) -> Status {
        self.status
            .lock()
            .map(|s| s.clone())
            .unwrap_or_else(|p| p.into_inner().clone())
    }

    /// The person typed the code the camera shows.
    pub fn pair_with_code(&self, code: &str) {
        let _ = self.tx.send(Msg::Code(code.trim().to_string()));
    }

    pub fn stop(&self) {
        let _ = self.tx.send(Msg::Stop);
    }
}

impl Drop for Receiver {
    fn drop(&mut self) {
        self.stop();
    }
}

fn set(st: &Arc<Mutex<Status>>, f: impl FnOnce(&mut Status)) {
    if let Ok(mut s) = st.lock() {
        f(&mut s);
        if s.log.len() > 200 {
            let n = s.log.len() - 200;
            s.log.drain(..n);
        }
    }
}

fn worker(
    st: Arc<Mutex<Status>>,
    rx: std::sync::mpsc::Receiver<Msg>,
    identity_path: PathBuf,
    out: PathBuf,
    host_name: String,
) {
    let note = |m: String| {
        log::info!("camera: {m}");
        set(&st, |s| s.log.push(m));
    };
    loop {
        if matches!(
            rx.try_recv(),
            Ok(Msg::Stop) | Err(std::sync::mpsc::TryRecvError::Disconnected)
        ) {
            set(&st, |s| s.state = State::Stopped);
            return;
        }
        set(&st, |s| s.state = State::Searching);
        let Some(found) = discover(Duration::from_secs(4)) else {
            continue;
        };
        // A saved pairing is reused even when the camera sits in its wizard:
        // the camera keeps accepting the identity it paired with, so only an
        // outright refusal leads to pairing again.
        let saved_works = found.mode == "PAIR"
            && Identity::load(&identity_path)
                .map(|(id, _)| {
                    Session::connect(found.ip, id, &host_name)
                        .map(|s| s.close())
                        .is_ok()
                })
                .unwrap_or(false);
        if found.mode == "PAIR" && !saved_works {
            // Pairing: ask for the code, show it, wait for the person.
            let id = Identity::fresh();
            let mut sess = match Session::connect(found.ip, id, &host_name) {
                Ok(s) => s,
                Err(e) => {
                    note(e);
                    std::thread::sleep(Duration::from_secs(3));
                    continue;
                }
            };
            let info = sess.device_info().unwrap_or_default();
            if sess.open().is_err() {
                continue;
            }
            let code = match sess.pairing_code() {
                Ok(c) => c,
                Err(e) => {
                    note(e);
                    sess.close();
                    std::thread::sleep(Duration::from_secs(3));
                    continue;
                }
            };
            let camera = format!("{} {}", info.manufacturer, info.model)
                .trim()
                .to_string();
            set(&st, |s| {
                s.state = State::NeedsPairing {
                    code: code.clone(),
                    camera: camera.clone(),
                }
            });
            // The camera waits; so do we, for the typed code.
            let typed = match rx.recv_timeout(Duration::from_secs(120)) {
                Ok(Msg::Code(c)) => c,
                Ok(Msg::Stop) | Err(_) => {
                    sess.close();
                    set(&st, |s| s.state = State::Stopped);
                    return;
                }
            };
            if typed != code {
                note("the code did not match; not pairing".into());
                sess.close();
                set(&st, |s| {
                    s.state = State::Error("The code did not match what the camera offered. Start the wizard on the camera again.".into())
                });
                return;
            }
            if let Err(e) = sess.confirm_pairing() {
                note(e);
                sess.close();
                continue;
            }
            let _ = id.save(&identity_path);
            note(format!(
                "paired with {camera}; finish the wizard on the camera"
            ));
            sess.close();
            std::thread::sleep(Duration::from_secs(3));
            continue;
        }
        // Transfer mode. The camera does not let a host read frames; it
        // pushes them to a host that advertises itself and listens. So:
        // advertise, listen, greet the camera once the way the utility does,
        // then take what it sends.
        let Some((id, source)) = Identity::load(&identity_path) else {
            set(&st, |s| {
                s.state = State::Error(
                    "Not paired yet. Start Connect to PC on the camera and choose Pair here."
                        .into(),
                )
            });
            std::thread::sleep(Duration::from_secs(5));
            continue;
        };
        set(&st, |s| s.state = State::Connecting);
        let listener = match std::net::TcpListener::bind(("0.0.0.0", PORT)) {
            Ok(l) => l,
            Err(e) => {
                set(&st, |s| {
                    s.state =
                        State::Error(format!("port {PORT} is in use ({e}); quit Nikon's utility"))
                });
                std::thread::sleep(Duration::from_secs(5));
                continue;
            }
        };
        let ip = local_ip_towards(found.ip);
        let mut adv = match advertise(id, &host_name) {
            Ok(c) => {
                note(format!(
                    "advertising {host_name} on _ptp._tcp at {:?}:{PORT}",
                    ip
                ));
                Some(c)
            }
            Err(e) => {
                note(format!("could not advertise: {e}"));
                None
            }
        };
        // Greet the camera as the utility does, and keep that link open.
        let greet = Session::connect(found.ip, id, &host_name).and_then(|mut sess| {
            let info = sess.device_info().unwrap_or_default();
            sess.open()?;
            let status = sess
                .op(0x952b, &[])
                .map(|r| r.iter().map(|b| format!("{b:02x}")).collect::<String>())
                .unwrap_or_default();
            sess.op(0x935a, &[0x2001])?;
            let _ = sess.op(OP_CLOSE_SESSION, &[]);
            Ok((sess, info, status))
        });
        let (keep, camera) = match greet {
            Ok((sess, info, status)) => {
                let camera = format!("{} {}", info.manufacturer, info.model)
                    .trim()
                    .to_string();
                note(format!(
                    "greeted {camera} (status {status}, identity from {source})"
                ));
                (Some(sess), camera)
            }
            Err(e) => {
                note(format!("{e}"));
                set(&st, |s| s.state = State::Error(e));
                std::thread::sleep(Duration::from_secs(5));
                continue;
            }
        };
        set(&st, |s| {
            s.state = State::Connected {
                camera: camera.clone(),
            }
        });
        // The camera sends on the link we opened: answer there as well.
        if let Some(sess) = keep {
            let Session { cmd, mut evt, .. } = sess;
            let st2 = st.clone();
            let out2 = out.clone();
            let name2 = host_name.clone();
            std::thread::spawn(move || {
                let say = move |m: String| {
                    log::info!("camera: {m}");
                    set(&st2, |s| s.log.push(m));
                };
                say("answering on the opened link".into());
                match serve_ops(cmd, &name2, &out2, &say) {
                    Ok(n) => say(format!("link closed after {n} frame(s)")),
                    Err(e) => say(format!("link: {e}")),
                }
            });
            let st3 = st.clone();
            std::thread::spawn(move || {
                evt.set_read_timeout(None).ok();
                while let Ok((t, b)) = read_pkt(&mut evt) {
                    if t == 13 {
                        let _ = evt.write_all(&pkt(14, &[]));
                    } else if t == 8 && b.len() >= 6 {
                        let code = u16::from_le_bytes([b[0], b[1]]);
                        log::info!("camera: event {code:#06x}");
                    } else {
                        log::info!("camera: event-channel packet type {t}, {} bytes", b.len());
                        set(&st3, |s| {
                            s.log.push(format!("event-channel packet type {t}"))
                        });
                    }
                }
            });
        }
        listener.set_nonblocking(true).ok();
        let started = Instant::now();
        loop {
            if matches!(
                rx.try_recv(),
                Ok(Msg::Stop) | Err(std::sync::mpsc::TryRecvError::Disconnected)
            ) {
                if let Some(c) = adv.as_mut() {
                    let _ = c.kill();
                }
                set(&st, |s| s.state = State::Stopped);
                return;
            }
            match listener.accept() {
                Ok((cmd, peer)) => {
                    note(format!("incoming connection from {peer}"));
                    let st2 = st.clone();
                    let say = move |m: String| {
                        log::info!("camera: {m}");
                        set(&st2, |s| s.log.push(m));
                    };
                    match serve_camera(&listener, cmd, id, &host_name, &out, &say) {
                        Ok(n) => set(&st, |s| s.frames += n),
                        Err(e) => note(format!("{e}")),
                    }
                    listener.set_nonblocking(true).ok();
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    std::thread::sleep(Duration::from_millis(200));
                    if started.elapsed() > Duration::from_secs(3600) {
                        break;
                    }
                }
                Err(e) => {
                    note(format!("listen: {e}"));
                    break;
                }
            }
        }
        if let Some(c) = adv.as_mut() {
            let _ = c.kill();
        }
        std::thread::sleep(Duration::from_secs(2));
    }
}

/* -- the host side: advertise, listen, and take what the camera pushes ---------- */

const OP_GET_STORAGE_IDS: u16 = 0x1004;
const OP_GET_STORAGE_INFO: u16 = 0x1005;
const OP_SEND_OBJECT_INFO: u16 = 0x100c;
const OP_SEND_OBJECT: u16 = 0x100d;
const RC_OPERATION_NOT_SUPPORTED: u16 = 0x2005;

fn ptp_str(s: &str) -> Vec<u8> {
    if s.is_empty() {
        return vec![0];
    }
    let units: Vec<u16> = s.encode_utf16().chain(std::iter::once(0)).collect();
    let mut v = vec![units.len() as u8];
    for u in units {
        v.extend_from_slice(&u.to_le_bytes());
    }
    v
}

fn u16_array(items: &[u16]) -> Vec<u8> {
    let mut v = (items.len() as u32).to_le_bytes().to_vec();
    for i in items {
        v.extend_from_slice(&i.to_le_bytes());
    }
    v
}

/// What Apature says it is when the camera asks.
fn host_device_info(name: &str) -> Vec<u8> {
    let mut v = Vec::new();
    v.extend_from_slice(&100u16.to_le_bytes()); // standard version 1.00
    v.extend_from_slice(&0u32.to_le_bytes()); // vendor extension id
    v.extend_from_slice(&0u16.to_le_bytes()); // vendor extension version
    v.extend_from_slice(&ptp_str("")); // vendor extension desc
    v.extend_from_slice(&0u16.to_le_bytes()); // functional mode
    v.extend_from_slice(&u16_array(&[
        OP_GET_DEVICE_INFO,
        OP_OPEN_SESSION,
        OP_CLOSE_SESSION,
        OP_GET_STORAGE_IDS,
        OP_GET_STORAGE_INFO,
        OP_SEND_OBJECT_INFO,
        OP_SEND_OBJECT,
    ]));
    v.extend_from_slice(&u16_array(&[])); // events
    v.extend_from_slice(&u16_array(&[])); // device properties
    v.extend_from_slice(&u16_array(&[])); // capture formats
    v.extend_from_slice(&u16_array(&[0x3801, 0x3000, 0x3002])); // JPEG, undefined, TIFF-ish RAW
    v.extend_from_slice(&ptp_str("Apature"));
    v.extend_from_slice(&ptp_str(name));
    v.extend_from_slice(&ptp_str(env!("CARGO_PKG_VERSION")));
    v.extend_from_slice(&ptp_str("1"));
    v
}

/// One storage, so SendObjectInfo has somewhere to point.
fn host_storage_info() -> Vec<u8> {
    let mut v = Vec::new();
    v.extend_from_slice(&0x0004u16.to_le_bytes()); // fixed RAM
    v.extend_from_slice(&0x0002u16.to_le_bytes()); // generic hierarchical
    v.extend_from_slice(&0x0000u16.to_le_bytes()); // read-write
    v.extend_from_slice(&(1u64 << 40).to_le_bytes()); // max capacity
    v.extend_from_slice(&(1u64 << 40).to_le_bytes()); // free bytes
    v.extend_from_slice(&0xFFFF_FFFFu32.to_le_bytes()); // free objects
    v.extend_from_slice(&ptp_str("Apature"));
    v.extend_from_slice(&ptp_str(""));
    v
}

/// Read a data phase sent by the camera (Start_Data, Data…, End_Data).
fn read_data_phase(s: &mut TcpStream) -> std::io::Result<Vec<u8>> {
    let mut data = Vec::new();
    loop {
        let (t, b) = read_pkt(s)?;
        match t {
            9 => {}
            10 => data.extend_from_slice(&b[4.min(b.len())..]),
            12 => {
                data.extend_from_slice(&b[4.min(b.len())..]);
                return Ok(data);
            }
            11 => return Err(std::io::Error::other("cancelled by the camera")),
            _ => {}
        }
    }
}

fn respond(s: &mut TcpStream, code: u16, tid: u32, params: &[u32]) -> std::io::Result<()> {
    let mut b = Vec::new();
    b.extend_from_slice(&code.to_le_bytes());
    b.extend_from_slice(&tid.to_le_bytes());
    for p in params {
        b.extend_from_slice(&p.to_le_bytes());
    }
    s.write_all(&pkt(7, &b))
}

fn send_data(s: &mut TcpStream, tid: u32, data: &[u8]) -> std::io::Result<()> {
    let mut start = tid.to_le_bytes().to_vec();
    start.extend_from_slice(&(data.len() as u64).to_le_bytes());
    s.write_all(&pkt(9, &start))?;
    let mut end = tid.to_le_bytes().to_vec();
    end.extend_from_slice(data);
    s.write_all(&pkt(12, &end))
}

/// Serve one camera that connected to us: the hello, the event channel,
/// then operations until it goes away. Frames pushed with SendObjectInfo +
/// SendObject land in `out`.
fn serve_camera(
    listener: &std::net::TcpListener,
    mut cmd: TcpStream,
    id: Identity,
    host_name: &str,
    out: &Path,
    note: &dyn Fn(String),
) -> Result<usize, String> {
    cmd.set_read_timeout(Some(Duration::from_secs(60))).ok();
    let (t, b) = read_pkt(&mut cmd).map_err(|e| e.to_string())?;
    if t != 1 || b.len() < 20 {
        return Err("the connection did not start with a hello".into());
    }
    let their_guid = &b[..16];
    let their_name = {
        let units: Vec<u16> = b[16..b.len().saturating_sub(4)]
            .chunks_exact(2)
            .map(|c| u16::from_le_bytes([c[0], c[1]]))
            .take_while(|u| *u != 0)
            .collect();
        String::from_utf16_lossy(&units)
    };
    note(format!(
        "camera {their_name} connected to Apature (identity {})",
        their_guid
            .iter()
            .map(|x| format!("{x:02x}"))
            .collect::<String>()
    ));
    let mut ack = 1u32.to_le_bytes().to_vec();
    ack.extend_from_slice(&id.0);
    ack.extend_from_slice(&utf16z(host_name));
    ack.extend_from_slice(&0x0001_0000u32.to_le_bytes());
    cmd.write_all(&pkt(2, &ack)).map_err(|e| e.to_string())?;
    // The event channel: the camera opens a second connection.
    listener.set_nonblocking(false).ok();
    let (mut evt, _) = listener.accept().map_err(|e| e.to_string())?;
    evt.set_read_timeout(Some(Duration::from_secs(30))).ok();
    let (t, _) = read_pkt(&mut evt).map_err(|e| e.to_string())?;
    if t != 3 {
        return Err("the second connection was not the event channel".into());
    }
    evt.write_all(&pkt(4, &[])).map_err(|e| e.to_string())?;
    note("event channel open; waiting for frames".into());
    serve_ops(cmd, host_name, out, note)
}

/// Answer the camera's operations on `cmd` until it goes away: device
/// info, session, storage, and the frames it pushes with SendObjectInfo +
/// SendObject, written into `out`. Returns how many frames landed.
fn serve_ops(
    mut cmd: TcpStream,
    host_name: &str,
    out: &Path,
    note: &dyn Fn(String),
) -> Result<usize, String> {
    cmd.set_read_timeout(None).ok();
    let mut frames = 0usize;
    let mut pending: Option<(String, usize)> = None;
    let mut next_handle = 1u32;
    loop {
        let (t, b) = match read_pkt(&mut cmd) {
            Ok(x) => x,
            Err(e) => {
                note(format!("camera went away: {e}"));
                return Ok(frames);
            }
        };
        match t {
            13 => {
                let _ = cmd.write_all(&pkt(14, &[]));
            }
            6 if b.len() >= 10 => {
                let op = u16::from_le_bytes([b[4], b[5]]);
                let tid = u32::from_le_bytes([b[6], b[7], b[8], b[9]]);
                let params: Vec<u32> = b[10..]
                    .chunks_exact(4)
                    .map(|c| u32::from_le_bytes([c[0], c[1], c[2], c[3]]))
                    .collect();
                let r: std::io::Result<()> = match op {
                    OP_GET_DEVICE_INFO => send_data(&mut cmd, tid, &host_device_info(host_name))
                        .and_then(|_| respond(&mut cmd, RC_OK, tid, &[])),
                    OP_OPEN_SESSION | OP_CLOSE_SESSION => respond(&mut cmd, RC_OK, tid, &[]),
                    OP_GET_STORAGE_IDS => send_data(&mut cmd, tid, &{
                        let mut v = 1u32.to_le_bytes().to_vec();
                        v.extend_from_slice(&0x0001_0001u32.to_le_bytes());
                        v
                    })
                    .and_then(|_| respond(&mut cmd, RC_OK, tid, &[])),
                    OP_GET_STORAGE_INFO => send_data(&mut cmd, tid, &host_storage_info())
                        .and_then(|_| respond(&mut cmd, RC_OK, tid, &[])),
                    OP_SEND_OBJECT_INFO => match read_data_phase(&mut cmd) {
                        Ok(info) if info.len() > 52 => {
                            let size = u32::from_le_bytes(info[8..12].try_into().unwrap()) as usize;
                            let (name, _) = ptp_string(&info, 52);
                            note(format!("incoming {name}, {:.1} MB", size as f64 / 1e6));
                            pending = Some((name, size));
                            let h = next_handle;
                            next_handle += 1;
                            respond(&mut cmd, RC_OK, tid, &[0x0001_0001, 0, h])
                        }
                        Ok(_) => respond(&mut cmd, 0x2002, tid, &[]),
                        Err(e) => Err(e),
                    },
                    OP_SEND_OBJECT => match read_data_phase(&mut cmd) {
                        Ok(data) => {
                            let (name, _) = pending.take().unwrap_or_else(|| {
                                (format!("WIFI_{:08X}.bin", next_handle), data.len())
                            });
                            let name = if name.is_empty() {
                                format!("WIFI_{:08X}.bin", next_handle)
                            } else {
                                name
                            };
                            let _ = std::fs::create_dir_all(out);
                            let path = out.join(&name);
                            let tmp = out.join(format!("{name}.part"));
                            let w = std::fs::write(&tmp, &data)
                                .and_then(|_| std::fs::rename(&tmp, &path));
                            match w {
                                Ok(()) => {
                                    frames += 1;
                                    note(format!(
                                        "received {name} ({:.1} MB)",
                                        data.len() as f64 / 1e6
                                    ));
                                    respond(&mut cmd, RC_OK, tid, &[])
                                }
                                Err(e) => {
                                    note(format!("could not write {name}: {e}"));
                                    respond(&mut cmd, 0x2002, tid, &[])
                                }
                            }
                        }
                        Err(e) => Err(e),
                    },
                    other => {
                        note(format!(
                            "camera asked for {other:#06x} {params:?}; not supported"
                        ));
                        respond(&mut cmd, RC_OPERATION_NOT_SUPPORTED, tid, &[])
                    }
                };
                if let Err(e) = r {
                    note(format!("camera went away: {e}"));
                    return Ok(frames);
                }
            }
            _ => {}
        }
    }
}

/// Advertise Apature as a `_ptp._tcp` service the camera can connect to.
/// The system's `dns-sd -R` keeps the registration alive for as long as the
/// returned child runs.
fn advertise(id: Identity, host_name: &str) -> Result<std::process::Child, String> {
    let b = id.0;
    let guid = format!(
        "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7], b[8], b[9], b[10], b[11], b[12], b[13], b[14], b[15]
    );
    std::process::Command::new("dns-sd")
        .args([
            "-R",
            host_name,
            "_ptp._tcp",
            "local",
            &PORT.to_string(),
            "apps=WT3T",
            &format!("guid={guid}"),
            &format!("name={host_name}"),
            "type=PC",
            "ver=1",
        ])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map_err(|e| e.to_string())
}

/// The address this machine has on the camera's network.
fn local_ip_towards(ip: IpAddr) -> Option<std::net::Ipv4Addr> {
    let s = std::net::UdpSocket::bind("0.0.0.0:0").ok()?;
    s.connect((ip, PORT)).ok()?;
    match s.local_addr().ok()?.ip() {
        IpAddr::V4(v4) => Some(v4),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn address_comes_out_of_the_hostname() {
        assert_eq!(
            ip_from_hostname("38e86a91-010-064-144-040.local."),
            Some(IpAddr::V4(Ipv4Addr::new(10, 64, 144, 40)))
        );
        assert_eq!(ip_from_hostname("camera.local."), None);
    }

    #[test]
    fn device_info_parses() {
        // version, vendor ext id, vendor ext version, desc "", functional mode, five arrays, then strings
        let mut b = vec![0u8; 8];
        b.push(0); // desc
        b.extend_from_slice(&[0, 0]); // functional mode
        for arr in [vec![0x1001u16, 0x1009], vec![], vec![], vec![], vec![]] {
            b.extend_from_slice(&(arr.len() as u32).to_le_bytes());
            for v in arr {
                b.extend_from_slice(&v.to_le_bytes());
            }
        }
        for s in ["Nikon Corporation", "Z 6_2", "V1.62", "1"] {
            let units: Vec<u16> = s.encode_utf16().chain(std::iter::once(0)).collect();
            b.push(units.len() as u8);
            for u in units {
                b.extend_from_slice(&u.to_le_bytes());
            }
        }
        let d = parse_device_info(&b);
        assert_eq!(d.model, "Z 6_2");
        assert!(d.operations.contains(&0x1009));
    }
}
