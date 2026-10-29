//! Export engines for Krebs documents: markdown, HTML, and PDF.
//!
//! All three engines share the same markdown source and the same RTL strategy:
//!
//! - [`markdown_to_html`] produces a self-contained, print-ready HTML file.
//!   Browsers perform full Arabic complex-text shaping, so this remains the
//!   print-perfect path for RTL documents. [`inject_bidi_attributes`] marks
//!   RTL blocks with `dir="rtl"` so the embedded stylesheet can mirror them.
//! - [`markdown_with_frontmatter`] serializes a document with a YAML front
//!   matter header carrying title, date, word count and language.
//! - [`export_pdf`] renders a paginated A4 PDF with the fonts bundled with
//!   Krebs (IBM Plex Mono, Vazirmatn) and an optional system CJK font.
//!
//! # PDF layout and bidi
//!
//! `genpdf 0.2` has no bidi support and its PDF backend always draws glyphs
//! left-to-right, which would visually reverse Arabic and Persian sentences.
//! [`bidi_visual_line`] therefore implements UAX #9 run reordering: every
//! wrapped line is converted to *visual* order before it is handed to a
//! right-aligned paragraph. Pure Latin text passes through unchanged.
//!
//! # Known PDF limitations (v1.0)
//!
//! - Arabic/Persian letterforms render in isolated (unshaped) form with
//!   correct visual order; the HTML export stays the print-perfect path for
//!   such documents.
//! - Italic runs map onto the regular typeface because no true italic variant
//!   is embedded.
//! - Inline styling inside an RTL line collapses to the style of the line.
//! - If no plain `.ttf` CJK fallback is found on the system, CJK codepoints
//!   degrade to tofu instead of failing the export.
//! - When a CJK fallback font is used, only its regular face is embedded;
//!   bold/italic slots fall back to IBM Plex Mono so CJK headings render
//!   without bold without doubling the file size.

use genpdf::Element;
use pulldown_cmark::html;
use pulldown_cmark::{CowStr, Event, HeadingLevel, Options, Parser, Tag, TagEnd};
use std::fs;
use std::path::Path;

use crate::editor::is_persian_or_arabic;

// ---------------------------------------------------------------------------
// HTML export
// ---------------------------------------------------------------------------

/// Converts markdown text into a styled, self-contained HTML document.
///
/// The result embeds both fonts (IBM Plex Mono, Vazirmatn) and print styles;
/// tables, footnotes, task lists and strikethrough are enabled. Blocks that
/// start with a Persian or Arabic character receive `dir="rtl"` so the browser
/// mirrors them.
///
/// # Examples
///
/// ```
/// use krebs::export::markdown_to_html;
///
/// let html = markdown_to_html("# Hallo\n\ndu **da**", "Titel");
/// assert!(html.contains("<h1>Hallo</h1>"));
/// assert!(html.contains("<title>Titel</title>"));
/// ```
#[must_use]
pub fn markdown_to_html(markdown: &str, title: &str) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);

    let parser = Parser::new_ext(markdown, options);
    let mut raw_html = String::new();
    html::push_html(&mut raw_html, parser);

    // Post-process HTML to add dir="rtl" to paragraphs/headings with Persian text
    let processed_html = inject_bidi_attributes(&raw_html);

    generate_html_template(title, &processed_html)
}

/// Injects `dir="rtl"` and `lang="fa"` on block tags starting with Persian
/// characters.
fn inject_bidi_attributes(html_content: &str) -> String {
    let mut output = String::with_capacity(html_content.len() + 1024);

    for line in html_content.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("<p>")
            || trimmed.starts_with("<h1>")
            || trimmed.starts_with("<h2>")
            || trimmed.starts_with("<h3>")
            || trimmed.starts_with("<blockquote>")
            || trimmed.starts_with("<li>")
        {
            // Extract text inside the tag to check direction
            let text_start = trimmed.find('>').map(|i| i + 1).unwrap_or(0);
            let inside_text = &trimmed[text_start..];
            if has_strong_rtl(inside_text) {
                let tag_end = trimmed.find('>').unwrap_or(0);
                let tag_name = &trimmed[1..tag_end];
                let modified = format!(
                    "<{} dir=\"rtl\" lang=\"fa\" class=\"rtl-block\">{}",
                    tag_name, inside_text
                );
                output.push_str(&modified);
                output.push('\n');
                continue;
            }
        }
        output.push_str(line);
        output.push('\n');
    }

    output
}

/// Returns whether the first alphabetic character of `text` is RTL.
fn has_strong_rtl(text: &str) -> bool {
    for c in text.chars() {
        if is_persian_or_arabic(c) {
            return true;
        } else if c.is_alphabetic() {
            return false;
        }
    }
    false
}

fn generate_html_template(title: &str, body_content: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>{}</title>
  <link rel="preconnect" href="https://fonts.googleapis.com">
  <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
  <link href="https://fonts.googleapis.com/css2?family=IBM+Plex+Mono:ital,wght@0,400;0,600;1,400&family=Vazirmatn:wght@400;600;700&display=swap" rel="stylesheet">
  <style>
    :root {{
      --bg: #111216;
      --text: #d8dee9;
      --text-muted: #7b8496;
      --accent: #88c0d0;
      --border: #23252e;
      --code-bg: #1a1c23;
    }}

    @media (prefers-color-scheme: light) {{
      :root {{
        --bg: #ffffff;
        --text: #1a1a1a;
        --text-muted: #666666;
        --accent: #0066cc;
        --border: #e0e0e0;
        --code-bg: #f5f5f5;
      }}
    }}

    * {{
      box-sizing: border-box;
    }}

    body {{
      font-family: 'IBM Plex Mono', 'Vazirmatn', -apple-system, BlinkMacSystemFont, monospace;
      background-color: var(--bg);
      color: var(--text);
      line-height: 1.75;
      max-width: 780px;
      margin: 0 auto;
      padding: 3rem 1.5rem;
      font-size: 15px;
      text-rendering: optimizeLegibility;
      -webkit-font-smoothing: antialiased;
    }}

    .rtl-block {{
      font-family: 'Vazirmatn', 'IBM Plex Mono', Tahoma, sans-serif;
      direction: rtl;
      text-align: right;
      line-height: 2.0;
    }}

    h1, h2, h3, h4 {{
      color: var(--text);
      font-weight: 600;
      margin-top: 2rem;
      margin-bottom: 0.8rem;
      line-height: 1.3;
    }}

    h1 {{
      font-size: 1.8rem;
      border-bottom: 1px solid var(--border);
      padding-bottom: 0.5rem;
    }}

    h2 {{
      font-size: 1.4rem;
    }}

    p {{
      margin-top: 0;
      margin-bottom: 1.25rem;
    }}

    blockquote {{
      border-left: 3px solid var(--accent);
      margin: 1.5rem 0;
      padding: 0.5rem 1rem;
      color: var(--text-muted);
      background-color: var(--code-bg);
      border-radius: 0 4px 4px 0;
    }}

    .rtl-block blockquote {{
      border-left: none;
      border-right: 3px solid var(--accent);
      border-radius: 4px 0 0 4px;
    }}

    code {{
      font-family: 'IBM Plex Mono', monospace;
      background-color: var(--code-bg);
      padding: 0.2rem 0.4rem;
      border-radius: 3px;
      font-size: 0.9em;
      border: 1px solid var(--border);
    }}

    pre {{
      background-color: var(--code-bg);
      padding: 1rem;
      border-radius: 6px;
      overflow-x: auto;
      border: 1px solid var(--border);
    }}

    pre code {{
      border: none;
      padding: 0;
    }}

    hr {{
      border: none;
      border-top: 1px solid var(--border);
      margin: 2.5rem 0;
    }}

    table {{
      width: 100%;
      border-collapse: collapse;
      margin: 1.5rem 0;
    }}

    th, td {{
      border: 1px solid var(--border);
      padding: 0.6rem 0.8rem;
      text-align: left;
    }}

    th {{
      background-color: var(--code-bg);
    }}

    @media print {{
      body {{
        background-color: #ffffff !important;
        color: #000000 !important;
        max-width: 100% !important;
        padding: 0 !important;
        font-size: 11pt !important;
      }}

      blockquote {{
        border-left-color: #333333 !important;
        background-color: #f9f9f9 !important;
      }}

      pre, code {{
        background-color: #f5f5f5 !important;
        border-color: #cccccc !important;
        color: #000000 !important;
      }}

      h1, h2, h3, hr {{
        page-break-after: avoid;
      }}

      table, pre, blockquote {{
        page-break-inside: avoid;
      }}
    }}
  </style>
</head>
<body>
  <main id="content">
    {}
  </main>
</body>
</html>"#,
        html_escape(title),
        body_content
    )
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

