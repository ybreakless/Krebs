# 🦀 Krebs Studio v1.0.0 — Autonomous Plan-Execute-Verify-Ship Protocol

Principal Systems Engineer ledger. Every phase follows the strict lifecycle:
feature branch → implement → `cargo check` / `cargo test` / `cargo clippy`
(self-healing until green) → conventional commit → `--no-ff` merge to main →
branch deleted → plan updated.

Repository: `ybreakless/Krebs` · Release target: **v1.0.0** (from `0.1.0`)

---

## Phase 1 — Installer & Build Fixes (`feat/fix-installer`)

- [x] Align `install.ps1` release URL with `release.yml` naming: `krebs-$tag-x86_64-pc-windows-msvc.zip`
- [x] Probe the newest GitHub release tag instead of hardcoding only `v0.1.0`
- [x] Remote fallback (no repo): download `archive/refs/heads/main.zip` into `$env:TEMP\krebs-build`, extract, locate `Cargo.toml`, `cargo build --release`
- [x] Never `Copy-Item` a binary that was never produced; fail with an actionable message
- [x] Remove hardcoded developer-machine paths (MinGW/LLVM DLL block, line 75)
- [x] Synchronize `install.sh` fallback source-build (source zip + temp dir + cleanup) with `install.ps1`
- [x] Verify: PowerShell syntax parse, shellcheck-style review of `install.sh`

## Phase 2 — Aesthetic Minimalist Crab Logo (`feat/crab-logo-svg`)

- [x] New geometric minimalist SVG: obsidian `#0b0f19` container, Nordic cyan `#38bdf8`/`#0284c7`, faceted carapace, centered stylus nib
- [x] Update `tests/generate_logo.rs` with the new artwork
- [x] `cargo test --test generate_logo` regenerates `assets/logo.svg`, `assets/logo.png`, `krebs_icon.png`, `krebs_icon.ico`

## Phase 3 — Button Ergonomics & Visual Clarity (`feat/button-clarity`)

- [x] Replace ambiguous/abbreviated labels with clear icon + label pills
- [x] Tooltips with shortcut on every action (`Zen Mode (Ctrl+H)`, `Whiteboard (Ctrl+M)`, `Split View (Ctrl+2)`, `Prompt Studio (Ctrl+Shift+P)`, …)
- [x] Active view: crisp high-contrast border + subtle glow; unmistakable current view
- [x] Minimum 32px hit targets for mouse & touch

## Phase 4 — 10-Language i18n & BiDi (`feat/multilingual-i18n`)

- [x] `src/i18n.rs` with typed keys for en, de, fa (RTL), ru, zh, ja, ko, ar (RTL), fr, es
- [x] `is_rtl()` detection with full LTR/RTL layout mirroring support
- [x] Translate toolbar, dialogs, whiteboard tools, AI coach prompts, tour slides
- [x] Language selector UI
- [x] Persist `language = "en"` in `./krebs_data/config.toml` (config struct + installer template)

## Phase 5 — Export PDF / Markdown / HTML (`feat/export-pdf-markdown`)

- [x] Markdown export with optional YAML frontmatter (title, date, word count, language)
- [x] Copy-Markdown to clipboard + save dialog via `rfd`
- [x] Paginated PDF: margins, headers, page numbers, UTF-8 (Latin/CJK/Arabic script support)
- [x] Unified export modal: PDF / Markdown / HTML options

## Phase 5b — Multi-Platform Release & Download Channels (`feat/multiplatform-release-ci`)

- [x] Windows installer (Inno Setup, per-user `%LOCALAPPDATA%\Programs\Krebs`) + portable zips for x64/x86/ARM64
- [x] macOS universal `.dmg` + `.app` zip verification in the release pipeline
- [x] Linux AppImage (linuxdeploy, auto-bundled libraries) for every distribution
- [x] Linux Flatpak: real single-file `.flatpak` bundle built in CI + manifest for Flathub (runtime 24.08, network-enabled cargo build)
- [x] Linux deb/rpm/pacman tarball pipeline (cargo-deb / generate-rpm) kept green
- [x] AUR packages `krebs` (source) & `krebs-bin` (prebuilt) with `.SRCINFO`, published from CI via optional `AUR_SSH_PRIVATE_KEY` secret, shipped as artifacts otherwise
- [x] GitHub Release publishes everything with SHA256SUMS and per-channel install instructions
- [x] Auto-detecting download page (`docs/download.html`) + GitHub Pages deployment workflow

