//! `apature-cli`: the engine without the window. Useful for testing format support
//! and for benchmarking without GUI overhead.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use rayon::prelude::*;

use apature_core::{ColorClass, ColorClassScheme, Marks, Rating, ScanOptions};
use apature_metadata::items;
use apature_preview::{Source, ThumbCache};

#[derive(Parser)]
#[command(name = "apature-cli", version, about = "Apature command line")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// List the photos in a folder with RAW+JPEG pairing and sidecar marks.
    Scan {
        dir: PathBuf,
        #[arg(long)]
        recursive: bool,
    },
    /// List the JPEG previews embedded in a file.
    Previews { file: PathBuf },
    /// Write the largest embedded preview out as a JPEG without re-encoding.
    Extract {
        file: PathBuf,
        #[arg(short, long)]
        out: PathBuf,
    },
    /// Print the EXIF summary for a file.
    Exif { file: PathBuf },
    /// Generate thumbnails for a whole folder and report timings.
    Bench {
        dir: PathBuf,
        /// Long side of the generated thumbnail in pixels.
        #[arg(long, default_value_t = 320)]
        size: u32,
        /// Use (and populate) the on-disk thumbnail cache.
        #[arg(long)]
        cache: bool,
        /// Worker threads (default: all cores).
        #[arg(long)]
        threads: Option<usize>,
        /// Include subfolders.
        #[arg(long)]
        recursive: bool,
    },
    /// Set rating / colour class / tag on a file's XMP sidecar.
    Mark {
        file: PathBuf,
        #[arg(long)]
        rating: Option<i8>,
        #[arg(long)]
        class: Option<u8>,
        #[arg(long)]
        tag: Option<bool>,
    },
    /// Verified copy of one file (refuses to overwrite).
    Copy { src: PathBuf, dst: PathBuf },
    /// Apply an assignment file's IPTC pad (and nothing else) to photos.
    Pad {
        /// The key: value assignment file.
        #[arg(long)]
        assignment: PathBuf,
        files: Vec<PathBuf>,
    },
    /// Set the caption (and optionally headline/keywords) on photos, expanding
    /// \code\ replacements from the assignment.
    Caption {
        #[arg(long)]
        assignment: Option<PathBuf>,
        #[arg(long)]
        text: String,
        #[arg(long)]
        headline: Option<String>,
        /// Semicolon-separated.
        #[arg(long)]
        keywords: Option<String>,
        files: Vec<PathBuf>,
    },
    /// Print the IPTC fields of a photo.
    Iptc { file: PathBuf },
    /// Plan and run a verified ingest (sequential; never deletes the source).
    Ingest {
        #[arg(long)]
        source: PathBuf,
        #[arg(long)]
        primary: PathBuf,
        #[arg(long)]
        secondary: Option<PathBuf>,
        /// Folder template under each destination, e.g. "{date}-{headline}".
        #[arg(long, default_value = "{date}-{headline}")]
        folder: String,
        /// File name template, e.g. "{headline}-{seqn}" (empty keeps names).
        #[arg(long, default_value = "")]
        rename: String,
        #[arg(long)]
        headline: Option<String>,
        /// Copy everything, ignoring the "already ingested" ledger.
        #[arg(long)]
        all: bool,
        /// Only print the plan.
        #[arg(long)]
        dry_run: bool,
    },
    /// Print where the thumbnail cache lives.
    CacheDir,
}

fn main() -> Result<()> {
    match Cli::parse().cmd {
        Cmd::Scan { dir, recursive } => scan(&dir, recursive),
        Cmd::Previews { file } => previews(&file),
        Cmd::Extract { file, out } => extract(&file, &out),
        Cmd::Exif { file } => exif(&file),
        Cmd::Bench {
            dir,
            size,
            cache,
            threads,
            recursive,
        } => bench(&dir, size, cache, threads, recursive),
        Cmd::Mark {
            file,
            rating,
            class,
            tag,
        } => mark(&file, rating, class, tag),
        Cmd::Copy { src, dst } => {
            let r = apature_ingest::copy_verified(&src, &dst)?;
            println!(
                "copied {} bytes to {} (sha256 {})",
                r.bytes,
                r.destination.display(),
                r.sha256
            );
            Ok(())
        }
        Cmd::Pad { assignment, files } => pad(&assignment, &files),
        Cmd::Caption {
            assignment,
            text,
            headline,
            keywords,
            files,
        } => caption(assignment.as_deref(), &text, headline, keywords, &files),
        Cmd::Ingest {
            source,
            primary,
            secondary,
            folder,
            rename,
            headline,
            all,
            dry_run,
        } => ingest(
            source, primary, secondary, folder, rename, headline, all, dry_run,
        ),
        Cmd::Iptc { file } => {
            let item = item_for(&file)?;
            let i = items::read_iptc(&item);
            println!("{}", serde_json::to_string_pretty(&i)?);
            Ok(())
        }
        Cmd::CacheDir => {
            println!("{}", cache_dir()?.display());
            Ok(())
        }
    }
}

