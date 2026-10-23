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

/// Look for a camera for `wait`. Returns the first one that resolves.
pub fn discover(wait: Duration) -> Option<Found> {
    let daemon = mdns_sd::ServiceDaemon::new().ok()?;
    let rx = daemon.browse("_ptp._tcp.local.").ok()?;
    let deadline = Instant::now() + wait;
    let mut found = None;
    while Instant::now() < deadline {
        let left = deadline.saturating_duration_since(Instant::now());
        let Ok(ev) = rx.recv_timeout(left) else { break };
        if let mdns_sd::ServiceEvent::ServiceResolved(info) = ev {
            let name = info
                .get_fullname()
                .split('.')
                .next()
                .unwrap_or("")
                .to_string();
            let mode = info.get_property_val_str("apps").unwrap_or("?").to_string();
            // Nikon encodes the address in the host name (…-010-064-144-040.local.)
            // and also answers with an A record; take whichever is there.
            let ip = info
                .get_addresses()
                .iter()
                .map(|a| a.to_ip_addr())
                .find(|ip| ip.is_ipv4())
                .or_else(|| ip_from_hostname(info.get_hostname()));
            if let Some(ip) = ip {
                found = Some(Found { ip, name, mode });
                break;
            }
        }
    }
    let _ = daemon.shutdown();
    found
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
const OP_GET_OBJECT_HANDLES: u16 = 0x1007;
const OP_GET_OBJECT_INFO: u16 = 0x1008;
const OP_GET_OBJECT: u16 = 0x1009;
const OP_GET_PARTIAL_OBJECT: u16 = 0x101b;
const EV_OBJECT_ADDED: u16 = 0x4002;
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
        self.op(OP_OPEN_SESSION, &[1]).map(|_| ())
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

    /// Fetch an announced object into `out`, trying the ways a camera in
    /// transfer mode may allow. Returns the path written.
    pub fn fetch(
        &mut self,
        handle: u32,
        out: &Path,
        log: &mut dyn FnMut(String),
    ) -> Result<PathBuf, String> {
        std::thread::sleep(Duration::from_millis(800));
        let mut handle = handle;
        let info = match self.op(OP_GET_OBJECT_INFO, &[handle]) {
            Ok(i) => i,
            Err(e) => {
                log(format!(
                    "object info on the announced handle: {e}; listing handles"
                ));
                let list = self.op(OP_GET_OBJECT_HANDLES, &[0xFFFF_FFFF, 0, 0])?;
                let n = u32::from_le_bytes(list[..4].try_into().unwrap()) as usize;
                let hs: Vec<u32> = list[4..(4 + 4 * n).min(list.len())]
                    .chunks_exact(4)
                    .map(|c| u32::from_le_bytes([c[0], c[1], c[2], c[3]]))
                    .collect();
                let last = *hs.last().ok_or("the camera lists no objects")?;
                log(format!("{n} objects listed; using the newest, {last:#x}"));
                handle = last;
                self.op(OP_GET_OBJECT_INFO, &[handle])?
            }
        };
        let size = u32::from_le_bytes(info[8..12].try_into().unwrap()) as usize;
        let (name, _) = ptp_string(&info, 52);
        let name = if name.is_empty() {
            format!("{handle:08x}.bin")
        } else {
            name
        };
        let t0 = Instant::now();
        let data = match self.op(OP_GET_OBJECT, &[handle]) {
            Ok(d) => d,
            Err(e) => {
                log(format!("whole object refused ({e}); reading in parts"));
                let mut d = Vec::with_capacity(size);
                while d.len() < size {
                    let chunk =
                        self.op(OP_GET_PARTIAL_OBJECT, &[handle, d.len() as u32, 1 << 20])?;
                    if chunk.is_empty() {
                        break;
                    }
                    d.extend_from_slice(&chunk);
                }
                d
            }
        };
        std::fs::create_dir_all(out).map_err(|e| e.to_string())?;
        let path = out.join(&name);
        let tmp = out.join(format!("{name}.part"));
        std::fs::write(&tmp, &data).map_err(|e| e.to_string())?;
        std::fs::rename(&tmp, &path).map_err(|e| e.to_string())?;
        let secs = t0.elapsed().as_secs_f64().max(0.001);
        log(format!(
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
        if found.mode == "PAIR" {
            // Pairing: ask for the code, show it, wait for the person.
            let id = Identity::fresh();
            let mut sess = match Session::connect(found.ip, id, &host_name) {
                Ok(s) => s,
                Err(e) => {
                    note(format!("{e}"));
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
        // Transfer mode: connect with the saved identity and receive.
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
        let mut sess = match Session::connect(found.ip, id, &host_name) {
            Ok(s) => s,
            Err(e) => {
                note(format!("{e} (identity from {source})"));
                set(&st, |s| s.state = State::Error(e));
                std::thread::sleep(Duration::from_secs(5));
                continue;
            }
        };
        let info = sess.device_info().unwrap_or_default();
        let camera = format!("{} {}", info.manufacturer, info.model)
            .trim()
            .to_string();
        if sess.open().is_err() {
            std::thread::sleep(Duration::from_secs(3));
            continue;
        }
        note(format!("connected to {camera}, firmware {}", info.firmware));
        set(&st, |s| {
            s.state = State::Connected {
                camera: camera.clone(),
            }
        });
        loop {
            if matches!(
                rx.try_recv(),
                Ok(Msg::Stop) | Err(std::sync::mpsc::TryRecvError::Disconnected)
            ) {
                sess.close();
                set(&st, |s| s.state = State::Stopped);
                return;
            }
            match sess.next_event(Duration::from_millis(500)) {
                Ok(Some((EV_OBJECT_ADDED, params))) if !params.is_empty() => {
                    let st2 = st.clone();
                    let mut log = |m: String| set(&st2, |s| s.log.push(m));
                    match sess.fetch(params[0], &out, &mut log) {
                        Ok(_) => set(&st, |s| s.frames += 1),
                        Err(e) => note(format!("frame not fetched: {e}")),
                    }
                }
                Ok(_) => {}
                Err(e) => {
                    note(format!("connection lost: {e}"));
                    break;
                }
            }
        }
        std::thread::sleep(Duration::from_secs(2));
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