/// Renders `markdown` as HTML and writes it to `path`.
///
/// # Errors
///
/// Returns an I/O error if the file cannot be written.
pub fn export_to_file(markdown: &str, title: &str, path: &Path) -> std::io::Result<()> {
    let html = markdown_to_html(markdown, title);
    fs::write(path, html)
}

/// Copies the document as rich HTML plus fallback markdown text to the system
/// clipboard.
///
/// # Errors
///
/// Returns an error string if the clipboard backend fails.
pub fn copy_html_to_clipboard(markdown: &str, title: &str) -> Result<(), String> {
    let html = markdown_to_html(markdown, title);
    let mut clipboard = arboard::Clipboard::new().map_err(|e| e.to_string())?;
    clipboard
        .set_html(html, Some(markdown.to_string()))
        .map_err(|e| e.to_string())
}

/// Opens the exported file in the platform default application (browser).
///
/// # Errors
///
/// Returns an error string if the OS fails to launch the file handler.
pub fn open_exported_html(path: &Path) -> Result<(), String> {
    open::that(path).map_err(|e| e.to_string())
}

// ---------------------------------------------------------------------------
// Markdown export
// ---------------------------------------------------------------------------

/// Returns the word count of a markdown document using the same trilingual,
/// ZWNJ-aware semantics as the in-editor status bar.
///
/// # Examples
///
/// ```
/// use krebs::export::word_count;
///
/// assert_eq!(word_count("one two three"), 3);
/// ```
#[must_use]
pub fn word_count(markdown: &str) -> usize {
    crate::editor::DocumentStats::compute(markdown).word_count
}

/// Serializes `markdown` with a YAML front matter header.
///
/// The header contains the title, the local date in ISO-8601 date form, the
/// trilingual-aware word count and the language code.
///
/// # Examples
///
/// ```
/// use krebs::export::markdown_with_frontmatter;
///
/// let out = markdown_with_frontmatter("وضعیت روشن است", "شروع", "fa");
/// assert!(out.starts_with("---\n"));
/// assert!(out.contains("language: fa"));
/// ```
#[must_use]
pub fn markdown_with_frontmatter(markdown: &str, title: &str, language: &str) -> String {
    let date = chrono::Local::now().format("%Y-%m-%d");
    format!(
        "---\ntitle: {}\ndate: {}\nwords: {}\nlanguage: {}\n---\n\n{}\n",
        yaml_scalar(title),
        date,
        word_count(markdown),
        language,
        markdown
    )
}

/// Wraps `value` into a double-quoted YAML scalar safe for the front matter.
fn yaml_scalar(value: &str) -> String {
    let escaped = value.replace('\\', "\\\\").replace('"', "\\\"");
    format!("\"{escaped}\"")
}

/// Writes `markdown` with a YAML front matter header to `path`.
///
/// # Errors
///
/// Returns an I/O error if the file cannot be written.
pub fn export_markdown_to_file(
    markdown: &str,
    title: &str,
    language: &str,
    path: &Path,
) -> std::io::Result<()> {
    let text = markdown_with_frontmatter(markdown, title, language);
    fs::write(path, text)
}

/// Copies the raw markdown body to the system clipboard.
///
/// In contrast to [`export_markdown_to_file`], the copy contains no front
/// matter so it can be pasted into other markdown tools directly.
///
/// # Errors
///
/// Returns an error string if the clipboard backend fails.
pub fn copy_markdown_to_clipboard(markdown: &str) -> Result<(), String> {
    let mut clipboard = arboard::Clipboard::new().map_err(|e| e.to_string())?;
    clipboard
        .set_text(markdown.to_string())
        .map_err(|e| e.to_string())
}

// ---------------------------------------------------------------------------
// PDF export: markdown block model
// ---------------------------------------------------------------------------

/// A span of inline text with its markdown emphasis.
#[derive(Debug, Clone, PartialEq, Eq)]
enum InlineRun {
    /// Unmarked text.
    Plain(String),
    /// `**bold**` text.
    Bold(String),
    /// `*italic*` text (mapped onto the regular typeface for PDF).
    Italic(String),
    /// `` `code` `` text.
    Code(String),
}

/// The inline emphasis of a run, as tracked by the parser.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RunKind {
    /// Unmarked text.
    Plain,
    /// `**bold**` text.
    Bold,
    /// `*italic*` text.
    Italic,
    /// `` `code` `` text.
    Code,
}

/// A parsed markdown block prepared for PDF rendering.
#[derive(Debug, Clone, PartialEq)]
enum Block {
    /// `#` heading with its level (1–3; deeper levels render as level 3).
    Heading(usize, Vec<InlineRun>),
    /// A plain paragraph.
    Paragraph(Vec<InlineRun>),
    /// `>` blockquote content.
    Quote(Vec<InlineRun>),
    /// One source line of a fenced code block.
    CodeLine(String),
    /// A `-` list item.
    Bullet(Vec<InlineRun>),
    /// A `1.` list item with its display number.
    Ordered(u64, Vec<InlineRun>),
    /// A markdown table; each entry is one row with cells joined by ` | `.
    Table(Vec<String>),
    /// A `---` thematic break.
    Rule,
    /// Vertical gap of `f64` lines, e.g. between list items.
    Spacer(f64),
}

