# OmaWrite-RS: Portable Rust Minimalist Writing Studio

> *"No vaults, no plugins, just you and the words."*  
> Inspired by Omarchy's `omawrite`, architected in **100% pure Rust** with first-class trilingual support (**English, German, and Persian/Farsi**), an ultra-minimal **vertical icon rail**, and **B2/C1 exam essay training tools**.

---

## 1. Highlights & Capabilities

- **100% Pure Rust & GPU Accelerated:** Built on modern `iced` (v0.13+) powered by `cosmic-text`, `wgpu`, `swash`, `rustybuzz` (HarfBuzz), and `unicode-bidi`.
- **True Portability ("Run Where Placed"):** Zero registry keys, zero `%APPDATA%` pollution. Resolves data relatively to `./omawrite_data/` alongside the executable (`config.toml`, `recovery/`, `snippets_de.toml`).
- **Trilingual BiDi Writing Engine:**
  - **Per-Paragraph Automatic BiDi:** Seamlessly right-aligns (RTL) Persian paragraphs and left-aligns (LTR) English and German paragraphs in the exact same document.
  - **Persian (Farsi) Enhancements:** `Shift + Space` and `Ctrl + Shift + 2` insert the Zero-Width Non-Joiner (ZWNJ / نیم‌فاصله, `\u{200C}`). Automatically normalizes Arabic Yeh (`ي`) and Kaf (`ك`) to Persian Yeh (`ی`) and Kaf (`ک`).
  - **German Exam Chords:** Non-German keyboard shortcuts (`Alt + a/o/u/s` -> `ä/ö/ü/ß`, `Alt + Shift + A/O/U` -> `Ä/Ö/Ü`).
  - **Smart Typography:** Converts `--` to en-dash `–` (Gedankenstrich) and straight quotes `"` to German opening and closing quotes `„...“`.
- **Bundled Offline Fonts (via `include_bytes!`):**
  - **Latin:** `IBM Plex Mono` (Regular & Bold)
  - **Persian:** `Vazirmatn` (Regular & Bold)
  - Renders cursive Persian ligatures and crisp monospace essays out-of-the-box on any machine without installing system fonts.
- **Exam & Essay Trainer Drawer (`Ctrl + E`):**
  - **Countdown Timer:** Presets (`30m`, `45m`, `60m`, Custom) with Start, Pause, Reset, and a top-edge progress bar.
  - **Word Count Target Tracker:** Configurable target range (e.g. `200–250 words`). Live word, character, and paragraph counter. Visual badge turns **green** when within target and **amber** when over/under.
  - **German Exam Redemittel Drawer:** One-click expandable chips for standard Goethe/Telc B2/C1 connectors (*Einleitung*, *Meinungsäußerung*, *Vor- und Nachteile*, *Schluss*).
- **Safety & Lifecycle:**
  - **Silent Crash Recovery:** Untitled drafts checkpoint every 5 seconds to `./omawrite_data/recovery/untitled_<timestamp>.md`.
  - **Debounced Auto-Save:** Writes to disk 1.5 seconds after typing stops for saved files.
  - **External Disk Watcher:** Detects external modifications and displays non-intrusive `[Reload from Disk] | [Keep Local]` action banner.
  - **Clean HTML / Print Export (`Ctrl + P`):** Self-contained HTML output with print styles (`@media print`) and automatic RTL markup.

---

## 2. Visual Aesthetic (Omarchy Dark Palette)

- **Canvas Background:** Deep matte charcoal (`#111216`)
- **Side Rails & Drawers:** Obsidian (`#0b0c0e`) with `1px` subtle border (`#23252e`)
- **Primary Text:** Warm soft off-white (`#d8dee9`)
- **Muted Syntax & Icons:** Dim slate (`#5c6370`)
- **Accent Nordic Cyan:** `#88c0d0`
- **Accent Amber:** `#ebcb8b`
- **Target In-Range Green:** `#a3be8c`
- **Alert Red:** `#bf616a`

---

## 3. Keyboard Shortcuts Cheat Sheet

| Shortcut | Description |
|---|---|
| `Ctrl + N` | Create new untitled document |
| `Ctrl + O` | Open document (`.md`, `.txt`) |
| `Ctrl + S` | Save document |
| `Ctrl + P` | Export clean styled HTML / Print view |
| `Ctrl + \` | Toggle left vertical icon rail (42px) |
| `Ctrl + E` | Toggle Exam & Essay Trainer panel |
| `Ctrl + =` / `+` | Zoom in writing canvas |
| `Ctrl + -` | Zoom out writing canvas |
| `Ctrl + 0` | Reset canvas zoom to 100% |
| `Ctrl + B` | Format Bold (`**text**`) |
| `Ctrl + I` | Format Italic (`*text*`) |
| `Ctrl + Shift + X` | Format Strikethrough (`~~text~~`) |
| `Ctrl + Z` / `Ctrl + Y` | Undo / Redo buffer history |
| `Shift + Space` | Insert Persian ZWNJ (نیم‌فاصله, `\u{200C}`) |
| `Ctrl + Shift + 2` | Alternative Persian ZWNJ shortcut |
| `Alt + a` / `o` / `u` / `s` | German quick chord for `ä`, `ö`, `ü`, `ß` |
| `Alt + Shift + A` / `O` / `U` | German quick chord for `Ä`, `Ö`, `Ü` |
| `F1` or `Ctrl + /` | Toggle centered keyboard shortcuts cheat sheet |
| `Escape` | Dismiss modal / external modification banner |

---

## 4. Architecture & Source Tree

```
c:/Mein Projekts/Krebs/
├── Cargo.toml
├── fonts/
│   ├── IBMPlexMono-Regular.ttf
│   ├── IBMPlexMono-Bold.ttf
│   ├── Vazirmatn-Regular.ttf
│   └── Vazirmatn-Bold.ttf
├── src/
│   ├── main.rs         # Entry point, CLI args, application lifecycle, global event subscriptions
│   ├── config.rs       # Portable data path resolution, config.toml, recovery manager, snippets_de.toml
│   ├── editor.rs       # BiDi paragraph detector, ZWNJ/Yeh/Kaf interceptors, German chords, word counter, undo/redo
│   ├── sidebar.rs      # Ultra-minimal 42px left rail, right exam & essay trainer drawer
│   ├── theme.rs        # Omarchy dark mode palette, custom iced widget styles
│   └── export.rs       # Markdown to print-ready HTML with embedded typography & BiDi tags
└── omawrite_data/      # Automatically created alongside binary
    ├── config.toml
    ├── recovery/
    └── snippets_de.toml
```

---

## 5. Building & Running

### Prerequisites
- Rust 1.80+ (Recommended: `stable-x86_64-pc-windows-gnullvm` or `stable-x86_64-pc-windows-msvc`)

### Development Run
```bash
cargo run
# Or open a specific markdown file directly:
cargo run -- path/to/essay.md
```

### Self-Contained Release Build
```bash
cargo build --release
```
The self-contained executable is produced at:
`target/release/omawrite-rs.exe` (~16.4 MB with all fonts embedded).
Simply copy `omawrite-rs.exe` anywhere (USB drive, desktop, portable folder) — it runs completely self-contained with zero installation required!

### Running Unit Tests
```bash
cargo test
```
All 10 unit tests verify bidirectional paragraph analysis, ZWNJ word boundary segmentation, Persian character normalization, German typographic replacement, undo/redo snapshotting, and recovery checkpoint lifecycles.
