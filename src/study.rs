//! Kartei — the Study Note engine.
//!
//! Any markdown note can become studyable: fenced blocks marked `karte` (or
//! `card`) describe flashcards with a `Q:` question and an `A:` answer:
//!
//! ```md
//! ::: karte
//! Q: What does `Due` mean in spaced repetition?
//! A: The card should be reviewed right now.
//! :::
//! ```
//!
//! A lightweight SM-2-style scheduler decides when a card is due again after
//! each grade ([`Grade::Again`], [`Grade::Good`], [`Grade::Easy`]). All
//! progress persists in `krebs_data/study.json`, keyed by a content hash of
//! the card so renaming the file never loses history.

use crate::config;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;

/// One calendar day in milliseconds.
pub const DAY_MS: u64 = 24 * 60 * 60 * 1000;

/// Interval growth ladder (in days) for successfully graded cards.
const INTERVAL_STEPS: [f64; 6] = [1.0, 3.0, 8.0, 21.0, 45.0, 90.0];

/// Lower bound of the easiness factor.
const MIN_EASE: f64 = 1.3;

/// Upper bound of the easiness factor.
const MAX_EASE: f64 = 3.0;

/// The observer's grade for one flashcard.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Grade {
    /// Could not recall it — card starts over, due in ten minutes.
    Again,
    /// Recalled with effort — normal interval growth.
    Good,
    /// Recalled instantly — longer interval and a small ease boost.
    Easy,
}

/// A flashcard parsed from the note text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KarteCard {
    /// Stable content hash — survives file renames and refactors.
    pub id: u64,
    /// The question text shown first.
    pub question: String,
    /// The answer text shown after the reveal.
    pub answer: String,
}

/// Scheduling memory for a single card.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct KarteRecord {
    /// Number of consecutive successful rehearsals.
    pub reps: u32,
    /// Easiness factor; grows with easy grades, shrinks with lapses.
    pub ease: f64,
    /// Epoch milliseconds of the last grade.
    pub last_ms: u64,
    /// Epoch milliseconds when the card is due again.
    pub due_ms: u64,
    /// Total number of forgotten rounds.
    pub lapses: u32,
}

/// All scheduling records, persisted next to the app data.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct StudyTable {
    /// Records keyed by the decimal card id.
    #[serde(default)]
    pub cards: BTreeMap<String, KarteRecord>,
}

/// Live review session over the due cards of one note.
#[derive(Debug, Clone, PartialEq)]
pub struct Review {
    /// Snapshot, in note order, of the cards that are due.
    pub cards: Vec<KarteCard>,
    /// Working copy of the persisted table; the caller saves after grades.
    pub table: StudyTable,
    /// Cursor into `cards` for the card being studied.
    pub index: usize,
    /// Whether the current card's answer is visible.
    pub revealed: bool,
    /// How many cards were graded in this session so far.
    pub graded_total: u32,
    /// Total number of cards found in the note (for the empty-state text).
    pub total_in_note: usize,
}

impl Review {
    /// Builds a review over the due cards of `markdown`.
    ///
    /// `now_ms` is the current epoch time in milliseconds; the table is a
    /// fresh snapshot loaded by the caller.
    #[must_use]
    pub fn open(markdown: &str, table: StudyTable, now_ms: u64) -> Self {
        let all = parse_cards(markdown);
        let total_in_note = all.len();
        let cards: Vec<KarteCard> = all
            .into_iter()
            .filter(|card| table.is_due(card.id, now_ms))
            .collect();
        Self {
            cards,
            table,
            index: 0,
            revealed: false,
            graded_total: 0,
            total_in_note,
        }
    }

    /// The card currently being studied, if any remain.
    #[must_use]
    pub fn current(&self) -> Option<&KarteCard> {
        self.cards.get(self.index)
    }

    /// Applies a grade to the current card and moves to the next one.
    pub fn advance(&mut self, grade: Grade, now_ms: u64) {
        if let Some(card) = self.cards.get(self.index) {
            let prev = self.table.cards.get(&card.id.to_string()).copied();
            let record = grade_record(prev, grade, now_ms);
            self.table.cards.insert(card.id.to_string(), record);
        }
        self.index += 1;
        self.revealed = false;
        self.graded_total += 1;
    }

    /// True once every due card has been graded.
    #[must_use]
    pub fn is_done(&self) -> bool {
        self.index >= self.cards.len()
    }

    /// How many due cards were still waiting when the review opened.
    #[must_use]
    pub fn planned_count(&self) -> usize {
        self.cards.len()
    }
}

/// Current epoch time in milliseconds (0 on systems without a wall clock).
#[must_use]
pub fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// Stable id for a card: a content hash that ignores surrounding whitespace.
#[must_use]
fn card_id(question: &str, answer: &str) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    question.trim().hash(&mut hasher);
    (0u8).hash(&mut hasher);
    answer.trim().hash(&mut hasher);
    hasher.finish()
}

