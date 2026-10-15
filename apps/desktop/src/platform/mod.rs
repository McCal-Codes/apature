//! The thin native layer. Everything egui cannot do itself lives here:
//! the menu bar and system accessibility preferences. Nothing in `ui/`
//! or the core crates imports from this module except through `Platform`.

use crate::commands::Command;

#[cfg(target_os = "macos")]
mod macos;

pub struct Platform {
    #[cfg(target_os = "macos")]
    menu: Option<macos::MenuBar>,
}

impl Platform {
    pub fn new() -> Self {
        Platform {
            #[cfg(target_os = "macos")]
            menu: macos::MenuBar::install()
                .map_err(|e| log::warn!("native menu unavailable: {e}"))
                .ok(),
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

    /// System "Reduce Motion", read once at startup.
    pub fn reduce_motion() -> bool {
        #[cfg(target_os = "macos")]
        {
            macos::reduce_motion()
        }
        #[cfg(not(target_os = "macos"))]
        {
            false
        }
    }

    /// Unmount the volume that holds `path` (macOS: `diskutil unmount`),
    /// off the UI thread; only volumes under /Volumes qualify. Returns
    /// the volume root that was asked to unmount, if any.
    pub fn eject(path: &std::path::Path) -> Option<std::path::PathBuf> {
        let mut comps = path.components();
        let root: std::path::PathBuf = match (comps.next(), comps.next(), comps.next()) {
            (Some(std::path::Component::RootDir), Some(v), Some(name))
                if v.as_os_str() == "Volumes" =>
            {
                std::path::PathBuf::from("/Volumes").join(name.as_os_str())
            }
            _ => return None,
        };
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
        Some(root)
    }

    pub fn reveal(path: &std::path::Path) {
        #[cfg(target_os = "macos")]
        {
            let _ = std::process::Command::new("open")
                .arg("-R")
                .arg(path)
                .spawn();
        }
        #[cfg(not(target_os = "macos"))]
        let _ = path;
    }
}