/// Converts a markdown document into a sequence of renderable blocks.
///
/// The parser follows the same conventions as the in-app markdown preview:
/// headings, paragraphs, block quotes (flattened), fenced code blocks,
/// flat and nested lists, tables (rendered as ` | `-joined rows) and thematic
/// breaks. Inline styles become [`InlineRun`]s; raw HTML and footnote bodies
/// are dropped.
#[must_use]
fn parse_blocks(markdown: &str) -> Vec<Block> {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);

    let mut state = ParserState::default();
    for event in Parser::new_ext(markdown, options) {
        state.step(event);
    }
    state.finish();
    state.blocks
}

/// Context of the currently open list item, if any.
#[derive(Debug, Clone, Copy)]
struct ItemContext {
    /// Whether the item belongs to an ordered list.
    ordered: bool,
    /// Number of the item (ordered lists only).
    number: u64,
}

/// The pulldown-cmark event machine behind [`parse_blocks`].
#[derive(Debug, Default)]
struct ParserState {
    blocks: Vec<Block>,
    runs: Vec<InlineRun>,
    bold_depth: usize,
    italic_depth: usize,
    heading_level: Option<usize>,
    quote_depth: usize,
    footnote_depth: usize,
    in_code_block: bool,
    code_buffer: String,
    /// `(ordered, next number)` per open list, outermost first.
    list_stack: Vec<(bool, u64)>,
    pending_item: Option<ItemContext>,
    in_table: bool,
    table_rows: Vec<String>,
    table_cells: Vec<String>,
    in_table_cell: bool,
    cell_buffer: String,
}

impl ParserState {
    /// Consumes one pulldown-cmark event.
    fn step(&mut self, event: Event<'_>) {
        match event {
            Event::Start(tag) => self.start_tag(tag),
            Event::End(tag) => self.end_tag(tag),
            Event::Text(text) => self.text_event(text),
            Event::Code(code) => self.code_event(code),
            Event::SoftBreak | Event::HardBreak => self.break_event(),
            Event::Rule => {
                self.flush_runs();
                self.blocks.push(Block::Rule);
            }
            Event::TaskListMarker(_)
            | Event::InlineHtml(_)
            | Event::Html(_)
            | Event::FootnoteReference(_) => {}
            _ => {}
        }
    }

    /// Handles the remaining events after the parser finished.
    fn finish(&mut self) {
        self.flush_runs();
    }

    fn start_tag(&mut self, tag: Tag<'_>) {
        match tag {
            Tag::Heading { level, .. } => {
                self.flush_runs();
                self.heading_level = Some(heading_index(level));
            }
            Tag::BlockQuote(_) => {
                self.flush_runs();
                self.quote_depth += 1;
            }
            Tag::CodeBlock(_) => {
                self.flush_runs();
                self.in_code_block = true;
                self.code_buffer.clear();
            }
            Tag::List(start) => {
                self.flush_runs();
                self.list_stack.push(match start {
                    Some(first) => (true, first),
                    None => (false, 0),
                });
            }
            Tag::Item => {
                self.flush_runs();
                // Assign the display number now so nested flushes inside the
                // same item reuse it.
                let ctx = match self.list_stack.last().copied() {
                    Some((true, number)) => {
                        let allocated = if number == 0 { 1 } else { number };
                        if let Some(slot) = self.list_stack.last_mut() {
                            *slot = (true, allocated + 1);
                        }
                        Some(ItemContext {
                            ordered: true,
                            number: allocated,
                        })
                    }
                    Some((false, _)) => Some(ItemContext {
                        ordered: false,
                        number: 0,
                    }),
                    None => None,
                };
                self.pending_item = ctx;
            }
            Tag::Table(_) => {
                self.flush_runs();
                self.in_table = true;
                self.table_rows.clear();
                self.table_cells.clear();
            }
            Tag::TableCell => {
                self.in_table_cell = true;
                self.cell_buffer.clear();
            }
            Tag::FootnoteDefinition(_) => {
                self.flush_runs();
                self.footnote_depth += 1;
            }
            Tag::Strong => self.bold_depth += 1,
            Tag::Emphasis => self.italic_depth += 1,
            _ => {}
        }
    }

    fn end_tag(&mut self, tag: TagEnd) {
        match tag {
            TagEnd::Heading(_) => {
                if let Some(level) = self.heading_level.take() {
                    let runs = self.take_runs();
                    if !runs.is_empty() {
                        self.blocks.push(Block::Heading(level, runs));
                    }
                }
            }
            TagEnd::Paragraph => self.flush_runs(),
            TagEnd::BlockQuote(_) => {
                self.flush_runs();
                self.quote_depth = self.quote_depth.saturating_sub(1);
            }
            TagEnd::CodeBlock => {
                self.in_code_block = false;
                while self.code_buffer.ends_with('\n') {
                    self.code_buffer.pop();
                }
                for line in self.code_buffer.lines() {
                    self.blocks.push(Block::CodeLine(line.to_string()));
                }
                self.code_buffer.clear();
                self.blocks.push(Block::Spacer(0.4));
            }
            TagEnd::List(_) => {
                let was_open = !self.list_stack.is_empty();
                self.list_stack.pop();
                if was_open && self.list_stack.is_empty() {
                    self.blocks.push(Block::Spacer(0.45));
                }
            }
            TagEnd::Item => {
                self.flush_runs();
                self.pending_item = None;
            }
            TagEnd::TableCell => {
                self.in_table_cell = false;
                self.table_cells.push(std::mem::take(&mut self.cell_buffer));
            }
            TagEnd::TableRow | TagEnd::TableHead => {
                if !self.table_cells.is_empty() {
                    self.table_rows.push(self.table_cells.join(" | "));
                    self.table_cells.clear();
                }
            }
            TagEnd::Table => {
                self.in_table = false;
                self.blocks
                    .push(Block::Table(std::mem::take(&mut self.table_rows)));
            }
            TagEnd::FootnoteDefinition => {
                self.footnote_depth = self.footnote_depth.saturating_sub(1);
            }
            TagEnd::Strong => self.bold_depth = self.bold_depth.saturating_sub(1),
            TagEnd::Emphasis => self.italic_depth = self.italic_depth.saturating_sub(1),
            _ => {}
        }
    }

    fn text_event(&mut self, text: CowStr<'_>) {
        if self.in_code_block {
            // Fenced block content is delivered via Event::Text.
            self.code_buffer.push_str(&text);
            return;
        }
        if self.footnote_depth > 0 {
            return;
        }
        if self.in_table_cell {
            self.cell_buffer.push_str(&text);
            return;
        }
        let kind = if self.bold_depth > 0 {
            RunKind::Bold
        } else if self.italic_depth > 0 {
            RunKind::Italic
        } else {
            RunKind::Plain
        };
        self.push_run(kind, text.to_string());
    }

