use iced::widget::text_editor::{self, Action, Edit};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use unicode_bidi::Direction;

pub const ZWNJ_CHAR: char = '\u{200C}';
pub const ZWNJ_STR: &str = "\u{200C}";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DirectionMode {
    Auto,
    Ltr,
    Rtl,
}

impl DirectionMode {
    pub fn label(&self) -> &'static str {
        match self {
            DirectionMode::Auto => "AUTO-BIDI",
            DirectionMode::Ltr => "LTR (EN/DE)",
            DirectionMode::Rtl => "RTL (FA)",
        }
    }

    pub fn next(&self) -> Self {
        match self {
            DirectionMode::Auto => DirectionMode::Ltr,
            DirectionMode::Ltr => DirectionMode::Rtl,
            DirectionMode::Rtl => DirectionMode::Auto,
        }
    }

    pub fn from_str_name(name: &str) -> Self {
        match name.to_lowercase().as_str() {
            "ltr" => DirectionMode::Ltr,
            "rtl" => DirectionMode::Rtl,
            _ => DirectionMode::Auto,
        }
    }

    pub fn to_str_name(&self) -> &'static str {
        match self {
            DirectionMode::Auto => "auto",
            DirectionMode::Ltr => "ltr",
            DirectionMode::Rtl => "rtl",
        }
    }
}

/// Detects the writing direction of a paragraph by checking the first strong directional character.
pub fn detect_paragraph_direction(text: &str) -> Direction {
    for c in text.chars() {
        if is_persian_or_arabic(c) {
            return Direction::Rtl;
        } else if c.is_alphabetic() {
            return Direction::Ltr;
        }
    }
    Direction::Ltr
}

/// Checks if character belongs to Persian/Arabic Unicode blocks.
pub fn is_persian_or_arabic(c: char) -> bool {
    matches!(
        c,
        '\u{0600}'..='\u{06FF}'
            | '\u{0750}'..='\u{077F}'
            | '\u{08A0}'..='\u{08FF}'
            | '\u{FB50}'..='\u{FDFF}'
            | '\u{FE70}'..='\u{FEFF}'
    )
}

/// Automatically normalizes Arabic Yeh (ي) and Kaf (ك) to Persian Yeh (ی) and Kaf (ک).
pub fn normalize_persian_char(c: char) -> char {
    match c {
        '\u{064A}' => '\u{06CC}', // Arabic Yeh -> Persian Yeh
        '\u{0643}' => '\u{06A9}', // Arabic Kaf -> Persian Kaf
        other => other,
    }
}

/// Normalizes an entire string for Persian typography.
pub fn normalize_persian_string(text: &str) -> String {
    text.chars().map(normalize_persian_char).collect()
}

/// Trilingual document statistics.
#[derive(Debug, Clone, Default)]
pub struct DocumentStats {
    pub word_count: usize,
    pub char_count: usize,
    pub char_count_no_spaces: usize,
    pub paragraph_count: usize,
    pub line_count: usize,
    pub reading_time_mins: f32,
}

impl DocumentStats {
    pub fn compute(text: &str) -> Self {
        let char_count = text.chars().count();
        let char_count_no_spaces = text.chars().filter(|c| !c.is_whitespace() && *c != ZWNJ_CHAR).count();
        let lines: Vec<&str> = text.lines().collect();
        let line_count = lines.len();
        let paragraph_count = lines.iter().filter(|l| !l.trim().is_empty()).count();

        // Trilingual-aware word counting:
        // ZWNJ is considered an internal letter connection, not a boundary.
        // Spaces and regular punctuation are boundaries.
        let mut words = 0;
        let mut in_word = false;

        for c in text.chars() {
            if c.is_alphanumeric() || c == ZWNJ_CHAR || c == '-' || c == '_' {
                if !in_word && (c.is_alphanumeric() || is_persian_or_arabic(c)) {
                    in_word = true;
                    words += 1;
                }
            } else if c.is_whitespace() || c.is_ascii_punctuation() || is_custom_punct(c) {
                in_word = false;
            }
        }

        let reading_time_mins = if words > 0 {
            (words as f32 / 200.0).max(0.5)
        } else {
            0.0
        };

        Self {
            word_count: words,
            char_count,
            char_count_no_spaces,
            paragraph_count,
            line_count,
            reading_time_mins,
        }
    }
}

fn is_custom_punct(c: char) -> bool {
    matches!(
        c,
        '„' | '“' | '”' | '«' | '»' | '–' | '—' | '،' | '؛' | '؟' | '…'
    )
}

/// Applies smart typography transformations:
/// - Double dashes `--` -> en-dash `–`
/// - Straight quotes `"` -> German quotation marks `„` and `“`
pub fn apply_smart_typography(text: &str, smart_quotes: bool, smart_dashes: bool) -> String {
    let mut result = text.to_string();

    if smart_dashes {
        result = result.replace("--", "–");
    }

    if smart_quotes {
        let mut output = String::with_capacity(result.len());
        let mut open_quote = false;
        let chars: Vec<char> = result.chars().collect();

        for i in 0..chars.len() {
            let c = chars[i];
            if c == '"' {
                let prev_is_space_or_start = i == 0 || chars[i - 1].is_whitespace() || is_custom_punct(chars[i - 1]);
                if prev_is_space_or_start || !open_quote {
                    output.push('„'); // German opening lower quote
                    open_quote = true;
                } else {
                    output.push('“'); // German closing upper quote
                    open_quote = false;
                }
            } else {
                output.push(c);
            }
        }
        result = output;
    }

    result
}

