#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

pub mod config;
pub mod editor;
pub mod export;
pub mod markdown_view;
pub mod sidebar;
pub mod theme;
pub mod whiteboard;

use std::borrow::Cow;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

use iced::event::{self, Event};
use iced::font;
use iced::keyboard::{self, Key, Modifiers};
use iced::mouse;
use iced::widget::{
    button, center, column, container, row, scrollable, stack, text, text_editor, text_input, Space,
};
use iced::window;
use iced::{Alignment, Color, Element, Font, Length, Size, Subscription, Task, Theme};

use config::{AppConfig, RecoveryManager, SnippetsConfig, SnippetsDeManager};
use editor::{
    apply_smart_typography, detect_paragraph_direction, insert_text_at_cursor,
    insert_zwnj_at_cursor, normalize_persian_char, normalize_persian_string, wrap_markdown_syntax,
    DirectionMode, DocumentStats, EditorHistory,
};
use sidebar::{view_left_rail, view_right_drawer, SidebarAction};
use theme::{
    active_tool_button_style, banner_container_style, canvas_container_style,
    modal_container_style, rail_button_style, search_container_style,
    tab_active_button_style, tab_inactive_button_style, text_editor_style, text_input_style,
    toolbar_floating_style, wipe_button_style, ACCENT_AMBER, ACCENT_CYAN, ACCENT_GREEN,
    ACCENT_RED, BORDER_COLOR, RAIL_BG, TEXT_MUTED, TEXT_PRIMARY,
};
use whiteboard::{PaperPattern, WhiteboardCanvas, WhiteboardMessage, WhiteboardState, WhiteboardTool};

// Bundled Offline Fonts & Assets
pub static KREBS_LOGO_PNG: &[u8] = include_bytes!("../assets/logo.png");
pub static IBM_PLEX_MONO_REGULAR: &[u8] = include_bytes!("../fonts/IBMPlexMono-Regular.ttf");
pub static IBM_PLEX_MONO_BOLD: &[u8] = include_bytes!("../fonts/IBMPlexMono-Bold.ttf");
pub static VAZIRMATN_REGULAR: &[u8] = include_bytes!("../fonts/Vazirmatn-Regular.ttf");
pub static VAZIRMATN_BOLD: &[u8] = include_bytes!("../fonts/Vazirmatn-Bold.ttf");
pub static JETBRAINS_MONO_REGULAR: &[u8] = include_bytes!("../fonts/JetBrainsMono-Regular.ttf");
pub static VT323_REGULAR: &[u8] = include_bytes!("../fonts/VT323-Regular.ttf");

pub fn font_ibm_plex_mono() -> Font {
    Font {
        family: font::Family::Name("IBM Plex Mono"),
        weight: font::Weight::Normal,
        stretch: font::Stretch::Normal,
        style: font::Style::Normal,
    }
}

pub fn font_jetbrains_mono() -> Font {
    Font {
        family: font::Family::Name("JetBrains Mono"),
        weight: font::Weight::Normal,
        stretch: font::Stretch::Normal,
        style: font::Style::Normal,
    }
}

pub fn font_vt323() -> Font {
    Font {
        family: font::Family::Name("VT323"),
        weight: font::Weight::Normal,
        stretch: font::Stretch::Normal,
        style: font::Style::Normal,
    }
}

pub fn font_vazirmatn() -> Font {
    Font {
        family: font::Family::Name("Vazirmatn"),
        weight: font::Weight::Normal,
        stretch: font::Stretch::Normal,
        style: font::Style::Normal,
    }
}

pub fn font_sans() -> Font {
    Font {
        family: font::Family::SansSerif,
        weight: font::Weight::Normal,
        stretch: font::Stretch::Normal,
        style: font::Style::Normal,
    }
}

