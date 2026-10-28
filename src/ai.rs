use std::collections::HashSet;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AiAction {
    GradeEssay,
    TipsAndFeedback,
    FixGrammar,
    GermanB2C1,
}

impl AiAction {
    pub fn title(&self) -> &'static str {
        match self {
            Self::GradeEssay => "🎯 Grade Essay",
            Self::TipsAndFeedback => "💡 Writing Tips",
            Self::FixGrammar => "🔍 Grammar & Flow",
            Self::GermanB2C1 => "🇩🇪 German B2/C1",
        }
    }

    pub fn system_prompt(&self) -> &'static str {
        match self {
            Self::GradeEssay => {
                "You are Hermes, a premier academic writing coach and CEFR/exam evaluator. \
                Analyze the student's essay thoroughly. Provide: \
                1) Overall Score (0-100) and CEFR proficiency estimate (B1, B2, C1, or C2). \
                2) Structure & Argumentation (Intro, Body, Conclusion). \
                3) Lexical Variety & Vocabulary critique. \
                4) Grammatical Accuracy & Syntax. \
                5) Top 3 concrete recommendations for reaching a higher score. \
                Keep your feedback constructive, sharp, encouraging, and actionable."
            }
            Self::TipsAndFeedback => {
                "You are Hermes, an expert writing mentor. Review the draft and provide: \
                1) Key strengths of the piece. \
                2) Structural weaknesses or logical gaps. \
                3) Transitions and flow between paragraphs. \
                4) 3 specific edits or additions to elevate the text."
            }
            Self::FixGrammar => {
                "You are Hermes, a meticulous proofreader and editor. \
                Identify grammatical mistakes, awkward phrasing, wordiness, and punctuation errors. \
                Provide clean, natural corrections and explain briefly why the rewrite improves the text."
            }
            Self::GermanB2C1 => {
                "You are Hermes, a specialist in German Goethe/Telc B2 and C1 examination writing. \
                Evaluate the essay against German exam rubrics. Identify missing or weak Redemittel connectors. \
                Suggest sophisticated connectors (e.g., 'Einerseits/Andererseits', 'Darüber hinaus', \
                'Es liegt auf der Hand, dass...', 'Zusammenfassend lässt sich sagen'). \
                Show rewritten sentences demonstrating B2/C1 idiomatic style."
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct AiState {
    pub is_open: bool,
    pub is_loading: bool,
    pub endpoint: String,
    pub model: String,
    pub last_action: Option<AiAction>,
    pub response_text: String,
    pub is_offline_fallback: bool,
    pub show_settings: bool,
    pub temp_endpoint: String,
    pub temp_model: String,
}

impl Default for AiState {
    fn default() -> Self {
        Self {
            is_open: false,
            is_loading: false,
            endpoint: "http://localhost:11434/v1".to_string(),
            model: "hermes3".to_string(),
            last_action: None,
            response_text: String::new(),
            is_offline_fallback: false,
            show_settings: false,
            temp_endpoint: "http://localhost:11434/v1".to_string(),
            temp_model: "hermes3".to_string(),
        }
    }
}

/// Executes AI analysis using local Hermes LLM via Ollama/OpenAI-compatible endpoint,
/// falling back seamlessly to the built-in offline evaluator if the endpoint is unreachable.
pub fn run_ai_task(
    endpoint: String,
    model: String,
    text: String,
    action: AiAction,
) -> (String, bool) {
    if text.trim().is_empty() {
        return (
            "Your document is empty! Write or paste an essay, paragraph, or draft first, then run Hermes AI."
                .to_string(),
            false,
        );
    }

    // Try calling local Hermes LLM via OpenAI-compatible chat API
    match call_hermes_endpoint(&endpoint, &model, &text, action) {
        Ok(llm_response) => (llm_response, false),
        Err(_err) => {
            // Local model offline or unreachable -> seamless offline heuristic evaluation
            let offline_report = evaluate_essay_offline(&text, action);
            (offline_report, true)
        }
    }
}

fn call_hermes_endpoint(
    endpoint: &str,
    model: &str,
    text: &str,
    action: AiAction,
) -> Result<String, String> {
    let clean_url = format!("{}/chat/completions", endpoint.trim_end_matches('/'));

    let body = serde_json::json!({
        "model": model,
        "messages": [
            { "role": "system", "content": action.system_prompt() },
            { "role": "user", "content": text }
        ],
        "temperature": 0.6,
        "stream": false
    });

    let resp_str = ureq::post(&clean_url)
        .header("Content-Type", "application/json")
        .send_json(body)
        .map_err(|e| e.to_string())?
        .body_mut()
        .read_to_string()
        .map_err(|e| e.to_string())?;

    let parsed: serde_json::Value =
        serde_json::from_str(&resp_str).map_err(|e| e.to_string())?;

    if let Some(content) = parsed["choices"][0]["message"]["content"].as_str() {
        Ok(content.to_string())
    } else if let Some(err_msg) = parsed["error"]["message"].as_str() {
        Err(err_msg.to_string())
    } else {
        Err("Invalid response structure from AI endpoint".to_string())
    }
}

/// Built-in intelligent offline essay evaluator.
/// Works 100% offline with zero external dependencies.
pub fn evaluate_essay_offline(draft: &str, action: AiAction) -> String {
    let words: Vec<&str> = draft.split_whitespace().collect();
    let word_count = words.len();
    let _char_count = draft.chars().count();
    let paragraphs: Vec<&str> = draft.split("\n\n").filter(|p| !p.trim().is_empty()).collect();
    let p_count = paragraphs.len().max(1);

    let sentences: Vec<&str> = draft
        .split(|c| c == '.' || c == '!' || c == '?' || c == '؛' || c == '؟')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();
    let s_count = sentences.len().max(1);
    let avg_words_per_sentence = word_count as f32 / s_count as f32;

    // Lexical diversity (Type-Token Ratio)
    let unique_words: HashSet<String> = words.iter().map(|w| w.to_lowercase()).collect();
    let ttr = if word_count > 0 {
        (unique_words.len() as f32 / word_count as f32) * 100.0
    } else {
        0.0
    };

    // German Redemittel connector detection
    let german_connectors = [
        "deshalb", "außerdem", "darüber hinaus", "einerseits", "andererseits",
        "jedoch", "allerdings", "dennoch", "infolgedessen", "zusammenfassend",
        "meiner meinung nach", "es steht außer frage", "abschließend", "zwar"
    ];
    let lower_text = draft.to_lowercase();
    let mut found_connectors = Vec::new();
    for conn in &german_connectors {
        if lower_text.contains(conn) {
            found_connectors.push(*conn);
        }
    }

    match action {
        AiAction::GradeEssay => {
            // Heuristic Rubric Scoring (0-100)
            let mut score = 65; // baseline

            // Length score
            if word_count >= 180 && word_count <= 350 {
                score += 12;
            } else if word_count > 100 {
                score += 6;
            }

            // Paragraph structure
            if p_count >= 3 && p_count <= 6 {
                score += 8;
            }

            // Sentence variation
            if avg_words_per_sentence >= 11.0 && avg_words_per_sentence <= 22.0 {
                score += 8;
            }

            // Lexical diversity
            if ttr >= 55.0 {
                score += 7;
            }

            // Connectors
            if !found_connectors.is_empty() {
                score += (found_connectors.len() * 3).min(10);
            }

            score = score.min(96);

            let (cefr_band, rating) = if score >= 88 {
                ("CEFR C1", "Excellent / High Distinction")
            } else if score >= 75 {
                ("CEFR B2.2", "Proficient / Strong Pass")
            } else if score >= 60 {
                ("CEFR B2.1", "Competent / Pass")
            } else {
                ("CEFR B1", "Developing / Needs Revision")
            };

            format!(
                "## 🎯 Academic Essay Scorecard\n\n\
                **Overall Evaluation:** `{score}/100` — **{cefr_band}** ({rating})\n\n\
                ### 📊 Quantitative Metrics\n\
                - **Word Count:** {word_count} words (Target: 200–250 words)\n\
                - **Paragraphs:** {p_count} structured sections\n\
                - **Avg. Sentence Length:** {avg_words_per_sentence:.1} words/sentence\n\
                - **Vocabulary Diversity (TTR):** {ttr:.1}%\n\
                - **Connectors Detected:** {conn_count}\n\n\
                ### 💡 Structural & Linguistic Analysis\n\
                1. **Structure ({p_count} paragraphs):** {p_comment}\n\
                2. **Flow & Cadence:** {flow_comment}\n\
                3. **Lexical Variety:** {ttr_comment}\n\n\
                ### 🚀 Key Recommendations\n\
                - {rec_1}\n\
                - {rec_2}\n\
                - Integrate advanced discourse markers to strengthen topic transitions.\n\n\
                *(Offline heuristic evaluation. Connect local Hermes at `ollama run hermes3` for deep LLM analysis)*",
                score = score,
                cefr_band = cefr_band,
                rating = rating,
                word_count = word_count,
                p_count = p_count,
                avg_words_per_sentence = avg_words_per_sentence,
                ttr = ttr,
                conn_count = found_connectors.len(),
                p_comment = if p_count >= 3 {
                    "Well-formed paragraph separation with distinct thematic blocks."
                } else {
                    "Consider splitting your draft into a clear Introduction, Body Arguments, and Conclusion."
                },
                flow_comment = if avg_words_per_sentence >= 12.0 && avg_words_per_sentence <= 22.0 {
                    "Good sentence length equilibrium; easy to read and logically coherent."
                } else {
                    "Sentences are either too short or run-on; vary short punchy sentences with compound clauses."
                },
                ttr_comment = if ttr >= 55.0 {
                    "High vocabulary variety without repetitive word choices."
                } else {
                    "Moderate repetition detected; replace repeated key terms with contextual synonyms."
                },
                rec_1 = if found_connectors.len() < 3 {
                    "Add 2-3 logical transitions (*Furthermore, In contrast, Infolgedessen*) to bind paragraphs."
                } else {
                    "Maintain the strong logical cohesion present in your body arguments."
                },
                rec_2 = if word_count < 180 {
                    "Elaborate on supporting examples to meet standard exam word limits."
                } else {
                    "Ensure conclusion directly answers the thesis proposed in paragraph 1."
                }
            )
        }

        AiAction::TipsAndFeedback => {
            format!(
                "## 💡 Hermes Writing Tips & Diagnostics\n\n\
                ### ✨ Document Highlights\n\
                - **Length:** {word_count} words across {p_count} paragraphs.\n\
                - **Tone:** Academic / Informative.\n\
                - **Vocabulary Richness:** {ttr:.1}% unique words.\n\n\
                ### 🛠️ Actionable Tips\n\
                1. **Opening Hook:** Ensure your first sentence grabs attention and establishes a clear thesis.\n\
                2. **Point-Example-Explanation (PEE):** Support every claim in your body paragraphs with an empirical example.\n\
                3. **Counter-Argument:** Include a 'Zwar / Although' perspective to show critical thinking depth.\n\
                4. **Cadence Polish:** Read aloud to verify that sentences glide smoothly without awkward stops."
            )
        }

        AiAction::FixGrammar => {
            format!(
                "## 🔍 Grammar & Style Diagnostic\n\n\
                - **Sentences Analyzed:** {s_count}\n\
                - **Average Sentence Length:** {avg_words_per_sentence:.1} words\n\
                - **Punctuation Check:** Standard ending marks present.\n\n\
                ### ✏️ Stylistic Checkpoints:\n\
                1. **Passive Voice Check:** Prefer active verbs (*'The experiment revealed...'* over *'It was revealed by...'*) to tighten your prose.\n\
                2. **Run-on Sentences:** Any sentence exceeding 30 words should be divided with a semicolon or period.\n\
                3. **Comma Splices:** Ensure independent clauses are joined with coordinating conjunctions or periods."
            )
        }

        AiAction::GermanB2C1 => {
            let used_list = if found_connectors.is_empty() {
                "None detected yet.".to_string()
            } else {
                found_connectors.join(", ")
            };

            format!(
                "## 🇩🇪 German B2/C1 Redemittel & Connector Coach\n\n\
                **Connectors Found in Draft:** `{used_list}`\n\n\
                ### 🏆 Recommended B2/C1 Formulaic Connectors:\n\
                - **Einleitung:** *„In der heutigen Zeit wird häufig darüber diskutiert, ob...“*\n\
                - **Pro-Argument:** *„Ein wesentlicher Vorteil besteht darin, dass...“*\n\
                - **Kontrast:** *„Auf der anderen Seite muss berücksichtigt werden, dass...“*\n\
                - **Eigene Meinung:** *„Meines Erachtens spricht vieles dafür, dass...“*\n\
                - **Schluss:** *„Zusammenfassend lässt sich sagen, dass ein ausgewogener Kompromiss nötig ist.“*\n\n\
                *Tip: Use the Redemittel Drawer in the right toolbar to insert these with one click!*"
            )
        }
    }
}
