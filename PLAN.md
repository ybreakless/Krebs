# ðŸ¦€ Krebs Studio v1.0.0 â€” Autonomous Plan-Execute-Verify-Ship Protocol

Principal Systems Engineer ledger. Every phase follows the strict lifecycle:
feature branch â†’ implement â†’ `cargo check` / `cargo test` / `cargo clippy`
(self-healing until green) â†’ conventional commit â†’ `--no-ff` merge to main â†’
branch deleted â†’ plan updated.

Repository: `ybreakless/Krebs` Â· Release target: **v1.0.0** (from `0.1.0`)

---

## Phase 1 â€” Installer & Build Fixes (`feat/fix-installer`)

- [x] Align `install.ps1` release URL with `release.yml` naming: `krebs-$tag-x86_64-pc-windows-msvc.zip`
- [x] Probe the newest GitHub release tag instead of hardcoding only `v0.1.0`
- [x] Remote fallback (no repo): download `archive/refs/heads/main.zip` into `$env:TEMP\krebs-build`, extract, locate `Cargo.toml`, `cargo build --release`
- [x] Never `Copy-Item` a binary that was never produced; fail with an actionable message
- [x] Remove hardcoded developer-machine paths (MinGW/LLVM DLL block, line 75)
- [x] Synchronize `install.sh` fallback source-build (source zip + temp dir + cleanup) with `install.ps1`
- [x] Verify: PowerShell syntax parse, shellcheck-style review of `install.sh`

## Phase 2 â€” Aesthetic Minimalist Crab Logo (`feat/crab-logo-svg`)

[x] New geometric minimalist SVG: obsidian `#0b0f19` container, Nordic cyan `#38bdf8`/`#0284c7`, faceted carapace, centered stylus nib
[x] Update `tests/generate_logo.rs` with the new artwork
[x] `cargo test --test generate_logo` regenerates `assets/logo.svg`, `assets/logo.png`, `krebs_icon.png`, `krebs_icon.ico`

## Phase 3 â€” Button Ergonomics & Visual Clarity (`feat/button-clarity`)

- [ ] Replace ambiguous/abbreviated labels with clear icon + label pills
- [ ] Tooltips with shortcut on every action (`Zen Mode (Ctrl+H)`, `Whiteboard (Ctrl+M)`, `Split View (Ctrl+2)`, `Prompt Studio (Ctrl+Shift+P)`, â€¦)
- [ ] Active view: crisp high-contrast border + subtle glow; unmistakable current view
- [ ] Minimum 32px hit targets for mouse & touch

## Phase 4 â€” 10-Language i18n & BiDi (`feat/multilingual-i18n`)

- [ ] `src/i18n.rs` with typed keys for en, de, fa (RTL), ru, zh, ja, ko, ar (RTL), fr, es
- [ ] `is_rtl()` detection with full LTR/RTL layout mirroring support
- [ ] Translate toolbar, dialogs, whiteboard tools, AI coach prompts, tour slides
- [ ] Language selector UI
- [ ] Persist `language = "en"` in `./krebs_data/config.toml` (config struct + installer template)

## Phase 5 â€” Export PDF / Markdown / HTML (`feat/export-pdf-markdown`)

- [ ] Markdown export with optional YAML frontmatter (title, date, word count, language)
- [ ] Copy-Markdown to clipboard + save dialog via `rfd`
- [ ] Paginated PDF: margins, headers, page numbers, UTF-8 (Latin/CJK/Arabic script support)
- [ ] Unified export modal: PDF / Markdown / HTML options

## Phase 6 â€” Prompt Studio, AI Workbench & MCP Hub (`feat/prompt-studio-mcp`)

- [ ] Visual prompt builder: persona, task/objectives, context/constraints, output format, live token & char counter
- [ ] Meta-prompt "Improve Prompt" optimizer through the configured AI provider
- [ ] MCP tool schema catalog: filesystem (read/write/list), web (fetch/search), database (sql), shell (run), git (status/commit) with toggle-to-inject
- [ ] Copy for Cursor/Windsurf/Claude Code/Antigravity + "Run in Krebs" live test

## Phase 7 â€” Beginner Onboarding Tour (`feat/expanded-tour`)

- [ ] 7-step rich walkthrough (welcome, distraction-free/BiDi, tri-view modes, whiteboard, AI coach, Prompt Studio & MCP, pro shortcuts/Zen)
- [ ] `Step X of 7`, Next/Previous, arrow-key navigation, Skip Tour
- [ ] Localized steps via i18n

## Phase 8 â€” Font Manager & Importer (`feat/font-importer`)

- [ ] Built-in font set expansion: JetBrains Mono, IBM Plex Mono, VT323, Vazirmatn, Fira Code, Inter, serif, sans
- [ ] "Import Font (.ttf/.otf)" via `rfd` â†’ copy into `./krebs_data/fonts/`
- [ ] Dynamic load via `iced::font::load`, register in picker, persist in `config.toml`

## Phase 9 â€” Whiteboard Patterns & Paper Colors (`feat/whiteboard-enhancements`)

- [ ] Patterns: DotGrid, Ruled, MathGrid, IsometricGrid, Cornell, HexGrid, Blank
- [ ] Paper colors: obsidian, slate navy, chalkboard, parchment, paper white, blueprint, charcoal
- [ ] Adaptive ink contrast (dark ink on light paper, light ink on dark paper)
- [ ] Quick pattern & color picker in the whiteboard toolbar, persisted

## Phase 10 â€” Killer Extra Features (`feat/killer-features`)

- [ ] Whiteboard export to SVG and PNG (one click)
- [ ] Word Sprint tracker: progress bar to configurable target (e.g. Goethe B2 250 words)
- [ ] Automatic draft recovery snapshots in `./krebs_data/recovery/` with instant rollback

## Phase 11 â€” Polish, Verification & Ship (`feat/ship-v1.0.0`)

- [ ] `cargo fmt --check` clean
- [ ] `cargo clippy --all-targets -- -D warnings` zero warnings (self-healing loop)
- [ ] `cargo test` fully green
- [ ] `cargo doc --no-deps` builds with zero warnings; Rustdoc on all public items
- [ ] README reflects: 10 languages, Prompt Studio & MCP, font importer, whiteboard templates, export formats
- [ ] Packaging scripts sanity: `package.ps1`, `build_packages.sh`, `release.yml` consistent with v1.0.0 artifact naming
- [ ] Bump version to 1.0.0, final merge `chore(release): v1.0.0 production ready release`, clean tree