/// Undo/Redo history stack with capped memory footprint.
#[derive(Debug, Clone)]
pub struct EditorHistory {
    undo_stack: Vec<String>,
    redo_stack: Vec<String>,
    max_history: usize,
}

impl Default for EditorHistory {
    fn default() -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            max_history: 100,
        }
    }
}

impl EditorHistory {
    pub fn push_snapshot(&mut self, text: String) {
        if self.undo_stack.last().map(|s| s == &text).unwrap_or(false) {
            return;
        }
        self.undo_stack.push(text);
        if self.undo_stack.len() > self.max_history {
            self.undo_stack.remove(0);
        }
        self.redo_stack.clear();
    }

    pub fn undo(&mut self, current: String) -> Option<String> {
        if let Some(prev) = self.undo_stack.pop() {
            self.redo_stack.push(current);
            Some(prev)
        } else {
            None
        }
    }

    pub fn redo(&mut self, current: String) -> Option<String> {
        if let Some(next) = self.redo_stack.pop() {
            self.undo_stack.push(current);
            Some(next)
        } else {
            None
        }
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }
}

/// Helper to insert a string at the current cursor in `iced::widget::text_editor::Content`.
pub fn insert_text_at_cursor(
    content: &mut text_editor::Content<iced::Renderer>,
    text: &str,
) {
    content.perform(Action::Edit(Edit::Paste(Arc::new(text.to_string()))));
}

/// Inserts Zero-Width Non-Joiner (ZWNJ / نیم‌فاصله) at the cursor.
pub fn insert_zwnj_at_cursor(content: &mut text_editor::Content<iced::Renderer>) {
    insert_text_at_cursor(content, ZWNJ_STR);
}

/// Wraps current selection or inserts markdown tags at cursor.
pub fn wrap_markdown_syntax(
    content: &mut text_editor::Content<iced::Renderer>,
    prefix: &str,
    suffix: &str,
    placeholder: &str,
) {
    if let Some(selected) = content.selection() {
        let wrapped = format!("{}{}{}", prefix, selected, suffix);
        insert_text_at_cursor(content, &wrapped);
    } else {
        let template = format!("{}{}{}", prefix, placeholder, suffix);
        insert_text_at_cursor(content, &template);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bidi_paragraph_detection() {
        // English / German should detect LTR
        assert_eq!(detect_paragraph_direction("Hello world"), Direction::Ltr);
        assert_eq!(
            detect_paragraph_direction("Einleitung: In der heutigen Zeit..."),
            Direction::Ltr
        );

        // Persian should detect RTL
        assert_eq!(detect_paragraph_direction("سلام دنیا"), Direction::Rtl);
        assert_eq!(
            detect_paragraph_direction("این یک متن نمونه فارسی است."),
            Direction::Rtl
        );
    }

    #[test]
    fn test_persian_normalization() {
        // Arabic Yeh (ي) -> Persian Yeh (ی)
        assert_eq!(normalize_persian_char('ي'), 'ی');
        // Arabic Kaf (ك) -> Persian Kaf (ک)
        assert_eq!(normalize_persian_char('ك'), 'ک');

        let raw = "كتاب يزدان";
        assert_eq!(normalize_persian_string(raw), "کتاب یزدان");
    }

    #[test]
    fn test_trilingual_word_counting_with_zwnj() {
        // English
        let stats_en = DocumentStats::compute("The quick brown fox jumps.");
        assert_eq!(stats_en.word_count, 5);

        // German
        let stats_de = DocumentStats::compute("Das Thema Umweltschutz spielt eine wichtige Rolle.");
        assert_eq!(stats_de.word_count, 7);

        // Persian with ZWNJ (نیم‌فاصله):
        // "می‌دانم" contains ZWNJ between "می" and "دانم". It is ONE word, not two!
        let text_fa = "من می‌دانم که این خانه‌ها زیبا هستند.";
        let stats_fa = DocumentStats::compute(text_fa);
        // "من", "می‌دانم", "که", "این", "خانه‌ها", "زیبا", "هستند" = 7 words
        assert_eq!(stats_fa.word_count, 7);
    }

    #[test]
    fn test_smart_typography() {
        let input = "Das ist -- wie gesagt -- ein \"guter Test\".";
        let transformed = apply_smart_typography(input, true, true);
        assert_eq!(transformed, "Das ist – wie gesagt – ein „guter Test“.");
    }

    #[test]
    fn test_editor_history_undo_redo() {
        let mut history = EditorHistory::default();
        history.push_snapshot("Initial text".to_string());
        history.push_snapshot("Second text".to_string());

        let undone = history.undo("Third text".to_string());
        assert_eq!(undone, Some("Second text".to_string()));

        let redone = history.redo("Second text".to_string());
        assert_eq!(redone, Some("Third text".to_string()));
    }
}

