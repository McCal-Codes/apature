use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// What kind of file a variant is. Only the kinds we can show previews for
/// are scanned; everything else is ignored.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Kind {
    Raw,
    Jpeg,
    Heic,
}

/// Map a file extension (any case) to a [`Kind`]. Unknown extensions return `None`.
pub fn kind_for_extension(ext: &str) -> Option<Kind> {
    match ext.to_ascii_lowercase().as_str() {
        "nef" | "nrw" | "cr2" | "cr3" | "arw" | "dng" | "raf" | "orf" | "rw2" | "pef" | "srw"
        | "3fr" | "iiq" => Some(Kind::Raw),
        "jpg" | "jpeg" | "jpe" => Some(Kind::Jpeg),
        "heic" | "heif" | "hif" => Some(Kind::Heic),
        _ => None,
    }
}

/// One physical file on disk.
#[derive(Clone, Debug)]
pub struct Variant {
    pub path: PathBuf,
    pub kind: Kind,
    pub size: u64,
    pub modified: Option<SystemTime>,
}

impl Variant {
    pub fn file_name(&self) -> String {
        self.path
            .file_name()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default()
    }
}

/// A logical photo: a RAW+JPEG pair collapses into one item, the way Photo
/// Mechanic shows a single contact-sheet cell for `DSC_0001.NEF` + `DSC_0001.JPG`.
///
/// `primary` is the RAW when one exists, otherwise the JPEG/HEIC. `secondary`
/// holds the other variants with the same stem in the same directory.
#[derive(Clone, Debug)]
pub struct PhotoItem {
    pub id: usize,
    pub stem: String,
    pub primary: Variant,
    pub secondary: Vec<Variant>,
}

impl PhotoItem {
    pub fn path(&self) -> &Path {
        &self.primary.path
    }

    pub fn display_name(&self) -> String {
        self.primary.file_name()
    }

    pub fn is_pair(&self) -> bool {
        !self.secondary.is_empty()
    }

    /// Where the XMP sidecar for this item lives: same directory, same stem,
    /// `.xmp` extension. This matches Photo Mechanic and Lightroom.
    pub fn sidecar_path(&self) -> PathBuf {
        self.primary.path.with_extension("xmp")
    }

    pub fn variants(&self) -> impl Iterator<Item = &Variant> {
        std::iter::once(&self.primary).chain(self.secondary.iter())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extensions_map_to_kinds() {
        assert_eq!(kind_for_extension("NEF"), Some(Kind::Raw));
        assert_eq!(kind_for_extension("jpg"), Some(Kind::Jpeg));
        assert_eq!(kind_for_extension("Jpeg"), Some(Kind::Jpeg));
        assert_eq!(kind_for_extension("heic"), Some(Kind::Heic));
        assert_eq!(kind_for_extension("xmp"), None);
        assert_eq!(kind_for_extension("txt"), None);
    }
}
