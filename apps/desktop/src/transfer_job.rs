//! Copy or move photos off the UI thread, one photo at a time, reporting
//! each file as it is verified. The engine is `apature_ingest::transfer`.

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use crossbeam_channel::{unbounded, Receiver, Sender};

use apature_core::fileops::{self, Trashed};
use apature_core::PhotoItem;
use apature_ingest::transfer::{index_alternates, transfer, FileResult, Mode, Report};

pub enum Msg {
    File {
        name: String,
        result: FileResult,
    },
    Finished {
        report: Report,
        trashed: Vec<Trashed>,
    },
}

/// What the dialog shows while and after a transfer runs.
pub struct Running {
    pub cancel: Arc<AtomicBool>,
    pub total_files: usize,
    pub done_files: usize,
    pub current: String,
    pub failed_files: Vec<(String, String)>,
    pub bytes: u64,
    pub finished: bool,
    pub report: Option<Report>,
    pub dest: PathBuf,
    pub mode: Mode,
}

pub struct TransferJobs {
    tx: Sender<Msg>,
    rx: Receiver<Msg>,
    ctx: egui::Context,
}

impl TransferJobs {
    pub fn new(ctx: egui::Context) -> Self {
        let (tx, rx) = unbounded();
        TransferJobs { tx, rx, ctx }
    }

    pub fn try_recv(&self) -> Option<Msg> {
        self.rx.try_recv().ok()
    }

    /// `card_roots`: folders to search for a file a cloud drive has not downloaded.
    pub fn start(
        &self,
        items: Vec<PhotoItem>,
        dest: PathBuf,
        mode: Mode,
        card_roots: Vec<PathBuf>,
    ) -> Running {
        let total_files = items.iter().map(|i| fileops::item_files(i).len()).sum();
        let cancel = Arc::new(AtomicBool::new(false));
        let (c2, tx, ctx, dest2) = (
            cancel.clone(),
            self.tx.clone(),
            self.ctx.clone(),
            dest.clone(),
        );
        std::thread::Builder::new()
            .name("transfer-run".into())
            .spawn(move || {
                let waiting = items
                    .iter()
                    .flat_map(fileops::item_files)
                    .any(|f| fileops::is_online_only(&f));
                let alternates = if waiting && !card_roots.is_empty() {
                    index_alternates(&card_roots)
                } else {
                    Vec::new()
                };
                let trashed = Mutex::new(Vec::<Trashed>::new());
                let trash = |files: &[PathBuf]| -> Result<(), String> {
                    let t = fileops::trash_tracked(files).map_err(|e| e.to_string())?;
                    trashed.lock().unwrap().extend(t);
                    Ok(())
                };
                let report = transfer(
                    &items,
                    &dest2,
                    mode,
                    &alternates,
                    &trash,
                    &|| c2.load(Ordering::Relaxed),
                    &mut |src, result| {
                        let name = src
                            .file_name()
                            .map(|n| n.to_string_lossy().into_owned())
                            .unwrap_or_default();
                        let _ = tx.send(Msg::File {
                            name,
                            result: result.clone(),
                        });
                        ctx.request_repaint();
                    },
                );
                let trashed = trashed.into_inner().unwrap_or_default();
                let _ = tx.send(Msg::Finished { report, trashed });
                ctx.request_repaint();
            })
            .expect("spawn transfer thread");
        Running {
            cancel,
            total_files,
            done_files: 0,
            current: String::new(),
            failed_files: Vec::new(),
            bytes: 0,
            finished: false,
            report: None,
            dest,
            mode,
        }
    }

    /// Fold one message into the running state.
    pub fn apply(r: &mut Running, msg: Msg) -> Option<Vec<Trashed>> {
        match msg {
            Msg::File { name, result } => {
                r.done_files += 1;
                match result {
                    FileResult::Copied { bytes, .. } => r.bytes += bytes,
                    FileResult::AlreadyThere => {}
                    FileResult::Failed(why) => r.failed_files.push((name.clone(), why)),
                }
                r.current = name;
                None
            }
            Msg::Finished { report, trashed } => {
                r.finished = true;
                r.report = Some(report);
                Some(trashed)
            }
        }
    }
}