    fn code_event(&mut self, code: CowStr<'_>) {
        if self.in_code_block {
            self.code_buffer.push_str(&code);
        } else if self.in_table_cell {
            self.cell_buffer.push_str(&code);
        } else {
            self.push_run(RunKind::Code, code.to_string());
        }
    }

    fn break_event(&mut self) {
        if self.in_code_block {
            self.code_buffer.push('\n');
        } else if self.in_table_cell {
            self.cell_buffer.push(' ');
        } else {
            self.push_run(RunKind::Plain, " ".to_string());
        }
    }

    fn push_run(&mut self, kind: RunKind, text: String) {
        if text.is_empty() {
            return;
        }
        let merges_with_last = matches!(
            (self.runs.last(), kind),
            (Some(InlineRun::Plain(_)), RunKind::Plain)
                | (Some(InlineRun::Bold(_)), RunKind::Bold)
                | (Some(InlineRun::Italic(_)), RunKind::Italic)
                | (Some(InlineRun::Code(_)), RunKind::Code)
        );
        if !merges_with_last {
            self.runs.push(match kind {
                RunKind::Plain => InlineRun::Plain(text),
                RunKind::Bold => InlineRun::Bold(text),
                RunKind::Italic => InlineRun::Italic(text),
                RunKind::Code => InlineRun::Code(text),
            });
            return;
        }
        let last = self
            .runs
            .last_mut()
            .expect("runs non-empty by matches above");
        match (kind, last) {
            (RunKind::Plain, InlineRun::Plain(existing)) => existing.push_str(&text),
            (RunKind::Bold, InlineRun::Bold(existing)) => existing.push_str(&text),
            (RunKind::Italic, InlineRun::Italic(existing)) => existing.push_str(&text),
            (RunKind::Code, InlineRun::Code(existing)) => existing.push_str(&text),
            _ => unreachable!("merges_with_last guarantees the same variant"),
        }
    }

    fn take_runs(&mut self) -> Vec<InlineRun> {
        std::mem::take(&mut self.runs)
    }

    /// Flushes buffered inline runs into a finished block. The destination is
    /// resolved by the enclosing context: list item > blockquote > paragraph.
    fn flush_runs(&mut self) {
        if self.runs.is_empty() {
            return;
        }
        if self.footnote_depth > 0 {
            self.runs.clear();
            return;
        }
        let runs = self.take_runs();
        if self.pending_item.is_some() || !self.list_stack.is_empty() {
            self.push_list_item(runs);
        } else if self.quote_depth > 0 {
            self.push_quote(runs);
        } else {
            self.blocks.push(Block::Paragraph(runs));
        }
    }

    fn push_list_item(&mut self, runs: Vec<InlineRun>) {
        match self.pending_item {
            Some(ItemContext {
                ordered: true,
                number,
            }) => self.blocks.push(Block::Ordered(number, runs)),
            _ => self.blocks.push(Block::Bullet(runs)),
        }
    }

    fn push_quote(&mut self, runs: Vec<InlineRun>) {
        match self.blocks.last_mut() {
            // Successive paragraphs inside one blockquote merge into a single
            // quote block separated by a space.
            Some(Block::Quote(existing)) => {
                if let Some(InlineRun::Plain(last)) = existing.last_mut() {
                    last.push(' ');
                } else {
                    existing.push(InlineRun::Plain(" ".into()));
                }
                existing.extend(runs);
            }
            _ => self.blocks.push(Block::Quote(runs)),
        }
    }
}

/// Maps a pulldown heading level to a 1-based block level capped at 3.
fn heading_index(level: HeadingLevel) -> usize {
    match level {
        HeadingLevel::H1 => 1,
        HeadingLevel::H2 => 2,
        HeadingLevel::H3 => 3,
        _ => 3,
    }
}

/// Flattens inline runs into the plain text of a block.
#[must_use]
fn runs_plain(runs: &[InlineRun]) -> String {
    let mut plain = String::new();
    for run in runs {
        match run {
            InlineRun::Plain(text)
            | InlineRun::Bold(text)
            | InlineRun::Italic(text)
            | InlineRun::Code(text) => plain.push_str(text),
        }
    }
    plain
}

// ---------------------------------------------------------------------------
// PDF export: scripts, fonts and bidi
// ---------------------------------------------------------------------------

/// The writing system a block belongs to, used for font selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ScriptKind {
    /// Latin and other left-to-right scripts (default family).
    Latin,
    /// Arabic or Persian (Vazirmatn family).
    Arabic,
    /// Chinese/Japanese/Korean (system fallback family).
    Cjk,
}

/// Checks whether `c` belongs to a CJK writing system (Han, Kana, Hangul).
#[must_use]
fn is_cjk_char(c: char) -> bool {
    matches!(
        c,
        '\u{AC00}'..='\u{D7AF}'    // Hangul syllables
            | '\u{1100}'..='\u{11FF}' // Hangul jamo
            | '\u{3130}'..='\u{318F}' // Hangul compatibility jamo
            | '\u{A960}'..='\u{A97F}' // Hangul jamo extended-A
            | '\u{D7B0}'..='\u{D7FF}' // Hangul jamo extended-B
            | '\u{3000}'..='\u{30FF}' // CJK symbols/punctuation, Hiragana, Katakana
            | '\u{31F0}'..='\u{31FF}' // Katakana phonetic extensions
            | '\u{4E00}'..='\u{9FFF}' // CJK unified ideographs
            | '\u{3400}'..='\u{4DBF}' // CJK extension A
            | '\u{F900}'..='\u{FAFF}' // CJK compatibility ideographs
            | '\u{FF00}'..='\u{FFEF}' // fullwidth and halfwidth forms
    )
}

/// Chooses the font family for a block from its characters.
///
/// CJK only wins when a CJK fallback font was discovered; otherwise those
/// codepoints fall through to the default family and render as tofu.
#[must_use]
fn block_script(text: &str, cjk_available: bool) -> ScriptKind {
    let mut arabic = false;
    for c in text.chars() {
        if is_cjk_char(c) && cjk_available {
            return ScriptKind::Cjk;
        } else if is_persian_or_arabic(c) {
            arabic = true;
        }
    }
    if arabic {
        ScriptKind::Arabic
    } else {
        ScriptKind::Latin
    }
}

/// Searches common system font locations for a plain CJK-capable TrueType
/// face.
///
/// `rusttype` cannot read font collections (`.ttc`/`.otf`), so only plain
/// `.ttf` candidates are probed. Returns the raw font data of the first
/// parseable candidate, or `None` when no usable face is installed on this
/// machine.
fn find_cjk_system_font_data() -> Option<Vec<u8>> {
    const CJK_CANDIDATES: &[&str] = &[
        "C:/Windows/Fonts/simhei.ttf",
        "C:/Windows/Fonts/simkai.ttf",
        "C:/Windows/Fonts/simfang.ttf",
        "C:/Windows/Fonts/Deng.ttf",
        "C:/Windows/Fonts/Dengl.ttf",
        "C:/Windows/Fonts/Dengb.ttf",
        "C:/Windows/Fonts/DFKai-SB.ttf",
        "C:/Windows/Fonts/ArialUni.ttf",
        "/System/Library/Fonts/Supplemental/Arial Unicode.ttf",
        "/usr/share/fonts/truetype/droid/DroidSansFallbackFull.ttf",
    ];

    for candidate in CJK_CANDIDATES {
        if fs::metadata(candidate).is_ok() {
            if let Ok(data) = fs::read(candidate) {
                if genpdf::fonts::FontData::new(data.clone(), None).is_ok() {
                    return Some(data);
                }
            }
        }
    }
    None
}

