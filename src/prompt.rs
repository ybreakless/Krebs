//! Prompt Studio & MCP Hub — build, refine and export reusable agent briefs.
//!
//! A [`PromptSpec`] is assembled visually in the right drawer (persona, task,
//! context, constraints, output format), merged with an optional
//! Machine-Communication-Protocol (MCP) tool catalog, and rendered as an agent
//! pack that can be pasted into coding agents:
//!
//! | Target | File |
//! |---|---|
//! | Cursor | `.cursor/rules/krebs.mdc` |
//! | Windsurf | `.windsurf/rules/krebs.md` |
//! | Claude Code | `CLAUDE.md` (append) |
//! | Antigravity | `.agent/rules/krebs.md` |

/// Recommended token budget estimate: ≈ 4 characters per token.
const CHARS_PER_TOKEN: f64 = 4.0;

/// One prompt-engineering field of the visual builder.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PromptField {
    /// Who the agent is.
    Persona,
    /// What it must do.
    Task,
    /// Background facts it should know.
    Context,
    /// Rules it must respect, one per line.
    Constraints,
    /// The shape of the expected answer.
    OutputFormat,
}

/// A prompt specification assembled in the builder.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct PromptSpec {
    /// Persona / role line.
    pub persona: String,
    /// Main objective.
    pub task: String,
    /// Working context.
    pub context: String,
    /// One constraint per line.
    pub constraints: String,
    /// Wanted output shape.
    pub output_format: String,
}

impl PromptSpec {
    /// Renders the builder fields into a readable brief (without tools).
    #[must_use]
    pub fn render(&self) -> String {
        let mut out = String::from("# Agent Brief\n\n");
        append_section(&mut out, "## Role", &self.persona);
        append_section(&mut out, "## Objective", &self.task);
        append_section(&mut out, "## Context", &self.context);
        if !self.constraints.trim().is_empty() {
            out.push_str("## Rules\n\n");
            for line in self.constraints.lines() {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }
                out.push_str("- ");
                out.push_str(line.trim_start_matches("- ").trim());
                out.push('\n');
            }
            out.push('\n');
        }
        append_section(&mut out, "## Output format", &self.output_format);
        out.trim_end().to_string()
    }
}

/// Appends a `## Heading` section unless the body is empty.
fn append_section(out: &mut String, heading: &str, body: &str) {
    let body = body.trim();
    if body.is_empty() {
        return;
    }
    out.push_str(heading);
    out.push_str("\n\n");
    out.push_str(body);
    out.push_str("\n\n");
}

/// One tool offered by the MCP catalog.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct McpTool {
    /// Short machine id, e.g. `fs.read`.
    pub id: &'static str,
    /// Human tool group, e.g. `Filesystem — Read`.
    pub name: &'static str,
    /// What the agent may do with it.
    pub description: &'static str,
    /// Example tool-call schema (YAML) injected into the pack.
    pub snippet: &'static str,
}

/// The built-in MCP tool catalog: filesystem, web, database, shell, git.
#[rustfmt::skip]
pub const MCP_TOOLS: [McpTool; 9] = [
    McpTool {
        id: "fs.read",
        name: "Filesystem — Read",
        description: "Read a file from the workspace.",
        snippet: "fs.read:\n  input:  { path: string }\n  output: { contents: string }",
    },
    McpTool {
        id: "fs.write",
        name: "Filesystem — Write",
        description: "Create or overwrite a file in the workspace.",
        snippet: "fs.write:\n  input:  { path: string, contents: string }\n  output: { ok: bool }",
    },
    McpTool {
        id: "fs.list",
        name: "Filesystem — List",
        description: "List files under a directory (recursive glob).",
        snippet: "fs.list:\n  input:  { dir: string, glob?: string }\n  output: { entries: string[] }",
    },
    McpTool {
        id: "web.fetch",
        name: "Web — Fetch",
        description: "Download a URL and return its text content.",
        snippet: "web.fetch:\n  input:  { url: string }\n  output: { text: string, status: number }",
    },
    McpTool {
        id: "web.search",
        name: "Web — Search",
        description: "Query a search engine, return ranked results.",
        snippet: "web.search:\n  input:  { query: string, limit?: number }\n  output: { results: { title: string, url: string }[] }",
    },
    McpTool {
        id: "db.sql",
        name: "Database — SQL",
        description: "Run read-only SQL against a configured database.",
        snippet: "db.sql:\n  input:  { sql: string, params?: any[] }\n  output: { rows: object[] }",
    },
    McpTool {
        id: "shell.run",
        name: "Shell — Run",
        description: "Execute an allowlisted shell command.",
        snippet: "shell.run:\n  input:  { command: string, timeout?: number }\n  output: { stdout: string, exit_code: number }",
    },
    McpTool {
        id: "git.status",
        name: "Git — Status",
        description: "Report changed files and the current branch.",
        snippet: "git.status:\n  input:  { repo: string }\n  output: { branch: string, changed: string[] }",
    },
    McpTool {
        id: "git.commit",
        name: "Git — Commit",
        description: "Stage selected files and create a commit.",
        snippet: "git.commit:\n  input:  { files: string[], message: string }\n  output: { hash: string }",
    },
];

