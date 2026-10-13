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
    pub fn sync_toggles(&self, sidebar: bool, inspector: bool, filmstrip: bool) {
        #[cfg(target_os = "macos")]
        if let Some(m) = &self.menu {
            m.sync_toggles(sidebar, inspector, filmstrip);
        }
        #[cfg(not(target_os = "macos"))]
        let _ = (sidebar, inspector, filmstrip);
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