/// Converts a *logical*-order line into *visual* order for PDF drawing.
///
/// The Unicode bidi algorithm assigns an embedding level to every byte; runs
/// of odd (RTL) levels have their characters reversed and runs are emitted in
/// the order returned by `visual_runs` (absolute byte ranges, visual order).
/// Pure LTR input is returned unchanged via a fast path.
#[must_use]
fn bidi_visual_line(logical_line: &str) -> String {
    use unicode_bidi::BidiInfo;

    let info = BidiInfo::new(logical_line, None);
    if !info.has_rtl() {
        return logical_line.to_string();
    }

    let mut visual = String::with_capacity(logical_line.len());
    for para in &info.paragraphs {
        let (levels, runs) = info.visual_runs(para, para.range.clone());
        for run in runs {
            let slice = &logical_line[run.clone()];
            if levels[run.start].is_rtl() {
                visual.extend(slice.chars().rev());
            } else {
                visual.push_str(slice);
            }
        }
    }
    visual
}

/// Returns whether the first alphabetic character of `text` is RTL; used to
/// select paragraph alignment.
fn is_text_rtl(text: &str) -> bool {
    has_strong_rtl(text)
}

// ---------------------------------------------------------------------------
// PDF export: wrapping and layout
// ---------------------------------------------------------------------------

/// Font sizes used by the PDF engine, in points.
mod pdf_sizes {
    /// Size of the page header line.
    pub const HEADER: u8 = 9;
    /// Size of fenced code blocks and inline code.
    pub const CODE: u8 = 10;
    /// Default paragraph size.
    pub const BODY: u8 = 11;
    /// Heading sizes for levels one to four.
    pub const HEADINGS: [u8; 4] = [20, 16, 13, 12];
}

/// Line spacing factor used inside the PDF document.
const PDF_LINE_SPACING: f64 = 1.15;
/// Left and right page margin in millimetres (A4 = 210 mm wide).
const PDF_PAGE_MARGIN_LR: f64 = 23.0;
/// Top page margin in millimetres; the header line sits inside this band.
const PDF_PAGE_MARGIN_TOP: f64 = 14.0;
/// Bottom page margin in millimetres.
const PDF_PAGE_MARGIN_BOTTOM: f64 = 20.0;
/// Width available to content on an A4 sheet, in millimetres.
const PDF_CONTENT_WIDTH_MM: f64 = 210.0 - 2.0 * PDF_PAGE_MARGIN_LR;
/// Horizontal indent of quoted text on each side, in millimetres.
const PDF_QUOTE_INDENT_MM: f64 = 10.0;
/// Extra width consumed by a bullet marker, in millimetres.
const PDF_BULLET_INDENT_MM: f64 = 5.0;
/// Extra width consumed by an ordered list marker, in millimetres.
const PDF_ORDERED_INDENT_MM: f64 = 7.0;
/// Code block frame indent on each side, in millimetres.
const PDF_CODE_INDENT_MM: f64 = 6.0;
/// Body text color (near black, print friendly).
const PDF_INK: genpdf::style::Color = genpdf::style::Color::Rgb(26, 26, 26);
/// Muted color for blockquotes, inline code and page furniture.
const PDF_MUTED: genpdf::style::Color = genpdf::style::Color::Rgb(85, 88, 96);
/// Muted color for the page header line.
const PDF_HEADER_INK: genpdf::style::Color = genpdf::style::Color::Rgb(140, 142, 150);
/// Line used to render a thematic break.
const PDF_RULE_TEXT: &str = "────────────────────────────────────────────────────";

/// Wraps `text` into lines that each measure at most `max_width` millimetres
/// when rendered with `style`.
///
/// Words are split on spaces (runs of spaces collapse); tokens wider than a
/// full line are hard-split by character (CJK text, long URLs). The greedy fit
/// uses the exact same condition as genpdf's internal wrapper
/// (`x + width > max` means the token must move), so pre-wrapped lines are
/// never re-wrapped by genpdf.
fn wrap_line_to_width(
    font_cache: &genpdf::fonts::FontCache,
    style: &genpdf::style::Style,
    text: &str,
    max_width: f64,
) -> Vec<String> {
    let max = genpdf::Mm::from(max_width);
    let space_width = style.str_width(font_cache, " ");

    let mut lines: Vec<String> = Vec::new();
    let mut current = String::new();
    let mut current_width = genpdf::Mm::default();

    for token in text.split(' ') {
        if token.is_empty() {
            continue;
        }
        let token_width = style.str_width(font_cache, token);
        if current.is_empty() {
            if token_width > max {
                lines.extend(hard_split_chars(font_cache, style, token, max));
            } else {
                current.push_str(token);
                current_width = token_width;
            }
        } else if current_width + space_width + token_width <= max {
            current.push(' ');
            current.push_str(token);
            current_width = current_width + space_width + token_width;
        } else {
            lines.push(std::mem::take(&mut current));
            if token_width > max {
                lines.extend(hard_split_chars(font_cache, style, token, max));
            } else {
                current.push_str(token);
                current_width = token_width;
            }
        }
    }

    if !current.is_empty() {
        lines.push(current);
    }
    if lines.is_empty() {
        lines.push(String::new());
    }
    lines
}

/// Splits an over-long token into chunks that each fit `max_width`.
fn hard_split_chars(
    font_cache: &genpdf::fonts::FontCache,
    style: &genpdf::style::Style,
    token: &str,
    max_width: genpdf::Mm,
) -> Vec<String> {
    let mut chunks = Vec::new();
    let mut chunk = String::new();
    let mut width = genpdf::Mm::default();

    for ch in token.chars() {
        let ch_width = style.char_width(font_cache, ch);
        if !chunk.is_empty() && width + ch_width > max_width {
            chunks.push(std::mem::take(&mut chunk));
            width = genpdf::Mm::default();
        }
        chunk.push(ch);
        width += ch_width;
    }
    if !chunk.is_empty() {
        chunks.push(chunk);
    }
    chunks
}

// ---------------------------------------------------------------------------
// PDF export: rendering
// ---------------------------------------------------------------------------

/// All font faces the PDF engine needs for one export run.
struct PdfFaces {
    /// Latin family — bundled IBM Plex Mono (the default family).
    plex: genpdf::fonts::FontFamily<genpdf::fonts::Font>,
    /// Arabic/Persian family — bundled Vazirmatn (regular and bold).
    vazir: genpdf::fonts::FontFamily<genpdf::fonts::Font>,
    /// CJK family — discovered system font, if any.
    cjk: Option<genpdf::fonts::FontFamily<genpdf::fonts::Font>>,
}

