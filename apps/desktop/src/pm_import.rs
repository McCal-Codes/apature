//! Import from Photo Mechanic: the parts of its files Apature understands.
//!
//! - An ingest snapshot (`.snap`, XML with a tab-separated `SerializedState`)
//!   gives the destination folder template and the rename template. Photo
//!   Mechanic's variable names (`{iptcyear4}`, `{iyr2}`, `{head}`, `{frame}`)
//!   are accepted by Apature's template engine as they are, so the strings
//!   are taken verbatim.
//! - A code replacement file (`code<TAB>replacement` lines) is used as is.
//!
//! - The IPTC stationery pad inside an ingest snapshot becomes an Apature
//!   assignment file (city, state, credit, source, copyright, creator,
//!   keywords), saved beside the settings and loaded.

use std::path::Path;

#[derive(Debug, Default)]
pub struct Imported {
    /// (folder template, rename template) from an ingest snapshot.
    pub ingest: Option<(String, String)>,
    /// The file is a code replacement table.
    pub codes: bool,
    /// The IPTC stationery pad inside a snapshot, as Apature assignment-file
    /// text (`key: value` lines), when it had anything in it.
    pub assignment: Option<String>,
}

/// One attribute or element value out of the (HTML-escaped) XMP pad.
fn xmp_attr(xmp: &str, name: &str) -> Option<String> {
    let key = format!("{name}=\"");
    let i = xmp.find(&key)? + key.len();
    let rest = &xmp[i..];
    let end = rest.find('"')?;
    let v = unescape(&rest[..end]);
    (!v.trim().is_empty()).then(|| v.trim().to_string())
}

/// The `rdf:li` items under an element such as `dc:subject`.
fn xmp_list(xmp: &str, name: &str) -> Vec<String> {
    let open = format!("<{name}>");
    let close = format!("</{name}>");
    let Some(i) = xmp.find(&open) else {
        return Vec::new();
    };
    let rest = &xmp[i..];
    let Some(j) = rest.find(&close) else {
        return Vec::new();
    };
    let block = &rest[..j];
    let mut out = Vec::new();
    let mut r = block;
    while let Some(a) = r.find("<rdf:li") {
        let after = &r[a..];
        let Some(gt) = after.find('>') else { break };
        let body = &after[gt + 1..];
        let Some(e) = body.find("</rdf:li>") else {
            break;
        };
        let v = unescape(body[..e].trim());
        if !v.is_empty() {
            out.push(v);
        }
        r = &body[e + 9..];
    }
    out
}

fn unescape(s: &str) -> String {
    s.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&#39;", "'")
        .replace("&amp;", "&")
}

/// The stationery pad of a snapshot as assignment-file lines.
fn pad_to_assignment(text: &str) -> Option<String> {
    let pad = text
        .split("<IPTCStationeryPadData>")
        .nth(1)
        .and_then(|s| s.split("</IPTCStationeryPadData>").next())?;
    let xmp = unescape(pad);
    let mut lines: Vec<String> = Vec::new();
    let mut put = |k: &str, v: Option<String>| {
        if let Some(v) = v {
            lines.push(format!("{k}: {v}"));
        }
    };
    put("slug", xmp_attr(&xmp, "photoshop:Headline"));
    put("city", xmp_attr(&xmp, "photoshop:City"));
    put("state", xmp_attr(&xmp, "photoshop:State"));
    put("country", xmp_attr(&xmp, "photoshop:Country"));
    put("country_code", xmp_attr(&xmp, "Iptc4xmpCore:CountryCode"));
    put("venue", xmp_attr(&xmp, "Iptc4xmpCore:Location"));
    put("credit", xmp_attr(&xmp, "photoshop:Credit"));
    put("source", xmp_attr(&xmp, "photoshop:Source"));
    put("creator_title", xmp_attr(&xmp, "photoshop:AuthorsPosition"));
    put("caption_writer", xmp_attr(&xmp, "photoshop:CaptionWriter"));
    let creators = xmp_list(&xmp, "dc:creator");
    put("creator", creators.first().cloned());
    put("copyright", xmp_list(&xmp, "dc:rights").into_iter().next());
    let kw = xmp_list(&xmp, "dc:subject");
    if !kw.is_empty() {
        lines.push(format!("keywords: {}", kw.join("; ")));
    }
    if lines.is_empty() {
        return None;
    }
    let mut out = String::from("# Imported from a Photo Mechanic snapshot by Apature.\n");
    out.push_str(&lines.join("\n"));
    out.push('\n');
    Some(out)
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
        got.assignment = pad_to_assignment(&text);
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

    #[test]
    fn reads_the_stationery_pad_as_an_assignment() {
        let snap = "<IngestSettings><SerializedState>IngestRenameEdit\t{frame}</SerializedState><IPTCStationeryPadData>&lt;rdf:Description photoshop:City=\"Sewickley\" photoshop:State=\"PA\" photoshop:Credit=\"Caleb McCartney\" photoshop:Source=\"The Globe\"&gt;&lt;dc:creator&gt;&lt;rdf:Seq&gt;&lt;rdf:li&gt;Caleb McCartney&lt;/rdf:li&gt;&lt;/rdf:Seq&gt;&lt;/dc:creator&gt;&lt;dc:subject&gt;&lt;rdf:Bag&gt;&lt;rdf:li&gt;news&lt;/rdf:li&gt;&lt;rdf:li&gt;Sewickley&lt;/rdf:li&gt;&lt;/rdf:Bag&gt;&lt;/dc:subject&gt;&lt;/rdf:Description&gt;</IPTCStationeryPadData></IngestSettings>";
        let a = pad_to_assignment(snap).unwrap();
        assert!(a.contains("city: Sewickley"));
        assert!(a.contains("credit: Caleb McCartney"));
        assert!(a.contains("creator: Caleb McCartney"));
        assert!(a.contains("keywords: news; Sewickley"));
    }
}