pub fn font_serif() -> Font {
    Font {
        family: font::Family::Serif,
        weight: font::Weight::Normal,
        stretch: font::Stretch::Normal,
        style: font::Style::Normal,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FontChoice {
    JetBrainsMono,
    IbmPlexMono,
    Vt323,
    Vazirmatn,
    Sans,
    Serif,
}

impl FontChoice {
    pub fn from_str_name(s: &str) -> Self {
        match s {
            "ibm_plex_mono" => Self::IbmPlexMono,
            "vt323" => Self::Vt323,
            "vazirmatn" => Self::Vazirmatn,
            "sans" => Self::Sans,
            "serif" => Self::Serif,
            _ => Self::JetBrainsMono,
        }
    }

    pub fn to_str_name(&self) -> &'static str {
        match self {
            Self::JetBrainsMono => "jetbrains_mono",
            Self::IbmPlexMono => "ibm_plex_mono",
            Self::Vt323 => "vt323",
            Self::Vazirmatn => "vazirmatn",
            Self::Sans => "sans",
            Self::Serif => "serif",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::JetBrainsMono => "JetBrains Mono",
            Self::IbmPlexMono => "IBM Plex Mono",
            Self::Vt323 => "VT323 Retro CRT",
            Self::Vazirmatn => "Vazirmatn Persian",
            Self::Sans => "System Sans",
            Self::Serif => "System Serif",
        }
    }

    pub fn to_font(&self) -> Font {
        match self {
            Self::JetBrainsMono => font_jetbrains_mono(),
            Self::IbmPlexMono => font_ibm_plex_mono(),
            Self::Vt323 => font_vt323(),
            Self::Vazirmatn => font_vazirmatn(),
            Self::Sans => font_sans(),
            Self::Serif => font_serif(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CanvasWidthMode {
    Standard,
    Wide,
    Full,
}

impl CanvasWidthMode {
    pub fn from_str_name(s: &str) -> Self {
        match s {
            "standard" => Self::Standard,
            "full" => Self::Full,
            _ => Self::Wide,
        }
    }

    pub fn to_str_name(&self) -> &'static str {
        match self {
            Self::Standard => "standard",
            Self::Wide => "wide",
            Self::Full => "full",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Standard => "Standard",
            Self::Wide => "Wide",
            Self::Full => "Full Width",
        }
    }

    pub fn next(&self) -> Self {
        match self {
            Self::Standard => Self::Wide,
            Self::Wide => Self::Full,
            Self::Full => Self::Standard,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewMode {
    Edit,
    Split,
    Preview,
}

impl ViewMode {
    pub fn from_str_name(s: &str) -> Self {
        match s {
            "split" => Self::Split,
            "preview" => Self::Preview,
            _ => Self::Edit,
        }
    }

    pub fn to_str_name(&self) -> &'static str {
        match self {
            Self::Edit => "edit",
            Self::Split => "split",
            Self::Preview => "preview",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Edit => "✏️ Edit",
            Self::Split => "◫ Live Split",
            Self::Preview => "👁️ Preview",
        }
    }

    pub fn next(&self) -> Self {
        match self {
            Self::Edit => Self::Split,
            Self::Split => Self::Preview,
            Self::Preview => Self::Edit,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppMode {
    Writing,
    Whiteboard,
}

#[derive(Debug, Clone, Default)]
pub struct SearchState {
    pub is_open: bool,
    pub query: String,
    pub matches: Vec<(usize, usize)>, // (line, col)
    pub current_idx: usize,
}

struct Krebs {
    config: AppConfig,
    snippets: SnippetsConfig,
    content: text_editor::Content<iced::Renderer>,
    history: EditorHistory,
    current_file_path: Option<PathBuf>,
    active_recovery_file: Option<PathBuf>,
    is_dirty: bool,
    last_edit_time: Instant,
    last_checkpoint_time: Instant,
    last_saved_time: Instant,
    external_modification_detected: bool,
    timer_remaining_secs: u64,
    timer_total_secs: u64,
    timer_is_running: bool,
    show_shortcuts_modal: bool,
    show_font_modal: bool,
    show_timer_presets: bool,
    is_german_bar_open: bool,
    status_notification: Option<(String, Instant)>,
    stats: DocumentStats,
    zoom_level: f32,
    direction_mode: DirectionMode,
    canvas_width_mode: CanvasWidthMode,
    font_choice: FontChoice,
    view_mode: ViewMode,
    app_mode: AppMode,
    whiteboard: WhiteboardState,
    search: SearchState,
}

#[derive(Debug, Clone)]
enum Message {
    EditorAction(text_editor::Action),
    Sidebar(SidebarAction),
    FileOpened(Option<PathBuf>),
    FileSaved(Option<PathBuf>),
    FileExported(Option<PathBuf>),
    FileDropped(PathBuf),
    Tick,
    KeyboardEvent(Key, Modifiers),
    MouseScrolled(f32, Modifiers),
    ReloadFromDisk,
    DismissExternalBanner,
    CloseShortcutsModal,
    CloseFontModal,
    SelectFont(FontChoice),
    ToggleFontModal,
    SetAppMode(AppMode),
    SetViewMode(ViewMode),
    CycleWidthMode,
    ToggleSearch,
    SearchInputChanged(String),
    SearchNext,
    SearchPrev,
    CloseSearch,
    ZenTimerToggle,
    ZenTimerSet(u64),
    ZenTimerTogglePresets,
    Whiteboard(WhiteboardMessage),
    WhiteboardToolSelect(WhiteboardTool),
    WhiteboardWidthSelect(f32),
    WhiteboardColorSelect(Color),
    WhiteboardPatternSelect(PaperPattern),
    WhiteboardUndo,
    WhiteboardRedo,
    WhiteboardClear,
    FontLoaded(Result<(), iced::font::Error>),
}

impl From<WhiteboardMessage> for Message {
    fn from(msg: WhiteboardMessage) -> Self {
        Message::Whiteboard(msg)
    }
}

impl Krebs {
    fn new(initial_file: Option<PathBuf>) -> (Self, Task<Message>) {
        let config = AppConfig::load_or_default();
        let snippets = SnippetsDeManager::load_or_create_default();
        let direction_mode = DirectionMode::from_str_name(&config.direction_mode);
        let zoom_level = config.zoom_level.clamp(0.5, 3.0);
        let timer_remaining_secs = config.timer_duration_secs;
        let timer_total_secs = config.timer_duration_secs;

        let canvas_width_mode = CanvasWidthMode::from_str_name(&config.canvas_width_mode);
        let font_choice = FontChoice::from_str_name(&config.font_choice);
        let view_mode = ViewMode::from_str_name(&config.view_mode);
        let app_mode = if config.app_mode == "whiteboard" {
            AppMode::Whiteboard
        } else {
            AppMode::Writing
        };

        let mut whiteboard = WhiteboardState::new();
        match config.paper_pattern.as_str() {
            "ruled" => whiteboard.paper_pattern = PaperPattern::Ruled,
            "grid" => whiteboard.paper_pattern = PaperPattern::MathGrid,
            "blank" => whiteboard.paper_pattern = PaperPattern::Blank,
            _ => whiteboard.paper_pattern = PaperPattern::DotGrid,
        }

        let mut initial_text = String::new();
        let mut loaded_path = None;

        if let Some(path) = initial_file {
            if path.exists() {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    initial_text = content;
                    loaded_path = Some(path);
                }
            }
        }

        if loaded_path.is_none() {
            let recoveries = RecoveryManager::list_recoveries();
            if let Some(first_recovery) = recoveries.first() {
                if let Ok(content) = std::fs::read_to_string(first_recovery) {
                    if !content.trim().is_empty() {
                        initial_text = content;
                    }
                }
            }
        }

        let stats = DocumentStats::compute(&initial_text);
        let content = text_editor::Content::with_text(&initial_text);
        let now = Instant::now();

        let app = Self {
            config,
            snippets,
            content,
            history: EditorHistory::default(),
            current_file_path: loaded_path,
            active_recovery_file: None,
            is_dirty: false,
            last_edit_time: now,
            last_checkpoint_time: now,
            last_saved_time: now,
            external_modification_detected: false,
            timer_remaining_secs,
            timer_total_secs,
            timer_is_running: false,
            show_shortcuts_modal: false,
            show_font_modal: false,
            show_timer_presets: false,
            is_german_bar_open: false,
            status_notification: None,
            stats,
            zoom_level,
            direction_mode,
            canvas_width_mode,
            font_choice,
            view_mode,
            app_mode,
            whiteboard,
            search: SearchState::default(),
        };

        let load_fonts_task = Task::batch(vec![
            font::load(Cow::Borrowed(IBM_PLEX_MONO_REGULAR)).map(Message::FontLoaded),
            font::load(Cow::Borrowed(IBM_PLEX_MONO_BOLD)).map(Message::FontLoaded),
            font::load(Cow::Borrowed(VAZIRMATN_REGULAR)).map(Message::FontLoaded),
            font::load(Cow::Borrowed(VAZIRMATN_BOLD)).map(Message::FontLoaded),
            font::load(Cow::Borrowed(JETBRAINS_MONO_REGULAR)).map(Message::FontLoaded),
            font::load(Cow::Borrowed(VT323_REGULAR)).map(Message::FontLoaded),
        ]);

        (app, load_fonts_task)
    }

    fn notify(&mut self, text: impl Into<String>) {
        self.status_notification = Some((text.into(), Instant::now()));
    }

    fn update_search_matches(&mut self) {
        self.search.matches.clear();
        self.search.current_idx = 0;
        if self.search.query.trim().is_empty() {
            return;
        }

        let query_lower = self.search.query.to_lowercase();
        let text_content = self.content.text();

        for (line_idx, line) in text_content.lines().enumerate() {
            let line_lower = line.to_lowercase();
            let mut start = 0;
            while let Some(pos) = line_lower[start..].find(&query_lower) {
                let col = start + pos;
                self.search.matches.push((line_idx + 1, col + 1));
                start = col + query_lower.len().max(1);
            }
        }
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::FontLoaded(_) => Task::none(),

            Message::SetAppMode(mode) => {
                self.app_mode = mode;
                self.config.app_mode = match mode {
                    AppMode::Writing => "writing".to_string(),
                    AppMode::Whiteboard => "whiteboard".to_string(),
                };
                let _ = self.config.save();
                Task::none()
            }

            Message::SetViewMode(mode) => {
                self.view_mode = mode;
                self.config.view_mode = mode.to_str_name().to_string();
                let _ = self.config.save();
                Task::none()
            }

            Message::CycleWidthMode => {
                self.canvas_width_mode = self.canvas_width_mode.next();
                self.config.canvas_width_mode = self.canvas_width_mode.to_str_name().to_string();
                let _ = self.config.save();
                self.notify(format!("Canvas: {}", self.canvas_width_mode.label()));
                Task::none()
            }

            Message::ToggleFontModal => {
                self.show_font_modal = !self.show_font_modal;
                Task::none()
            }

            Message::CloseFontModal => {
                self.show_font_modal = false;
                Task::none()
            }

            Message::SelectFont(choice) => {
                self.font_choice = choice;
                self.config.font_choice = choice.to_str_name().to_string();
                let _ = self.config.save();
                self.show_font_modal = false;
                self.notify(format!("Font: {}", choice.display_name()));
                Task::none()
            }

            Message::ToggleSearch => {
                self.search.is_open = !self.search.is_open;
                if self.search.is_open {
                    self.update_search_matches();
                }
                Task::none()
            }

            Message::SearchInputChanged(query) => {
                self.search.query = query;
                self.update_search_matches();
                Task::none()
            }

            Message::SearchNext => {
                if !self.search.matches.is_empty() {
                    self.search.current_idx = (self.search.current_idx + 1) % self.search.matches.len();
                    let (line, col) = self.search.matches[self.search.current_idx];
                    self.notify(format!("Match {} of {} (Line {}, Col {})", self.search.current_idx + 1, self.search.matches.len(), line, col));
                }
                Task::none()
            }

            Message::SearchPrev => {
                if !self.search.matches.is_empty() {
                    if self.search.current_idx == 0 {
                        self.search.current_idx = self.search.matches.len() - 1;
                    } else {
                        self.search.current_idx -= 1;
                    }
                    let (line, col) = self.search.matches[self.search.current_idx];
                    self.notify(format!("Match {} of {} (Line {}, Col {})", self.search.current_idx + 1, self.search.matches.len(), line, col));
                }
                Task::none()
            }

            Message::CloseSearch => {
                self.search.is_open = false;
                Task::none()
            }

            Message::ZenTimerToggle => {
                self.timer_is_running = !self.timer_is_running;
                Task::none()
            }

            Message::ZenTimerSet(secs) => {
                self.timer_total_secs = secs;
                self.timer_remaining_secs = secs;
                self.timer_is_running = true;
                self.show_timer_presets = false;
                self.config.timer_duration_secs = secs;
                let _ = self.config.save();
                Task::none()
            }

            Message::ZenTimerTogglePresets => {
                self.show_timer_presets = !self.show_timer_presets;
                Task::none()
            }

            Message::Whiteboard(w_msg) => {
                match w_msg {
                    WhiteboardMessage::PointPressed(pt) => self.whiteboard.start_stroke(pt),
                    WhiteboardMessage::PointDragged(pt) => self.whiteboard.continue_stroke(pt),
                    WhiteboardMessage::PointReleased => self.whiteboard.end_stroke(),
                }
                Task::none()
            }

            Message::WhiteboardToolSelect(tool) => {
                self.whiteboard.active_tool = tool;
                Task::none()
            }

            Message::WhiteboardWidthSelect(width) => {
                self.whiteboard.pen_width = width;
                Task::none()
            }

            Message::WhiteboardColorSelect(color) => {
                self.whiteboard.pen_color = color;
                Task::none()
            }

            Message::WhiteboardPatternSelect(pattern) => {
                self.whiteboard.paper_pattern = pattern;
                self.config.paper_pattern = match pattern {
                    PaperPattern::Ruled => "ruled".to_string(),
                    PaperPattern::MathGrid => "grid".to_string(),
                    PaperPattern::Blank => "blank".to_string(),
                    PaperPattern::DotGrid => "dot".to_string(),
                };
                let _ = self.config.save();
                Task::none()
            }

            Message::WhiteboardUndo => {
                self.whiteboard.undo();
                Task::none()
            }

            Message::WhiteboardRedo => {
                self.whiteboard.redo();
                Task::none()
            }

            Message::WhiteboardClear => {
                self.whiteboard.clear_board();
                self.notify("Whiteboard cleared");
                Task::none()
            }

            Message::EditorAction(action) => {
                let prev_text = self.content.text();

                let modified_action = match &action {
                    text_editor::Action::Edit(text_editor::Edit::Insert(c)) => {
                        let normalized_c = normalize_persian_char(*c);
                        text_editor::Action::Edit(text_editor::Edit::Insert(normalized_c))
                    }
                    text_editor::Action::Edit(text_editor::Edit::Paste(arc_str)) => {
                        let normalized_str = normalize_persian_string(arc_str);
                        text_editor::Action::Edit(text_editor::Edit::Paste(Arc::new(normalized_str)))
                    }
                    _ => action,
                };

                self.content.perform(modified_action);
                let new_text = self.content.text();

                if new_text != prev_text {
                    self.history.push_snapshot(prev_text);
                    self.is_dirty = true;
                    self.last_edit_time = Instant::now();
                    self.stats = DocumentStats::compute(&new_text);

                    if self.config.smart_dashes || self.config.smart_quotes_de {
                        let transformed = apply_smart_typography(
                            &new_text,
                            self.config.smart_quotes_de,
                            self.config.smart_dashes,
                        );
                        if transformed != new_text {
                            self.content = text_editor::Content::with_text(&transformed);
                        }
                    }

                    if self.search.is_open {
                        self.update_search_matches();
                    }
                }

                Task::none()
            }

            Message::Sidebar(action) => self.handle_sidebar_action(action),

            Message::FileOpened(path_opt) => {
                if let Some(path) = path_opt {
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        self.content = text_editor::Content::with_text(&content);
                        self.stats = DocumentStats::compute(&content);
                        self.current_file_path = Some(path.clone());
                        self.is_dirty = false;
                        self.external_modification_detected = false;
                        self.last_saved_time = Instant::now();

                        let path_str = path.to_string_lossy().to_string();
                        if !self.config.recent_files.contains(&path_str) {
                            self.config.recent_files.insert(0, path_str);
                            if self.config.recent_files.len() > 10 {
                                self.config.recent_files.truncate(10);
                            }
                            let _ = self.config.save();
                        }

                        self.notify("Opened file");
                    }
                }
                Task::none()
            }

            Message::FileDropped(path) => {
                if path.is_file() {
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        self.content = text_editor::Content::with_text(&content);
                        self.stats = DocumentStats::compute(&content);
                        self.current_file_path = Some(path.clone());
                        self.is_dirty = false;
                        self.external_modification_detected = false;
                        self.last_saved_time = Instant::now();
                        self.notify("Loaded dropped file");
                    }
                }
                Task::none()
            }

            Message::FileSaved(path_opt) => {
                if let Some(path) = path_opt {
                    let text = self.content.text();
                    if std::fs::write(&path, &text).is_ok() {
                        self.current_file_path = Some(path.clone());
                        self.is_dirty = false;
                        self.last_saved_time = Instant::now();
                        self.external_modification_detected = false;
                        if let Some(rec) = self.active_recovery_file.take() {
                            RecoveryManager::cleanup_checkpoint(&rec);
                        }
                        self.notify("Saved successfully");
                    }
                }
                Task::none()
            }

            Message::FileExported(path_opt) => {
                if let Some(path) = path_opt {
                    let text = self.content.text();
                    let title = path
                        .file_stem()
                        .map(|s| s.to_string_lossy().to_string())
                        .unwrap_or_else(|| "Krebs Export".to_string());

                    if export::export_to_file(&text, &title, &path).is_ok() {
                        let _ = export::open_exported_html(&path);
                        self.notify("Exported and opened in browser");
                    }
                }
                Task::none()
            }

            Message::Tick => {
                if self.timer_is_running && self.timer_remaining_secs > 0 {
                    self.timer_remaining_secs -= 1;
                    if self.timer_remaining_secs == 0 {
                        self.timer_is_running = false;
                        self.notify("⏱ Focus time is up!");
                    }
                }

                if let Some(path) = self.current_file_path.clone() {
                    if self.is_dirty && self.last_edit_time.elapsed() >= Duration::from_millis(1500) {
                        let text = self.content.text();
                        if std::fs::write(&path, &text).is_ok() {
                            self.is_dirty = false;
                            self.last_saved_time = Instant::now();
                            self.notify("Auto-saved");
                        }
                    }

                    if !self.is_dirty && path.exists() {
                        if let Ok(metadata) = std::fs::metadata(&path) {
                            if let Ok(modified) = metadata.modified() {
                                if let Ok(elapsed_since_save) = modified.elapsed() {
                                    if elapsed_since_save.as_secs() < 3 && self.last_saved_time.elapsed() > Duration::from_secs(4) {
                                        self.external_modification_detected = true;
                                    }
                                }
                            }
                        }
                    }
                } else {
                    if self.is_dirty && self.last_checkpoint_time.elapsed() >= Duration::from_secs(5) {
                        let text = self.content.text();
                        if let Some(rec_path) = RecoveryManager::save_untitled_checkpoint(&text, &mut self.active_recovery_file) {
                            self.active_recovery_file = Some(rec_path);
                            self.last_checkpoint_time = Instant::now();
                        }
                    }
                }

                if let Some((_, time)) = &self.status_notification {
                    if time.elapsed() >= Duration::from_secs(3) {
                        self.status_notification = None;
                    }
                }

                Task::none()
            }

            Message::KeyboardEvent(key, modifiers) => {
                self.handle_keyboard_shortcut(key, modifiers)
            }

            Message::MouseScrolled(delta, modifiers) => {
                if modifiers.control() {
                    if delta > 0.0 {
                        self.zoom_level = (self.zoom_level + 0.1).min(3.0);
                    } else if delta < 0.0 {
                        self.zoom_level = (self.zoom_level - 0.1).max(0.5);
                    }
                    self.config.zoom_level = self.zoom_level;
                    let _ = self.config.save();
                }
                Task::none()
            }

            Message::ReloadFromDisk => {
                if let Some(ref path) = self.current_file_path {
                    if let Ok(content) = std::fs::read_to_string(path) {
                        self.content = text_editor::Content::with_text(&content);
                        self.stats = DocumentStats::compute(&content);
                        self.is_dirty = false;
                        self.external_modification_detected = false;
                        self.last_saved_time = Instant::now();
                        self.notify("Reloaded from disk");
                    }
                }
                Task::none()
            }

            Message::DismissExternalBanner => {
                self.external_modification_detected = false;
                self.last_saved_time = Instant::now();
                Task::none()
            }

            Message::CloseShortcutsModal => {
                self.show_shortcuts_modal = false;
                Task::none()
            }
        }
    }

    fn handle_sidebar_action(&mut self, action: SidebarAction) -> Task<Message> {
        match action {
            SidebarAction::ToggleAppMode => {
                self.app_mode = match self.app_mode {
                    AppMode::Writing => AppMode::Whiteboard,
                    AppMode::Whiteboard => AppMode::Writing,
                };
                Task::none()
            }

            SidebarAction::ToggleSearch => {
                self.search.is_open = !self.search.is_open;
                if self.search.is_open {
                    self.update_search_matches();
                }
                Task::none()
            }

            SidebarAction::CycleWidthMode => {
                self.canvas_width_mode = self.canvas_width_mode.next();
                self.config.canvas_width_mode = self.canvas_width_mode.to_str_name().to_string();
                let _ = self.config.save();
                self.notify(format!("Width: {}", self.canvas_width_mode.label()));
                Task::none()
            }

            SidebarAction::CycleViewMode => {
                self.view_mode = self.view_mode.next();
                self.config.view_mode = self.view_mode.to_str_name().to_string();
                let _ = self.config.save();
                self.notify(format!("Mode: {}", self.view_mode.label()));
                Task::none()
            }

            SidebarAction::ToggleFontModal => {
                self.show_font_modal = !self.show_font_modal;
                Task::none()
            }

            SidebarAction::NewFile => {
                let current_text = self.content.text();
                if self.is_dirty && !current_text.trim().is_empty() {
                    let _ = RecoveryManager::save_untitled_checkpoint(&current_text, &mut self.active_recovery_file);
                }
                self.content = text_editor::Content::with_text("");
                self.current_file_path = None;
                self.active_recovery_file = None;
                self.is_dirty = false;
                self.stats = DocumentStats::compute("");
                self.notify("New untitled buffer");
                Task::none()
            }

            SidebarAction::OpenFile => Task::perform(
                async {
                    rfd::FileDialog::new()
                        .add_filter("Markdown & Text Files", &["md", "markdown", "mdown", "mkd", "txt", "text", "note", "org", "rst"])
                        .add_filter("All Files", &["*"])
                        .pick_file()
                },
                Message::FileOpened,
            ),

            SidebarAction::SaveFile => {
                if let Some(ref path) = self.current_file_path {
                    let text = self.content.text();
                    if std::fs::write(path, &text).is_ok() {
                        self.is_dirty = false;
                        self.last_saved_time = Instant::now();
                        self.external_modification_detected = false;
                        if let Some(rec) = self.active_recovery_file.take() {
                            RecoveryManager::cleanup_checkpoint(&rec);
                        }
                        self.notify("Saved");
                    }
                    Task::none()
                } else {
                    Task::perform(
                        async {
                            rfd::FileDialog::new()
                                .add_filter("Markdown (*.md)", &["md"])
                                .add_filter("Text File (*.txt)", &["txt"])
                                .set_file_name("untitled.md")
                                .save_file()
                        },
                        Message::FileSaved,
                    )
                }
            }

            SidebarAction::ExportHtml => Task::perform(
                async {
                    rfd::FileDialog::new()
                        .add_filter("HTML Document", &["html", "htm"])
                        .set_file_name("document.html")
                        .save_file()
                },
                Message::FileExported,
            ),

            SidebarAction::ToggleLeftRail => {
                self.config.show_left_rail = !self.config.show_left_rail;
                let _ = self.config.save();
                Task::none()
            }

            SidebarAction::ToggleRightDrawer => {
                self.config.show_right_drawer = !self.config.show_right_drawer;
                let _ = self.config.save();
                Task::none()
            }

            SidebarAction::ToggleShortcutsModal => {
                self.show_shortcuts_modal = !self.show_shortcuts_modal;
                Task::none()
            }

            SidebarAction::FormatHeading1 => {
                insert_text_at_cursor(&mut self.content, "# ");
                Task::none()
            }

            SidebarAction::FormatHeading2 => {
                insert_text_at_cursor(&mut self.content, "## ");
                Task::none()
            }

            SidebarAction::FormatBold => {
                wrap_markdown_syntax(&mut self.content, "**", "**", "bold text");
                Task::none()
            }

            SidebarAction::FormatItalic => {
                wrap_markdown_syntax(&mut self.content, "*", "*", "italic text");
                Task::none()
            }

            SidebarAction::FormatStrikethrough => {
                wrap_markdown_syntax(&mut self.content, "~~", "~~", "strikethrough");
                Task::none()
            }

            SidebarAction::FormatBlockquote => {
                insert_text_at_cursor(&mut self.content, "> ");
                Task::none()
            }

            SidebarAction::FormatList => {
                insert_text_at_cursor(&mut self.content, "- ");
                Task::none()
            }

            SidebarAction::FormatCode => {
                wrap_markdown_syntax(&mut self.content, "`", "`", "code");
                Task::none()
            }

            SidebarAction::CycleDirection => {
                self.direction_mode = match self.direction_mode {
                    DirectionMode::Auto => DirectionMode::Ltr,
                    DirectionMode::Ltr => DirectionMode::Rtl,
                    DirectionMode::Rtl => DirectionMode::Auto,
                };
                self.config.direction_mode = self.direction_mode.to_str_name().to_string();
                let _ = self.config.save();
                self.notify(format!("Direction: {}", self.direction_mode.label()));
                Task::none()
            }

            SidebarAction::ToggleGermanBar => {
                self.is_german_bar_open = !self.is_german_bar_open;
                Task::none()
            }

            SidebarAction::InsertGermanChar(ch) => {
                insert_text_at_cursor(&mut self.content, ch);
                self.stats = DocumentStats::compute(&self.content.text());
                Task::none()
            }

            SidebarAction::InsertSnippet(snippet_text) => {
                insert_text_at_cursor(&mut self.content, &snippet_text);
                self.stats = DocumentStats::compute(&self.content.text());
                self.notify("Inserted Redemittel connector");
                Task::none()
            }

            SidebarAction::TimerStart => {
                self.timer_is_running = true;
                Task::none()
            }

            SidebarAction::TimerPause => {
                self.timer_is_running = false;
                Task::none()
            }

            SidebarAction::TimerReset => {
                self.timer_is_running = false;
                self.timer_remaining_secs = self.timer_total_secs;
                Task::none()
            }

            SidebarAction::TimerSetDuration(secs) => {
                self.timer_total_secs = secs;
                self.timer_remaining_secs = secs;
                self.timer_is_running = false;
                self.config.timer_duration_secs = secs;
                let _ = self.config.save();
                Task::none()
            }

            SidebarAction::TargetMinChanged(min) => {
                self.config.target_min_words = min;
                let _ = self.config.save();
                Task::none()
            }

            SidebarAction::TargetMaxChanged(max) => {
                self.config.target_max_words = max;
                let _ = self.config.save();
                Task::none()
            }
        }
    }

    fn handle_keyboard_shortcut(&mut self, key: Key, modifiers: Modifiers) -> Task<Message> {
        let is_ctrl = modifiers.control() || modifiers.command();

        if is_ctrl {
            match key {
                Key::Character(ref s) if s.eq_ignore_ascii_case("n") => {
                    return self.handle_sidebar_action(SidebarAction::NewFile);
                }
                Key::Character(ref s) if s.eq_ignore_ascii_case("o") => {
                    return self.handle_sidebar_action(SidebarAction::OpenFile);
                }
                Key::Character(ref s) if s.eq_ignore_ascii_case("s") => {
                    return self.handle_sidebar_action(SidebarAction::SaveFile);
                }
                Key::Character(ref s) if s.eq_ignore_ascii_case("p") => {
                    return self.handle_sidebar_action(SidebarAction::ExportHtml);
                }
                Key::Character(ref s) if s.eq_ignore_ascii_case("f") => {
                    return self.update(Message::ToggleSearch);
                }
                Key::Character(ref s) if s.eq_ignore_ascii_case("w") => {
                    return self.update(Message::CycleWidthMode);
                }
                Key::Character(ref s) if s.eq_ignore_ascii_case("m") => {
                    let next = match self.app_mode {
                        AppMode::Writing => AppMode::Whiteboard,
                        AppMode::Whiteboard => AppMode::Writing,
                    };
                    return self.update(Message::SetAppMode(next));
                }
                Key::Character(ref s) if s == "1" => {
                    return self.update(Message::SetViewMode(ViewMode::Edit));
                }
                Key::Character(ref s) if s == "2" => {
                    return self.update(Message::SetViewMode(ViewMode::Split));
                }
                Key::Character(ref s) if s == "3" => {
                    return self.update(Message::SetViewMode(ViewMode::Preview));
                }
                Key::Character(ref s) if s.eq_ignore_ascii_case("e") => {
                    return self.handle_sidebar_action(SidebarAction::ToggleRightDrawer);
                }
                Key::Character(ref s) if s == "\\" => {
                    return self.handle_sidebar_action(SidebarAction::ToggleLeftRail);
                }
                Key::Character(ref s) if s == "=" || s == "+" => {
                    self.zoom_level = (self.zoom_level + 0.1).min(3.0);
                    self.config.zoom_level = self.zoom_level;
                    let _ = self.config.save();
                    return Task::none();
                }
                Key::Character(ref s) if s == "-" => {
                    self.zoom_level = (self.zoom_level - 0.1).max(0.5);
                    self.config.zoom_level = self.zoom_level;
                    let _ = self.config.save();
                    return Task::none();
                }
                Key::Character(ref s) if s == "0" => {
                    self.zoom_level = 1.0;
                    self.config.zoom_level = self.zoom_level;
                    let _ = self.config.save();
                    return Task::none();
                }
                Key::Character(ref s) if s.eq_ignore_ascii_case("b") => {
                    wrap_markdown_syntax(&mut self.content, "**", "**", "bold text");
                    return Task::none();
                }
                Key::Character(ref s) if s.eq_ignore_ascii_case("i") => {
                    wrap_markdown_syntax(&mut self.content, "*", "*", "italic text");
                    return Task::none();
                }
                Key::Character(ref s) if s.eq_ignore_ascii_case("z") => {
                    let current_text = self.content.text();
                    if modifiers.shift() {
                        if let Some(snapshot) = self.history.redo(current_text) {
                            self.content = text_editor::Content::with_text(&snapshot);
                        }
                    } else if let Some(snapshot) = self.history.undo(current_text) {
                        self.content = text_editor::Content::with_text(&snapshot);
                    }
                    return Task::none();
                }
                Key::Character(ref s) if s.eq_ignore_ascii_case("y") => {
                    let current_text = self.content.text();
                    if let Some(snapshot) = self.history.redo(current_text) {
                        self.content = text_editor::Content::with_text(&snapshot);
                    }
                    return Task::none();
                }
                Key::Character(ref s) if s == "2" && modifiers.shift() => {
                    insert_zwnj_at_cursor(&mut self.content);
                    self.notify("Inserted ZWNJ (نیم‌فاصله)");
                    return Task::none();
                }
                _ => {}
            }
        }

        if modifiers.shift() && matches!(key, Key::Named(keyboard::key::Named::Space)) {
            insert_zwnj_at_cursor(&mut self.content);
            self.notify("Inserted ZWNJ (نیم‌فاصله)");
            return Task::none();
        }

        if matches!(key, Key::Named(keyboard::key::Named::F1)) {
            self.show_shortcuts_modal = !self.show_shortcuts_modal;
            return Task::none();
        }

        if matches!(key, Key::Named(keyboard::key::Named::Escape)) {
            if self.search.is_open {
                self.search.is_open = false;
                return Task::none();
            }
            if self.show_shortcuts_modal {
                self.show_shortcuts_modal = false;
                return Task::none();
            }
            if self.show_font_modal {
                self.show_font_modal = false;
                return Task::none();
            }
            if self.external_modification_detected {
                self.external_modification_detected = false;
                return Task::none();
            }
        }

        if modifiers.alt() {
            match key {
                Key::Character(ref s) if s.eq_ignore_ascii_case("a") => {
                    let ch = if modifiers.shift() { "Ä" } else { "ä" };
                    insert_text_at_cursor(&mut self.content, ch);
                    return Task::none();
                }
                Key::Character(ref s) if s.eq_ignore_ascii_case("o") => {
                    let ch = if modifiers.shift() { "Ö" } else { "ö" };
                    insert_text_at_cursor(&mut self.content, ch);
                    return Task::none();
                }
                Key::Character(ref s) if s.eq_ignore_ascii_case("u") => {
                    let ch = if modifiers.shift() { "Ü" } else { "ü" };
                    insert_text_at_cursor(&mut self.content, ch);
                    return Task::none();
                }
                Key::Character(ref s) if s.eq_ignore_ascii_case("s") => {
                    insert_text_at_cursor(&mut self.content, "ß");
                    return Task::none();
                }
                _ => {}
            }
        }

        Task::none()
    }

    fn subscription(&self) -> Subscription<Message> {
        let tick_sub = iced::time::every(Duration::from_secs(1)).map(|_| Message::Tick);

        let event_sub = event::listen_with(|event, _status, _window| match event {
            Event::Keyboard(keyboard::Event::KeyPressed {
                key, modifiers, ..
            }) => Some(Message::KeyboardEvent(key, modifiers)),
            Event::Mouse(mouse::Event::WheelScrolled { delta }) => {
                let y = match delta {
                    mouse::ScrollDelta::Lines { y, .. } => y,
                    mouse::ScrollDelta::Pixels { y, .. } => y / 20.0,
                };
                Some(Message::MouseScrolled(y, Modifiers::default()))
            }
            Event::Window(window::Event::FileDropped(path)) => Some(Message::FileDropped(path)),
            _ => None,
        });

        Subscription::batch(vec![tick_sub, event_sub])
    }

    fn view(&self) -> Element<'_, Message> {
        let base_font_size = 15.0 * self.zoom_level;

        let is_persian_doc = {
            let first_line = self.content.text().lines().next().unwrap_or("").to_string();
            detect_paragraph_direction(&first_line) == unicode_bidi::Direction::Rtl
        };

        let active_font = if is_persian_doc || self.direction_mode == DirectionMode::Rtl {
            font_vazirmatn()
        } else {
            self.font_choice.to_font()
        };

        // Top hairline zen timer progress bar (if timer is running)
        let maybe_timer_progress = if self.timer_total_secs > 0 && self.timer_is_running {
            let elapsed = self.timer_total_secs.saturating_sub(self.timer_remaining_secs) as f32;
            let pct = (elapsed / self.timer_total_secs as f32).clamp(0.0, 1.0);
            Some(
                container(
                    row![
                        container(Space::with_height(2))
                            .width(Length::FillPortion((pct * 1000.0) as u16 + 1))
                            .style(|_t| container::Style {
                                background: Some(iced::Background::Color(ACCENT_CYAN)),
                                ..Default::default()
                            }),
                        container(Space::with_height(2))
                            .width(Length::FillPortion(((1.0 - pct) * 1000.0) as u16 + 1))
                            .style(|_t| container::Style {
                                background: Some(iced::Background::Color(Color::TRANSPARENT)),
                                ..Default::default()
                            }),
                    ],
                )
                .width(Length::Fill)
                .height(2),
            )
        } else {
            None
        };

        // Main Studio Canvas View (Writing vs Whiteboard)
        let main_canvas: Element<'_, Message> = match self.app_mode {
            AppMode::Whiteboard => self.view_whiteboard_studio(),
            AppMode::Writing => self.view_writing_studio(base_font_size, active_font),
        };

        // Left 42px Vertical Rail
        let maybe_left_rail = if self.config.show_left_rail {
            Some(
                view_left_rail(
                    self.direction_mode,
                    self.config.show_right_drawer,
                    self.is_german_bar_open,
                    self.app_mode == AppMode::Whiteboard,
                    self.search.is_open,
                )
                .map(Message::Sidebar),
            )
        } else {
            None
        };

        // Right Exam & Redemittel Drawer
        let maybe_right_drawer = if self.config.show_right_drawer && self.app_mode == AppMode::Writing {
            Some(
                view_right_drawer(
                    self.timer_remaining_secs,
                    self.timer_is_running,
                    self.timer_total_secs,
                    self.config.target_min_words,
                    self.config.target_max_words,
                    &self.stats,
                    &self.snippets,
                )
                .map(Message::Sidebar),
            )
        } else {
            None
        };

        let mut center_row = row![].width(Length::Fill).height(Length::Fill);
        if let Some(rail) = maybe_left_rail {
            center_row = center_row.push(rail);
        }
        center_row = center_row.push(main_canvas);
        if let Some(drawer) = maybe_right_drawer {
            center_row = center_row.push(drawer);
        }

        let mut root_column = column![];
        if let Some(prog) = maybe_timer_progress {
            root_column = root_column.push(prog);
        }
        root_column = root_column.push(center_row);

        let mut root_stack = stack![container(root_column)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(canvas_container_style)];

        if self.show_shortcuts_modal {
            root_stack = root_stack.push(
                container(self.view_shortcuts_modal())
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .center(Length::Fill)
                    .style(|_t| container::Style {
                        background: Some(iced::Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.65))),
                        ..Default::default()
                    }),
            );
        }

        if self.show_font_modal {
            root_stack = root_stack.push(
                container(self.view_font_modal())
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .center(Length::Fill)
                    .style(|_t| container::Style {
                        background: Some(iced::Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.65))),
                        ..Default::default()
                    }),
            );
        }

        root_stack.into()
    }

    fn view_writing_studio<'a>(&'a self, base_font_size: f32, active_font: Font) -> Element<'a, Message> {
        // Text Editor Widget
        let editor_widget = text_editor(&self.content)
            .font(active_font)
            .size(base_font_size)
            .style(text_editor_style)
            .placeholder("No vaults, no plugins, just you and the words...\nBegin writing your notes or exam essay here.")
            .on_action(Message::EditorAction);

        // Rendered Markdown View (supports Bold, Italic, Quotes, Lists, Headings)
        let rendered_markdown = scrollable(markdown_view::render_markdown(
            &self.content.text(),
            active_font,
            self.zoom_level,
        ))
        .height(Length::Fill)
        .width(Length::Fill);

        // Center Content depending on ViewMode (Edit / Live Split / Preview)
        let center_content: Element<'a, Message> = match self.view_mode {
            ViewMode::Edit => editor_widget.into(),
            ViewMode::Split => {
                row![
                    container(editor_widget)
                        .width(Length::FillPortion(1))
                        .height(Length::Fill),
                    container(Space::with_width(1))
                        .width(1)
                        .height(Length::Fill)
                        .style(|_t| container::Style {
                            background: Some(iced::Background::Color(BORDER_COLOR)),
                            ..Default::default()
                        }),
                    container(rendered_markdown)
                        .width(Length::FillPortion(1))
                        .height(Length::Fill)
                        .padding(iced::Padding {
                            top: 0.0,
                            bottom: 0.0,
                            left: 16.0,
                            right: 8.0,
                        }),
                ]
                .spacing(12)
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
            }
            ViewMode::Preview => rendered_markdown.into(),
        };

        // Adjustable Canvas Width calculation
        let center_column = match self.canvas_width_mode {
            CanvasWidthMode::Standard => container(center_content)
                .max_width(850.0 * self.zoom_level)
                .width(Length::Fill)
                .height(Length::Fill)
                .padding(iced::Padding {
                    top: 24.0,
                    bottom: 24.0,
                    left: 20.0,
                    right: 20.0,
                }),
            CanvasWidthMode::Wide => container(center_content)
                .max_width(1350.0 * self.zoom_level)
                .width(Length::Fill)
                .height(Length::Fill)
                .padding(iced::Padding {
                    top: 24.0,
                    bottom: 24.0,
                    left: 24.0,
                    right: 24.0,
                }),
            CanvasWidthMode::Full => container(center_content)
                .width(Length::Fill)
                .height(Length::Fill)
                .padding(iced::Padding {
                    top: 24.0,
                    bottom: 24.0,
                    left: 32.0,
                    right: 32.0,
                }),
        };

        // Top Subtle Control Header
        let top_header = self.view_top_header();

        // Optional Search Bar Overlay (Ctrl+F)
        let maybe_search = if self.search.is_open {
            Some(self.view_search_bar())
        } else {
            None
        };

        // Optional External modification banner
        let maybe_banner = if self.external_modification_detected {
            Some(
                container(
                    row![
                        text("⚠️ File modified externally on disk.")
                            .size(12)
                            .color(ACCENT_AMBER),
                        Space::with_width(Length::Fill),
                        button(text("Reload from Disk").size(11))
                            .on_press(Message::ReloadFromDisk)
                            .style(rail_button_style)
                            .padding(iced::Padding::new(4.0)),
                        Space::with_width(8),
                        button(text("Keep Local Changes").size(11))
                            .on_press(Message::DismissExternalBanner)
                            .style(rail_button_style)
                            .padding(iced::Padding::new(4.0)),
                    ]
                    .align_y(Alignment::Center),
                )
                .padding(8)
                .width(Length::Fill)
                .style(banner_container_style),
            )
        } else {
            None
        };

        // Bottom Status Bar
        let bottom_status_bar = self.view_bottom_status_bar();

        let mut studio_col = column![top_header];
        if let Some(search_widget) = maybe_search {
            studio_col = studio_col.push(search_widget);
        }
        if let Some(banner) = maybe_banner {
            studio_col = studio_col.push(banner);
        }
        studio_col = studio_col.push(
            container(center(center_column))
                .width(Length::Fill)
                .height(Length::Fill),
        );
        studio_col = studio_col.push(bottom_status_bar);

        container(studio_col)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn view_whiteboard_studio<'a>(&'a self) -> Element<'a, Message> {
        let toolbar = self.view_whiteboard_toolbar();

        let canvas_widget = iced::widget::canvas(WhiteboardCanvas {
            state: &self.whiteboard,
        })
        .width(Length::Fill)
        .height(Length::Fill);

        let content_col = column![toolbar, canvas_widget]
            .width(Length::Fill)
            .height(Length::Fill);

        container(content_col)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(canvas_container_style)
            .into()
    }

    fn view_top_header<'a>(&'a self) -> Element<'a, Message> {
        // Mode Switcher Chips: Writing Studio vs Whiteboard
        let write_tab_style = if self.app_mode == AppMode::Writing {
            tab_active_button_style
        } else {
            tab_inactive_button_style
        };
        let btn_write = button(text("📝 Write").size(11))
            .on_press(Message::SetAppMode(AppMode::Writing))
            .style(write_tab_style)
            .padding(iced::Padding { top: 3.0, bottom: 3.0, left: 10.0, right: 10.0 });

        let board_tab_style = if self.app_mode == AppMode::Whiteboard {
            tab_active_button_style
        } else {
            tab_inactive_button_style
        };
        let btn_board = button(text("🎨 Whiteboard").size(11))
            .on_press(Message::SetAppMode(AppMode::Whiteboard))
            .style(board_tab_style)
            .padding(iced::Padding { top: 3.0, bottom: 3.0, left: 10.0, right: 10.0 });

        // View Mode Segment: Edit | Split | Preview
        let btn_mode_edit = button(text("Edit").size(11))
            .on_press(Message::SetViewMode(ViewMode::Edit))
            .style(if self.view_mode == ViewMode::Edit { tab_active_button_style } else { tab_inactive_button_style })
            .padding(iced::Padding { top: 3.0, bottom: 3.0, left: 8.0, right: 8.0 });

        let btn_mode_split = button(text("◫ Live Split").size(11))
            .on_press(Message::SetViewMode(ViewMode::Split))
            .style(if self.view_mode == ViewMode::Split { tab_active_button_style } else { tab_inactive_button_style })
            .padding(iced::Padding { top: 3.0, bottom: 3.0, left: 8.0, right: 8.0 });

        let btn_mode_preview = button(text("👁️ Preview").size(11))
            .on_press(Message::SetViewMode(ViewMode::Preview))
            .style(if self.view_mode == ViewMode::Preview { tab_active_button_style } else { tab_inactive_button_style })
            .padding(iced::Padding { top: 3.0, bottom: 3.0, left: 8.0, right: 8.0 });

        // Width Cycle Button
        let btn_width = button(text(format!("↔ {}", self.canvas_width_mode.label())).size(11))
            .on_press(Message::CycleWidthMode)
            .style(rail_button_style)
            .padding(iced::Padding { top: 3.0, bottom: 3.0, left: 8.0, right: 8.0 });

        // Font Switcher Button
        let btn_font = button(text(format!("🔤 {}", self.font_choice.display_name())).size(11))
            .on_press(Message::ToggleFontModal)
            .style(rail_button_style)
            .padding(iced::Padding { top: 3.0, bottom: 3.0, left: 8.0, right: 8.0 });

        // Minimal Zen Clock
        let mins = self.timer_remaining_secs / 60;
        let secs = self.timer_remaining_secs % 60;
        let clock_str = format!("{:02}:{:02}", mins, secs);
        let clock_color = if self.timer_is_running {
            ACCENT_CYAN
        } else {
            TEXT_MUTED
        };

        let btn_clock = button(text(format!("⏱ {}", clock_str)).size(11).color(clock_color))
            .on_press(Message::ZenTimerToggle)
            .style(rail_button_style)
            .padding(iced::Padding { top: 3.0, bottom: 3.0, left: 8.0, right: 8.0 });

        let btn_clock_presets = button(text("▾").size(10).color(TEXT_MUTED))
            .on_press(Message::ZenTimerTogglePresets)
            .style(rail_button_style)
            .padding(iced::Padding { top: 3.0, bottom: 3.0, left: 4.0, right: 4.0 });

        let mut header_row = row![
            btn_write,
            btn_board,
            Space::with_width(12),
            btn_mode_edit,
            btn_mode_split,
            btn_mode_preview,
            Space::with_width(Length::Fill),
            btn_width,
            btn_font,
            btn_clock,
            btn_clock_presets,
        ]
        .spacing(4)
        .align_y(Alignment::Center);

        if self.show_timer_presets {
            let p15 = button(text("15m").size(10)).on_press(Message::ZenTimerSet(900)).style(rail_button_style).padding(iced::Padding::new(3.0));
            let p25 = button(text("25m").size(10)).on_press(Message::ZenTimerSet(1500)).style(rail_button_style).padding(iced::Padding::new(3.0));
            let p45 = button(text("45m").size(10)).on_press(Message::ZenTimerSet(2700)).style(rail_button_style).padding(iced::Padding::new(3.0));
            let p60 = button(text("60m").size(10)).on_press(Message::ZenTimerSet(3600)).style(rail_button_style).padding(iced::Padding::new(3.0));
            let rst = button(text("↺").size(10)).on_press(Message::Sidebar(SidebarAction::TimerReset)).style(rail_button_style).padding(iced::Padding::new(3.0));

            header_row = header_row.push(row![p15, p25, p45, p60, rst].spacing(2).align_y(Alignment::Center));
        }

        container(header_row)
            .width(Length::Fill)
            .padding(iced::Padding {
                top: 6.0,
                bottom: 6.0,
                left: 14.0,
                right: 14.0,
            })
            .style(|_t| container::Style {
                background: Some(iced::Background::Color(RAIL_BG)),
                border: iced::Border {
                    color: BORDER_COLOR,
                    width: 1.0,
                    ..Default::default()
                },
                ..Default::default()
            })
            .into()
    }

    fn view_search_bar<'a>(&'a self) -> Element<'a, Message> {
        let input = text_input("Find in document...", &self.search.query)
            .on_input(Message::SearchInputChanged)
            .on_submit(Message::SearchNext)
            .style(text_input_style)
            .size(12)
            .width(220);

        let match_label = if self.search.query.trim().is_empty() {
            "".to_string()
        } else if self.search.matches.is_empty() {
            "No matches".to_string()
        } else {
            format!("{}/{}", self.search.current_idx + 1, self.search.matches.len())
        };

        let btn_prev = button(text("↑").size(11))
            .on_press(Message::SearchPrev)
            .style(rail_button_style)
            .padding(iced::Padding::new(4.0));

        let btn_next = button(text("↓").size(11))
            .on_press(Message::SearchNext)
            .style(rail_button_style)
            .padding(iced::Padding::new(4.0));

        let btn_close = button(text("✕").size(11))
            .on_press(Message::CloseSearch)
            .style(rail_button_style)
            .padding(iced::Padding::new(4.0));

        let search_row = row![
            text("🔍").size(12).color(ACCENT_CYAN),
            input,
            text(match_label).size(11).color(ACCENT_CYAN),
            btn_prev,
            btn_next,
            btn_close,
        ]
        .spacing(8)
        .align_y(Alignment::Center);

        container(search_row)
            .padding(iced::Padding { top: 6.0, bottom: 6.0, left: 12.0, right: 12.0 })
            .style(search_container_style)
            .into()
    }

    fn view_whiteboard_toolbar<'a>(&'a self) -> Element<'a, Message> {
        let btn_write = button(text("📝 Write Studio").size(11))
            .on_press(Message::SetAppMode(AppMode::Writing))
            .style(tab_inactive_button_style)
            .padding(iced::Padding { top: 3.0, bottom: 3.0, left: 8.0, right: 8.0 });

        let is_pen = self.whiteboard.active_tool == WhiteboardTool::Pen;
        let is_hl = self.whiteboard.active_tool == WhiteboardTool::Highlighter;
        let is_er = self.whiteboard.active_tool == WhiteboardTool::Eraser;

        let btn_pen = button(text("✏️ Pen").size(11))
            .on_press(Message::WhiteboardToolSelect(WhiteboardTool::Pen))
            .style(if is_pen { active_tool_button_style } else { rail_button_style })
            .padding(iced::Padding { top: 3.0, bottom: 3.0, left: 8.0, right: 8.0 });

        let btn_hl = button(text("🖍️ Highlighter").size(11))
            .on_press(Message::WhiteboardToolSelect(WhiteboardTool::Highlighter))
            .style(if is_hl { active_tool_button_style } else { rail_button_style })
            .padding(iced::Padding { top: 3.0, bottom: 3.0, left: 8.0, right: 8.0 });

        let btn_eraser = button(text("🧹 Eraser").size(11))
            .on_press(Message::WhiteboardToolSelect(WhiteboardTool::Eraser))
            .style(if is_er { active_tool_button_style } else { rail_button_style })
            .padding(iced::Padding { top: 3.0, bottom: 3.0, left: 8.0, right: 8.0 });

        // Stroke Width chips
        let w_fine = button(text("1.5px").size(10))
            .on_press(Message::WhiteboardWidthSelect(1.5))
            .style(if (self.whiteboard.pen_width - 1.5).abs() < 0.1 { active_tool_button_style } else { rail_button_style })
            .padding(iced::Padding::new(3.0));

        let w_med = button(text("3.0px").size(10))
            .on_press(Message::WhiteboardWidthSelect(3.0))
            .style(if (self.whiteboard.pen_width - 3.0).abs() < 0.1 { active_tool_button_style } else { rail_button_style })
            .padding(iced::Padding::new(3.0));

        let w_bold = button(text("6.0px").size(10))
            .on_press(Message::WhiteboardWidthSelect(6.0))
            .style(if (self.whiteboard.pen_width - 6.0).abs() < 0.1 { active_tool_button_style } else { rail_button_style })
            .padding(iced::Padding::new(3.0));

        // Color chips
        let c_white = button(text("●").size(13).color(Color::from_rgb(0.88, 0.90, 0.94)))
            .on_press(Message::WhiteboardColorSelect(Color::from_rgb(0.88, 0.90, 0.94)))
            .style(rail_button_style)
            .padding(iced::Padding::new(2.0));

        let c_cyan = button(text("●").size(13).color(ACCENT_CYAN))
            .on_press(Message::WhiteboardColorSelect(ACCENT_CYAN))
            .style(rail_button_style)
            .padding(iced::Padding::new(2.0));

        let c_amber = button(text("●").size(13).color(ACCENT_AMBER))
            .on_press(Message::WhiteboardColorSelect(ACCENT_AMBER))
            .style(rail_button_style)
            .padding(iced::Padding::new(2.0));

        let c_green = button(text("●").size(13).color(ACCENT_GREEN))
            .on_press(Message::WhiteboardColorSelect(ACCENT_GREEN))
            .style(rail_button_style)
            .padding(iced::Padding::new(2.0));

        let c_pink = button(text("●").size(13).color(ACCENT_RED))
            .on_press(Message::WhiteboardColorSelect(ACCENT_RED))
            .style(rail_button_style)
            .padding(iced::Padding::new(2.0));

        // Paper Patterns
        let p_dots = button(text("Dots").size(10))
            .on_press(Message::WhiteboardPatternSelect(PaperPattern::DotGrid))
            .style(if self.whiteboard.paper_pattern == PaperPattern::DotGrid { active_tool_button_style } else { rail_button_style })
            .padding(iced::Padding::new(3.0));

        let p_ruled = button(text("Ruled").size(10))
            .on_press(Message::WhiteboardPatternSelect(PaperPattern::Ruled))
            .style(if self.whiteboard.paper_pattern == PaperPattern::Ruled { active_tool_button_style } else { rail_button_style })
            .padding(iced::Padding::new(3.0));

        let p_math = button(text("Math Grid").size(10))
            .on_press(Message::WhiteboardPatternSelect(PaperPattern::MathGrid))
            .style(if self.whiteboard.paper_pattern == PaperPattern::MathGrid { active_tool_button_style } else { rail_button_style })
            .padding(iced::Padding::new(3.0));

        let p_blank = button(text("Blank").size(10))
            .on_press(Message::WhiteboardPatternSelect(PaperPattern::Blank))
            .style(if self.whiteboard.paper_pattern == PaperPattern::Blank { active_tool_button_style } else { rail_button_style })
            .padding(iced::Padding::new(3.0));

        let btn_undo = button(text("↶ Undo").size(10))
            .on_press(Message::WhiteboardUndo)
            .style(rail_button_style)
            .padding(iced::Padding::new(3.0));

        let btn_redo = button(text("↷ Redo").size(10))
            .on_press(Message::WhiteboardRedo)
            .style(rail_button_style)
            .padding(iced::Padding::new(3.0));

        // Student Wipe / Delete Work Action
        let btn_wipe = button(text("🗑️ Wipe Board").size(11))
            .on_press(Message::WhiteboardClear)
            .style(wipe_button_style)
            .padding(iced::Padding { top: 3.0, bottom: 3.0, left: 8.0, right: 8.0 });

        let tb_row = row![
            btn_write,
            Space::with_width(8),
            btn_pen,
            btn_hl,
            btn_eraser,
            Space::with_width(8),
            w_fine,
            w_med,
            w_bold,
            Space::with_width(8),
            c_white,
            c_cyan,
            c_amber,
            c_green,
            c_pink,
            Space::with_width(12),
            text("Paper:").size(10).color(TEXT_MUTED),
            p_dots,
            p_ruled,
            p_math,
            p_blank,
            Space::with_width(Length::Fill),
            btn_undo,
            btn_redo,
            btn_wipe,
        ]
        .spacing(4)
        .align_y(Alignment::Center);

        container(tb_row)
            .width(Length::Fill)
            .padding(iced::Padding { top: 6.0, bottom: 6.0, left: 12.0, right: 12.0 })
            .style(toolbar_floating_style)
            .into()
    }

    fn view_bottom_status_bar<'a>(&'a self) -> Element<'a, Message> {
        let file_name = self
            .current_file_path
            .as_ref()
            .and_then(|p| p.file_name())
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| {
                if self.is_dirty {
                    "Untitled (Auto-checkpoints active)".to_string()
                } else {
                    "Untitled".to_string()
                }
            });

        let dirty_indicator = if self.is_dirty { " • Unsaved" } else { "" };
        let file_label = format!("{}{}", file_name, dirty_indicator);

        let stats_label = format!(
            "{} words  |  {} chars  |  {:.1} min",
            self.stats.word_count, self.stats.char_count, self.stats.reading_time_mins
        );

        let zoom_pct = (self.zoom_level * 100.0).round() as u32;
        let mode_label = format!("{} | {}% | {}", self.direction_mode.label(), zoom_pct, self.canvas_width_mode.label());

        let maybe_toast = self.status_notification.as_ref().map(|(msg, _)| {
            text(msg).size(10).color(ACCENT_CYAN)
        });

        let mut status_row = row![
            text(file_label).size(10).color(TEXT_MUTED),
            Space::with_width(16),
        ]
        .align_y(Alignment::Center);

        if let Some(toast) = maybe_toast {
            status_row = status_row.push(toast);
        }

        status_row = status_row.push(Space::with_width(Length::Fill));
        status_row = status_row.push(text(stats_label).size(10).color(TEXT_MUTED));
        status_row = status_row.push(Space::with_width(Length::Fill));
        status_row = status_row.push(text(mode_label).size(10).color(TEXT_MUTED));

        container(status_row)
            .padding(iced::Padding {
                top: 4.0,
                bottom: 4.0,
                left: 16.0,
                right: 16.0,
            })
            .width(Length::Fill)
            .style(|_t| container::Style {
                background: Some(iced::Background::Color(RAIL_BG)),
                border: iced::Border {
                    color: BORDER_COLOR,
                    width: 1.0,
                    ..Default::default()
                },
                ..Default::default()
            })
            .into()
    }

    fn view_font_modal<'a>(&self) -> Element<'a, Message> {
        let title_row = row![
            text("🔤 Select Typography").size(15).color(ACCENT_CYAN),
            Space::with_width(Length::Fill),
            button(text("✕").size(12))
                .on_press(Message::CloseFontModal)
                .style(rail_button_style)
                .width(28)
                .height(28),
        ]
        .align_y(Alignment::Center);

        let font_items = [
            (FontChoice::JetBrainsMono, "JetBrains Mono", "Famous Modern Coding & Prose Font (Default)"),
            (FontChoice::IbmPlexMono, "IBM Plex Mono", "Crisp Modernist Typewriter / Monospace"),
            (FontChoice::Vt323, "VT323 Retro CRT", "Iconic 80s CRT Terminal / Retro Computing"),
            (FontChoice::Vazirmatn, "Vazirmatn", "Premier Persian / RTL Cursive Ligature Shaping"),
            (FontChoice::Sans, "System Sans-Serif", "Clean, Distraction-Free Modern Sans"),
            (FontChoice::Serif, "System Serif", "Classic Literary Book & Editorial Serif"),
        ];

        let mut font_list = column![title_row, Space::with_height(8)].spacing(8);

        for (choice, name, desc) in font_items {
            let is_selected = self.font_choice == choice;
            let card_style = if is_selected {
                active_tool_button_style
            } else {
                rail_button_style
            };

            let item_btn = button(
                column![
                    text(name).size(13).color(if is_selected { ACCENT_CYAN } else { TEXT_PRIMARY }),
                    text(desc).size(10).color(TEXT_MUTED),
                ]
                .spacing(2),
            )
            .on_press(Message::SelectFont(choice))
            .style(card_style)
            .padding(10)
            .width(Length::Fill);

            font_list = font_list.push(item_btn);
        }

        container(font_list)
            .width(460)
            .padding(18)
            .style(modal_container_style)
            .into()
    }

    fn view_shortcuts_modal<'a>(&self) -> Element<'a, Message> {
        let title_row = row![
            text("⌨️ Krebs Shortcuts & Quick Guide")
                .size(15)
                .color(ACCENT_CYAN),
            Space::with_width(Length::Fill),
            button(text("✕").size(12))
                .on_press(Message::CloseShortcutsModal)
                .style(rail_button_style)
                .width(28)
                .height(28),
        ]
        .align_y(Alignment::Center);

        let file_section = column![
            text("WORKSPACE & STUDIO").size(10).color(TEXT_MUTED),
            row![text("Ctrl + M").size(11).color(ACCENT_CYAN), Space::with_width(Length::Fill), text("Toggle Write / Whiteboard Studio").size(11).color(TEXT_PRIMARY)],
            row![text("Ctrl + 1 / 2 / 3").size(11).color(ACCENT_CYAN), Space::with_width(Length::Fill), text("Edit / Live Split / Preview View").size(11).color(TEXT_PRIMARY)],
            row![text("Ctrl + W").size(11).color(ACCENT_CYAN), Space::with_width(Length::Fill), text("Cycle Width (Standard / Wide / Full)").size(11).color(TEXT_PRIMARY)],
            row![text("Ctrl + F").size(11).color(ACCENT_CYAN), Space::with_width(Length::Fill), text("Find / Search in Document").size(11).color(TEXT_PRIMARY)],
            row![text("Ctrl + N / O / S").size(11).color(ACCENT_CYAN), Space::with_width(Length::Fill), text("New / Open / Save Document").size(11).color(TEXT_PRIMARY)],
            row![text("Ctrl + P").size(11).color(ACCENT_CYAN), Space::with_width(Length::Fill), text("Export Print / HTML").size(11).color(TEXT_PRIMARY)],
        ]
        .spacing(6);

        let format_section = column![
            text("MARKDOWN FORMATTING").size(10).color(TEXT_MUTED),
            row![text("Ctrl + B / I").size(11).color(ACCENT_CYAN), Space::with_width(Length::Fill), text("Bold (**) / Italic (*)").size(11).color(TEXT_PRIMARY)],
            row![text("Ctrl + + / - / 0").size(11).color(ACCENT_CYAN), Space::with_width(Length::Fill), text("Zoom In / Out / Reset (50%-300%)").size(11).color(TEXT_PRIMARY)],
            row![text("Shift + Space").size(11).color(ACCENT_CYAN), Space::with_width(Length::Fill), text("Persian ZWNJ (نیم‌فاصله)").size(11).color(TEXT_PRIMARY)],
            row![text("Alt + a/o/u/s").size(11).color(ACCENT_CYAN), Space::with_width(Length::Fill), text("German Chords (ä, ö, ü, ß)").size(11).color(TEXT_PRIMARY)],
        ]
        .spacing(6);

        let content_col = column![
            title_row,
            create_divider_simple(),
            file_section,
            create_divider_simple(),
            format_section,
        ]
        .spacing(12);

        container(content_col)
            .width(520)
            .padding(18)
            .style(modal_container_style)
            .into()
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }
}