/// Emits parsed blocks into a genpdf document.
struct PdfWriter<'a> {
    faces: &'a PdfFaces,
    document: &'a mut genpdf::Document,
}

impl PdfWriter<'_> {
    /// Returns the family registered for `script`.
    fn family(&self, script: ScriptKind) -> genpdf::fonts::FontFamily<genpdf::fonts::Font> {
        match script {
            ScriptKind::Latin => self.faces.plex,
            ScriptKind::Arabic => self.faces.vazir,
            ScriptKind::Cjk => self.faces.cjk.unwrap_or(self.faces.vazir),
        }
    }

    /// Builds the render style for a script/size/color triple.
    fn style(
        &self,
        script: ScriptKind,
        size: u8,
        color: genpdf::style::Color,
    ) -> genpdf::style::Style {
        genpdf::style::Style::new()
            .with_font_family(self.family(script))
            .with_font_size(size)
            .with_color(color)
    }

    /// Emits a plain-text block whose lines are wrapped and — for RTL —
    /// bidi-reordered in-house; each wrapped line becomes its own aligned
    /// paragraph so genpdf never re-wraps RTL content in logical order.
    fn emit_literal_lines(
        &mut self,
        text: &str,
        size: u8,
        color: genpdf::style::Color,
        width_mm: f64,
        bold: bool,
        reorder: bool,
    ) {
        let script = block_script(text, self.faces.cjk.is_some());
        let style = self.style(script, size, color);
        let style = if bold { style.bold() } else { style };
        let align_right = is_text_rtl(text);

        for line in wrap_line_to_width(self.document.font_cache(), &style, text, width_mm) {
            let visual = if reorder {
                bidi_visual_line(&line)
            } else {
                line
            };
            let alignment = if align_right {
                genpdf::Alignment::Right
            } else {
                genpdf::Alignment::Left
            };
            self.document.push(
                genpdf::elements::Paragraph::default()
                    .styled_string(visual, style)
                    .aligned(alignment),
            );
        }
    }

    /// Emits an inline-styled paragraph directly into genpdf's own wrapping
    /// pipeline; used only for pure Latin content where no reordering is
    /// required and bold/inline-code styling should survive.
    fn emit_styled_paragraph(
        &mut self,
        runs: &[InlineRun],
        size: u8,
        color: genpdf::style::Color,
        pad_lr: f64,
    ) {
        let mut para = genpdf::elements::Paragraph::default();
        for run in runs {
            let base = genpdf::style::Style::new()
                .with_font_size(size)
                .with_color(color);
            match run {
                InlineRun::Plain(text) | InlineRun::Italic(text) => {
                    para.push_styled(text.clone(), base);
                }
                InlineRun::Bold(text) => para.push_styled(text.clone(), base.bold()),
                InlineRun::Code(text) => para.push_styled(
                    text.clone(),
                    genpdf::style::Style::new()
                        .with_font_size(pdf_sizes::CODE)
                        .with_color(PDF_MUTED),
                ),
            }
        }
        if pad_lr > 0.0 {
            self.document.push(
                para.aligned(genpdf::Alignment::Left)
                    .padded(genpdf::Margins::trbl(0.0, pad_lr, 0.0, pad_lr)),
            );
        } else {
            self.document.push(para);
        }
    }

    /// Emits one fenced-code line in logical order with a mono frame indent.
    fn emit_code_line(&mut self, line: &str) {
        let style = self.style(ScriptKind::Latin, pdf_sizes::CODE, PDF_INK);
        let width = PDF_CONTENT_WIDTH_MM - 2.0 * PDF_CODE_INDENT_MM;
        for wrapped in wrap_line_to_width(self.document.font_cache(), &style, line, width) {
            self.document.push(
                genpdf::elements::Paragraph::default()
                    .styled_string(wrapped, style)
                    .padded(genpdf::Margins::trbl(
                        0.0,
                        PDF_CODE_INDENT_MM,
                        0.0,
                        PDF_CODE_INDENT_MM,
                    )),
            );
        }
    }

    /// Renders a text block as either the styled Latin path or the manual
    /// visual-order path, depending on its script.
    fn emit_runs(
        &mut self,
        runs: &[InlineRun],
        size: u8,
        color: genpdf::style::Color,
        indent: f64,
    ) {
        let plain = runs_plain(runs);
        if plain.trim().is_empty() {
            return;
        }
        match block_script(&plain, self.faces.cjk.is_some()) {
            ScriptKind::Latin => self.emit_styled_paragraph(runs, size, color, indent),
            ScriptKind::Arabic | ScriptKind::Cjk => {
                let width = PDF_CONTENT_WIDTH_MM - 2.0 * indent;
                self.emit_literal_lines(&plain, size, color, width, false, true);
            }
        }
    }

    /// Dispatches one block to the right emit path, including the surrounding
    /// line-break furniture.
    fn emit_block(&mut self, block: Block) {
        match block {
            Block::Heading(level, runs) => {
                let plain = runs_plain(&runs);
                if plain.trim().is_empty() {
                    return;
                }
                self.document.push(genpdf::elements::Break::new(0.9));
                let size = pdf_sizes::HEADINGS[(level.saturating_sub(1)).min(3)];
                let script = block_script(&plain, self.faces.cjk.is_some());
                if script == ScriptKind::Latin {
                    self.document.push(
                        genpdf::elements::Paragraph::default()
                            .styled_string(
                                plain,
                                self.style(ScriptKind::Latin, size, PDF_INK).bold(),
                            )
                            .aligned(genpdf::Alignment::Left),
                    );
                } else {
                    self.emit_literal_lines(
                        &plain,
                        size,
                        PDF_INK,
                        PDF_CONTENT_WIDTH_MM,
                        true,
                        true,
                    );
                }
            }
            Block::Paragraph(runs) => {
                self.emit_runs(&runs, pdf_sizes::BODY, PDF_INK, 0.0);
                self.document.push(genpdf::elements::Break::new(0.4));
            }
            Block::Quote(runs) => {
                self.emit_runs(&runs, pdf_sizes::BODY, PDF_MUTED, PDF_QUOTE_INDENT_MM);
                self.document.push(genpdf::elements::Break::new(0.45));
            }
            Block::CodeLine(line) => self.emit_code_line(&line),
            Block::Bullet(runs) => {
                let plain = runs_plain(&runs);
                if plain.trim().is_empty() {
                    return;
                }
                let text = format!("•   {}", plain.trim_start());
                self.emit_literal_lines(
                    &text,
                    pdf_sizes::BODY,
                    PDF_INK,
                    PDF_CONTENT_WIDTH_MM - PDF_BULLET_INDENT_MM,
                    false,
                    true,
                );
            }
            Block::Ordered(number, runs) => {
                let plain = runs_plain(&runs);
                if plain.trim().is_empty() {
                    return;
                }
                let text = format!("{number}.  {}", plain.trim_start());
                self.emit_literal_lines(
                    &text,
                    pdf_sizes::BODY,
                    PDF_INK,
                    PDF_CONTENT_WIDTH_MM - PDF_ORDERED_INDENT_MM,
                    false,
                    true,
                );
            }
            Block::Table(rows) => {
                for row in rows {
                    self.emit_literal_lines(
                        &row,
                        pdf_sizes::CODE,
                        PDF_MUTED,
                        PDF_CONTENT_WIDTH_MM,
                        false,
                        true,
                    );
                }
                self.document.push(genpdf::elements::Break::new(0.4));
            }
            Block::Rule => {
                self.emit_literal_lines(
                    PDF_RULE_TEXT,
                    pdf_sizes::HEADER,
                    PDF_MUTED,
                    PDF_CONTENT_WIDTH_MM,
                    false,
                    false,
                );
                self.document.push(genpdf::elements::Break::new(0.9));
            }
            Block::Spacer(lines) => self.document.push(genpdf::elements::Break::new(lines)),
        }
    }
}