fn cache_dir() -> Result<PathBuf> {
    let dirs =
        directories::ProjectDirs::from("dev", "mccal", "Apature").context("no home directory")?;
    Ok(dirs.cache_dir().join("thumbs"))
}

fn scan(dir: &Path, recursive: bool) -> Result<()> {
    let t0 = Instant::now();
    let items = apature_core::scan(dir, &ScanOptions { recursive })?;
    let scheme = ColorClassScheme::default();
    for it in &items {
        let src = items::read(it);
        let m = src.effective(it, &scheme);
        let pair = if it.is_pair() {
            format!(
                " +{}",
                it.secondary
                    .iter()
                    .map(|v| v.file_name())
                    .collect::<Vec<_>>()
                    .join(",")
            )
        } else {
            String::new()
        };
        let has_any = |m: &apature_metadata::XmpMarks| {
            m.rating.is_some() || m.label.is_some() || m.color_class.is_some() || m.tagged.is_some()
        };
        let marked = src.sidecar.as_ref().is_some_and(has_any)
            || src.embedded.iter().any(|(_, m)| has_any(m));
        let mtext = if marked {
            format!(
                "  rating={} class={} tagged={}",
                m.rating.0, m.color_class.0, m.tagged
            )
        } else {
            String::new()
        };
        println!("{:>5}  {}{}{}", it.id, it.display_name(), pair, mtext);
        for (p, c) in src.conflicts() {
            println!(
                "       CONFLICT {}: sidecar={} embedded({})={}",
                c.field,
                c.sidecar,
                p.file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_default(),
                c.embedded
            );
        }
    }
    eprintln!(
        "{} items in {:.1} ms",
        items.len(),
        t0.elapsed().as_secs_f64() * 1000.0
    );
    Ok(())
}

fn previews(file: &Path) -> Result<()> {
    let t0 = Instant::now();
    let src = Source::open(file)?;
    let dt = t0.elapsed();
    if src.candidates().is_empty() {
        bail!("no embedded JPEG found in {}", file.display());
    }
    println!(
        "{} ({} bytes), indexed in {:.2} ms",
        file.display(),
        src.len(),
        dt.as_secs_f64() * 1000.0
    );
    for c in src.candidates() {
        println!(
            "  {:>5}x{:<5} {:>9} bytes @ {:<10} {}",
            c.width, c.height, c.length, c.offset, c.source
        );
    }
    Ok(())
}

fn extract(file: &Path, out: &Path) -> Result<()> {
    let src = Source::open(file)?;
    let c = src.largest().context("no embedded JPEG")?;
    std::fs::write(out, src.bytes(c))?;
    println!(
        "wrote {}x{} ({} bytes) from {} to {}",
        c.width,
        c.height,
        c.length,
        c.source,
        out.display()
    );
    Ok(())
}

fn exif(file: &Path) -> Result<()> {
    let s = apature_metadata::exif::read(file)?;
    println!("camera:      {}", s.camera_line());
    println!("lens:        {}", s.lens.as_deref().unwrap_or("-"));
    println!(
        "captured:    {}",
        s.date_time_original.as_deref().unwrap_or("-")
    );
    println!("exposure:    {}", s.exposure_line());
    println!("orientation: {}", s.orientation);
    Ok(())
}