/// Builds the `## Tools (MCP)` section from the enabled toggles.
///
/// `enabled` maps positionally onto [`MCP_TOOLS`]; missing flags count as off.
#[must_use]
pub fn render_tools_section(enabled: &[usize], indent: bool) -> String {
    let mut out = String::new();
    for (index, tool) in MCP_TOOLS.iter().enumerate() {
        if !enabled.contains(&index) {
            continue;
        }
        if indent {
            out.push_str("  ");
        }
        out.push_str("- ");
        out.push_str(&tool.snippet.replace('\n', "\n  "));
        out.push_str("  // ");
        out.push_str(tool.description);
        out.push('\n');
    }
    if out.is_empty() {
        return out;
    }
    let mut section = String::from("## Tools (MCP)\n\nCall tools when they are needed; otherwise answer directly.\n\n");
    if indent {
        section.push_str("  ");
    }
    section.push_str(&out);
    section.push('\n');
    section
}

/// Combines the rendered brief with the enabled tool subset.
#[must_use]
pub fn render_agent_pack(spec: &PromptSpec, enabled: &[usize]) -> String {
    let mut pack = spec.render();
    let tools = render_tools_section(enabled, false);
    if !tools.is_empty() {
        pack.push_str("\n\n");
        pack.push_str(&tools);
    }
    pack.trim_end().to_string()
}

/// Where the agent pack is shipped.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CopyTarget {
    /// `.cursor/rules/krebs.mdc`.
    Cursor,
    /// `.windsurf/rules/krebs.md`.
    Windsurf,
    /// `CLAUDE.md` (project memory).
    Claude,
    /// `.agent/rules/krebs.md`.
    Antigravity,
}

impl CopyTarget {
    /// Every supported target, in display order.
    pub const ALL: [CopyTarget; 4] = [
        CopyTarget::Cursor,
        CopyTarget::Windsurf,
        CopyTarget::Claude,
        CopyTarget::Antigravity,
    ];

    /// Destination file shown next to the copy button.
    #[must_use]
    pub fn filename(&self) -> &'static str {
        match self {
            Self::Cursor => ".cursor/rules/krebs.mdc",
            Self::Windsurf => ".windsurf/rules/krebs.md",
            Self::Claude => "CLAUDE.md (append)",
            Self::Antigravity => ".agent/rules/krebs.md",
        }
    }

    /// One-word label for the copy button.
    #[must_use]
    pub fn label(&self) -> &'static str {
        match self {
            Self::Cursor => "Cursor",
            Self::Windsurf => "Windsurf",
            Self::Claude => "Claude Code",
            Self::Antigravity => "Antigravity",
        }
    }
}

/// Wraps the pack in the destination-specific file format.
#[must_use]
pub fn render_for_target(target: CopyTarget, pack: &str) -> String {
    match target {
        CopyTarget::Cursor => format!(
            "---\ndescription: Krebs Studio generated rules\nglobs: \nalwaysApply: true\n---\n\n{pack}\n"
        ),
        CopyTarget::Windsurf => format!("# Krebs Rules\n\n{pack}\n"),
        CopyTarget::Claude => format!("# Krebs\n\n{pack}\n"),
        CopyTarget::Antigravity => format!("# Antigravity Guidelines — Krebs\n\n{pack}\n"),
    }
}

/// Rough token estimate (≈ 4 characters per token for prose).
#[must_use]
pub fn estimate_tokens(text: &str) -> usize {
    (text.chars().count() as f64 / CHARS_PER_TOKEN).ceil() as usize
}