/// Composes the page header line for page `page`.
///
/// RTL documents read `Page N — Title`; LTR documents read
/// `Title — Page N`.
fn page_header_text(title: &str, page_label: &str, page: usize) -> String {
    let rtl = has_strong_rtl(&format!("{title} {page_label}"));
    if rtl {
        format!("{page_label} {page} — {title}")
    } else {
        format!("{title} — {page_label} {page}")
    }
}

/// Loads a bundled TrueType face into genpdf.
///
/// # Errors
///
/// Returns a readable error when the embedded font data cannot be parsed.
fn font_data(data: Vec<u8>) -> Result<genpdf::fonts::FontData, String> {
    genpdf::fonts::FontData::new(data, None).map_err(|e| format!("font load failed: {e}"))
}

/// Loads a bundled TrueType face referenced by an `include_bytes!` static.
///
/// # Errors
///
/// Returns a readable error when the embedded font data cannot be parsed.
fn font_data_bundled(data: &'static [u8]) -> Result<genpdf::fonts::FontData, String> {
    font_data(data.to_vec())
}

/// Builds the optional CJK fallback family from a discovered system font.
///
/// Only the regular slot uses the system font; the remaining slots fall back
/// to bundled Plex faces so a CJK document does not pay for six embedded
/// copies of a multi-megabyte face.
fn cjk_family_from_system(
) -> Result<Option<genpdf::fonts::FontFamily<genpdf::fonts::FontData>>, String> {
    match find_cjk_system_font_data() {
        None => Ok(None),
        Some(data) => {
            let regular = font_data(data)?;
            let bold = font_data_bundled(crate::IBM_PLEX_MONO_BOLD)?;
            let italic = font_data_bundled(crate::IBM_PLEX_MONO_REGULAR)?;
            let bold_italic = font_data_bundled(crate::IBM_PLEX_MONO_BOLD)?;
            Ok(Some(genpdf::fonts::FontFamily {
                regular,
                bold,
                italic,
                bold_italic,
            }))
        }
    }
}

/// Composes the set of faces used for one PDF export.
///
/// The default IBM Plex Mono family is installed by [`genpdf::Document::new`];
/// Vazirmatn and the optional CJK fallback are registered afterwards.
fn load_faces(document: &mut genpdf::Document) -> Result<PdfFaces, String> {
    let vazir_data = genpdf::fonts::FontFamily {
        regular: font_data_bundled(crate::VAZIRMATN_REGULAR)?,
        bold: font_data_bundled(crate::VAZIRMATN_BOLD)?,
        italic: font_data_bundled(crate::VAZIRMATN_REGULAR)?,
        bold_italic: font_data_bundled(crate::VAZIRMATN_BOLD)?,
    };
    let vazir = document.add_font_family(vazir_data);
    let cjk = cjk_family_from_system()?.map(|data| document.add_font_family(data));
    Ok(PdfFaces {
        plex: document.font_cache().default_font_family(),
        vazir,
        cjk,
    })
}

/// Builds the page decorator: page margins plus a centered title/page header.
fn build_page_decorator(
    title: &str,
    page_label: &str,
    header_plex: genpdf::fonts::FontFamily<genpdf::fonts::Font>,
    header_vazir: genpdf::fonts::FontFamily<genpdf::fonts::Font>,
) -> genpdf::SimplePageDecorator {
    let title = title.to_string();
    let page_label = page_label.to_string();
    let mut decorator = genpdf::SimplePageDecorator::new();
    decorator.set_margins(genpdf::Margins::trbl(
        PDF_PAGE_MARGIN_TOP,
        PDF_PAGE_MARGIN_LR,
        PDF_PAGE_MARGIN_BOTTOM,
        PDF_PAGE_MARGIN_LR,
    ));
    decorator.set_header(move |page: usize| {
        let text = page_header_text(&title, &page_label, page);
        let family = if has_strong_rtl(&text) {
            header_vazir
        } else {
            header_plex
        };
        genpdf::elements::Paragraph::default()
            .styled_string(
                bidi_visual_line(&text),
                genpdf::style::Style::new()
                    .with_font_size(pdf_sizes::HEADER)
                    .with_color(PDF_HEADER_INK)
                    .with_font_family(family),
            )
            .aligned(genpdf::Alignment::Center)
            .padded(genpdf::Margins::trbl(2.0, 0.0, 4.0, 0.0))
    });
    decorator
}