fn bench(
    dir: &Path,
    size: u32,
    use_cache: bool,
    threads: Option<usize>,
    recursive: bool,
) -> Result<()> {
    if let Some(n) = threads {
        rayon::ThreadPoolBuilder::new()
            .num_threads(n)
            .build_global()
            .ok();
    }
    let t0 = Instant::now();
    let items = apature_core::scan(dir, &ScanOptions { recursive })?;
    let scan_ms = t0.elapsed().as_secs_f64() * 1000.0;
    let cache = if use_cache {
        Some(ThumbCache::new(cache_dir()?)?)
    } else {
        None
    };
    let hits = AtomicUsize::new(0);
    let errors = AtomicUsize::new(0);

    let t1 = Instant::now();
    let mut times: Vec<f64> = items
        .par_iter()
        .map(|it| {
            let t = Instant::now();
            let key = ThumbCache::key(it.path(), it.primary.size, it.primary.modified, size);
            if let Some(c) = &cache {
                if c.get(&key).is_some() {
                    hits.fetch_add(1, Ordering::Relaxed);
                    return t.elapsed().as_secs_f64() * 1000.0;
                }
            }
            let result = (|| -> Result<()> {
                let src = Source::open(it.path())?;
                let orientation = apature_metadata::exif::from_bytes(src.data())
                    .map(|e| e.orientation)
                    .unwrap_or(1);
                let rgb = src.decode_for(size * 2)?;
                let rgb = apature_preview::rotate(rgb, orientation);
                let thumb = apature_preview::thumbnail(&rgb, size);
                if let Some(c) = &cache {
                    c.put(&key, &thumb)?;
                }
                Ok(())
            })();
            if let Err(e) = result {
                errors.fetch_add(1, Ordering::Relaxed);
                eprintln!("{}: {e}", it.display_name());
            }
            t.elapsed().as_secs_f64() * 1000.0
        })
        .collect();
    let total = t1.elapsed().as_secs_f64();
    times.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let pct = |p: f64| {
        times
            .get(((times.len() as f64 * p) as usize).min(times.len().saturating_sub(1)))
            .copied()
            .unwrap_or(0.0)
    };
    println!("folder:     {}", dir.display());
    println!("items:      {}  (scan {scan_ms:.1} ms)", items.len());
    println!("threads:    {}", rayon::current_num_threads());
    println!(
        "thumb size: {size}px  cache: {}",
        if use_cache { "on" } else { "off" }
    );
    println!(
        "wall:       {:.2} s  ({:.1} items/s)",
        total,
        items.len() as f64 / total.max(1e-9)
    );
    println!(
        "per item:   p50 {:.1} ms  p95 {:.1} ms  max {:.1} ms",
        pct(0.5),
        pct(0.95),
        times.last().copied().unwrap_or(0.0)
    );
    println!(
        "cache hits: {}  errors: {}",
        hits.load(Ordering::Relaxed),
        errors.load(Ordering::Relaxed)
    );
    Ok(())
}

/// Resolve a file to its item so a pair gets both the sidecar and the JPEG.
fn item_for(file: &Path) -> Result<apature_core::PhotoItem> {
    let dir = file.parent().context("file has no parent directory")?;
    let stem = file
        .file_stem()
        .context("file has no stem")?
        .to_string_lossy()
        .to_lowercase();
    let items = apature_core::scan(dir, &ScanOptions::default())?;
    items
        .into_iter()
        .find(|it| it.stem.to_lowercase() == stem)
        .with_context(|| format!("{} is not a photo in {}", file.display(), dir.display()))
}

