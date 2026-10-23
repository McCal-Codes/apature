//! The thin native layer. Everything egui cannot do itself lives here:
//! the menu bar, volumes, opening things, secrets and accessibility
//! preferences, for macOS, Linux and Windows. Nothing in `ui/`
//! or the core crates imports from this module except through `Platform`.

use crate::commands::Command;

#[cfg(target_os = "macos")]
mod macos;

pub struct Platform {
    #[cfg(target_os = "macos")]
    menu: Option<macos::MenuBar>,
    #[cfg(target_os = "macos")]
    status: Option<macos::StatusItem>,
}

impl Platform {
    pub fn new() -> Self {
        Platform {
            #[cfg(target_os = "macos")]
            menu: macos::MenuBar::install()
                .map_err(|e| log::warn!("native menu unavailable: {e}"))
                .ok(),
            #[cfg(target_os = "macos")]
            status: None,
        }
    }

    /// True when the native menu bar owns the ⌘ shortcuts, so the egui key
    /// handler must leave command-modified keys alone.
    pub fn has_menu(&self) -> bool {
        #[cfg(target_os = "macos")]
        {
            self.menu.is_some()
        }
        #[cfg(not(target_os = "macos"))]
        {
            false
        }
    }

    /// Finish native setup that must wait for the app to be fully launched.
    pub fn on_first_frame(&mut self) {
        #[cfg(target_os = "macos")]
        if let Some(m) = &mut self.menu {
            m.install_if_needed();
        }
        #[cfg(target_os = "macos")]
        if self.status.is_none() {
            self.status = macos::StatusItem::install()
                .map_err(|e| log::warn!("menu bar extra unavailable: {e}"))
                .ok();
        }
    }

    /// Menu bar extra title (empty hides it) and the Live Ingest check.
    pub fn status_item(&self, title: &str, live: bool) {
        #[cfg(target_os = "macos")]
        if let Some(s) = &self.status {
            s.set_title(title);
            s.set_live(live);
        }
        #[cfg(not(target_os = "macos"))]
        {
            let _ = (title, live);
        }
    }

    pub fn drain_menu(&self) -> Vec<Command> {
        #[cfg(target_os = "macos")]
        {
            self.menu.as_ref().map(|m| m.drain()).unwrap_or_default()
        }
        #[cfg(not(target_os = "macos"))]
        {
            Vec::new()
        }
    }

    /// Reflect the state of toggles in the menu (checkmarks).
    pub fn sync_toggles(&self, sidebar: bool, inspector: bool, filmstrip: bool, advance: bool) {
        #[cfg(target_os = "macos")]
        if let Some(m) = &self.menu {
            m.sync_toggles(sidebar, inspector, filmstrip, advance);
        }
        #[cfg(not(target_os = "macos"))]
        let _ = (sidebar, inspector, filmstrip, advance);
    }

    pub fn set_class_names(&self, names: &[String; 8]) {
        #[cfg(target_os = "macos")]
        if let Some(m) = &self.menu {
            m.set_class_names(names);
        }
        #[cfg(not(target_os = "macos"))]
        let _ = names;
    }

    /// System "Reduce Motion", read once at startup (macOS accessibility
    /// setting; GNOME's "enable animations" on Linux).
    pub fn reduce_motion() -> bool {
        #[cfg(target_os = "macos")]
        {
            macos::reduce_motion()
        }
        #[cfg(target_os = "linux")]
        {
            std::process::Command::new("gsettings")
                .args(["get", "org.gnome.desktop.interface", "enable-animations"])
                .output()
                .ok()
                .map(|o| String::from_utf8_lossy(&o.stdout).trim() == "false")
                .unwrap_or(false)
        }
        #[cfg(target_os = "windows")]
        {
            false
        }
    }

    /// Where removable volumes appear: `/Volumes` on macOS; the user's
    /// `/media` and `/run/media` folders plus `/media` and `/mnt` on Linux;
    /// every drive root except the system drive on Windows.
    pub fn volume_roots() -> Vec<std::path::PathBuf> {
        #[cfg(target_os = "macos")]
        {
            vec![std::path::PathBuf::from("/Volumes")]
        }
        #[cfg(target_os = "linux")]
        {
            let mut v = Vec::new();
            if let Ok(user) = std::env::var("USER") {
                v.push(std::path::PathBuf::from("/media").join(&user));
                v.push(std::path::PathBuf::from("/run/media").join(&user));
            }
            v.push(std::path::PathBuf::from("/media"));
            v.push(std::path::PathBuf::from("/mnt"));
            v
        }
        #[cfg(target_os = "windows")]
        {
            Vec::new()
        }
    }

