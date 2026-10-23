//! Update check against GitHub Releases: one request a day when enabled,
//! nothing else leaves the Mac. The result is a toast with a Download button;
//! the DMG opens in the browser and the person drags it to Applications.

use std::sync::mpsc::{channel, Receiver};

pub const REPO: &str = "McCal-Codes/apature";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Update {
    pub version: String,
    pub prerelease: bool,
    pub page: String,
    /// Direct link to this platform's download (DMG, AppImage or Windows
    /// zip), when the release has one.
    pub asset: Option<String>,
    pub notes: String,
}

/// Semver-ish compare of "1.2.3" / "1.2.3-beta.2"; a pre-release sorts below
/// its stable. Returns true when `a` is newer than `b`.
pub fn is_newer(a: &str, b: &str) -> bool {
    fn parse(v: &str) -> (Vec<u64>, Option<String>) {
        let v = v.trim().trim_start_matches('v');
        let (num, pre) = match v.split_once('-') {
            Some((n, p)) => (n, Some(p.to_string())),
            None => (v, None),
        };
        let nums = num.split('.').map(|x| x.parse().unwrap_or(0)).collect();
        (nums, pre)
    }
    let (an, ap) = parse(a);
    let (bn, bp) = parse(b);
    if an != bn {
        return an > bn;
    }
    match (ap, bp) {
        (None, Some(_)) => true,
        (Some(_), None) => false,
        (Some(x), Some(y)) => x > y,
        (None, None) => false,
    }
}

/// Fetch the newest release that is newer than `current`; `beta` includes
/// pre-releases. Blocking; call from a thread.
pub fn check(current: &str, beta: bool) -> Result<Option<Update>, String> {
    // APATURE_UPDATE_REPO lets a test point at another public repository.
    let repo = std::env::var("APATURE_UPDATE_REPO").unwrap_or_else(|_| REPO.to_string());
    let url = format!("https://api.github.com/repos/{repo}/releases?per_page=10");
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .tls_config(
            ureq::tls::TlsConfig::builder()
                .provider(ureq::tls::TlsProvider::NativeTls)
                .build(),
        )
        .timeout_global(Some(std::time::Duration::from_secs(15)))
        .build()
        .into();
    let body: serde_json::Value = agent
        .get(&url)
        .header("User-Agent", &format!("Apature/{current}"))
        .header("Accept", "application/vnd.github+json")
        .call()
        .map_err(|e| match e {
            ureq::Error::StatusCode(404) => {
                "no releases visible (a private repository hides them)".to_string()
            }
            other => other.to_string(),
        })?
        .body_mut()
        .read_json()
        .map_err(|e| e.to_string())?;
    let mut best: Option<Update> = None;
    for r in body.as_array().cloned().unwrap_or_default() {
        if r.get("draft").and_then(|d| d.as_bool()).unwrap_or(false) {
            continue;
        }
        let pre = r
            .get("prerelease")
            .and_then(|p| p.as_bool())
            .unwrap_or(false);
        if pre && !beta {
            continue;
        }
        let tag = r
            .get("tag_name")
            .and_then(|t| t.as_str())
            .unwrap_or("")
            .trim_start_matches('v')
            .to_string();
        if tag.is_empty() || !is_newer(&tag, current) {
            continue;
        }
        if best.as_ref().is_some_and(|b| !is_newer(&tag, &b.version)) {
            continue;
        }
        let asset = r.get("assets").and_then(|a| a.as_array()).and_then(|a| {
            a.iter().find_map(|x| {
                let n = x.get("name")?.as_str()?;
                is_download_for_this_platform(n)
                    .then(|| {
                        x.get("browser_download_url")?
                            .as_str()
                            .map(|s| s.to_string())
                    })
                    .flatten()
            })
        });
        best = Some(Update {
            version: tag,
            prerelease: pre,
            page: r
                .get("html_url")
                .and_then(|u| u.as_str())
                .unwrap_or("")
                .to_string(),
            asset,
            notes: r
                .get("body")
                .and_then(|b| b.as_str())
                .unwrap_or("")
                .to_string(),
        });
    }
    Ok(best)
}

pub struct Checker {
    rx: Receiver<Result<Option<Update>, String>>,
}

impl Checker {
    pub fn spawn(current: String, beta: bool, ctx: egui::Context) -> Checker {
        let (tx, rx) = channel();
        std::thread::Builder::new()
            .name("update-check".into())
            .spawn(move || {
                let _ = tx.send(check(&current, beta));
                ctx.request_repaint();
            })
            .expect("spawn update check");
        Checker { rx }
    }
    pub fn try_recv(&self) -> Option<Result<Option<Update>, String>> {
        self.rx.try_recv().ok()
    }
}

/// The release asset a user of this platform should download.
fn is_download_for_this_platform(name: &str) -> bool {
    if cfg!(target_os = "macos") {
        name.ends_with(".dmg")
    } else if cfg!(target_os = "windows") {
        name.ends_with("-setup.exe")
    } else {
        name.ends_with(".AppImage")
    }
}

#[cfg(test)]
mod tests {
    use super::is_newer;
    #[test]
    fn version_order() {
        assert!(is_newer("0.2.0", "0.1.0"));
        assert!(is_newer("v0.1.1", "0.1.0"));
        assert!(!is_newer("0.1.0", "0.1.0"));
        assert!(is_newer("0.2.0", "0.2.0-beta.1"));
        assert!(!is_newer("0.2.0-beta.1", "0.2.0"));
        assert!(is_newer("0.2.0-beta.2", "0.2.0-beta.1"));
        assert!(!is_newer("0.1.9", "0.10.0"));
    }
}