/// Renders `markdown` as a paginated A4 PDF at `path`.
///
/// The PDF uses the fonts bundled with Krebs: IBM Plex Mono for Latin
/// scripts, Vazirmatn for Arabic/Persian and — when a suitable system font
/// exists — a CJK fallback. Every page carries a centered header with the
/// document title and the localized page label (`page_label`).
///
/// # Errors
///
/// Returns a human-readable error when font loading, layout or file writing
/// fails.
pub fn export_pdf(
    markdown: &str,
    title: &str,
    path: &Path,
    page_label: &str,
) -> Result<(), String> {
    let plex_data = genpdf::fonts::FontFamily {
        regular: font_data_bundled(crate::IBM_PLEX_MONO_REGULAR)?,
        bold: font_data_bundled(crate::IBM_PLEX_MONO_BOLD)?,
        italic: font_data_bundled(crate::IBM_PLEX_MONO_REGULAR)?,
        bold_italic: font_data_bundled(crate::IBM_PLEX_MONO_BOLD)?,
    };

    let mut document = genpdf::Document::new(plex_data);
    document.set_title(title);
    document.set_font_size(pdf_sizes::BODY);
    document.set_line_spacing(PDF_LINE_SPACING);
    document.set_minimal_conformance();

    let faces = load_faces(&mut document)?;
    let header_plex = faces.plex;
    let header_vazir = faces.vazir;
    document.set_page_decorator(build_page_decorator(
        title,
        page_label,
        header_plex,
        header_vazir,
    ));

    {
        let mut writer = PdfWriter {
            faces: &faces,
            document: &mut document,
        };
        for block in parse_blocks(markdown) {
            writer.emit_block(block);
        }
    }

    document
        .render_to_file(path)
        .map_err(|e| format!("PDF render failed: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_markdown_to_html_export() {
        let md = "# Einleitung\n\nDas ist ein wichtiger **Test**.\n\n- Punkt 1\n- Punkt 2";
        let html = markdown_to_html(md, "B2 Aufsatz");
        assert!(html.contains("<title>B2 Aufsatz</title>"));
        assert!(html.contains("<h1>Einleitung</h1>"));
        assert!(html.contains("<strong>Test</strong>"));
        assert!(html.contains("IBM Plex Mono"));
        assert!(html.contains("@media print"));
    }

    #[test]
    fn test_bidi_html_injection() {
        let md = "سلام این یک پاراگراف فارسی است.";
        let html = markdown_to_html(md, "Persian Essay");
        assert!(html.contains("dir=\"rtl\""));
        assert!(html.contains("lang=\"fa\""));
        assert!(html.contains("class=\"rtl-block\""));
    }

    #[test]
    fn test_frontmatter_contains_metadata() {
        let out = markdown_with_frontmatter("نور نور است", "شروع", "fa");
        assert!(out.starts_with("---\n"));
        assert!(out.contains("title: \"شروع\""));
        assert!(out.contains("words: 3"));
        assert!(out.contains("language: fa"));
        assert!(out.contains("\n---\n\n"));
        assert!(out.trim_end().ends_with("نور نور است"));
    }

    #[test]
    fn test_frontmatter_escapes_quotes() {
        let out = markdown_with_frontmatter("body", "Der \"K\"affee", "de");
        assert!(out.contains("title: \"Der \\\"K\\\"affee\""));
    }

    #[test]
    fn test_word_count_matches_document_stats() {
        assert_eq!(word_count("one two three"), 3);
        assert_eq!(word_count("زیر نوران نور"), 3);
        assert_eq!(word_count(""), 0);
    }

    #[test]
    fn test_bidi_visual_line_pure_rtl_reverses() {
        let input = "سلام";
        let out = bidi_visual_line(input);
        let reversed: String = input.chars().rev().collect();
        assert_eq!(out, reversed);
    }

    #[test]
    fn test_bidi_visual_line_pure_ltr_unchanged() {
        assert_eq!(bidi_visual_line("Hello world!"), "Hello world!");
        assert_eq!(bidi_visual_line("230.- 42"), "230.- 42");
    }

    #[test]
    fn test_bidi_visual_line_keeps_char_pool() {
        let input = "سلام دوست";
        let out = bidi_visual_line(input);
        let mut a: Vec<char> = input.chars().collect();
        let mut b: Vec<char> = out.chars().collect();
        a.sort_unstable();
        b.sort_unstable();
        assert_eq!(a, b);
        assert_eq!(input.chars().count(), out.chars().count());
    }

    #[test]
    fn test_block_script_detection() {
        assert_eq!(block_script("Hello world", false), ScriptKind::Latin);
        assert_eq!(block_script("سلام دنیا", true), ScriptKind::Arabic);
        assert_eq!(block_script("中文测试", false), ScriptKind::Latin);
        assert_eq!(block_script("中文测试", true), ScriptKind::Cjk);
        assert_eq!(
            block_script("Eins zwei سه", true),
            ScriptKind::Arabic,
            "any Arabic char wins unless CJK"
        );
    }

    #[test]
    fn test_is_cjk_char() {
        assert!(is_cjk_char('中'));
        assert!(is_cjk_char('あ'));
        assert!(is_cjk_char('한'));
        assert!(!is_cjk_char('a'));
        assert!(!is_cjk_char('س'));
    }

    #[test]
    fn test_parse_blocks_headings_and_paragraphs() {
        let blocks = parse_blocks("# Titel\n\nEin *kurzer* Absatz mit **fett**.\n");
        assert_eq!(
            blocks[0],
            Block::Heading(1, vec![InlineRun::Plain("Titel".into())])
        );
        assert_eq!(
            blocks[1],
            Block::Paragraph(vec![
                InlineRun::Plain("Ein ".into()),
                InlineRun::Italic("kurzer".into()),
                InlineRun::Plain(" Absatz mit ".into()),
                InlineRun::Bold("fett".into()),
                InlineRun::Plain(".".into()),
            ])
        );
    }

    #[test]
    fn test_parse_blocks_lists() {
        let blocks = parse_blocks("1. eerste\n2. tweede\n\n- bullet a\n- bullet b\n");
        assert_eq!(
            blocks[0],
            Block::Ordered(1, vec![InlineRun::Plain("eerste".into())])
        );
        assert_eq!(
            blocks[1],
            Block::Ordered(2, vec![InlineRun::Plain("tweede".into())])
        );
        assert!(matches!(blocks[2], Block::Spacer(_)), "gap after the list");
        assert_eq!(
            blocks[3],
            Block::Bullet(vec![InlineRun::Plain("bullet a".into())])
        );
        assert_eq!(
            blocks[4],
            Block::Bullet(vec![InlineRun::Plain("bullet b".into())])
        );
    }

    #[test]
    fn test_parse_blocks_quote_rule_and_code() {
        let blocks = parse_blocks("> ein Zitat\n\n---\n\n```rust\nfn a() {}\n```\n\nEnde\n");
        assert!(blocks
            .iter()
            .any(|b| matches!(b, Block::Quote(runs) if runs_plain(runs).contains("ein Zitat"))));
        assert!(blocks.iter().any(|b| matches!(b, Block::Rule)));
        assert!(blocks
            .iter()
            .any(|b| matches!(b, Block::CodeLine(line) if line == "fn a() {}")));
        assert!(blocks
            .iter()
            .any(|b| matches!(b, Block::Paragraph(runs) if runs_plain(runs).contains("Ende"))));
    }

    #[test]
    fn test_parse_blocks_table() {
        let blocks = parse_blocks("| a | b |\n|---|---|\n| 1 | 2 |\n");
        assert_eq!(
            blocks[0],
            Block::Table(vec!["a | b".to_string(), "1 | 2".to_string()])
        );
    }

    #[test]
    fn test_runs_plain_concatenates() {
        let runs = vec![
            InlineRun::Plain("a ".into()),
            InlineRun::Bold("b".into()),
            InlineRun::Italic(" c".into()),
        ];
        assert_eq!(runs_plain(&runs), "a b c");
    }

    #[test]
    fn test_page_header_text_sides() {
        assert_eq!(page_header_text("Essay", "Page", 4), "Essay — Page 4");
        let rtl_header = page_header_text("مقاله", "صفحه", 2);
        assert!(rtl_header.starts_with("صفحه 2 — مقاله"));
    }

    #[test]
    fn test_yaml_scalar_escaping() {
        assert_eq!(yaml_scalar("plain"), "\"plain\"");
        assert_eq!(yaml_scalar("back\\slash"), "\"back\\\\slash\"");
    }
}