    /// The mounted volumes to list in the sidebar: (display name, root).
    /// System volumes are filtered by the caller through `volume_kind`.
    pub fn volumes() -> Vec<(String, std::path::PathBuf)> {
        let mut v = Vec::new();
        #[cfg(target_os = "windows")]
        {
            let system = std::env::var("SystemDrive").unwrap_or_else(|_| "C:".into());
            for letter in b'A'..=b'Z' {
                let root = format!("{}:\\", letter as char);
                if root.trim_end_matches('\\').eq_ignore_ascii_case(&system) {
                    continue;
                }
                let p = std::path::PathBuf::from(&root);
                if p.is_dir() {
                    v.push((format!("{}:", letter as char), p));
                }
            }
            return v;
        }
        #[cfg(not(target_os = "windows"))]
        {
            for root in Self::volume_roots() {
                let Ok(rd) = std::fs::read_dir(&root) else {
                    continue;
                };
                for e in rd.flatten() {
                    let p = e.path();
                    let name = e.file_name().to_string_lossy().into_owned();
                    if p.is_dir() && !name.starts_with('.') && !v.iter().any(|(_, q)| *q == p) {
                        v.push((name, p));
                    }
                }
            }
            v
        }
    }

    /// True when `path` is on a removable volume (so it may be ejected, and
    /// stays in the sidebar while unplugged).
    pub fn is_volume(path: &std::path::Path) -> bool {
        Self::volume_root_of(path).is_some()
    }

    /// The volume root that holds `path`, if it is under one of the roots.
    pub fn volume_root_of(path: &std::path::Path) -> Option<std::path::PathBuf> {
        #[cfg(target_os = "windows")]
        {
            let mut comps = path.components();
            if let Some(std::path::Component::Prefix(p)) = comps.next() {
                let root = std::path::PathBuf::from(p.as_os_str()).join("\\");
                let system = std::env::var("SystemDrive").unwrap_or_else(|_| "C:".into());
                if p.as_os_str()
                    .to_string_lossy()
                    .eq_ignore_ascii_case(&system)
                {
                    return None;
                }
                return Some(root);
            }
            None
        }
        #[cfg(not(target_os = "windows"))]
        {
            for root in Self::volume_roots() {
                if let Ok(rest) = path.strip_prefix(&root) {
                    if let Some(name) = rest.components().next() {
                        return Some(root.join(name.as_os_str()));
                    }
                }
            }
            None
        }
    }

    /// Unmount the volume that holds `path`, off the UI thread (macOS:
    /// `diskutil unmount`; Linux: `gio mount -e`, then `udisksctl`). Returns
    /// the volume root that was asked to unmount, if any. Windows: none yet.
    pub fn eject(path: &std::path::Path) -> Option<std::path::PathBuf> {
        let root = Self::volume_root_of(path)?;
        #[cfg(target_os = "macos")]
        {
            let r = root.clone();
            std::thread::spawn(move || {
                let _ = std::process::Command::new("diskutil")
                    .arg("unmount")
                    .arg(&r)
                    .output();
            });
        }
        #[cfg(target_os = "linux")]
        {
            let r = root.clone();
            std::thread::spawn(move || {
                let ok = std::process::Command::new("gio")
                    .args(["mount", "-e"])
                    .arg(&r)
                    .output()
                    .map(|o| o.status.success())
                    .unwrap_or(false);
                if !ok {
                    let _ = std::process::Command::new("udisksctl")
                        .args(["unmount", "-b"])
                        .arg(&r)
                        .output();
                }
            });
        }
        #[cfg(target_os = "windows")]
        {
            // Shell.Application's Eject verb on the drive, the way the
            // "Safely Remove Hardware" menu does it.
            let letter = root.to_string_lossy().trim_end_matches('\\').to_string();
            std::thread::spawn(move || {
                let script = format!(
                    "(New-Object -ComObject Shell.Application).NameSpace(17).ParseName('{letter}').InvokeVerb('Eject')"
                );
                let _ = std::process::Command::new("powershell")
                    .args(["-NoProfile", "-NonInteractive", "-Command", &script])
                    .output();
            });
        }
        Some(root)
    }

    /// Show `path` in the system file manager, selected where the platform
    /// supports it (Finder, Explorer); otherwise its folder.
    pub fn reveal(path: &std::path::Path) {
        #[cfg(target_os = "macos")]
        {
            let _ = std::process::Command::new("open")
                .arg("-R")
                .arg(path)
                .spawn();
        }
        #[cfg(target_os = "windows")]
        {
            let _ = std::process::Command::new("explorer")
                .arg(format!("/select,{}", path.display()))
                .spawn();
        }
        #[cfg(target_os = "linux")]
        {
            let dir = if path.is_dir() {
                path.to_path_buf()
            } else {
                path.parent().map(|p| p.to_path_buf()).unwrap_or_default()
            };
            let _ = std::process::Command::new("xdg-open").arg(dir).spawn();
        }
    }

