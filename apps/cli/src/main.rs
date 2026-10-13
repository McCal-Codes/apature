//! `pw`: the engine without the window. Useful for testing format support
//! and for benchmarking without GUI overhead.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use rayon::prelude::*;

use photo_core::{ColorClass, ColorClassScheme, Marks, Rating, ScanOptions};
use photo_metadata::xmp::{self, XmpMarks};
use photo_preview::{Source, ThumbCache};

#[derive(Parser)]
#[command(name = "pw", version, about = "photo-workflow command line")]
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
            let r = photo_ingest::copy_verified(&src, &dst)?;
            println!(
                "copied {} bytes to {} (sha256 {})",
                r.bytes,
                r.destination.display(),
                r.sha256
            );
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
        directories::ProjectDirs::from("", "", "photo-workflow").context("no home directory")?;
    Ok(dirs.cache_dir().join("thumbs"))
}

fn scan(dir: &Path, recursive: bool) -> Result<()> {
    let t0 = Instant::now();
    let items = photo_core::scan(dir, &ScanOptions { recursive })?;
    let scheme = ColorClassScheme::default();
    for it in &items {
        let marks = xmp::read_sidecar(&it.sidecar_path())
            .ok()
            .flatten()
            .map(|m| m.to_marks(&scheme));
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
        let m = marks
            .map(|m| {
                format!(
                    "  rating={} class={} tagged={}",
                    m.rating.0, m.color_class.0, m.tagged
                )
            })
            .unwrap_or_default();
        println!("{:>5}  {}{}{}", it.id, it.display_name(), pair, m);
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
    let s = photo_metadata::exif::read(file)?;
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
    let items = photo_core::scan(dir, &ScanOptions { recursive })?;
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
                let orientation = photo_metadata::exif::from_bytes(src.data())
                    .map(|e| e.orientation)
                    .unwrap_or(1);
                let rgb = src.decode_for(size * 2)?;
                let rgb = photo_preview::rotate(rgb, orientation);
                let thumb = photo_preview::thumbnail(&rgb, size);
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

fn mark(file: &Path, rating: Option<i8>, class: Option<u8>, tag: Option<bool>) -> Result<()> {
    let sidecar = file.with_extension("xmp");
    let scheme = ColorClassScheme::default();
    let current = xmp::read_sidecar(&sidecar)?
        .map(|m| m.to_marks(&scheme))
        .unwrap_or_default();
    let next = Marks {
        rating: rating.map(Rating::clamp).unwrap_or(current.rating),
        color_class: class.map(ColorClass::clamp).unwrap_or(current.color_class),
        tagged: tag.unwrap_or(current.tagged),
    };
    xmp::write_marks(&sidecar, &XmpMarks::from_marks(&next, &scheme))?;
    println!(
        "{}: rating={} class={} ({}) tagged={}",
        sidecar.display(),
        next.rating.0,
        next.color_class.0,
        scheme.name(next.color_class).unwrap_or("none"),
        next.tagged
    );
    Ok(())
}