## Phase 5c — Kartei Study Note (`feat/study-note-kartei`)

- [x] Flashcard parsing from plain markdown (`::: karte` blocks with `Q:` / `A:` lines) as a first-class mode on any note
- [x] SM-2-lite scheduler (Again / Good / Easy; 1–90 day intervals, 10-minute re-go) persisted in `krebs_data/study.json`, cards keyed by content hash
- [x] Study overlay (Ctrl+K + rail button): due-only queue, reveal answer, grade buttons, progress counter, done state, no-card hint with syntax guide
- [x] Escape handling, 10 new i18n keys translated across all 10 languages, README + shortcuts documentation

## Phase 6 — Prompt Studio, AI Workbench & MCP Hub (`feat/prompt-studio-mcp`)

- [ ] Visual prompt builder: persona, task/objectives, context/constraints, output format, live token & char counter
- [ ] Meta-prompt "Improve Prompt" optimizer through the configured AI provider
- [ ] MCP tool schema catalog: filesystem (read/write/list), web (fetch/search), database (sql), shell (run), git (status/commit) with toggle-to-inject
- [ ] Copy for Cursor/Windsurf/Claude Code/Antigravity + "Run in Krebs" live test

## Phase 7 — Beginner Onboarding Tour (`feat/expanded-tour`)

- [ ] 7-step rich walkthrough (welcome, distraction-free/BiDi, tri-view modes, whiteboard, AI coach, Prompt Studio & MCP, pro shortcuts/Zen)
- [ ] `Step X of 7`, Next/Previous, arrow-key navigation, Skip Tour
- [ ] Localized steps via i18n

## Phase 8 — Font Manager & Importer (`feat/font-importer`)

- [ ] Built-in font set expansion: JetBrains Mono, IBM Plex Mono, VT323, Vazirmatn, Fira Code, Inter, serif, sans
- [ ] "Import Font (.ttf/.otf)" via `rfd` → copy into `./krebs_data/fonts/`
- [ ] Dynamic load via `iced::font::load`, register in picker, persist in `config.toml`

## Phase 9 — Whiteboard Patterns & Paper Colors (`feat/whiteboard-enhancements`)

- [ ] Patterns: DotGrid, Ruled, MathGrid, IsometricGrid, Cornell, HexGrid, Blank
- [ ] Paper colors: obsidian, slate navy, chalkboard, parchment, paper white, blueprint, charcoal
- [ ] Adaptive ink contrast (dark ink on light paper, light ink on dark paper)
- [ ] Quick pattern & color picker in the whiteboard toolbar, persisted

## Phase 10 — Killer Extra Features (`feat/killer-features`)

- [ ] Whiteboard export to SVG and PNG (one click)
- [ ] Word Sprint tracker: progress bar to configurable target (e.g. Goethe B2 250 words)
- [ ] Automatic draft recovery snapshots in `./krebs_data/recovery/` with instant rollback

## Phase 11 — Polish, Verification & Ship (`feat/ship-v1.0.0`)

- [ ] `cargo fmt --check` clean
- [ ] `cargo clippy --all-targets -- -D warnings` zero warnings (self-healing loop)
- [ ] `cargo test` fully green
- [ ] `cargo doc --no-deps` builds with zero warnings; Rustdoc on all public items
- [ ] README reflects: 10 languages, Prompt Studio & MCP, font importer, whiteboard templates, export formats
- [ ] Packaging scripts sanity: `package.ps1`, `build_packages.sh`, `release.yml` consistent with v1.0.0 artifact naming
- [ ] Bump version to 1.0.0, final merge `chore(release): v1.0.0 production ready release`, clean tree
