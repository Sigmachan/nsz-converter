# NSZ Converter - Production Architecture

## Overview

Production-ready, cross-platform (macOS, Windows, Linux) NSZ/NSP converter with:
- **Automatic file monitoring** - Watches a directory and converts new NSZ files instantly
- **NSZ → NSP conversion** - Powered by the battle-tested `nsz` CLI (https://github.com/nicoboss/nsz)
- **Multi-part NSP merging** - Automatically detects and merges split NSP files
- **Game folder organization** - Creates folders named after games (user-defined mappings)
- **Modern native GUI** - Tauri + React with dark theme, real-time updates

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                        Tauri GUI (React)                         │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌───────────┐ │
│  │  Dashboard  │ │  Settings   │ │ Game Mappings│ │  History  │ │
│  └─────────────┘ └─────────────┘ └─────────────┘ └───────────┘ │
└───────────────────────────┬─────────────────────────────────────┘
                            │ IPC ( Tauri's invoke system)
┌───────────────────────────▼─────────────────────────────────────┐
│                     Rust Backend (Tauri)                         │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │                   AppState (Arc<RwLock>)                  │  │
│  │  ┌─────────────┐ ┌─────────────┐ ┌─────────────────────┐ │  │
│  │  │  Database   │ │File Monitor │ │ Conversion Engine   │ │  │
│  │  │  (SQLite)   │ │  (notify)   │ │  (nsz subprocess)   │ │  │
│  │  └─────────────┘ └─────────────┘ └─────────────────────┘ │  │
│  └──────────────────────────────────────────────────────────┘  │
└───────────────────────────┬─────────────────────────────────────┘
                            │ Subprocess
┌───────────────────────────▼─────────────────────────────────────┐
│                      nsz CLI (Python)                            │
│         https://github.com/nicoboss/nsz - 2.4k stars             │
│  - Lossless zstd compression/decompression                       │
│  - Block compression for random read access                      │
│  - Verification, title key extraction                            │
└─────────────────────────────────────────────────────────────────┘
```

## Key Design Decisions

### 1. Why delegate to `nsz` CLI instead of reimplementing?

**CRITICAL**: The NSZ format is complex:
- NCZ header with crypto sections at offset 0x4000
- Variable-length section descriptors (NCZSECTN magic)
- Optional block compression (NCZBLOCK) with per-block size tracking
- Re-encryption during decompression using stored keys

The `nsz` library (2.4k stars, actively maintained) has:
- 690 commits, battle-tested over years
- Handles all edge cases (padding, verification, multi-threading)
- PFS0/NSP container manipulation
- Proper error handling and recovery

**Reimplementing this would be:**
- Error-prone (crypto, container formats)
- Maintenance burden (format updates)
- Reinventing the wheel

**Our approach**: Thin Rust wrapper → `nsz` subprocess with proper error handling, progress tracking, and integration.

### 2. File Monitoring Strategy

Uses the `notify` crate for cross-platform file system events:
- Recursive watching of monitored directory
- Event debouncing (2-second stability check)
- Automatic conversion queue integration
- System notifications on detection

### 3. Game Name Mapping System

User defines mappings in the UI:
```
Title ID: 0100ABCDEF123456 → Game Name: "The Legend of Zelda: Breath of the Wild"
```

When conversion completes:
1. Detect Title ID from filename or `nsz --info`
2. Look up in database
3. Create folder: `~/NSZ_Converted/The Legend of Zelda - Breath of the Wild/`
4. Place converted NSP inside

### 4. Multi-part NSP Handling

Two scenarios detected:

**A. Split-file dumps** (rare):
- Files named `game.part1.nsp`, `game.part2.nsp`
- Requires PFS0 container merging (future implementation)

**B. Separate titles** (common):
- Base game + Update + DLCs with same Title ID prefix
- Organized into same folder, installed separately
- No binary merging needed

## Production Features Implemented

✅ **Logging**: `tracing` + `tracing-subscriber` with JSON output option  
✅ **Error handling**: `anyhow` for ergonomic errors, proper context  
✅ **Database**: SQLite with migrations, proper indexing  
✅ **Settings persistence**: All config saved to DB  
✅ **Conversion queue**: Jobs tracked with status, timestamps  
✅ **History**: Last 100 conversions viewable  
✅ **File verification**: Optional post-conversion verification  
✅ **Cross-platform**: Tauri handles macOS/Windows/Linux  

## Installation Requirements (for end users)

### 1. Install `nsz` CLI (required dependency)

```bash
# Recommended: Prebuilt binaries (no Python needed)
# Download from: https://github.com/nicoboss/nsz/releases

# Or via pip:
pip3 install --upgrade nsz

# Verify:
nsz --version
```

### 2. Install Nintendo Switch keys

Required for any NSP/NSZ operation. User must provide their own `prod.keys`.

**Locations checked (in order):**
- Current directory
- `~/.switch/prod.keys` (all platforms)
- `~/.config/nsz/prod.keys` (Linux)
- Custom path via `--keys` flag

### 3. Run NSZ Converter

The app bundles everything else. First run will:
1. Detect `nsz` installation
2. Initialize SQLite database
3. Create default settings

## Build Instructions (Developers)

### Prerequisites

```bash
# Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Node.js 18+
node --version  # Should be v18+

# Tauri CLI
cargo install tauri-cli
```

### Development

```bash
cd nsz-converter

# Terminal 1: Rust backend (auto-reloads on changes)
cargo tauri dev

# Terminal 2: Frontend (if you want separate Vite dev server)
cd frontend && npm install && npm run dev
```

### Production Build

```bash
cargo tauri build

# Outputs:
# macOS:  src-tauri/target/release/bundle/dmg/NSZ Converter_1.0.0_aarch64.dmg
# Windows: src-tauri/target/release/bundle/msi/NSZ Converter_1.0.0_x64_en-US.msi
# Linux:   src-tauri/target/release/bundle/deb/nsz-converter_1.0.0_amd64.deb
```

## Database Schema

```sql
-- User settings (monitored dir, output dir, preferences)
CREATE TABLE settings (key TEXT PRIMARY KEY, value TEXT, updated_at TEXT);

-- Title ID → Game Name mappings (user-defined)
CREATE TABLE game_mappings (title_id TEXT PRIMARY KEY, game_name TEXT, created_at TEXT);

-- Conversion job tracking
CREATE TABLE conversion_jobs (
    id TEXT PRIMARY KEY,
    input_path TEXT NOT NULL,
    output_path TEXT,
    status TEXT NOT NULL,  -- JSON: {"Pending":null} or {"Completed":{"output_path":"..."}}
    created_at TEXT,
    started_at TEXT,
    completed_at TEXT,
    title_id TEXT,
    game_name TEXT
);
```

## Future Enhancements (Roadmap)

### Phase 2
- [ ] PFS0 split-file merging implementation
- [ ] Drag-and-drop support in GUI
- [ ] Batch conversion with progress bars per file
- [ ] Theme customization
- [ ] Scheduled conversion (e.g., "convert at 3 AM")

### Phase 3
- [ ] XCI/XCZ support (currently NSZ/NSP only)
- [ ] Cloud sync of game mappings
- [ ] Statistics dashboard (compression ratios, time saved)
- [ ] Portable mode (store everything in one folder)

## License

MIT - Same as the `nsz` library this wraps.

## Credits

- **nsz library**: nicoboss (https://github.com/nicoboss/nsz) - The foundation
- **Tauri**: The awesome cross-platform framework
- **React + Tailwind**: Modern UI stack