/// The system instruction for the "Improve Prompt" meta-prompt.
pub const IMPL_SYSTEM_PROMPT: &str = "You are a world-class prompt engineer. \
Improve the given agent brief: sharpen the objective, remove contradictions, \
add concrete success criteria and guardrails, keep every original intent. \
Reply with the improved brief in the same markdown structure — no commentary.";

/// Deterministic offline fallback used when no AI endpoint is reachable:
/// normalizes the brief and injects standard quality guardrails.
#[must_use]
pub fn offline_improve(brief: &str) -> String {
    let mut out = String::from(brief.trim_end());
    out.push_str("\n\n## Quality bar\n\n");
    out.push_str("- Verify your answer answers the objective directly; cut anything unrelated.\n");
    out.push_str("- When facts are missing, state the assumption explicitly instead of guessing.\n");
    out.push_str("- Prefer concrete examples over abstract advice.\n");
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spec() -> PromptSpec {
        PromptSpec {
            persona: "You are a German B2 exam grader.".into(),
            task: "Review the essay and grade it.".into(),
            context: "The essay is by a beginner.".into(),
            constraints: "- Be concise\n- Cite quotes".into(),
            output_format: "Markdown with 3 sections.".into(),
        }
    }

    #[test]
    fn test_render_includes_all_filled_sections() {
        let text = spec().render();
        assert!(text.starts_with("# Agent Brief"));
        assert!(text.contains("## Role"));
        assert!(text.contains("## Objective"));
        assert!(text.contains("## Context"));
        assert!(text.contains("## Rules"));
        assert!(text.contains("- Be concise"));
        assert!(text.contains("## Output format"));
        // Constraint dashes are normalized onto list markers.
        assert!(text.contains("- Cite quotes"));
    }

    #[test]
    fn test_render_skips_empty_fields() {
        let mut s = spec();
        s.context.clear();
        s.persona.clear();
        let text = s.render();
        assert!(!text.contains("## Role"));
        assert!(!text.contains("## Context"));
        assert!(text.contains("## Objective"));
    }

    #[test]
    fn test_tools_section_lists_only_enabled() {
        let tools = render_tools_section(&[1, 4], false);
        assert!(tools.contains("fs.write"));
        assert!(tools.contains("web.search"));
        assert!(!tools.contains("db.sql"));
        assert!(render_tools_section(&[], false).is_empty());
        let indented = render_tools_section(&[0], true);
        assert!(indented.starts_with("## Tools (MCP)"));
        assert!(indented.contains("  - fs.read"));
    }

    #[test]
    fn test_agent_pack_combines_brief_and_tools() {
        let pack = render_agent_pack(&spec(), &[3]);
        assert!(pack.contains("# Agent Brief"));
        assert!(pack.contains("web.fetch"));
        assert!(!pack.contains("fs.read"));
    }

    #[test]
    fn test_cursor_frontmatter_and_filenames() {
        let out = render_for_target(CopyTarget::Cursor, "BRIEF");
        assert!(out.starts_with("---\ndescription"));
        assert!(out.contains("alwaysApply: true"));
        assert_eq!(CopyTarget::Cursor.filename(), ".cursor/rules/krebs.mdc");
        assert_eq!(CopyTarget::Antigravity.filename(), ".agent/rules/krebs.md");
        assert!(
            render_for_target(CopyTarget::Claude, "BRIEF")
                .contains("# Krebs")
        );
    }

    #[test]
    fn test_estimate_tokens_rounds_up() {
        assert_eq!(estimate_tokens(""), 0);
        assert_eq!(estimate_tokens("abcd"), 1);
        assert_eq!(estimate_tokens("abcde"), 2);
    }

    #[test]
    fn test_offline_improve_adds_quality_bar() {
        let improved = offline_improve("# Agent Brief\n## Objective\n\nWrite it.");
        assert!(improved.contains("## Quality bar"));
        assert!(improved.contains("## Objective"));
    }

    #[test]
    fn test_mcp_catalog_shapes() {
        assert_eq!(MCP_TOOLS.len(), 9);
        assert!(MCP_TOOLS.iter().any(|t| t.id == "db.sql"));
        assert!(MCP_TOOLS.iter().all(|t| !t.description.is_empty()));
        assert!(MCP_TOOLS.iter().all(|t| t.snippet.contains(':')));
    }
}
