use arboard::Clipboard;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;
use std::thread;
use std::time::Duration;

// ---------------------------------------------------------------------------
// Config — each term has a free-form type defined in the JSON
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct Config {
    poll_ms: Option<u64>,
    groups: Vec<Group>,
}

#[derive(Deserialize)]
struct Group {
    terms: Vec<Term>,
}

#[derive(Deserialize, Clone)]
struct Term {
    #[serde(rename = "type")]
    kind: String,  // e.g. "Cluster", "Label", "Product Code" — user-defined
    value: String,
}

// ---------------------------------------------------------------------------
// Index: term value (lowercase) → all sibling Terms in the same group
// ---------------------------------------------------------------------------

struct Index {
    map: HashMap<String, Vec<Term>>,
    poll: Duration,
}

impl Index {
    fn build(config: Config) -> Self {
        let poll = Duration::from_millis(config.poll_ms.unwrap_or(400));
        let mut map: HashMap<String, Vec<Term>> = HashMap::new();

        for group in &config.groups {
            for term in &group.terms {
                map.insert(term.value.trim().to_lowercase(), group.terms.clone());
            }
        }

        Self { map, poll }
    }

    fn lookup(&self, raw: &str) -> Option<&Vec<Term>> {
        self.map.get(&raw.trim().to_lowercase())
    }
}

// ---------------------------------------------------------------------------
// Config loading — external file only
// ---------------------------------------------------------------------------

fn config_path() -> PathBuf {
    let beside_binary = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.join("mappings.json")));

    if let Some(p) = beside_binary.filter(|p| p.exists()) {
        return p;
    }
    PathBuf::from("mappings.json")
}

fn load_index() -> Index {
    let path = config_path();
    let raw = std::fs::read_to_string(&path).unwrap_or_else(|e| {
        eprintln!("✗ Cannot read {}: {}", path.display(), e);
        eprintln!("  Place a mappings.json next to the binary (or in the working dir).");
        std::process::exit(1);
    });
    let config: Config = serde_json::from_str(&raw).unwrap_or_else(|e| {
        eprintln!("✗ Invalid mappings.json: {}", e);
        std::process::exit(1);
    });
    Index::build(config)
}

// ---------------------------------------------------------------------------
// Notification formatting
// ---------------------------------------------------------------------------

fn format_notification(matched_value: &str, siblings: &[Term]) -> (String, String) {
    // Find the matched term to use its type as the subtitle context.
    let matched_term = siblings
        .iter()
        .find(|t| t.value.trim().to_lowercase() == matched_value.trim().to_lowercase());

    // Title: "cluster-abc123 (Cluster)" — what you copied + its type
    let title = match matched_term {
        Some(t) => format!("{} — {}", t.value, t.kind),
        None    => matched_value.to_string(),
    };

    // Body: aligned "Type:  Value" lines for every sibling, skipping the matched one.
    // Find the longest type name for alignment.
    let others: Vec<&Term> = siblings
        .iter()
        .filter(|t| t.value.trim().to_lowercase() != matched_value.trim().to_lowercase())
        .collect();

    let max_len = others.iter().map(|t| t.kind.len()).max().unwrap_or(0);

    let body = others
        .iter()
        .map(|t| format!("{:<width$}  {}", t.kind, t.value, width = max_len))
        .collect::<Vec<_>>()
        .join("\n");

    (title, body)
}

fn notify(matched_value: &str, siblings: &[Term]) {
    let (title, body) = format_notification(matched_value, siblings);

    // Log to stdout with the same structure
    println!("✓  {}", title);
    for line in body.lines() {
        println!("   {}", line);
    }
    println!();

    // macOS notification: title is the app header, subtitle is the matched term+type,
    // body is the sibling list. Newlines in body are supported by osascript.
    let script = format!(
        r#"display notification "{body}" with title "clipmap" subtitle "{title}""#
    );
    Command::new("osascript")
        .args(["-e", &script])
        .spawn()
        .ok();
}

// ---------------------------------------------------------------------------
// Main loop
// ---------------------------------------------------------------------------

fn main() {
    let index = load_index();
    let mut clipboard = Clipboard::new().expect("Cannot open clipboard");
    let mut last = String::new();

    println!(
        "clipmap ready — {} terms indexed, polling every {}ms\n",
        index.map.len(),
        index.poll.as_millis()
    );

    loop {
        if let Ok(text) = clipboard.get_text() {
            let trimmed = text.trim().to_string();
            if trimmed != last {
                last = trimmed.clone();
                if let Some(siblings) = index.lookup(&trimmed) {
                    notify(&trimmed, siblings);
                }
            }
        }
        thread::sleep(index.poll);
    }
}
