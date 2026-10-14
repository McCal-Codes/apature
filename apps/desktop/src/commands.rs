//! Every user action is a `Command`, whether it came from the menu bar, a
//! key, a toolbar button or a sidebar row. One dispatcher, one keymap.

use std::path::PathBuf;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Workspace {
    Sheet,
    Preview,
    Caption,
    Ingest,
}

impl Workspace {
    pub fn label(self) -> &'static str {
        match self {
            Workspace::Sheet => "Sheet",
            Workspace::Preview => "Preview",
            Workspace::Caption => "Caption",
            Workspace::Ingest => "Ingest",
        }
    }
    /// Workspaces that exist today. The others are shown disabled with the
    /// milestone that brings them, never as controls that do nothing.
    pub fn available(self) -> Result<(), &'static str> {
        match self {
            Workspace::Sheet | Workspace::Preview => Ok(()),
            Workspace::Caption => Err("Captions arrive in M4"),
            Workspace::Ingest => Err("Ingest arrives in M5"),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Command {
    OpenFolder(PathBuf),
    /// Click on a cell with modifiers.
    Click {
        idx: usize,
        shift: bool,
        command: bool,
    },
    /// Keyboard focus move by `delta` positions in the view, with modifiers.
    MoveFocus {
        delta: isize,
        shift: bool,
        command: bool,
    },
    SelectAll,
    SelectNone,
    Undo,
    Redo,
    SetFilter(crate::filter::Filter),
    ClearFilter(Option<crate::filter::Clause>),
    SetSort(crate::filter::Sort),
    ToggleSortReversed,
    ToggleAdvanceAfterMark,
    PickFolder,
    AddFavorite,
    RemoveFavorite(PathBuf),
    RevealInFinder,
    ToggleSidebar,
    ToggleInspector,
    ToggleFilmstrip,
    SetWorkspace(Workspace),
    ZoomIn,
    ZoomOut,
    Move(isize),
    MovePage(isize),
    Home,
    End,
    SetRating(i8),
    SetClass(u8),
    ToggleTag,
    Escape,
    Activate,
}

impl Command {
    /// Menu identifiers are strings so the native menu can round-trip them.
    pub fn menu_id(&self) -> String {
        match self {
            Command::PickFolder => "file.open".into(),
            Command::AddFavorite => "file.favorite".into(),
            Command::RevealInFinder => "file.reveal".into(),
            Command::ToggleSidebar => "view.sidebar".into(),
            Command::ToggleInspector => "view.inspector".into(),
            Command::ToggleFilmstrip => "view.filmstrip".into(),
            Command::SetWorkspace(w) => format!("view.workspace.{}", w.label().to_lowercase()),
            Command::SelectAll => "edit.selectall".into(),
            Command::SelectNone => "edit.selectnone".into(),
            Command::Undo => "edit.undo".into(),
            Command::Redo => "edit.redo".into(),
            Command::ClearFilter(None) => "view.clearfilter".into(),
            Command::SetSort(s) => format!("view.sort.{}", s.label().replace(' ', "")),
            Command::ToggleSortReversed => "view.sortreversed".into(),
            Command::ToggleAdvanceAfterMark => "image.advance".into(),
            Command::ZoomIn => "view.zoomin".into(),
            Command::ZoomOut => "view.zoomout".into(),
            Command::SetRating(r) => format!("image.rating.{r}"),
            Command::SetClass(c) => format!("image.class.{c}"),
            Command::ToggleTag => "image.tag".into(),
            other => format!("other.{other:?}"),
        }
    }

    pub fn from_menu_id(id: &str) -> Option<Command> {
        Some(match id {
            "file.open" => Command::PickFolder,
            "file.favorite" => Command::AddFavorite,
            "file.reveal" => Command::RevealInFinder,
            "view.sidebar" => Command::ToggleSidebar,
            "view.inspector" => Command::ToggleInspector,
            "view.filmstrip" => Command::ToggleFilmstrip,
            "view.workspace.sheet" => Command::SetWorkspace(Workspace::Sheet),
            "view.workspace.preview" => Command::SetWorkspace(Workspace::Preview),
            "view.workspace.caption" => Command::SetWorkspace(Workspace::Caption),
            "view.workspace.ingest" => Command::SetWorkspace(Workspace::Ingest),
            "edit.selectall" => Command::SelectAll,
            "edit.selectnone" => Command::SelectNone,
            "edit.undo" => Command::Undo,
            "edit.redo" => Command::Redo,
            "view.clearfilter" => Command::ClearFilter(None),
            "view.sortreversed" => Command::ToggleSortReversed,
            "image.advance" => Command::ToggleAdvanceAfterMark,
            "view.zoomin" => Command::ZoomIn,
            "view.zoomout" => Command::ZoomOut,
            "image.tag" => Command::ToggleTag,
            _ => {
                if let Some(k) = id.strip_prefix("view.sort.") {
                    Command::SetSort(
                        *crate::filter::Sort::ALL
                            .iter()
                            .find(|s| s.label().replace(' ', "") == k)?,
                    )
                } else if let Some(r) = id.strip_prefix("image.rating.") {
                    Command::SetRating(r.parse().ok()?)
                } else if let Some(c) = id.strip_prefix("image.class.") {
                    Command::SetClass(c.parse().ok()?)
                } else {
                    return None;
                }
            }
        })
    }
}
