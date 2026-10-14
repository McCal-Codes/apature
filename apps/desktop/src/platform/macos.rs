//! Native menu bar via `muda`, mapped onto `Command`s.

use muda::accelerator::{Accelerator, Code, Modifiers};
use muda::{AboutMetadata, CheckMenuItem, Menu, MenuEvent, MenuItem, PredefinedMenuItem, Submenu};

use crate::commands::{Command, Workspace};

pub struct MenuBar {
    menu: Menu,
    window: Submenu,
    help: Submenu,
    installed: bool,
    sidebar: CheckMenuItem,
    advance: CheckMenuItem,
    inspector: CheckMenuItem,
    filmstrip: CheckMenuItem,
    class_items: Vec<MenuItem>,
}

fn item(cmd: Command, text: &str, acc: Option<Accelerator>) -> MenuItem {
    MenuItem::with_id(cmd.menu_id(), text, true, acc)
}

fn disabled(text: &str) -> MenuItem {
    MenuItem::new(text, false, None)
}

fn acc(mods: Modifiers, code: Code) -> Option<Accelerator> {
    Some(Accelerator::new(mods, code))
}

impl MenuBar {
    pub fn install() -> muda::Result<Self> {
        let menu = Menu::new();
        let cmd = Modifiers::META;
        let cmd_shift = Modifiers::META | Modifiers::SHIFT;
        let cmd_alt = Modifiers::META | Modifiers::ALT;
        let ctrl_cmd = Modifiers::META | Modifiers::CONTROL;

        let app = Submenu::new("Apature", true);
        app.append_items(&[
            &PredefinedMenuItem::about(
                Some("About Apature"),
                Some(AboutMetadata {
                    name: Some("Apature".into()),
                    version: Some(env!("CARGO_PKG_VERSION").into()),
                    comments: Some(
                        "Keyboard-first contact sheet, culling and metadata for photojournalists."
                            .into(),
                    ),
                    ..Default::default()
                }),
            ),
            &PredefinedMenuItem::separator(),
            &item(Command::OpenSettings, "Settings…", acc(cmd, Code::Comma)),
            &PredefinedMenuItem::separator(),
            &PredefinedMenuItem::hide(None),
            &PredefinedMenuItem::hide_others(None),
            &PredefinedMenuItem::show_all(None),
            &PredefinedMenuItem::separator(),
            &PredefinedMenuItem::quit(None),
        ])?;

        let file = Submenu::new("File", true);
        file.append_items(&[
            &item(
                Command::PickFolder,
                "Open Folder in New Tab…",
                acc(cmd, Code::KeyO),
            ),
            &item(Command::PickFolder, "New Tab…", acc(cmd, Code::KeyT)),
            &item(Command::CloseTab, "Close Tab", acc(cmd, Code::KeyW)),
            &item(Command::AddFavorite, "Add Folder to Favorites", None),
            &PredefinedMenuItem::separator(),
            &item(Command::LoadAssignment, "Load Assignment…", None),
            &item(Command::LoadCodes, "Load Code Replacements…", None),
            &PredefinedMenuItem::separator(),
            &item(Command::IngestStart, "Ingest…", acc(cmd_shift, Code::KeyI)),
            &item(Command::ExportStart, "Export…", acc(cmd, Code::KeyE)),
            &PredefinedMenuItem::separator(),
            &item(
                Command::RevealInFinder,
                "Reveal in Finder",
                acc(cmd, Code::KeyR),
            ),
            &PredefinedMenuItem::separator(),
            &PredefinedMenuItem::close_window(Some("Close Window")),
        ])?;

        let edit = Submenu::new("Edit", true);
        edit.append_items(&[
            &item(Command::Undo, "Undo", acc(cmd, Code::KeyZ)),
            &item(Command::Redo, "Redo", acc(cmd_shift, Code::KeyZ)),
            &PredefinedMenuItem::separator(),
            &PredefinedMenuItem::cut(None),
            &PredefinedMenuItem::copy(None),
            &PredefinedMenuItem::paste(None),
            &PredefinedMenuItem::separator(),
            &item(Command::SelectAll, "Select All", acc(cmd, Code::KeyA)),
            &item(
                Command::SelectNone,
                "Deselect All",
                acc(cmd_shift, Code::KeyA),
            ),
        ])?;

        let sidebar = CheckMenuItem::with_id(
            Command::ToggleSidebar.menu_id(),
            "Show Sidebar",
            true,
            true,
            acc(ctrl_cmd, Code::KeyS),
        );
        let inspector = CheckMenuItem::with_id(
            Command::ToggleInspector.menu_id(),
            "Show Inspector",
            true,
            true,
            acc(cmd_alt, Code::KeyI),
        );
        let filmstrip = CheckMenuItem::with_id(
            Command::ToggleFilmstrip.menu_id(),
            "Show Filmstrip",
            true,
            false,
            acc(Modifiers::SHIFT, Code::KeyF),
        );
        let sort = Submenu::new("Sort By", true);
        for so in crate::filter::Sort::ALL {
            sort.append(&item(Command::SetSort(so), &capitalize(so.label()), None))?;
        }
        sort.append_items(&[
            &PredefinedMenuItem::separator(),
            &item(Command::ToggleSortReversed, "Reverse Order", None),
        ])?;
        let view = Submenu::new("View", true);
        view.append_items(&[
            &item(
                Command::SetWorkspace(Workspace::Sheet),
                "Contact Sheet",
                acc(cmd_shift, Code::Digit1),
            ),
            &item(
                Command::SetWorkspace(Workspace::Preview),
                "Preview",
                acc(cmd_shift, Code::Digit2),
            ),
            &item(
                Command::SetWorkspace(Workspace::Caption),
                "Caption",
                acc(cmd_shift, Code::Digit3),
            ),
            &item(
                Command::SetWorkspace(Workspace::Ingest),
                "Ingest",
                acc(cmd_shift, Code::Digit4),
            ),
            &item(
                Command::SetWorkspace(Workspace::Export),
                "Export",
                acc(cmd_shift, Code::Digit5),
            ),
            &PredefinedMenuItem::separator(),
            &sidebar,
            &inspector,
            &filmstrip,
            &PredefinedMenuItem::separator(),
            &sort,
            &item(
                Command::ClearFilter(None),
                "Clear Filters",
                acc(cmd_alt, Code::KeyF),
            ),
            &PredefinedMenuItem::separator(),
            &item(Command::ZoomIn, "Larger Thumbnails", acc(cmd, Code::Equal)),
            &item(
                Command::ZoomOut,
                "Smaller Thumbnails",
                acc(cmd, Code::Minus),
            ),
            &PredefinedMenuItem::separator(),
            &item(
                Command::ToggleZoom,
                "Zoom 1:1 / Fit",
                Some(Accelerator::new(Modifiers::empty(), Code::KeyZ)),
            ),
            &item(
                Command::ToggleCompare,
                "Compare Selected",
                Some(Accelerator::new(Modifiers::empty(), Code::KeyC)),
            ),
            &PredefinedMenuItem::separator(),
            &PredefinedMenuItem::fullscreen(None),
        ])?;

        let class = Submenu::new("Color Class", true);
        let names = apature_core::ColorClassScheme::default().names;
        let mut class_items = Vec::new();
        for (i, n) in names.iter().enumerate() {
            let code = [
                Code::Digit1,
                Code::Digit2,
                Code::Digit3,
                Code::Digit4,
                Code::Digit5,
                Code::Digit6,
                Code::Digit7,
                Code::Digit8,
            ][i];
            let it = item(
                Command::SetClass(i as u8 + 1),
                &format!("{} {n}", i + 1),
                Some(Accelerator::new(Modifiers::empty(), code)),
            );
            class.append(&it)?;
            class_items.push(it);
        }
        class.append_items(&[
            &PredefinedMenuItem::separator(),
            &item(
                Command::SetClass(0),
                "None",
                Some(Accelerator::new(Modifiers::empty(), Code::Digit0)),
            ),
        ])?;
        let rating = Submenu::new("Rating", true);
        for r in (1..=5).rev() {
            let code = [
                Code::Digit0,
                Code::Digit1,
                Code::Digit2,
                Code::Digit3,
                Code::Digit4,
                Code::Digit5,
            ][r as usize];
            rating.append(&item(
                Command::SetRating(r),
                &format!(
                    "{} {}",
                    "★".repeat(r as usize),
                    if r == 1 { "Star" } else { "Stars" }
                ),
                acc(cmd, code),
            ))?;
        }
        rating.append_items(&[
            &PredefinedMenuItem::separator(),
            &item(Command::SetRating(0), "Unrated", acc(cmd, Code::Digit0)),
            &item(
                Command::SetRating(-1),
                "Reject",
                Some(Accelerator::new(Modifiers::empty(), Code::KeyX)),
            ),
        ])?;
        let image = Submenu::new("Image", true);
        let advance = CheckMenuItem::with_id(
            Command::ToggleAdvanceAfterMark.menu_id(),
            "Advance After Marking",
            true,
            true,
            None,
        );
        image.append_items(&[
            &class,
            &rating,
            &PredefinedMenuItem::separator(),
            &item(
                Command::ToggleTag,
                "Toggle Tag",
                Some(Accelerator::new(Modifiers::empty(), Code::KeyT)),
            ),
            &PredefinedMenuItem::separator(),
            &item(
                Command::ApplyStationery,
                "Apply Assignment Pad to Selection",
                None,
            ),
            &item(Command::SaveCaption, "Save Caption", None),
            &PredefinedMenuItem::separator(),
            &advance,
        ])?;

        let window = Submenu::new("Window", true);
        window.append_items(&[
            &PredefinedMenuItem::minimize(None),
            &PredefinedMenuItem::maximize(None),
            &PredefinedMenuItem::separator(),
            &item(
                Command::NextTab,
                "Show Next Tab",
                acc(cmd_shift, Code::BracketRight),
            ),
            &item(
                Command::PrevTab,
                "Show Previous Tab",
                acc(cmd_shift, Code::BracketLeft),
            ),
        ])?;
        let help = Submenu::new("Help", true);
        help.append(&disabled("Apature Help (M6)"))?;

        menu.append_items(&[&app, &file, &edit, &view, &image, &window, &help])?;
        Ok(MenuBar {
            menu,
            window,
            help,
            installed: false,
            sidebar,
            inspector,
            filmstrip,
            advance,
            class_items,
        })
    }

