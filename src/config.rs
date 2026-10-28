use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// Returns the portable data directory `./krebs_data/` alongside the executable.
/// Zero system pollution: never touches AppData or registry.
pub fn get_portable_data_dir() -> PathBuf {
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()))
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    let krebs_dir = exe_dir.join("krebs_data");
    let omawrite_dir = exe_dir.join("omawrite_data");
    let data_dir = if omawrite_dir.exists() && !krebs_dir.exists() {
        omawrite_dir
    } else {
        krebs_dir
    };

    if !data_dir.exists() {
        let _ = fs::create_dir_all(&data_dir);
    }
    data_dir
}

/// Returns the recovery directory `./krebs_data/recovery/`.
pub fn get_recovery_dir() -> PathBuf {
    let rec_dir = get_portable_data_dir().join("recovery");
    if !rec_dir.exists() {
        let _ = fs::create_dir_all(&rec_dir);
    }
    rec_dir
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub window_width: f32,
    pub window_height: f32,
    pub zoom_level: f32,
    pub theme: String,
    pub target_min_words: usize,
    pub target_max_words: usize,
    pub timer_duration_secs: u64,
    pub recent_files: Vec<String>,
    pub show_left_rail: bool,
    pub show_right_drawer: bool,
    pub show_zwnj_indicator: bool,
    pub direction_mode: String, // "auto", "ltr", "rtl"
    pub smart_quotes_de: bool,
    pub smart_dashes: bool,
    pub canvas_max_width: f32,
    #[serde(default = "default_canvas_width_mode")]
    pub canvas_width_mode: String, // "standard", "wide", "full"
    #[serde(default = "default_font_choice")]
    pub font_choice: String, // "jetbrains_mono", "ibm_plex_mono", "vt323", "vazirmatn", "sans", "serif"
    #[serde(default = "default_view_mode")]
    pub view_mode: String, // "edit", "split", "preview"
    #[serde(default = "default_app_mode")]
    pub app_mode: String, // "writing", "whiteboard"
    #[serde(default = "default_paper_pattern")]
    pub paper_pattern: String, // "dot", "ruled", "grid", "blank"
}

fn default_canvas_width_mode() -> String {
    "wide".to_string()
}
fn default_font_choice() -> String {
    "jetbrains_mono".to_string()
}
fn default_view_mode() -> String {
    "edit".to_string()
}
fn default_app_mode() -> String {
    "writing".to_string()
}
fn default_paper_pattern() -> String {
    "dot".to_string()
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            window_width: 1200.0,
            window_height: 820.0,
            zoom_level: 1.0,
            theme: "dark".to_string(),
            target_min_words: 200,
            target_max_words: 250,
            timer_duration_secs: 1500, // 25 minutes Pomodoro / Zen focus
            recent_files: Vec::new(),
            show_left_rail: true,
            show_right_drawer: false,
            show_zwnj_indicator: true,
            direction_mode: "auto".to_string(),
            smart_quotes_de: true,
            smart_dashes: true,
            canvas_max_width: 120.0,
            canvas_width_mode: "wide".to_string(),
            font_choice: "jetbrains_mono".to_string(),
            view_mode: "edit".to_string(),
            app_mode: "writing".to_string(),
            paper_pattern: "dot".to_string(),
        }
    }
}

impl AppConfig {
    pub fn config_file_path() -> PathBuf {
        get_portable_data_dir().join("config.toml")
    }

    pub fn load_or_default() -> Self {
        let path = Self::config_file_path();
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(cfg) = toml::from_str::<AppConfig>(&content) {
                    return cfg;
                }
            }
        }
        let cfg = Self::default();
        let _ = cfg.save();
        cfg
    }

    pub fn save(&self) -> Result<(), std::io::Error> {
        let path = Self::config_file_path();
        let content = toml::to_string_pretty(self).unwrap_or_default();
        fs::write(path, content)
    }

    pub fn add_recent_file(&mut self, file_path: &str) {
        self.recent_files.retain(|p| p != file_path);
        self.recent_files.insert(0, file_path.to_string());
        if self.recent_files.len() > 10 {
            self.recent_files.truncate(10);
        }
        let _ = self.save();
    }
}

