# Krebs: Portable Minimalist Writing & Whiteboard Studio

> *"No vaults, no plugins, just you, the words, and the canvas."*  
> Built in **100% pure Rust** with modern GPU acceleration (`iced` 0.13 + `wgpu`), trilingual support (**English, German, and Persian/Farsi**), an **Rnote-inspired Whiteboard Studio for pen tablets & students**, live Markdown formatting with visual bold/italic rendering, an adjustable-width canvas, in-document search, and an ultra-minimal Zen timer.

---

## 1. Key Capabilities

### 📝 1. Writing Studio with Live Markdown Preview
- **Visible Bold, Italic & Formatting (`Ctrl + 1 / 2 / 3`):**
  - **Edit View (`Ctrl + 1`):** Pure distraction-free monospace buffer.
  - **Live Split View (`Ctrl + 2`):** Side-by-side editing and live rendered preview! As you type `**bold**`, `*italic*`, headings, or blockquotes, the right pane immediately renders actual bold, italic, code chips, and styled quotes.
  - **Reader / Preview View (`Ctrl + 3`):** Full-page distraction-free rendered Markdown reader.
- **Adjustable Writing Area Width (`Ctrl + W`):**
  - Toggle between **Standard** (85ch), **Wide** (130ch / ~1350px - default), and **Full Width** (100% screen fill) so your writing area is never cramped.
- **In-Document Search Engine (`Ctrl + F`):**
  - Instant non-intrusive floating search bar with match counter (`"Match 2 of 5"`), Next (`Enter` / `↓`), Previous (`↑`), and line jumping.
- **Offline Famous & Retro Typography:**
  - One-click font selector:
    - **JetBrains Mono:** The world's premier developer & writing font (Default).
    - **IBM Plex Mono:** Crisp modernist typewriter monospace.
    - **VT323:** Iconic 80s retro CRT terminal font.
    - **Vazirmatn:** Flawless Persian / Arabic ligature shaping.
    - **System Sans & System Serif:** Clean modern sans or classic Georgia editorial serif.
- **Trilingual BiDi Engine:**
  - Automatic per-paragraph RTL (Persian) and LTR (English/German).
  - Persian ZWNJ (`Shift + Space`, `Ctrl + Shift + 2`).
  - German Umlaut chords (`Alt + a/o/u/s`).
  - Goethe/Telc B2/C1 Redemittel connector drawer.

---

### 🎨 2. Whiteboard & Pen Tablet Studio (Rnote-Inspired for Students)
- Switch seamlessly between Writing Studio and Whiteboard with `Ctrl + M` or the top/rail button.
- **Pen Tablet & Stylus Support:** High-precision, zero-latency drawing with smooth antialiased strokes and round caps.
- **Tools:**
  - **✏️ Ballpoint Pen:** 1.5px (Fine), 3.0px (Medium), 6.0px (Bold) for calculations, formulas, diagrams, and handwriting.
  - **🖍️ Highlighter:** 16px translucent accent stroke for highlighting notes.
  - **🧹 Eraser:** Radius stroke eraser for fast corrections.
- **Student Notebook Paper Patterns:**
  - **Dots:** Subtle engineering/bullet grid.
  - **Ruled:** School notebook lines.
  - **Math Grid:** Square graph paper for math graphs and physics vectors.
  - **Blank:** Pure dark matte canvas.
- **Color Palette:** Nordic Snow, Cyan, Amber, Aurora Green, Rose Pink.
- **Student Wipe Button (`🗑️ Wipe Board`):**
  - Instantly wipes the scratchpad clean after completing a calculation or problem, with full `Undo` (`Ctrl + Z`) protection!

---

### ⏱️ 3. Minimal Zen Timer & Clock
- Pure, quiet, distraction-free clock in the top/bottom bar (`25:00`).
- Click to start/pause.
- 1-click presets: `[ 15m ] [ 25m Pomodoro ] [ 45m Exam ] [ 60m ] [ ↺ Reset ]`.
- Top hairline (1px) progress bar smoothly tracking remaining focus time.

---

## 2. Keyboard Shortcuts Cheat Sheet

| Shortcut | Action |
|---|---|
| `Ctrl + M` | **Toggle Writing Studio ↔ Whiteboard Studio** |
| `Ctrl + 1` | **Edit View** (Writing buffer only) |
| `Ctrl + 2` | **Live Split View** (Side-by-side live rendered bold/italic preview) |
| `Ctrl + 3` | **Reader View** (Full-width rendered Markdown) |
| `Ctrl + W` | **Cycle Canvas Width** (Standard ↔ Wide ↔ Full Width) |
| `Ctrl + F` | **Toggle In-Document Search** |
| `Ctrl + N` / `O` / `S` | New / Open / Save Document |
| `Ctrl + P` | Export clean styled HTML / Print view |
| `Ctrl + B` / `I` | Format Bold (`**text**`) / Italic (`*text*`) |
| `Ctrl + =` / `-` / `0` | Text Zoom In / Out / Reset (50% to 300%) |
| `Ctrl + Z` / `Ctrl + Y` | Undo / Redo (Text buffer or Whiteboard strokes) |
| `Shift + Space` | Insert Persian ZWNJ (نیم‌فاصله, `\u{200C}`) |
| `Alt + a` / `o` / `u` / `s` | German quick chord for `ä`, `ö`, `ü`, `ß` |
| `F1` or `Ctrl + /` | Toggle centered keyboard shortcuts cheat sheet |
| `Escape` | Close search bar, modals, or banners |

---

## 3. Supported File Types
- **Markdown:** `.md`, `.markdown`, `.mdown`, `.mkd`
- **Plain Text:** `.txt`, `.text`, `.note`, `.org`, `.rst`
- **Drag & Drop:** Simply drag any text file from Windows Explorer into Krebs to open it instantly.
- **Auto-Recovery:** Untitled buffers checkpoint silently to `./krebs_data/recovery/untitled_<timestamp>.md`.

---

## 4. Installation & Launching

Krebs is self-contained in `%LOCALAPPDATA%\Programs\Krebs` and bundled with all runtime DLLs (`libunwind.dll`, `libc++.dll`, `libwinpthread-1.dll`).

- **Start Menu:** Open Windows Start, type **`Krebs`**, and press Enter.
- **Desktop:** Double-click the **`Krebs`** desktop icon.
- **Installer / Updater:**
  ```powershell
  .\install.ps1
  ```
