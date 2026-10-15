//! Every user action is a `Command`, whether it came from the menu bar, a
//! key, a toolbar button or a sidebar row. One dispatcher, one keymap.

use std::path::PathBuf;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Workspace {
    Sheet,
    Preview,
    Caption,
    Ingest,
    Export,
}

impl Workspace {
    pub fn label(self) -> &'static str {
        match self {
            Workspace::Sheet => "Sheet",
            Workspace::Preview => "Preview",
            Workspace::Caption => "Caption",
            Workspace::Ingest => "Ingest",
            Workspace::Export => "Export",
        }
    }
    /// Workspaces that exist today. The others are shown disabled with the
    /// milestone that brings them, never as controls that do nothing.
    pub fn available(self) -> Result<(), &'static str> {
        match self {
            Workspace::Sheet
            | Workspace::Preview
            | Workspace::Caption
            | Workspace::Ingest
            | Workspace::Export => Ok(()),
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
    LoadAssignment,
    LoadCodes,
    ApplyStationery,
    InsertCaptionTemplate,
    SaveCaption,
    SaveCaptionAndNext,
    RevertCaption,
    IngestChooseSource,
    IngestChoosePrimary,
    IngestChooseSecondary,
    IngestStart,
    IngestCancel,
    IngestOpenResult,
    /// Unmount the ingested card (after a verified ingest).
    IngestEject,
    /// Copy the focused photo's marks and IPTC; paste onto the selection.
    CopyMetadata,
    PasteMetadata,
    ExportChooseDest,
    ExportStart,
    ExportCancel,
    ExportOpenResult,
    /// Preview: toggle 1:1 and fit.
    ToggleZoom,
    /// File ops: open the Rename dialog, apply it, open the Trash dialog, confirm it.
    RenamePhotos,
    RenameApply,
    MoveToTrash,
    TrashConfirm,
    CloseDialog,
    /// Preview: compare the selection side by side.
    ToggleCompare,
    CloseTab,
    NextTab,
    PrevTab,
    SelectTab(usize),
    OpenSettings,
    ClearThumbCache,
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
    /// A quarter turn, losslessly, in the sidecar (`true` = clockwise).
    Rotate(bool),
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
            Command::LoadAssignment => "file.assignment".into(),
            Command::LoadCodes => "file.codes".into(),
            Command::ApplyStationery => "image.stationery".into(),
            Command::SaveCaption => "image.savecaption".into(),
            Command::IngestStart => "file.ingest".into(),
            Command::CopyMetadata => "edit.copymeta".into(),
            Command::PasteMetadata => "edit.pastemeta".into(),
            Command::ExportStart => "file.export".into(),
            Command::RenamePhotos => "file.rename".into(),
            Command::MoveToTrash => "file.trash".into(),
            Command::CloseTab => "file.closetab".into(),
            Command::NextTab => "window.nexttab".into(),
            Command::PrevTab => "window.prevtab".into(),
            Command::OpenSettings => "app.settings".into(),
            Command::ZoomIn => "view.zoomin".into(),
            Command::ZoomOut => "view.zoomout".into(),
            Command::ToggleZoom => "view.zoom11".into(),
            Command::ToggleCompare => "view.compare".into(),
            Command::SetRating(r) => format!("image.rating.{r}"),
            Command::SetClass(c) => format!("image.class.{c}"),
            Command::ToggleTag => "image.tag".into(),
            Command::Rotate(true) => "image.rotate.cw".into(),
            Command::Rotate(false) => "image.rotate.ccw".into(),
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
            "file.assignment" => Command::LoadAssignment,
            "file.codes" => Command::LoadCodes,
            "image.stationery" => Command::ApplyStationery,
            "image.savecaption" => Command::SaveCaption,
            "file.ingest" => Command::SetWorkspace(Workspace::Ingest),
            "edit.copymeta" => Command::CopyMetadata,
            "edit.pastemeta" => Command::PasteMetadata,
            "file.export" => Command::SetWorkspace(Workspace::Export),
            "file.rename" => Command::RenamePhotos,
            "file.trash" => Command::MoveToTrash,
            "view.workspace.export" => Command::SetWorkspace(Workspace::Export),
            "file.closetab" => Command::CloseTab,
            "window.nexttab" => Command::NextTab,
            "window.prevtab" => Command::PrevTab,
            "app.settings" => Command::OpenSettings,
            "view.zoomin" => Command::ZoomIn,
            "view.zoomout" => Command::ZoomOut,
            "view.zoom11" => Command::ToggleZoom,
            "view.compare" => Command::ToggleCompare,
            "image.tag" => Command::ToggleTag,
            "image.rotate.cw" => Command::Rotate(true),
            "image.rotate.ccw" => Command::Rotate(false),
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