// ---------------- Recovery Manager ----------------

pub struct RecoveryManager;

impl RecoveryManager {
    /// Silently checkpoints an untitled draft buffer to `./krebs_data/recovery/untitled_<timestamp>.md`.
    pub fn save_untitled_checkpoint(
        content: &str,
        active_checkpoint: &mut Option<PathBuf>,
    ) -> Option<PathBuf> {
        if content.trim().is_empty() {
            return None;
        }

        let recovery_dir = get_recovery_dir();
        let target_path = if let Some(existing) = active_checkpoint {
            existing.clone()
        } else {
            let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
            let file_name = format!("untitled_{}.md", timestamp);
            let path = recovery_dir.join(file_name);
            *active_checkpoint = Some(path.clone());
            path
        };

        if fs::write(&target_path, content).is_ok() {
            Some(target_path)
        } else {
            None
        }
    }

    /// Cleans up an active checkpoint file once explicitly saved or cleanly closed.
    pub fn cleanup_checkpoint(path: &Path) {
        if path.exists() {
            let _ = fs::remove_file(path);
        }
    }

    /// Lists any recovered untitled drafts left in the recovery directory.
    pub fn list_recoveries() -> Vec<PathBuf> {
        let mut list = Vec::new();
        let dir = get_recovery_dir();
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_file() && p.extension().map(|e| e == "md").unwrap_or(false) {
                    list.push(p);
                }
            }
        }
        list.sort();
        list.reverse(); // newest first
        list
    }
}