    /// Open a folder or file with its default application.
    #[allow(dead_code)]
    pub fn open_path(path: &std::path::Path) {
        #[cfg(target_os = "macos")]
        {
            let _ = std::process::Command::new("open").arg(path).spawn();
        }
        #[cfg(target_os = "windows")]
        {
            let _ = std::process::Command::new("explorer").arg(path).spawn();
        }
        #[cfg(target_os = "linux")]
        {
            let _ = std::process::Command::new("xdg-open").arg(path).spawn();
        }
    }

    /// Open a URL in the default browser.
    pub fn open_url(url: &str) {
        #[cfg(target_os = "macos")]
        {
            let _ = std::process::Command::new("open").arg(url).spawn();
        }
        #[cfg(target_os = "windows")]
        {
            let _ = std::process::Command::new("cmd")
                .args(["/C", "start", "", url])
                .spawn();
        }
        #[cfg(target_os = "linux")]
        {
            let _ = std::process::Command::new("xdg-open").arg(url).spawn();
        }
    }

    /// The name of the system file manager, for button labels.
    pub fn file_manager() -> &'static str {
        if cfg!(target_os = "macos") {
            "Finder"
        } else if cfg!(target_os = "windows") {
            "Explorer"
        } else {
            "file manager"
        }
    }

    /// Where delivery passwords live, for Settings copy.
    pub fn secret_store() -> &'static str {
        if cfg!(target_os = "macos") {
            "the Keychain"
        } else if cfg!(target_os = "windows") {
            "Windows Credential Manager"
        } else {
            "the system keyring"
        }
    }
}

/// What a mounted volume is, for the icon and for hiding system volumes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VolumeKind {
    /// Has a DCIM folder: a memory card or a tethered camera.
    Camera,
    /// Removable or external disk (USB, Thunderbolt, SD without DCIM).
    External,
    /// The Mac's own disk.
    Internal,
    /// Recovery and other system volumes: never an ingest source.
    System,
}

impl Platform {
    /// Classify a volume root. DCIM wins outright (a camera over USB reports
    /// the same bus as an SSD, so the bus alone cannot tell them apart);
    /// otherwise macOS `diskutil info -plist` says whether the disk is
    /// internal. One process per new mount, cached by the caller.
    pub fn volume_kind(path: &std::path::Path) -> VolumeKind {
        if path.join("DCIM").is_dir() {
            return VolumeKind::Camera;
        }
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_lowercase())
            .unwrap_or_default();
        if name == "recovery"
            || name.starts_with("com.apple")
            || name == "preboot"
            || name == "vm"
            || name == "update"
        {
            return VolumeKind::System;
        }
        #[cfg(target_os = "macos")]
        {
            if let Ok(out) = std::process::Command::new("/usr/sbin/diskutil")
                .args(["info", "-plist"])
                .arg(path)
                .output()
            {
                let text = String::from_utf8_lossy(&out.stdout);
                // <key>Internal</key> <true/> ... without pulling in a plist crate.
                if let Some(i) = text.find("<key>Internal</key>") {
                    let tail = &text[i..];
                    if tail
                        .trim_start_matches("<key>Internal</key>")
                        .trim_start()
                        .starts_with("<true/>")
                    {
                        return VolumeKind::Internal;
                    }
                }
            }
        }
        VolumeKind::External
    }
}

/// Keychain service name for delivery passwords (account = target name).
const SECRET_SERVICE: &str = "dev.mccal.Apature.delivery";

impl Platform {
    /// Read a delivery password from the macOS Keychain; elsewhere from the
    /// platform store through `keyring` (Secret Service on Linux, Credential
    /// Manager on Windows).
    pub fn secret_get(account: &str) -> Option<String> {
        #[cfg(target_os = "macos")]
        {
            security_framework::passwords::get_generic_password(SECRET_SERVICE, account)
                .ok()
                .and_then(|b| String::from_utf8(b).ok())
        }
        #[cfg(not(target_os = "macos"))]
        {
            keyring::Entry::new(SECRET_SERVICE, account)
                .ok()
                .and_then(|e| e.get_password().ok())
        }
    }

    /// Store (or replace) a delivery password in the macOS Keychain.
    pub fn secret_set(account: &str, secret: &str) -> Result<(), String> {
        #[cfg(target_os = "macos")]
        {
            security_framework::passwords::set_generic_password(
                SECRET_SERVICE,
                account,
                secret.as_bytes(),
            )
            .map_err(|e| e.to_string())
        }
        #[cfg(not(target_os = "macos"))]
        {
            keyring::Entry::new(SECRET_SERVICE, account)
                .and_then(|e| e.set_password(secret))
                .map_err(|e| e.to_string())
        }
    }

    pub fn secret_delete(account: &str) {
        #[cfg(target_os = "macos")]
        {
            let _ = security_framework::passwords::delete_generic_password(SECRET_SERVICE, account);
        }
        #[cfg(not(target_os = "macos"))]
        {
            if let Ok(e) = keyring::Entry::new(SECRET_SERVICE, account) {
                let _ = e.delete_credential();
            }
        }
    }
}