/// Parses every `::: karte` block into a flashcard.
///
/// Lines inside the block before `Q:` are ignored, so decorative comments are
/// allowed there. A block without a complete question/answer pair is skipped.
#[must_use]
fn parse_cards_impl(markdown: &str) -> Vec<KarteCard> {
    let lines: Vec<&str> = markdown.lines().collect();
    let mut cards = Vec::new();

    let mut i = 0usize;
    while i < lines.len() {
        if !is_open_fence(lines[i]) {
            i += 1;
            continue;
        }

        let mut question_lines: Vec<String> = Vec::new();
        let mut answer_lines: Vec<String> = Vec::new();
        let mut in_answer = false;

        i += 1;
        while i < lines.len() && !is_close_fence(lines[i]) {
            let trimmed = lines[i].trim();
            if !in_answer && trimmed.starts_with('A') {
                // Permit plain "A:" and bullet variants of the counter.
                let after = trimmed.trim_start_matches(['A', 'a']).trim();
                if after.starts_with(':') || after.starts_with('.') {
                    in_answer = true;
                    let rest = after.trim_start_matches([':', '.']).trim();
                    if !rest.is_empty() {
                        answer_lines.push(rest.to_string());
                    }
                    i += 1;
                    continue;
                }
            } else if !in_answer && trimmed.starts_with('Q') {
                let after = trimmed.trim_start_matches(['Q', 'q']).trim();
                if after.starts_with(':') || after.starts_with('.') {
                    let rest = after.trim_start_matches([':', '.']).trim();
                    if !rest.is_empty() {
                        question_lines.push(rest.to_string());
                    }
                    i += 1;
                    continue;
                }
            }
            if in_answer {
                answer_lines.push(trimmed.to_string());
            } else if !question_lines.is_empty() {
                question_lines.push(trimmed.to_string());
            }
            i += 1;
        }
        // `i` now rests on the close fence (or end of input); continue from it.
        i += 1;

        let question = question_lines.join("\n");
        let answer = answer_lines.join("\n");
        if question.trim().is_empty() || answer.trim().is_empty() {
            continue;
        }
        cards.push(KarteCard {
            id: card_id(&question, &answer),
            question,
            answer,
        });
    }

    cards
}

/// Bare `:::` plus an accepted marker word opens a flashcard block.
#[must_use]
fn is_open_fence(line: &str) -> bool {
    let Some(rest) = line.trim().strip_prefix(":::") else {
        return false;
    };
    let marker = rest.trim();
    matches!(
        marker,
        "karte" | "card" | "karteikarte" | "flashcard" | "flashcards" | "cards"
    )
}

/// A bare `:::` line closes the block.
#[must_use]
fn is_close_fence(line: &str) -> bool {
    line.trim() == ":::"
}

/// The scheduler: applies one grade to a card record.
///
/// `prev` is `None` for cards never graded before (they start at the default
/// easiness and interval ladder).
#[must_use]
fn grade_record(prev: Option<KarteRecord>, grade: Grade, now_ms: u64) -> KarteRecord {
    let (reps, ease, lapses) = match prev {
        Some(record) => (record.reps, record.ease, record.lapses),
        None => (0, 2.5, 0),
    };

    match grade {
        Grade::Again => KarteRecord {
            reps: 0,
            ease: (ease - 0.2).max(MIN_EASE),
            last_ms: now_ms,
            due_ms: now_ms + 10 * 60 * 1000,
            lapses: lapses + 1,
        },
        Grade::Good => {
            let step = INTERVAL_STEPS[(reps as usize).min(INTERVAL_STEPS.len() - 1)];
            let interval = step * ease;
            KarteRecord {
                reps: reps + 1,
                ease,
                last_ms: now_ms,
                due_ms: now_ms + (interval * DAY_MS as f64) as u64,
                lapses,
            }
        }
        Grade::Easy => {
            let step = INTERVAL_STEPS[((reps + 1) as usize).min(INTERVAL_STEPS.len() - 1)];
            let interval = step * (ease + 0.1).min(MAX_EASE);
            KarteRecord {
                reps: reps + 1,
                ease: (ease + 0.1).min(MAX_EASE),
                last_ms: now_ms,
                due_ms: now_ms + (interval * DAY_MS as f64) as u64,
                lapses,
            }
        }
    }
}

impl StudyTable {
    /// Path of the persisted table inside the portable data directory.
    #[must_use]
    pub fn path() -> PathBuf {
        config::get_portable_data_dir().join("study.json")
    }

