//! Import from Photo Mechanic: the parts of its files Apature understands.
//!
//! - An ingest snapshot (`.snap`, XML with a tab-separated `SerializedState`)
//!   gives the destination folder template and the rename template. Photo
//!   Mechanic's variable names (`{iptcyear4}`, `{iyr2}`, `{head}`, `{frame}`)
//!   are accepted by Apature's template engine as they are, so the strings
//!   are taken verbatim.
//! - A code replacement file (`code<TAB>replacement` lines) is used as is.
//!
//! The IPTC stationery pad inside a snapshot is not imported yet; it maps
//! onto an assignment file, which is a later step.

use std::path::Path;

#[derive(Debug, Default)]
pub struct Imported {
    /// (folder template, rename template) from an ingest snapshot.
    pub ingest: Option<(String, String)>,
    /// The file is a code replacement table.
    pub codes: bool,
}

pub fn import(path: &Path) -> Result<Imported, String> {
    let text = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    let mut got = Imported::default();
    if text.contains("<IngestSettings>") || text.contains("<SerializedState>") {
        let state = text
            .split("<SerializedState>")
            .nth(1)
            .and_then(|s| s.split("</SerializedState>").next())
            .ok_or("no SerializedState in the snapshot")?;
        let fields: Vec<&str> = state.split('\t').collect();
        let get = |key: &str| {
            fields
                .iter()
                .position(|f| *f == key)
                .and_then(|i| fields.get(i + 1))
                .map(|v| v.trim().to_string())
                .filter(|v| !v.is_empty())
        };
        let folder = get("IngestDestFolderNameString");
        let rename = get("IngestRenameEdit");
        if folder.is_none() && rename.is_none() {
            return Err("the snapshot has no folder or rename template".into());
        }
        got.ingest = Some((
            folder
                .unwrap_or_else(|| "{year4}/{month0} {monthname}/{mnn3} {day0} {headline}".into()),
            rename.unwrap_or_else(|| "{filename}".into()),
        ));
        return Ok(got);
    }
    let table = apature_metadata::codes::CodeTable::parse(&text);
    if !table.codes.is_empty() {
        got.codes = true;
        return Ok(got);
    }
    Err("not an ingest snapshot or a code replacement file".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_templates_from_an_ingest_snapshot() {
        let dir = std::env::temp_dir().join(format!("pm-import-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let p = dir.join("ingest.snap");
        std::fs::write(&p, "<?xml version=\"1.0\"?>\n<IngestSettings>\n  <SerializedState>IngestDisks\t1\tIngestDestFolderNameString\t{iptcyear4}/{iptcmonth0} {iptcmonthname}/{iptcmonthname3} {iptcday0} {head}\tIngestRenameCheck\t1\tIngestRenameEdit\t{iyr2}{imn0}{idy0}{head}_CAL{frame}\tIngestUseSeqnCheck\t0</SerializedState>\n</IngestSettings>\n").unwrap();
        let got = import(&p).unwrap();
        let (folder, rename) = got.ingest.unwrap();
        assert_eq!(
            folder,
            "{iptcyear4}/{iptcmonth0} {iptcmonthname}/{iptcmonthname3} {iptcday0} {head}"
        );
        assert_eq!(rename, "{iyr2}{imn0}{idy0}{head}_CAL{frame}");
        // and the engine expands those names as Photo Mechanic would
        let f = apature_ingest::template::Facts {
            stem: "DSC_0500".into(),
            ext: "NEF".into(),
            date: Some((2026, 5, 5)),
            headline: Some("groundbreak".into()),
            ..Default::default()
        };
        assert_eq!(
            apature_ingest::template::expand(&rename, &f, 1),
            "260505groundbreak_CAL0500"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }
}
