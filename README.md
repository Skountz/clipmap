# clipmap

Copy any identifier. Instantly know what it is.

A zero-overhead macOS daemon that watches your clipboard and maps identifiers to human-readable context — cross-referenced, typed, and formatted as a native notification.

```
You copy:   cluster-abc123

You see:  ┌─ clipmap ──────────────────────────────┐
          │ cluster-abc123 — Cluster               │
          │                                        │
          │ Label         Cluster Tooling          │
          │ Product Code  AB42                     │
          │ Team          Platform Eng             │
          └────────────────────────────────────────┘
```

---

## Requirements

- macOS 12+
- [Rust](https://rustup.rs) (only to build)

---

## Install

```bash
# 1. Clone or download the source
git clone https://github.com/you/clipmap && cd clipmap

# 2. Build (optimized, stripped binary ~400 KB)
cargo build --release

# 3. Move the binary somewhere permanent
cp target/release/clipmap /usr/local/bin/clipmap

# 4. Put your mappings file next to the binary
cp mappings.json /usr/local/bin/mappings.json
```

---

## Run

**Once (foreground, good for testing):**
```bash
clipmap
```

**At login, silently in the background:**
```bash
# Edit the plist — update the path if you installed elsewhere
nano com.yourname.clipmap.plist

# Register with launchd
cp com.yourname.clipmap.plist ~/Library/LaunchAgents/
launchctl load ~/Library/LaunchAgents/com.yourname.clipmap.plist
```

To stop it:
```bash
launchctl unload ~/Library/LaunchAgents/com.yourname.clipmap.plist
```

Logs go to `/tmp/clipmap.log` and `/tmp/clipmap.err`.

---

## Configure `mappings.json`

Each group is a set of terms that refer to the same thing.  
Every term has a **type** (your label) and a **value** (what to match on clipboard).  
Types are free-form — define whatever makes sense for your data.

```json
{
  "poll_ms": 400,
  "groups": [
    {
      "terms": [
        { "type": "Cluster",      "value": "cluster-abc123" },
        { "type": "Label",        "value": "Cluster Tooling" },
        { "type": "Product Code", "value": "AB42" },
        { "type": "Team",         "value": "Platform Eng" }
      ]
    },
    {
      "terms": [
        { "type": "Service",  "value": "svc-auth-prod" },
        { "type": "Label",    "value": "Auth Service" },
        { "type": "Owner",    "value": "security@company.com" }
      ]
    }
  ]
}
```

Copy any `value` from any group — you'll get all sibling terms displayed with their types.  
Matching is **case-insensitive**. Reload the daemon after editing.

| Field | Description |
|---|---|
| `poll_ms` | Clipboard check interval in ms (default: `400`) |
| `groups` | Array of term groups |
| `type` | Display label, free-form string |
| `value` | The string to match when copied |

---

## Resource usage

| | |
|---|---|
| Idle CPU | ~0% (sleeping between polls) |
| RAM | ~2–3 MB |
| Binary size | ~400 KB |
| Notification latency | < 1 poll cycle |

Runs entirely on-device. No network calls, no telemetry.