    /// Loads the table, falling back to an empty one on any error.
    #[must_use]
    pub fn load() -> Self {
        match fs::read_to_string(Self::path()) {
            Ok(text) => serde_json::from_str(&text).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    /// Writes the table back to disk.
    ///
    /// # Errors
    ///
    /// Returns the underlying I/O error when writing fails.
    pub fn save(&self) -> std::io::Result<()> {
        let text = serde_json::to_string(self).unwrap_or_default();
        fs::write(Self::path(), text)
    }

    /// True when the card with `id` should be practised at `now_ms`.
    #[must_use]
    pub fn is_due(&self, id: u64, now_ms: u64) -> bool {
        match self.cards.get(&id.to_string()) {
            Some(record) => record.due_ms <= now_ms,
            None => true,
        }
    }
}

/// Parses every `::: karte` block into a flashcard.
///
/// Public wrapper for reuse by the UI and exports.
#[must_use]
pub fn parse_cards(markdown: &str) -> Vec<KarteCard> {
    let mut cards = parse_cards_impl(markdown);
    // Stable, human-friendly order: as written in the note.
    cards.dedup();
    cards
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_single_card() {
        let md = "# Notes\n\n::: karte\nQ: What is aDEM?\nA: A German exam script.\n:::\n\nafter\n";
        let cards = parse_cards(md);
        assert_eq!(cards.len(), 1);
        assert_eq!(cards[0].question, "What is aDEM?");
        assert_eq!(cards[0].answer, "A German exam script.");
    }

    #[test]
    fn test_parse_multiple_cards_and_aliases() {
        let md = "::: card\nQ: one?\nA: 1\n:::\nskip\n\n::: flashcard\nQ: two?\nA: 2\n:::";
        let cards = parse_cards(md);
        assert_eq!(cards.len(), 2);
        assert_eq!(cards[0].question, "one?");
        assert_eq!(cards[1].answer, "2");
    }

    #[test]
    fn test_multiline_answers_and_q_prefix() {
        let md = "::: karte\nQ: Define bilinear.\nQ: Extra clarifier\nA: Linear twice.\nsecond line\n:::";
        let cards = parse_cards(md);
        assert_eq!(cards.len(), 1);
        assert!(cards[0].question.contains("Extra clarifier"));
        assert!(cards[0].answer.contains("second line"));
    }

    #[test]
    fn test_incomplete_blocks_are_skipped() {
        // Missing answer — must not become a card.
        let cards = parse_cards("::: karte\nQ: only a question\n:::");
        assert!(cards.is_empty());
        // Untightened fences in normal writing must be left alone.
        assert!(parse_cards("just text\n:::\nnothing\n").is_empty());
    }

    #[test]
    fn test_card_ids_are_stable_and_unique() {
        let first = parse_cards("::: karte\nQ: a\nA: b\n:::\n");
        let again = parse_cards("::: card\nQ:  a\nA: b\n:::\n");
        assert_eq!(first[0].id, again[0].id);
        let other = parse_cards("::: karte\nQ: a\nA: c\n:::\n");
        assert_ne!(first[0].id, other[0].id);
    }

    #[test]
    fn test_grade_again_resets_and_ease_falls() {
        let now = 1_000_000_000_u64;
        let after = grade_record(None, Grade::Again, now);
        assert_eq!(after.reps, 0);
        assert!(after.due_ms <= now + DAY_MS, "again due quickly");
    }

    #[test]
    fn test_grade_good_grows_interval() {
        let now = 1_000_000_000_u64;
        let first = grade_record(None, Grade::Good, now);
        assert!(first.due_ms > now);
        let second = grade_record(Some(first), Grade::Good, now);
        assert!(
            second.due_ms - now > first.due_ms - now,
            "interval must grow with reps"
        );
    }

    #[test]
    fn test_grade_easy_grows_interval_faster() {
        let now = 1_000_000_000_u64;
        let good = grade_record(None, Grade::Good, now);
        let easy = grade_record(None, Grade::Easy, now);
        assert!(easy.due_ms > good.due_ms);
        assert!(easy.ease > good.ease || easy.reps > good.reps);
    }

    #[test]
    fn test_table_roundtrip_and_due() {
        let table = StudyTable::default();
        assert!(table.is_due(42, 0));
        let mut table = table;
        table.cards.insert(
            "42".to_string(),
            KarteRecord {
                reps: 2,
                ease: 2.5,
                last_ms: 0,
                due_ms: 5_000,
                lapses: 0,
            },
        );
        assert!(!table.is_due(42, 4_999));
        assert!(table.is_due(42, 5_000));
        assert!(table.is_due(43, 0), "unknown cards count as due");
    }

    #[test]
    fn test_review_flow_reaches_the_end() {
        let md = "::: karte\nQ: 1\nA: x\n:::\n::: karte\nQ: 2\nA: y\n:::";
        let mut review = Review::open(md, StudyTable::default(), 10);
        assert!(!review.is_done(), "planned: {}", review.planned_count());
        assert_eq!(review.planned_count(), 2);

        let first_card = review.current().cloned();
        assert!(first_card.is_some());
        review.advance(Grade::Good, 1_000_000);
        assert!(!review.revealed);
        assert!(review.current() != first_card.as_ref(), "cursor advanced");
        review.advance(Grade::Again, 1_000_001);
        assert!(review.is_done());
        assert_eq!(review.graded_total, 2);
    }
}
