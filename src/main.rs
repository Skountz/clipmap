use arboard::Clipboard;
use notify_rust::Notification;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;
use dirs;
use std::thread;
use std::time::{Duration, Instant};

// ---------------------------------------------------------------------------
// Config — local per machine
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct Config {
    mappings_url:    String,
    poll_ms:         Option<u64>,
    refresh_minutes: Option<u64>,
}

// ---------------------------------------------------------------------------
// Mappings — fetched from remote URL, shared across the team
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct Mappings {
    groups: Vec<Group>,
}

#[derive(Deserialize)]
struct Group {
    terms: Vec<Term>,
}

#[derive(Deserialize, Clone)]
struct Term {
    #[serde(rename = "type")]
    kind:  String,
    value: String,
}

// ---------------------------------------------------------------------------
// Index — flat HashMap built from mappings
// ---------------------------------------------------------------------------

struct Index {
    map: HashMap<String, Vec<Term>>,
}

impl Index {
    fn build(mappings: Mappings) -> Self {
        let mut map = HashMap::new();
        for group in &mappings.groups {
            for term in &group.terms {
                map.insert(term.value.trim().to_lowercase(), group.terms.clone());
            }
        }
        Self { map }
    }

    fn lookup(&self, raw: &str) -> Option<&Vec<Term>> {
        self.map.get(&raw.trim().to_lowercase())
    }
}

// ---------------------------------------------------------------------------
// Config loading
// ---------------------------------------------------------------------------

fn user_config_dir() -> PathBuf {
    #[cfg(target_os = "windows")]
    let base = dirs::config_dir();

    #[cfg(not(target_os = "windows"))]
    let base = dirs::home_dir().map(|h| h.join(".config"));

    base.map(|d| d.join("clipmap"))
        .unwrap_or_else(|| PathBuf::from("clipmap"))
}

fn bundled_resources_dir() -> Option<PathBuf> {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent()?.parent().map(|d| d.join("Resources")))
}

// On first launch, copy bundled files to the user config dir so the user
// has a starting point to edit, and the app can run without manual setup.
fn bootstrap_config() {
    let dir = user_config_dir();
    if dir.join("config.json").exists() {
        return;
    }
    let Some(resources) = bundled_resources_dir() else { return };
    if std::fs::create_dir_all(&dir).is_err() { return }

    for filename in &["config.json", "mappings.json"] {
        let src = resources.join(filename);
        let dst = dir.join(filename);
        if src.exists() && !dst.exists() {
            if std::fs::copy(&src, &dst).is_ok() {
                eprintln!("✓  Installed default {} to {}", filename, dst.display());
            }
        }
    }
}

fn config_path() -> PathBuf {
    user_config_dir().join("config.json")
}

fn load_config() -> Config {
    let path = config_path();
    let raw = std::fs::read_to_string(&path).unwrap_or_else(|e| {
        eprintln!("✗ Cannot read {}: {}", path.display(), e);
        std::process::exit(1);
    });
    serde_json::from_str(&raw).unwrap_or_else(|e| {
        eprintln!("✗ Invalid config.json: {}", e);
        std::process::exit(1);
    })
}

// ---------------------------------------------------------------------------
// Remote fetch
// ---------------------------------------------------------------------------

fn resolve_source(source: &str) -> String {
    if source.starts_with("http://") || source.starts_with("https://") {
        return source.to_string();
    }
    let path = std::path::Path::new(source);
    if path.is_absolute() {
        return source.to_string();
    }
    // Relative: resolve against the config file's directory
    config_path()
        .parent()
        .map(|d| d.join(path).to_string_lossy().into_owned())
        .unwrap_or_else(|| source.to_string())
}

fn fetch_index(source: &str) -> Result<Index, String> {
    let source = resolve_source(source);
    let body = if source.starts_with("http://") || source.starts_with("https://") {
        ureq::get(&source)
            .call()
            .map_err(|e| e.to_string())?
            .body_mut()
            .read_to_string()
            .map_err(|e| e.to_string())?
    } else {
        std::fs::read_to_string(&source).map_err(|e| e.to_string())?
    };

    let mappings: Mappings = serde_json::from_str(&body)
        .map_err(|e| e.to_string())?;

    Ok(Index::build(mappings))
}

// ---------------------------------------------------------------------------
// Notification
// ---------------------------------------------------------------------------

fn notify(matched_value: &str, siblings: &[Term]) {
    let matched = siblings
        .iter()
        .find(|t| t.value.trim().to_lowercase() == matched_value.trim().to_lowercase());

    let title = match matched {
        Some(t) => format!("{} — {}", t.value, t.kind),
        None    => matched_value.to_string(),
    };

    let others: Vec<&Term> = siblings
        .iter()
        .filter(|t| t.value.trim().to_lowercase() != matched_value.trim().to_lowercase())
        .collect();

    // Body: "Type: Value" per line, works with proportional fonts
    let body = others
        .iter()
        .map(|t| format!("{}: {}", t.kind, t.value))
        .collect::<Vec<_>>()
        .join("\n");

    // Terminal output: padded table (monospace is fine here)
    let pad = others.iter().map(|t| t.kind.len()).max().unwrap_or(0);
    println!("✓  {}", title);
    for t in &others {
        println!("   {:<pad$}  {}", t.kind, t.value);
    }
    println!();

    // Native notification
    let mut n = Notification::new();
    n.summary(&title).body(&body).sound_name("default");

    #[cfg(target_os = "macos")]
    n.subtitle("clipmap");

    if let Err(e) = n.show() {
        eprintln!("⚠  notification: {e}");
    }
}

// ---------------------------------------------------------------------------
// Main loop
// ---------------------------------------------------------------------------

fn main() {
    bootstrap_config();
    let config      = load_config();
    let poll        = Duration::from_millis(config.poll_ms.unwrap_or(400));
    let refresh_int = Duration::from_secs(config.refresh_minutes.unwrap_or(30) * 60);

    println!("clipmap starting…");
    println!("  source:  {}", config.mappings_url);
    println!("  refresh: every {} min\n", config.refresh_minutes.unwrap_or(30));

    let mut index = fetch_index(&config.mappings_url).unwrap_or_else(|e| {
        eprintln!("✗ Failed to fetch mappings: {}", e);
        std::process::exit(1);
    });
    println!("✓  {} terms loaded\n", index.map.len());

    let mut clipboard    = Clipboard::new().expect("Cannot open clipboard");
    let mut last_text    = String::new();
    let mut last_refresh = Instant::now();

    loop {
        // Refresh mappings from remote when interval has elapsed
        if last_refresh.elapsed() >= refresh_int {
            match fetch_index(&config.mappings_url) {
                Ok(new_index) => {
                    index        = new_index;
                    last_refresh = Instant::now();
                    println!("↻  Mappings refreshed — {} terms", index.map.len());
                }
                Err(e) => eprintln!("⚠  Refresh failed (using cache): {}", e),
            }
        }

        // Clipboard check
        if let Ok(text) = clipboard.get_text() {
            let trimmed = text.trim().to_string();
            if trimmed != last_text {
                last_text = trimmed.clone();
                if let Some(siblings) = index.lookup(&trimmed) {
                    notify(&trimmed, siblings);
                }
            }
        }

        thread::sleep(poll);
    }
}