#[allow(clippy::too_many_arguments)]
fn ingest(
    source: PathBuf,
    primary: PathBuf,
    secondary: Option<PathBuf>,
    folder: String,
    rename: String,
    headline: Option<String>,
    all: bool,
    dry_run: bool,
) -> Result<()> {
    use apature_ingest::{build_plan, run, Event, Ledger, Options};
    let ledger_path = directories::ProjectDirs::from("dev", "mccal", "Apature")
        .context("no home directory")?
        .data_dir()
        .join("ingested.json");
    let mut ledger = Ledger::load(&ledger_path);
    let opts = Options {
        source,
        primary,
        secondary,
        folder_template: folder,
        rename_template: rename,
        headline,
        only_new: !all,
        ..Default::default()
    };
    let plan = build_plan(&opts, &ledger)?;
    println!(
        "plan: {} files in {} photos, {:.1} MB, {} skipped, {} collisions",
        plan.entries.len(),
        plan.items,
        plan.total_bytes as f64 / 1e6,
        plan.skipped.len(),
        plan.collisions.len()
    );
    for c in &plan.collisions {
        println!("  COLLISION {} — {}", c.destination.display(), c.reason);
    }
    for e in plan.entries.iter().take(5) {
        println!("  {} → {}", e.source.display(), e.primary.display());
    }
    if dry_run {
        return Ok(());
    }
    if !plan.can_run() {
        bail!("plan cannot run");
    }
    let cancel = std::sync::atomic::AtomicBool::new(false);
    let report = run(&plan, &mut ledger, &cancel, |ev| match ev {
        Event::Verified { destination, .. } => println!("  ✓ {}", destination.display()),
        Event::Failed {
            destination, error, ..
        } => println!("  ✕ {} — {error}", destination.display()),
        _ => {}
    });
    println!(
        "{}: {} ok, {} failed, {:.1} MB in {:.1} s",
        if report.all_verified(&plan) {
            "every file verified at every destination"
        } else {
            "INCOMPLETE"
        },
        report.files_ok(),
        report.files_failed(),
        report.bytes_copied as f64 / 1e6,
        report.elapsed_secs
    );
    Ok(())
}

fn pad(assignment: &Path, files: &[PathBuf]) -> Result<()> {
    let a = apature_metadata::stationery::Assignment::parse(&std::fs::read_to_string(assignment)?)?;
    let fields = a.iptc();
    for f in files {
        let item = item_for(f)?;
        let w = items::write_iptc(&item, &fields)?;
        println!("{}: pad applied → {} file(s)", item.display_name(), w.len());
    }
    Ok(())
}

fn caption(
    assignment: Option<&Path>,
    text: &str,
    headline: Option<String>,
    keywords: Option<String>,
    files: &[PathBuf],
) -> Result<()> {
    use apature_metadata::codes::{self, Vars};
    let a = match assignment {
        Some(p) => Some(apature_metadata::stationery::Assignment::parse(
            &std::fs::read_to_string(p)?,
        )?),
        None => None,
    };
    let codes_table = a.as_ref().map(|a| a.codes()).unwrap_or_default();
    for f in files {
        let item = item_for(f)?;
        let current = items::read_iptc(&item);
        let exif_date = apature_metadata::exif::read(item.path())
            .ok()
            .and_then(|e| e.date_time_original);
        let date = current
            .date_created
            .as_deref()
            .and_then(codes::parse_date)
            .or_else(|| exif_date.as_deref().and_then(codes::parse_date));
        let vars = Vars {
            date,
            credit: current
                .credit
                .clone()
                .or_else(|| a.as_ref().and_then(|a| a.credit_line())),
            headline: current.headline.clone(),
        };
        let expanded = codes::expand(text, &codes_table, &vars, '\\');
        let fields = apature_metadata::Iptc {
            description: Some(expanded.clone()),
            headline: headline.clone(),
            keywords: keywords.as_ref().map(|k| {
                k.split(';')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect()
            }),
            ..Default::default()
        };
        let w = items::write_iptc(&item, &fields)?;
        println!(
            "{}: {expanded}\n  → {} file(s)",
            item.display_name(),
            w.len()
        );
    }
    Ok(())
}

fn mark(file: &Path, rating: Option<i8>, class: Option<u8>, tag: Option<bool>) -> Result<()> {
    let item = item_for(file)?;
    let scheme = ColorClassScheme::default();
    let current = items::read_marks(&item, &scheme);
    let next = Marks {
        rating: rating.map(Rating::clamp).unwrap_or(current.rating),
        color_class: class.map(ColorClass::clamp).unwrap_or(current.color_class),
        tagged: tag.unwrap_or(current.tagged),
    };
    let written = items::write(&item, &next, &scheme)?;
    for p in &written {
        println!("wrote {}", p.display());
    }
    println!(
        "{}: rating={} class={} ({}) tagged={}",
        item.display_name(),
        next.rating.0,
        next.color_class.0,
        scheme.name(next.color_class).unwrap_or("none"),
        next.tagged
    );
    Ok(())
}
