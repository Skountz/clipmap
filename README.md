# clipmap

Copy any identifier. Instantly know what it is.

A lightweight background daemon that watches your clipboard and surfaces related context via native notifications. Mappings are fetched from a shared URL, so the whole team stays in sync automatically.

```
You copy:   cluster-abc123

You see:  ┌─ cluster-abc123 — Cluster ─────────┐
          │ Label:        Cluster Tooling       │
          │ Product Code: AB42                  │
          │ Team:         Platform Eng          │
          └─────────────────────────────────────┘
```

Works on **macOS**, **Windows**, and **Linux**.

---

## Install

### macOS

Download the latest `.dmg` from [Releases](../../releases), open it, drag **clipmap** to `/Applications`.

On first launch, clipmap installs a default `config.json` and `mappings.json` to `~/.config/clipmap/`. Edit them to point to your team's mappings source.

**Run at login:**
```sh
cp /Applications/clipmap.app/Contents/Resources/fr.frenchbytes.clipmap.plist \
   ~/Library/LaunchAgents/
launchctl load ~/Library/LaunchAgents/fr.frenchbytes.clipmap.plist
```

Stop: `launchctl unload ~/Library/LaunchAgents/fr.frenchbytes.clipmap.plist`  
Logs: `/tmp/clipmap.log` and `/tmp/clipmap.err`

### Windows

```powershell
# Run once as Administrator
.\windows.ps1
```

### Linux

```sh
cargo build --release
cp target/release/clipmap ~/.local/bin/
```

Add to your session autostart or create a systemd user unit.

---

## Configure

Config lives at:

| Platform | Path |
|---|---|
| macOS / Linux | `~/.config/clipmap/config.json` |
| Windows | `%APPDATA%\clipmap\config.json` |

### `config.json`

```json
{
  "mappings_url":    "https://raw.githubusercontent.com/yourorg/yourrepo/main/mappings.json",
  "poll_ms":         400,
  "refresh_minutes": 30
}
```

| Field | Description |
|---|---|
| `mappings_url` | URL or local path to the shared mappings file |
| `poll_ms` | Clipboard check interval in ms (default: 400) |
| `refresh_minutes` | How often to re-fetch mappings (default: 30) |

### `mappings.json`

Host this file anywhere reachable — GitHub raw, S3, an internal server. One person edits it, everyone gets the update within `refresh_minutes`. No restart needed.

```json
{
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
        { "type": "Service", "value": "svc-auth-prod" },
        { "type": "Label",   "value": "Auth Service" },
        { "type": "Owner",   "value": "security@example.com" }
      ]
    }
  ]
}
```

- Each group links any number of terms
- `type` is free-form — use whatever labels make sense for your team
- Copying **any** term in a group surfaces all the others
- Matching is case-insensitive

---

## How team updates work

```
You (maintainer)              Everyone else
      │                             │
      ├─ edit mappings.json         │
      └─ push to GitHub / upload    │
                                    │
                              (within 30 min)
                              clipmap silently
                              re-fetches and reloads
```

---

## Build from source

```sh
cargo build --release
```

Requires [Rust](https://rustup.rs). On Linux, also requires `libdbus-1-dev`.

---

## Resource usage

| | |
|---|---|
| Idle CPU | ~0% |
| RAM | ~3–4 MB |
| Network | One small HTTP GET per refresh interval |
| Binary size | ~500 KB |
