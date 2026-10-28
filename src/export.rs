use pulldown_cmark::{html, Options, Parser};
use std::fs;
use std::path::Path;

use crate::editor::is_persian_or_arabic;

/// Converts markdown text into a styled, print-ready, self-contained HTML document.
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

/// Injects dir="rtl" and lang="fa" on block tags starting with Persian characters.
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

pub fn export_to_file(markdown: &str, title: &str, path: &Path) -> std::io::Result<()> {
    let html = markdown_to_html(markdown, title);
    fs::write(path, html)
}

pub fn copy_html_to_clipboard(markdown: &str, title: &str) -> Result<(), String> {
    let html = markdown_to_html(markdown, title);
    let mut clipboard = arboard::Clipboard::new().map_err(|e| e.to_string())?;
    clipboard
        .set_html(html, Some(markdown.to_string()))
        .map_err(|e| e.to_string())
}

pub fn open_exported_html(path: &Path) -> Result<(), String> {
    open::that(path).map_err(|e| e.to_string())
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
}