fn create_divider_simple<'a, Message: 'a>() -> Element<'a, Message> {
    container(Space::with_height(1))
        .style(|_t| container::Style {
            background: Some(iced::Background::Color(BORDER_COLOR)),
            ..Default::default()
        })
        .width(Length::Fill)
        .height(1)
        .into()
}

fn create_app_icon() -> Option<window::Icon> {
    if let Ok(img) = image::load_from_memory(KREBS_LOGO_PNG) {
        let resized = img.resize_exact(64, 64, image::imageops::FilterType::Lanczos3);
        let rgba = resized.to_rgba8();
        let (width, height) = rgba.dimensions();
        window::icon::from_rgba(rgba.into_raw(), width, height).ok()
    } else {
        None
    }
}

fn main() -> iced::Result {
    std::panic::set_hook(Box::new(|info| {
        let log_dir = config::get_portable_data_dir();
        let msg = format!("PANIC: {:?}\n", info);
        let _ = std::fs::write(log_dir.join("panic.log"), msg);
    }));

    let args: Vec<String> = std::env::args().collect();
    let initial_file = if args.len() > 1 {
        Some(PathBuf::from(&args[1]))
    } else {
        None
    };

    let config = AppConfig::load_or_default();

    let res = iced::application(
        "Krebs — Minimalist Writing & Whiteboard Studio",
        Krebs::update,
        Krebs::view,
    )
    .window(window::Settings {
        size: Size::new(config.window_width, config.window_height),
        min_size: Some(Size::new(640.0, 420.0)),
        icon: create_app_icon(),
        ..Default::default()
    })
    .subscription(Krebs::subscription)
    .theme(Krebs::theme)
    .run_with(move || Krebs::new(initial_file));

    if let Err(ref e) = res {
        let log_dir = config::get_portable_data_dir();
        let _ = std::fs::write(log_dir.join("error.log"), format!("RUN ERROR: {:?}\n", e));
    }
    res
}