    /// Attach to NSApp. Must run after the application has finished launching,
    /// or winit's default menu replaces it; the app calls this on its first frame.
    pub fn install_if_needed(&mut self) {
        if self.installed {
            return;
        }
        self.menu.init_for_nsapp();
        self.window.set_as_windows_menu_for_nsapp();
        self.help.set_as_help_menu_for_nsapp();
        self.installed = true;
    }

    pub fn set_class_names(&self, names: &[String; 8]) {
        for (i, it) in self.class_items.iter().enumerate() {
            it.set_text(format!("{} {}", i + 1, names[i]));
        }
    }

    pub fn drain(&self) -> Vec<Command> {
        let mut out = Vec::new();
        while let Ok(ev) = MenuEvent::receiver().try_recv() {
            if let Some(c) = Command::from_menu_id(ev.id().as_ref()) {
                out.push(c);
            }
        }
        out
    }

    pub fn sync_toggles(&self, sidebar: bool, inspector: bool, filmstrip: bool, advance: bool) {
        self.sidebar.set_checked(sidebar);
        self.inspector.set_checked(inspector);
        self.filmstrip.set_checked(filmstrip);
        self.advance.set_checked(advance);
    }
}

fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        None => String::new(),
    }
}

/// `defaults read com.apple.universalaccess reduceMotion`, once at startup.
pub fn reduce_motion() -> bool {
    std::process::Command::new("defaults")
        .args(["read", "com.apple.universalaccess", "reduceMotion"])
        .output()
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim() == "1")
        .unwrap_or(false)
}