// ---------------- German Exam Redemittel Manager ----------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnippetItem {
    pub label: String,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnippetCategory {
    pub name: String,
    pub items: Vec<SnippetItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnippetsConfig {
    pub categories: Vec<SnippetCategory>,
}

impl Default for SnippetsConfig {
    fn default() -> Self {
        Self {
            categories: vec![
                SnippetCategory {
                    name: "Einleitung (Intro)".to_string(),
                    items: vec![
                        SnippetItem {
                            label: "Wichtige Rolle".to_string(),
                            text: "In der heutigen Zeit spielt dieses Thema eine immer wichtigere Rolle. ".to_string(),
                        },
                        SnippetItem {
                            label: "Einleitend betonen".to_string(),
                            text: "Einleitend möchte ich betonen, dass die vorliegende Thematik von großer Relevanz ist. ".to_string(),
                        },
                        SnippetItem {
                            label: "Stellungnahme".to_string(),
                            text: "Im Folgenden möchte ich zu den wesentlichen Aspekten dieser Debatte Stellung nehmen. ".to_string(),
                        },
                        SnippetItem {
                            label: "Häufig diskutiert".to_string(),
                            text: "Es wird heutzutage häufig darüber diskutiert, inwieweit diese Entwicklung positiv zu bewerten ist. ".to_string(),
                        },
                    ],
                },
                SnippetCategory {
                    name: "Meinung (Opinion)".to_string(),
                    items: vec![
                        SnippetItem {
                            label: "Meiner Ansicht nach".to_string(),
                            text: "Meiner Ansicht nach sollte man diesen Punkt keineswegs außer Acht lassen. ".to_string(),
                        },
                        SnippetItem {
                            label: "Feste Überzeugung".to_string(),
                            text: "Ich bin der festen Überzeugung, dass ein Umdenken in diesem Bereich dringend erforderlich ist. ".to_string(),
                        },
                        SnippetItem {
                            label: "Eigene Erfahrung".to_string(),
                            text: "Aus meiner persönlichen Erfahrung lässt sich bestätigen, dass dieser Ansatz sinnvoll ist. ".to_string(),
                        },
                        SnippetItem {
                            label: "Außer Zweifel".to_string(),
                            text: "Es steht völlig außer Zweifel, dass dieser Aspekt eine zentrale Rolle einnimmt. ".to_string(),
                        },
                        SnippetItem {
                            label: "Standpunkt".to_string(),
                            text: "Ich vertrete hierbei den Standpunkt, dass Qualität stets Vorrang haben sollte. ".to_string(),
                        },
                    ],
                },
                SnippetCategory {
                    name: "Vor- & Nachteile (Pros/Cons)".to_string(),
                    items: vec![
                        SnippetItem {
                            label: "Einerseits / Andererseits".to_string(),
                            text: "Einerseits spricht dafür, dass ... Andererseits muss bedacht werden, dass ... ".to_string(),
                        },
                        SnippetItem {
                            label: "Entscheidender Vorteil".to_string(),
                            text: "Ein entscheidender Vorteil besteht darin, dass dadurch wertvolle Zeit gespart wird. ".to_string(),
                        },
                        SnippetItem {
                            label: "Einwand".to_string(),
                            text: "Demgegenüber lässt sich jedoch einwenden, dass auch Risiken vorhanden sind. ".to_string(),
                        },
                        SnippetItem {
                            label: "Nicht vernachlässigen".to_string(),
                            text: "Nicht zu vernachlässigen ist ferner der Gesichtspunkt der Nachhaltigkeit. ".to_string(),
                        },
                        SnippetItem {
                            label: "Zwar ... aber dennoch".to_string(),
                            text: "Zwar lässt sich argumentieren, dass die Kosten hoch sind, dennoch überwiegen die Vorzüge. ".to_string(),
                        },
                    ],
                },
                SnippetCategory {
                    name: "Schluss (Conclusion)".to_string(),
                    items: vec![
                        SnippetItem {
                            label: "Zusammenfassend".to_string(),
                            text: "Zusammenfassend lässt sich sagen, dass eine differenzierte Betrachtung notwendig ist. ".to_string(),
                        },
                        SnippetItem {
                            label: "Schlussfolgerung".to_string(),
                            text: "Abschließend bin ich zu dem Schluss gekommen, dass die positiven Aspekte deutlich überwiegen. ".to_string(),
                        },
                        SnippetItem {
                            label: "Sinnvolle Lösung".to_string(),
                            text: "Vor diesem Hintergrund halte ich es für sinnvoll, einen Mittelweg einzuschlagen. ".to_string(),
                        },
                        SnippetItem {
                            label: "Alles in allem".to_string(),
                            text: "Alles in allem zeigt sich deutlich, dass eine zukunftsorientierte Handlungsweise gefordert ist. ".to_string(),
                        },
                    ],
                },
            ],
        }
    }
}

pub struct SnippetsDeManager;

impl SnippetsDeManager {
    pub fn snippets_file_path() -> PathBuf {
        get_portable_data_dir().join("snippets_de.toml")
    }

    pub fn load_or_create_default() -> SnippetsConfig {
        let path = Self::snippets_file_path();
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(cfg) = toml::from_str::<SnippetsConfig>(&content) {
                    return cfg;
                }
            }
        }

        let default_config = SnippetsConfig::default();
        let content = toml::to_string_pretty(&default_config).unwrap_or_default();
        let _ = fs::write(path, content);
        default_config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_portable_paths() {
        let dir = get_portable_data_dir();
        assert!(dir.ends_with("krebs_data") || dir.ends_with("omawrite_data"));
        assert!(dir.exists());

        let rec_dir = get_recovery_dir();
        assert!(rec_dir.ends_with("recovery"));
        assert!(rec_dir.exists());
    }

    #[test]
    fn test_snippets_default() {
        let snippets = SnippetsConfig::default();
        assert_eq!(snippets.categories.len(), 4);
        assert!(snippets.categories.iter().any(|c| c.name.contains("Einleitung")));
        assert!(snippets.categories.iter().any(|c| c.name.contains("Meinung")));
        assert!(snippets.categories.iter().any(|c| c.name.contains("Vor- & Nachteile")));
        assert!(snippets.categories.iter().any(|c| c.name.contains("Schluss")));
    }

    #[test]
    fn test_recovery_checkpoint_lifecycle() {
        let mut active = None;
        let test_content = "Temporary draft content for exam essay.";
        let checkpoint_path = RecoveryManager::save_untitled_checkpoint(test_content, &mut active);
        assert!(checkpoint_path.is_some());
        let path = checkpoint_path.unwrap();
        assert!(path.exists());
        assert_eq!(fs::read_to_string(&path).unwrap(), test_content);

        // Cleanup
        RecoveryManager::cleanup_checkpoint(&path);
        assert!(!path.exists());
    }
}

