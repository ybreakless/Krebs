#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

pub mod config;
pub mod editor;
pub mod ai;
pub mod export;
pub mod icons;
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
    button, center, column, container, mouse_area, row, scrollable, stack, text, text_editor, text_input, Space,
};
use iced::window;
use iced::{Alignment, Color, Element, Font, Length, Size, Subscription, Task, Theme};

use config::{AppConfig, RecoveryManager, SnippetsConfig, SnippetsDeManager};
use editor::{
    apply_smart_typography, detect_paragraph_direction, insert_text_at_cursor,
    insert_zwnj_at_cursor, normalize_persian_char, normalize_persian_string, wrap_markdown_syntax,
    DirectionMode, DocumentStats, EditorHistory,
};
use sidebar::{view_left_rail, SidebarAction};
use crate::icons::*;
use ai::{AiAction, AiProvider, AiState};
use theme::{
    active_tool_button_style, banner_container_style, canvas_container_style, card_container_style,
    drawer_container_style,
    modal_container_style, rail_button_style, search_container_style,
    tab_active_button_style, tab_inactive_button_style, text_editor_style, text_input_style,
    toolbar_floating_style, wipe_button_style, ACCENT_AMBER, ACCENT_CYAN, ACCENT_GREEN,
    ACCENT_RED, BORDER_COLOR, RAIL_BG, TEXT_MUTED, TEXT_PRIMARY, AccentChoice, ThemeMode,
};
use whiteboard::{PaperPattern, WhiteboardCanvas, WhiteboardMessage, WhiteboardState, WhiteboardTool};

// Bundled Offline Fonts & Assets
pub static KREBS_LOGO_PNG: &[u8] = include_bytes!("../assets/logo.png");pub static IBM_PLEX_MONO_REGULAR: &[u8] = include_bytes!("../fonts/IBMPlexMono-Regular.ttf");
pub static IBM_PLEX_MONO_BOLD: &[u8] = include_bytes!("../fonts/IBMPlexMono-Bold.ttf");
pub static VAZIRMATN_REGULAR: &[u8] = include_bytes!("../fonts/Vazirmatn-Regular.ttf");
pub static VAZIRMATN_BOLD: &[u8] = include_bytes!("../fonts/Vazirmatn-Bold.ttf");
pub static JETBRAINS_MONO_REGULAR: &[u8] = include_bytes!("../fonts/JetBrainsMono-Regular.ttf");
pub static VT323_REGULAR: &[u8] = include_bytes!("../fonts/VT323-Regular.ttf");

/// Wraps any widget in a bottom-anchored hover tooltip.
///
/// Every toolbar affordance gets one so compact labels stay discoverable; the
/// hint text is conventionally spelled `Name (Shortcut)`, for example
/// `Zen Mode (Ctrl+H)`.
///
/// # Arguments
/// - `content` — the widget to annotate, typically an icon or pill button.
/// - `label` — the full hint text shown on hover.
///
/// # Examples
/// ```
/// with_tooltip(btn_zen, "Zen Mode (Ctrl+H)").into()
/// ```
pub fn with_tooltip<'a, Msg: 'a, Label>(
    content: impl Into<Element<'a, Msg>>,
    label: Label,
) -> Element<'a, Msg>
where
    Label: Into<String>,
{
    iced::widget::Tooltip::new(
        content.into(),
        iced::widget::text(label.into()),
        iced::widget::tooltip::Position::Bottom,
    )
    .style(|_theme| container::Style {
        background: Some(iced::Background::Color(Color::from_rgba(0.03, 0.04, 0.07, 0.97))),
        border: iced::Border {
            color: ACCENT_CYAN,
            width: 1.0,
            radius: iced::border::Radius::from(6.0),
        },
        text_color: Some(TEXT_PRIMARY),
        shadow: iced::Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.45),
            offset: iced::Vector::new(0.0, 3.0),
            blur_radius: 8.0,
        },
    })
    .gap(6)
    .into()
}

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
    theme_mode: ThemeMode,
    accent_choice: AccentChoice,
    show_theme_modal: bool,
    show_tour_modal: bool,
    tour_step: usize,
    is_maximized: bool,
    show_top_toolbar: bool,
    ai_state: AiState,
    right_drawer_tab: RightDrawerTab,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RightDrawerTab {
    AiCoach,
    Redemittel,
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
    ToggleTopToolbar,
    ToggleAiDrawer,
    SetRightDrawerTab(RightDrawerTab),
    SelectAiProvider(AiProvider),
    AiApiKeyChanged(String),
    AdjustSplitRatio(f32),
    #[allow(dead_code)]
    SetSplitRatio(f32),
    CycleSplitPresets,
    AdjustDrawerWidth(f32),
    RunAiAction(AiAction),
    AiCompleted(String, bool),
    AiCopyFeedback,
    AiInsertFeedback,
    AiSetEndpoint(String),
    AiSetModel(String),
    AiToggleSettings,
    ToggleThemeModal,
    CloseThemeModal,
    SelectTheme(ThemeMode),
    SelectAccent(AccentChoice),
    ToggleTourModal,
    CloseTourModal,
    TourNext,
    TourPrev,
    TourFinish,
    WindowMinimize,
    WindowToggleMaximize,
    WindowClose,
    WindowDrag,
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
                if let Ok(raw) = std::fs::read_to_string(&path) {
                        let content = raw.replace('\t', "    ");
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

        initial_text = initial_text.replace('\t', "    ");
        let stats = DocumentStats::compute(&initial_text);
        let content = text_editor::Content::with_text(&initial_text);
        let now = Instant::now();

        let theme_mode = ThemeMode::from_str_name(&config.theme);
        let accent_choice = AccentChoice::from_str_name(&config.accent_color);
        let show_tour_modal = !config.has_seen_tour;

        let show_top_toolbar = config.show_top_toolbar;
        let ai_endpoint = config.ai_endpoint.clone();
        let ai_model = config.ai_model.clone();
        let ai_provider = AiProvider::from_str_id(&config.ai_provider);
        let ai_api_key = config.ai_api_key.clone();

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
            theme_mode,
            accent_choice,
            show_theme_modal: false,
            show_tour_modal,
            tour_step: 0,
            is_maximized: false,
            show_top_toolbar,
            ai_state: AiState {
                provider: ai_provider,
                endpoint: ai_endpoint.clone(),
                api_key: ai_api_key.clone(),
                temp_api_key: ai_api_key,
                model: ai_model.clone(),
                temp_endpoint: ai_endpoint,
                temp_model: ai_model,
                ..Default::default()
            },
            right_drawer_tab: RightDrawerTab::AiCoach,
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

            Message::ToggleTopToolbar => {
                self.show_top_toolbar = !self.show_top_toolbar;
                self.config.show_top_toolbar = self.show_top_toolbar;
                let _ = self.config.save();
                self.notify(if self.show_top_toolbar {
                    "Toolbar Visible"
                } else {
                    "Zen Mode: Toolbar Hidden (Press Ctrl+H to restore)"
                });
                Task::none()
            }

            Message::ToggleAiDrawer => {
                self.config.show_right_drawer = !self.config.show_right_drawer;
                self.right_drawer_tab = RightDrawerTab::AiCoach;
                let _ = self.config.save();
                Task::none()
            }

            Message::SelectAiProvider(provider) => {
                self.ai_state.provider = provider;
                self.ai_state.endpoint = provider.default_endpoint().to_string();
                self.ai_state.model = provider.default_model().to_string();
                self.ai_state.temp_endpoint = self.ai_state.endpoint.clone();
                self.ai_state.temp_model = self.ai_state.model.clone();
                self.config.ai_provider = provider.to_str_id().to_string();
                self.config.ai_endpoint = self.ai_state.endpoint.clone();
                self.config.ai_model = self.ai_state.model.clone();
                let _ = self.config.save();
                self.notify(format!("Active Agent: {}", provider.label()));
                Task::none()
            }

            Message::AiApiKeyChanged(key) => {
                self.ai_state.api_key = key.clone();
                self.ai_state.temp_api_key = key.clone();
                self.config.ai_api_key = key;
                let _ = self.config.save();
                Task::none()
            }

            Message::AdjustSplitRatio(delta) => {
                let current = self.config.split_ratio;
                let new_ratio = (current + delta).clamp(0.15, 0.85);
                self.config.split_ratio = (new_ratio * 100.0).round() / 100.0;
                let _ = self.config.save();
                let pct = (self.config.split_ratio * 100.0).round() as u16;
                self.notify(format!("Split: {}:{}", pct, 100 - pct));
                Task::none()
            }

            Message::SetSplitRatio(val) => {
                self.config.split_ratio = val.clamp(0.15, 0.85);
                let _ = self.config.save();
                let pct = (self.config.split_ratio * 100.0).round() as u16;
                self.notify(format!("Split: {}:{}", pct, 100 - pct));
                Task::none()
            }

            Message::CycleSplitPresets => {
                let current = self.config.split_ratio;
                let next = if (current - 0.50).abs() < 0.04 {
                    0.60
                } else if (current - 0.60).abs() < 0.04 {
                    0.70
                } else if (current - 0.70).abs() < 0.04 {
                    0.30
                } else if (current - 0.30).abs() < 0.04 {
                    0.40
                } else {
                    0.50
                };
                self.config.split_ratio = next;
                let _ = self.config.save();
                let pct = (next * 100.0).round() as u16;
                self.notify(format!("Split: {}:{}", pct, 100 - pct));
                Task::none()
            }

            Message::AdjustDrawerWidth(delta) => {
                self.config.drawer_width = (self.config.drawer_width + delta).clamp(280.0, 600.0);
                let _ = self.config.save();
                Task::none()
            }

            Message::SetRightDrawerTab(tab) => {
                self.right_drawer_tab = tab;
                Task::none()
            }

            Message::RunAiAction(action) => {
                self.ai_state.is_loading = true;
                self.ai_state.last_action = Some(action);
                let provider = self.ai_state.provider;
                let endpoint = self.ai_state.endpoint.clone();
                let model = self.ai_state.model.clone();
                let api_key = self.ai_state.api_key.clone();
                let text = self.content.text();
                Task::perform(
                    async move {
                        ai::run_ai_task(provider, endpoint, model, api_key, text, action)
                    },
                    |(res, is_offline)| Message::AiCompleted(res, is_offline),
                )
            }

            Message::AiCompleted(res, is_offline) => {
                self.ai_state.is_loading = false;
                self.ai_state.response_text = res;
                self.ai_state.is_offline_fallback = is_offline;
                self.notify(if is_offline {
                    "Analyzed via Offline Evaluator"
                } else {
                    "Hermes AI Analysis Complete"
                });
                Task::none()
            }

            Message::AiCopyFeedback => {
                if !self.ai_state.response_text.is_empty() {
                    if let Ok(mut cb) = arboard::Clipboard::new() {
                        let _ = cb.set_text(&self.ai_state.response_text);
                        self.notify("Feedback copied to clipboard");
                    }
                }
                Task::none()
            }

            Message::AiInsertFeedback => {
                if !self.ai_state.response_text.is_empty() {
                    let formatted = format!("\n\n---\n{}\n---\n", self.ai_state.response_text);
                    insert_text_at_cursor(&mut self.content, &formatted);
                    self.stats = DocumentStats::compute(&self.content.text());
                    self.notify("Inserted feedback into editor");
                }
                Task::none()
            }

            Message::AiSetEndpoint(val) => {
                self.ai_state.temp_endpoint = val;
                Task::none()
            }

            Message::AiSetModel(val) => {
                self.ai_state.temp_model = val;
                Task::none()
            }

            Message::AiToggleSettings => {
                self.ai_state.show_settings = !self.ai_state.show_settings;
                if !self.ai_state.show_settings {
                    self.ai_state.endpoint = self.ai_state.temp_endpoint.clone();
                    self.ai_state.model = self.ai_state.temp_model.clone();
                    self.config.ai_endpoint = self.ai_state.endpoint.clone();
                    self.config.ai_model = self.ai_state.model.clone();
                    let _ = self.config.save();
                    self.notify("AI settings saved");
                }
                Task::none()
            }

            Message::ToggleThemeModal => {
                self.show_theme_modal = !self.show_theme_modal;
                Task::none()
            }

            Message::CloseThemeModal => {
                self.show_theme_modal = false;
                Task::none()
            }

            Message::SelectTheme(t) => {
                self.theme_mode = t;
                self.config.theme = t.to_str_name().to_string();
                let _ = self.config.save();
                self.notify(format!("Theme: {}", t.display_name()));
                Task::none()
            }

            Message::SelectAccent(a) => {
                self.accent_choice = a;
                self.config.accent_color = a.to_str_name().to_string();
                let _ = self.config.save();
                self.notify(format!("Accent: {}", a.display_name()));
                Task::none()
            }

            Message::ToggleTourModal => {
                self.show_tour_modal = !self.show_tour_modal;
                self.tour_step = 0;
                Task::none()
            }

            Message::CloseTourModal => {
                self.show_tour_modal = false;
                Task::none()
            }

            Message::TourNext => {
                if self.tour_step < 4 {
                    self.tour_step += 1;
                } else {
                    self.show_tour_modal = false;
                    self.config.has_seen_tour = true;
                    let _ = self.config.save();
                }
                Task::none()
            }

            Message::TourPrev => {
                if self.tour_step > 0 {
                    self.tour_step -= 1;
                }
                Task::none()
            }

            Message::TourFinish => {
                self.show_tour_modal = false;
                self.config.has_seen_tour = true;
                let _ = self.config.save();
                Task::none()
            }

            Message::WindowMinimize => {
                window::get_latest().and_then(|id| window::minimize(id, true))
            }

            Message::WindowToggleMaximize => {
                self.is_maximized = !self.is_maximized;
                window::get_latest().and_then(window::toggle_maximize)
            }

            Message::WindowClose => {
                window::get_latest().and_then(window::close)
            }

            Message::WindowDrag => {
                window::get_latest().and_then(window::drag)
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
                        if *c == '\t' {
                            text_editor::Action::Edit(text_editor::Edit::Paste(Arc::new("    ".to_string())))
                        } else {
                            let normalized_c = normalize_persian_char(*c);
                            text_editor::Action::Edit(text_editor::Edit::Insert(normalized_c))
                        }
                    }
                    text_editor::Action::Edit(text_editor::Edit::Paste(arc_str)) => {
                        let replaced = arc_str.replace('\t', "    ");
                        let normalized_str = normalize_persian_string(&replaced);
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
                    if let Ok(raw) = std::fs::read_to_string(&path) {
                        let content = raw.replace('\t', "    ");
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
                    if let Ok(raw) = std::fs::read_to_string(&path) {
                        let content = raw.replace('\t', "    ");
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
                    if let Ok(raw) = std::fs::read_to_string(path) {
                        let content = raw.replace('\t', "    ");
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
                self.config.app_mode = match self.app_mode {
                    AppMode::Writing => "writing".to_string(),
                    AppMode::Whiteboard => "whiteboard".to_string(),
                };
                let _ = self.config.save();
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

            SidebarAction::ToggleAiCoach => {
                self.config.show_right_drawer = !self.config.show_right_drawer;
                self.right_drawer_tab = RightDrawerTab::AiCoach;
                let _ = self.config.save();
                Task::none()
            }

            SidebarAction::ToggleRightDrawer => {
                self.config.show_right_drawer = !self.config.show_right_drawer;
                let _ = self.config.save();
                Task::none()
            }

            SidebarAction::ToggleShortcutsModal => {
                self.update(Message::ToggleTourModal)
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

            SidebarAction::WhiteboardToolPen => {
                self.whiteboard.active_tool = WhiteboardTool::Pen;
                Task::none()
            }

            SidebarAction::WhiteboardToolHighlighter => {
                self.whiteboard.active_tool = WhiteboardTool::Highlighter;
                Task::none()
            }

            SidebarAction::WhiteboardToolEraser => {
                self.whiteboard.active_tool = WhiteboardTool::Eraser;
                Task::none()
            }

            SidebarAction::WhiteboardUndo => {
                self.whiteboard.undo();
                Task::none()
            }

            SidebarAction::WhiteboardRedo => {
                self.whiteboard.redo();
                Task::none()
            }

            SidebarAction::WhiteboardWipe => {
                self.whiteboard.clear_board();
                self.notify("Whiteboard cleared");
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
                Key::Character(ref s) if s.eq_ignore_ascii_case("h") => {
                    return self.update(Message::ToggleTopToolbar);
                }
                Key::Character(ref s) if s.eq_ignore_ascii_case("j") => {
                    return self.update(Message::ToggleAiDrawer);
                }
                Key::Character(ref s) if s.eq_ignore_ascii_case("f") => {
                    return self.update(Message::ToggleSearch);
                }
                Key::Character(ref s) if s.eq_ignore_ascii_case("t") => {
                    return self.update(Message::ToggleThemeModal);
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
                Key::Character(ref s) if s == "[" => {
                    return self.update(Message::AdjustSplitRatio(-0.05));
                }
                Key::Character(ref s) if s == "]" => {
                    return self.update(Message::AdjustSplitRatio(0.05));
                }
                Key::Character(ref s) if s == "2" && modifiers.shift() => {
                    insert_zwnj_at_cursor(&mut self.content);
                    self.notify("Inserted ZWNJ");
                    return Task::none();
                }
                _ => {}
            }
        }

        if modifiers.shift() && matches!(key, Key::Named(keyboard::key::Named::Space)) {
            insert_zwnj_at_cursor(&mut self.content);
            self.notify("Inserted ZWNJ");
            return Task::none();
        }

        if matches!(key, Key::Named(keyboard::key::Named::F9)) {
            return self.update(Message::ToggleTopToolbar);
        }

        if matches!(key, Key::Named(keyboard::key::Named::F11)) {
            return self.update(Message::WindowToggleMaximize);
        }

        if matches!(key, Key::Named(keyboard::key::Named::F1)) {
            return self.update(Message::ToggleTourModal);
        }

        if matches!(key, Key::Named(keyboard::key::Named::Escape)) {
            if self.show_tour_modal {
                self.show_tour_modal = false;
                return Task::none();
            }
            if self.show_theme_modal {
                self.show_theme_modal = false;
                return Task::none();
            }
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
                    self.config.show_right_drawer && self.right_drawer_tab == RightDrawerTab::AiCoach,
                )
                .map(Message::Sidebar),
            )
        } else {
            None
        };

        // Right Exam & Hermes AI Assistant Drawer
        let maybe_right_drawer = if self.config.show_right_drawer && self.app_mode == AppMode::Writing {
            Some(self.view_right_drawer())
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
        if self.show_top_toolbar {
            root_column = root_column.push(self.view_top_header());
        }
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

        if self.show_theme_modal {
            root_stack = root_stack.push(
                container(self.view_theme_modal())
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .center(Length::Fill)
                    .style(|_t| container::Style {
                        background: Some(iced::Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.65))),
                        ..Default::default()
                    }),
            );
        }

        if self.show_tour_modal {
            root_stack = root_stack.push(
                container(self.view_tour_modal())
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .center(Length::Fill)
                    .style(|_t| container::Style {
                        background: Some(iced::Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.70))),
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
            .wrapping(iced::widget::text::Wrapping::Word)
            .height(Length::Fill)
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
                let ratio = self.config.split_ratio.clamp(0.15, 0.85);
                let editor_portion = (ratio * 100.0).round().clamp(15.0, 85.0) as u16;
                let preview_portion = 100 - editor_portion;

                let splitter_bar = container(
                    column![
                        with_tooltip(
                            button(center(text("◀").size(10).color(TEXT_MUTED)))
                                .on_press(Message::AdjustSplitRatio(-0.05))
                                .style(rail_button_style)
                                .width(20)
                                .height(32),
                            "Shrink Editor Pane (Ctrl+[)",
                        ),
                        with_tooltip(
                            button(center(text(format!("{}:{}", editor_portion, preview_portion)).size(9).color(ACCENT_CYAN)))
                                .on_press(Message::CycleSplitPresets)
                                .style(rail_button_style)
                                .width(42)
                                .height(24),
                            "Cycle Split Presets",
                        ),
                        with_tooltip(
                            button(center(text("▶").size(10).color(TEXT_MUTED)))
                                .on_press(Message::AdjustSplitRatio(0.05))
                                .style(rail_button_style)
                                .width(20)
                                .height(32),
                            "Grow Editor Pane (Ctrl+]",
                        ),
                    ]
                    .spacing(4)
                    .align_x(Alignment::Center),
                )
                .center_y(Length::Fill)
                .width(46)
                .height(Length::Fill);

                row![
                    container(editor_widget)
                        .width(Length::FillPortion(editor_portion))
                        .height(Length::Fill),
                    splitter_bar,
                    container(rendered_markdown)
                        .width(Length::FillPortion(preview_portion))
                        .height(Length::Fill)
                        .padding(iced::Padding {
                            top: 0.0,
                            bottom: 0.0,
                            left: 12.0,
                            right: 8.0,
                        }),
                ]
                .spacing(4)
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

        let mut studio_col = column![];
        if let Some(search_widget) = maybe_search {
            studio_col = studio_col.push(search_widget);
        }
        if let Some(banner) = maybe_banner {
            studio_col = studio_col.push(banner);
        }
        studio_col = studio_col.push(
            container(center_column)
                .center_x(Length::Fill)
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

        let mut content_col = column![];
        if self.show_top_toolbar {
            content_col = content_col.push(toolbar);
        }
        content_col = content_col.push(canvas_widget);
        let content_col = content_col.width(Length::Fill).height(Length::Fill);

        container(content_col)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(canvas_container_style)
            .into()
    }

    fn view_right_drawer<'a>(&'a self) -> Element<'a, Message> {
        let accent = self.accent_choice.color();

        let is_ai = self.right_drawer_tab == RightDrawerTab::AiCoach;
        let ai_tab_style = if is_ai { tab_active_button_style } else { tab_inactive_button_style };
        let ai_title = format!("✨ {}", self.ai_state.provider.label());
        let btn_ai_tab = button(
            row![
                render_svg_icon(SVG_ROBOT, 14, if is_ai { accent } else { TEXT_MUTED }),
                text(ai_title).size(12),
            ]
            .spacing(5)
            .align_y(Alignment::Center),
        )
        .on_press(Message::SetRightDrawerTab(RightDrawerTab::AiCoach))
        .style(ai_tab_style)
        .padding(iced::Padding { top: 4.0, bottom: 4.0, left: 10.0, right: 10.0 });

        let is_de = self.right_drawer_tab == RightDrawerTab::Redemittel;
        let de_tab_style = if is_de { tab_active_button_style } else { tab_inactive_button_style };
        let btn_de_tab = button(
            row![
                render_svg_icon(SVG_TIMER, 14, if is_de { accent } else { TEXT_MUTED }),
                text("Exam & Goals").size(12),
            ]
            .spacing(5)
            .align_y(Alignment::Center),
        )
        .on_press(Message::SetRightDrawerTab(RightDrawerTab::Redemittel))
        .style(de_tab_style)
        .padding(iced::Padding { top: 4.0, bottom: 4.0, left: 10.0, right: 10.0 });

        let btn_shrink = button(center(text("◀").size(10).color(TEXT_MUTED)))
            .on_press(Message::AdjustDrawerWidth(-30.0))
            .style(rail_button_style)
            .width(22)
            .height(24);

        let btn_expand = button(center(text("▶").size(10).color(TEXT_MUTED)))
            .on_press(Message::AdjustDrawerWidth(30.0))
            .style(rail_button_style)
            .width(22)
            .height(24);

        let btn_close = button(center(render_svg_icon(SVG_CLOSE, 12, TEXT_MUTED)))
            .on_press(Message::Sidebar(SidebarAction::ToggleRightDrawer))
            .style(rail_button_style)
            .width(26)
            .height(26);

        let drawer_header = row![
            btn_ai_tab,
            btn_de_tab,
            Space::with_width(Length::Fill),
            btn_shrink,
            btn_expand,
            btn_close,
        ]
        .spacing(4)
        .align_y(Alignment::Center);

        let drawer_body: Element<'a, Message> = match self.right_drawer_tab {
            RightDrawerTab::AiCoach => self.view_ai_drawer_content(accent),
            RightDrawerTab::Redemittel => self.view_redemittel_drawer_content(),
        };

        let drawer_col = column![
            drawer_header,
            container(Space::with_height(1))
                .style(|_t| container::Style {
                    background: Some(iced::Background::Color(BORDER_COLOR)),
                    ..Default::default()
                })
                .width(Length::Fill)
                .height(1),
            drawer_body,
        ]
        .spacing(8)
        .padding(10);

        let drawer_width = self.config.drawer_width.clamp(280.0, 600.0);
        container(drawer_col)
            .width(drawer_width)
            .height(Length::Fill)
            .style(drawer_container_style)
            .into()
    }

    fn view_ai_drawer_content<'a>(&'a self, accent: Color) -> Element<'a, Message> {
        let provider_chip = |prov: AiProvider, name: &'static str| {
            let is_sel = self.ai_state.provider == prov;
            button(text(name).size(10.0).color(if is_sel { accent } else { TEXT_PRIMARY }))
                .on_press(Message::SelectAiProvider(prov))
                .style(if is_sel { tab_active_button_style } else { rail_button_style })
                .padding(iced::Padding { top: 3.5, bottom: 3.5, left: 7.0, right: 7.0 })
        };

        let providers_row = row![
            provider_chip(AiProvider::Hermes, "🦙 Hermes 3"),
            provider_chip(AiProvider::Claude, "🧠 Claude 3.5"),
            provider_chip(AiProvider::ChatGPT, "🟢 GPT-4o"),
            provider_chip(AiProvider::DeepSeek, "⚡ DeepSeek"),
            provider_chip(AiProvider::Custom, "⚙️ Custom"),
        ]
        .spacing(4)
        .align_y(Alignment::Center);

        let provider_header = column![
            text("CHOOSE AI AGENT MODEL").size(9.0).color(TEXT_MUTED),
            providers_row,
        ]
        .spacing(4);

        let maybe_api_key = if self.ai_state.provider.requires_api_key() {
            let key_placeholder = match self.ai_state.provider {
                AiProvider::Claude => "Enter Anthropic API key (sk-ant-...)",
                AiProvider::ChatGPT => "Enter OpenAI API key (sk-...)",
                AiProvider::DeepSeek => "Enter DeepSeek API key (sk-...)",
                _ => "Enter API Key / Bearer Token",
            };
            Some(
                container(
                    column![
                        row![
                            render_svg_icon(SVG_KEY, 12, ACCENT_AMBER),
                            text("API KEY").size(9.0).color(ACCENT_AMBER),
                            Space::with_width(Length::Fill),
                            text(if self.ai_state.api_key.trim().is_empty() { "⚠️ Key Required" } else { "✓ Configured" })
                                .size(9.0)
                                .color(if self.ai_state.api_key.trim().is_empty() { ACCENT_AMBER } else { ACCENT_GREEN }),
                        ]
                        .spacing(5)
                        .align_y(Alignment::Center),
                        text_input(key_placeholder, &self.ai_state.api_key)
                            .on_input(Message::AiApiKeyChanged)
                            .secure(true)
                            .size(11)
                            .style(text_input_style),
                    ]
                    .spacing(4),
                )
                .padding(6)
                .width(Length::Fill)
                .style(card_container_style)
            )
        } else {
            None
        };

        let action_btn = |action: AiAction, label: &'static str, svg_icon: &'static [u8]| {
            let is_active = self.ai_state.last_action == Some(action) && !self.ai_state.is_loading;
            button(
                row![
                    render_svg_icon(svg_icon, 13, if is_active { accent } else { TEXT_MUTED }),
                    text(label).size(11).color(if is_active { accent } else { TEXT_PRIMARY }),
                ]
                .spacing(5)
                .align_y(Alignment::Center),
            )
            .on_press(Message::RunAiAction(action))
            .style(if is_active { tab_active_button_style } else { rail_button_style })
            .padding(iced::Padding { top: 6.0, bottom: 6.0, left: 10.0, right: 10.0 })
            .width(Length::Fill)
        };

        let row1 = row![
            action_btn(AiAction::GradeEssay, "Grade Essay", SVG_CHECK),
            action_btn(AiAction::TipsAndFeedback, "Writing Tips", SVG_AI),
        ]
        .spacing(6);

        let row2 = row![
            action_btn(AiAction::FixGrammar, "Fix Grammar", SVG_WRITE),
            action_btn(AiAction::GermanB2C1, "German B2/C1", SVG_LIST),
        ]
        .spacing(6);

        let actions_section = column![row1, row2].spacing(6);

        let result_section: Element<'a, Message> = if self.ai_state.is_loading {
            let agent_name = self.ai_state.provider.label();
            container(
                column![
                    text(format!("✨ {} is evaluating your text...", agent_name)).size(12).color(accent),
                    text("Analyzing structure, CEFR rubric, and argumentation...").size(10).color(TEXT_MUTED),
                ]
                .spacing(6)
            )
            .padding(14)
            .width(Length::Fill)
            .style(card_container_style)
            .into()
        } else if !self.ai_state.response_text.is_empty() {
            let offline_note: Option<Element<'a, Message>> = if self.ai_state.is_offline_fallback {
                Some(
                    container(
                        text(format!("⚡ Offline Evaluator • Model: {}", self.ai_state.provider.label()))
                            .size(10)
                            .color(ACCENT_AMBER)
                    )
                    .padding(5)
                    .width(Length::Fill)
                    .style(card_container_style)
                    .into()
                )
            } else {
                None
            };

            let scroll_box = scrollable(
                container(text(&self.ai_state.response_text).size(11.5).color(TEXT_PRIMARY))
                    .padding(8)
                    .width(Length::Fill)
            )
            .height(Length::Fill);

            let btn_copy = button(
                row![
                    render_svg_icon(SVG_CHECK, 11, TEXT_MUTED),
                    text("Copy Feedback").size(10.5),
                ]
                .spacing(4)
                .align_y(Alignment::Center),
            )
            .on_press(Message::AiCopyFeedback)
            .style(rail_button_style)
            .padding(iced::Padding { top: 4.0, bottom: 4.0, left: 8.0, right: 8.0 });

            let btn_insert = button(
                row![
                    render_svg_icon(SVG_WRITE, 11, TEXT_MUTED),
                    text("Insert at Cursor").size(10.5),
                ]
                .spacing(4)
                .align_y(Alignment::Center),
            )
            .on_press(Message::AiInsertFeedback)
            .style(rail_button_style)
            .padding(iced::Padding { top: 4.0, bottom: 4.0, left: 8.0, right: 8.0 });

            let action_bar = row![btn_copy, btn_insert].spacing(6);

            let mut out_col = column![];
            if let Some(note) = offline_note {
                out_col = out_col.push(note);
            }
            out_col = out_col.push(container(scroll_box).height(Length::Fill).style(card_container_style));
            out_col = out_col.push(action_bar);

            out_col.spacing(6).height(Length::Fill).into()
        } else {
            let agent_label = self.ai_state.provider.label();
            let privacy_text = if self.ai_state.provider == AiProvider::Hermes {
                "🔒 100% Private & Local\nRuns directly on your hardware via local Hermes/Ollama or the built-in offline engine."
            } else {
                "☁️ Cloud Model Active\nDirect API connection with your private key, or instant offline fallback if disconnected."
            };
            container(
                column![
                    text(format!("✨ {} Writing Studio", agent_label)).size(13).color(accent),
                    text("Grade essays against CEFR rubrics, diagnose grammatical cadence, and receive Goethe/Telc B2/C1 suggestions.").size(11).color(Color::from_rgb(0.80, 0.84, 0.90)),
                    container(
                        text(privacy_text).size(10).color(ACCENT_AMBER)
                    )
                    .padding(6)
                    .width(Length::Fill)
                    .style(card_container_style),
                ]
                .spacing(8)
            )
            .padding(12)
            .width(Length::Fill)
            .style(card_container_style)
            .into()
        };

        let settings_section: Element<'a, Message> = if self.ai_state.show_settings {
            column![
                text("AI ENDPOINT").size(9).color(TEXT_MUTED),
                text_input("http://localhost:11434/v1", &self.ai_state.temp_endpoint)
                    .on_input(Message::AiSetEndpoint)
                    .size(11)
                    .style(text_input_style),
                text("MODEL NAME").size(9).color(TEXT_MUTED),
                text_input("hermes3", &self.ai_state.temp_model)
                    .on_input(Message::AiSetModel)
                    .size(11)
                    .style(text_input_style),
                button(text("Save Settings").size(10))
                    .on_press(Message::AiToggleSettings)
                    .style(tab_active_button_style)
                    .padding(4),
            ]
            .spacing(4)
            .into()
        } else {
            row![
                text(format!("Model: {} • {}", self.ai_state.model, if self.ai_state.is_offline_fallback { "Offline" } else { "Online/Local" })).size(9.5).color(TEXT_MUTED),
                Space::with_width(Length::Fill),
                button(text("⚙️ Config").size(9.5))
                    .on_press(Message::AiToggleSettings)
                    .style(rail_button_style)
                    .padding(2),
            ]
            .align_y(Alignment::Center)
            .into()
        };

        let mut full_col = column![provider_header];
        if let Some(key_box) = maybe_api_key {
            full_col = full_col.push(key_box);
        }
        full_col = full_col
            .push(actions_section)
            .push(result_section)
            .push(settings_section)
            .spacing(8)
            .height(Length::Fill);

        full_col.into()
    }

    fn view_redemittel_drawer_content<'a>(&'a self) -> Element<'a, Message> {
        let is_in_range = self.stats.word_count >= self.config.target_min_words && self.stats.word_count <= self.config.target_max_words;
        let badge_color = if self.stats.word_count == 0 {
            TEXT_MUTED
        } else if is_in_range {
            ACCENT_GREEN
        } else {
            ACCENT_AMBER
        };

        let target_row = row![
            text(format!("Target: {} - {} words", self.config.target_min_words, self.config.target_max_words))
                .size(11)
                .color(TEXT_MUTED),
            Space::with_width(Length::Fill),
            text(format!("{} words", self.stats.word_count))
                .size(12)
                .color(badge_color),
        ]
        .align_y(Alignment::Center);

        let target_card = container(target_row)
            .padding(8)
            .width(Length::Fill)
            .style(card_container_style);

        let mut snippets_col = column![
            text("B2/C1 CONNECTORS (CLICK TO INSERT)")
                .size(9.5)
                .color(TEXT_MUTED),
        ]
        .spacing(6);

        for cat in &self.snippets.categories {
            let mut cat_col = column![
                text(&cat.name).size(10.5).color(ACCENT_AMBER),
            ]
            .spacing(3);

            for item in &cat.items {
                let snippet_text = item.text.clone();
                let chip = button(
                    text(&item.label)
                        .size(11)
                        .color(TEXT_PRIMARY),
                )
                .on_press(Message::Sidebar(SidebarAction::InsertSnippet(snippet_text)))
                .style(rail_button_style)
                .padding(iced::Padding { top: 4.0, bottom: 4.0, left: 8.0, right: 8.0 })
                .width(Length::Fill);

                cat_col = cat_col.push(chip);
            }

            snippets_col = snippets_col.push(cat_col);
        }

        let scrollable_snippets = scrollable(snippets_col).height(Length::Fill);

        column![
            target_card,
            container(Space::with_height(1))
                .style(|_t| container::Style {
                    background: Some(iced::Background::Color(BORDER_COLOR)),
                    ..Default::default()
                })
                .width(Length::Fill)
                .height(1),
            scrollable_snippets,
        ]
        .spacing(8)
        .height(Length::Fill)
        .into()
    }

    fn view_top_header<'a>(&'a self) -> Element<'a, Message> {
        let accent = self.accent_choice.color();

        // 1. Mode Switcher Chips (Writing Studio vs Whiteboard) - Bigger buttons
        let is_writing = self.app_mode == AppMode::Writing;
        let write_tab_style = if is_writing { tab_active_button_style } else { tab_inactive_button_style };
        let btn_write = button(
            center(
                row![
                    render_svg_icon(SVG_WRITE, 15, if is_writing { accent } else { TEXT_MUTED }),
                    text("Write").size(12.5),
                ]
                .spacing(6)
                .align_y(Alignment::Center),
            ),
        )
        .on_press(Message::SetAppMode(AppMode::Writing))
        .height(32)
        .style(write_tab_style)
        .padding(iced::Padding { left: 12.0, right: 12.0, ..Default::default() });

        let is_whiteboard = self.app_mode == AppMode::Whiteboard;
        let board_tab_style = if is_whiteboard { tab_active_button_style } else { tab_inactive_button_style };
        let btn_board = button(
            center(
                row![
                    render_svg_icon(SVG_WHITEBOARD, 15, if is_whiteboard { accent } else { TEXT_MUTED }),
                    text("Whiteboard").size(12.5),
                ]
                .spacing(6)
                .align_y(Alignment::Center),
            ),
        )
        .on_press(Message::SetAppMode(AppMode::Whiteboard))
        .height(32)
        .style(board_tab_style)
        .padding(iced::Padding { left: 12.0, right: 12.0, ..Default::default() });

        // 2. View Mode Segment: Edit | Split | Preview
        let btn_mode_edit = button(
            center(
                row![
                    render_svg_icon(SVG_WRITE, 13, if self.view_mode == ViewMode::Edit { accent } else { TEXT_MUTED }),
                    text("Edit").size(12),
                ]
                .spacing(5)
                .align_y(Alignment::Center),
            ),
        )
        .on_press(Message::SetViewMode(ViewMode::Edit))
        .height(32)
        .style(if self.view_mode == ViewMode::Edit { tab_active_button_style } else { tab_inactive_button_style })
        .padding(iced::Padding { left: 10.0, right: 10.0, ..Default::default() });

        let btn_mode_split = button(
            center(
                row![
                    render_svg_icon(SVG_SPLIT, 13, if self.view_mode == ViewMode::Split { accent } else { TEXT_MUTED }),
                    text("Live Split").size(12),
                ]
                .spacing(5)
                .align_y(Alignment::Center),
            ),
        )
        .on_press(Message::SetViewMode(ViewMode::Split))
        .height(32)
        .style(if self.view_mode == ViewMode::Split { tab_active_button_style } else { tab_inactive_button_style })
        .padding(iced::Padding { left: 10.0, right: 10.0, ..Default::default() });

        let btn_mode_preview = button(
            center(
                row![
                    render_svg_icon(SVG_PREVIEW, 13, if self.view_mode == ViewMode::Preview { accent } else { TEXT_MUTED }),
                    text("Preview").size(12),
                ]
                .spacing(5)
                .align_y(Alignment::Center),
            ),
        )
        .on_press(Message::SetViewMode(ViewMode::Preview))
        .height(32)
        .style(if self.view_mode == ViewMode::Preview { tab_active_button_style } else { tab_inactive_button_style })
        .padding(iced::Padding { left: 10.0, right: 10.0, ..Default::default() });

        let left_cluster = if self.app_mode == AppMode::Writing {
            row![
                with_tooltip(btn_write, "Writing Studio - Markdown editor & reader"),
                with_tooltip(btn_board, "Whiteboard (Ctrl+M)"),
                Space::with_width(8),
                with_tooltip(btn_mode_edit, "Edit - Markdown source (Ctrl+1)"),
                with_tooltip(btn_mode_split, "Split View - editor + live preview (Ctrl+2)"),
                with_tooltip(btn_mode_preview, "Reader - rendered preview (Ctrl+3)"),
            ]
        } else {
            row![
                with_tooltip(btn_write, "Writing Studio - Markdown editor & reader"),
                with_tooltip(btn_board, "Whiteboard (Ctrl+M)"),
            ]
        }
        .spacing(4)
        .align_y(Alignment::Center);

        // 3. Central Draggable Title Area with Blue Crab Logo
        let title_text = match self.app_mode {
            AppMode::Writing => "Krebs Studio",
            AppMode::Whiteboard => "Krebs Studio — Whiteboard",
        };
        let drag_center = mouse_area(
            container(
                row![
                    render_svg_icon(SVG_TOUR, 14, accent),
                    text(title_text).size(12).color(Color::from_rgba(1.0, 1.0, 1.0, 0.45)),
                ]
                .spacing(6)
                .align_y(Alignment::Center)
            )
            .center_x(Length::Fill)
            .width(Length::Fill)
        )
        .on_press(Message::WindowDrag);

        // 4. Right Utilities & Tools (Hermes AI, Width, Font, Clock, Zen Hide, Window controls)
        let is_ai_open = self.config.show_right_drawer && self.right_drawer_tab == RightDrawerTab::AiCoach;
        let agent_label = format!("✨ Agent: {} ▼", self.ai_state.provider.label());
        let btn_ai = button(
            center(
                row![
                    render_svg_icon(SVG_ROBOT, 14, if is_ai_open { accent } else { ACCENT_CYAN }),
                    text(agent_label).size(12).color(if is_ai_open { accent } else { TEXT_PRIMARY }),
                ]
                .spacing(5)
                .align_y(Alignment::Center),
            ),
        )
        .on_press(Message::ToggleAiDrawer)
        .height(32)
        .style(if is_ai_open { tab_active_button_style } else { rail_button_style })
        .padding(iced::Padding { left: 10.0, right: 10.0, ..Default::default() });

        let maybe_split_pill = if self.app_mode == AppMode::Writing && self.view_mode == ViewMode::Split {
            let pct = (self.config.split_ratio * 100.0).round() as u16;
            Some(
                button(
                    center(
                        row![
                            render_svg_icon(SVG_SPLIT, 12, ACCENT_CYAN),
                            text(format!("Split {}:{}", pct, 100 - pct)).size(11.5).color(TEXT_PRIMARY),
                        ]
                        .spacing(4)
                        .align_y(Alignment::Center),
                    ),
                )
                .on_press(Message::CycleSplitPresets)
                .height(32)
                .style(rail_button_style)
                .padding(iced::Padding { left: 9.0, right: 9.0, ..Default::default() })
            )
        } else {
            None
        };

        let btn_width = button(
            center(
                row![
                    render_svg_icon(SVG_WIDTH, 13, TEXT_MUTED),
                    text(self.canvas_width_mode.label()).size(12),
                ]
                .spacing(5)
                .align_y(Alignment::Center),
            ),
        )
        .on_press(Message::CycleWidthMode)
        .height(32)
        .style(rail_button_style)
        .padding(iced::Padding { left: 10.0, right: 10.0, ..Default::default() });

        let btn_font = button(
            center(
                row![
                    render_svg_icon(SVG_FONT, 13, TEXT_MUTED),
                    text(self.font_choice.display_name()).size(12),
                ]
                .spacing(5)
                .align_y(Alignment::Center),
            ),
        )
        .on_press(Message::ToggleFontModal)
        .height(32)
        .style(rail_button_style)
        .padding(iced::Padding { left: 10.0, right: 10.0, ..Default::default() });

        let mins = self.timer_remaining_secs / 60;
        let secs = self.timer_remaining_secs % 60;
        let clock_str = format!("{:02}:{:02}", mins, secs);
        let clock_color = if self.timer_is_running { accent } else { TEXT_MUTED };

        let btn_clock = button(
            center(
                row![
                    render_svg_icon(SVG_TIMER, 13, clock_color),
                    text(clock_str).size(12).color(clock_color),
                ]
                .spacing(5)
                .align_y(Alignment::Center),
            ),
        )
        .on_press(Message::ZenTimerToggle)
        .height(32)
        .style(rail_button_style)
        .padding(iced::Padding { left: 10.0, right: 10.0, ..Default::default() });

        let btn_clock_presets = button(center(render_svg_icon(SVG_CHEVRON_DOWN, 11, TEXT_MUTED)))
            .on_press(Message::ZenTimerTogglePresets)
            .width(32)
            .height(32)
            .style(rail_button_style);

        let btn_theme = button(center(render_svg_icon(SVG_THEME, 15, TEXT_MUTED)))
            .on_press(Message::ToggleThemeModal)
            .width(32)
            .height(32)
            .style(rail_button_style);

        let btn_tour = button(center(render_svg_icon(SVG_TOUR, 15, TEXT_MUTED)))
            .on_press(Message::ToggleTourModal)
            .width(32)
            .height(32)
            .style(rail_button_style);

        let btn_zen_hide = button(center(render_svg_icon(SVG_EYE, 15, TEXT_MUTED)))
            .on_press(Message::ToggleTopToolbar)
            .width(32)
            .height(32)
            .style(rail_button_style);

        let btn_min = button(center(render_svg_icon(SVG_MINIMIZE, 13, TEXT_MUTED)))
            .on_press(Message::WindowMinimize)
            .style(rail_button_style)
            .width(32)
            .height(32);

        let btn_max = button(center(render_svg_icon(SVG_FULLSCREEN, 13, TEXT_MUTED)))
            .on_press(Message::WindowToggleMaximize)
            .style(rail_button_style)
            .width(32)
            .height(32);

        let btn_close = button(center(render_svg_icon(SVG_CLOSE, 13, TEXT_MUTED)))
            .on_press(Message::WindowClose)
            .style(rail_button_style)
            .width(32)
            .height(32);

        let mut right_cluster = row![with_tooltip(btn_ai, "AI Agent Drawer - Hermes, Claude, ChatGPT, DeepSeek (Ctrl+J)")];
        if let Some(split_pill) = maybe_split_pill {
            right_cluster = right_cluster.push(with_tooltip(split_pill, "Cycle Split Presets - editor:previewer ratio"));
        }
        right_cluster = right_cluster
            .push(with_tooltip(btn_width, "Cycle Text Width - narrow / normal / wide (Ctrl+W)"))
            .push(with_tooltip(btn_font, "Typeface Selection"))
            .push(with_tooltip(btn_clock, "Focus Timer - start / pause"))
            .push(with_tooltip(btn_clock_presets, "Timer Presets - 15 / 25 / 45 / 60 min"))
            .push(with_tooltip(btn_theme, "Theme Selector (Ctrl+T)"))
            .push(with_tooltip(btn_tour, "Onboarding Tour (F1)"))
            .push(with_tooltip(btn_zen_hide, "Zen Mode - hide toolbar (Ctrl+H)"))
            .push(Space::with_width(6))
            .push(with_tooltip(btn_min, "Minimize"))
            .push(with_tooltip(btn_max, "Maximize / Restore (F11)"))
            .push(with_tooltip(btn_close, "Close Krebs"))
            .spacing(4)
            .align_y(Alignment::Center);

        if self.show_timer_presets {
            let p15 = button(center(text("15m").size(10.5))).on_press(Message::ZenTimerSet(900)).height(32).style(rail_button_style).padding(iced::Padding::new(6.0));
            let p25 = button(center(text("25m").size(10.5))).on_press(Message::ZenTimerSet(1500)).height(32).style(rail_button_style).padding(iced::Padding::new(6.0));
            let p45 = button(center(text("45m").size(10.5))).on_press(Message::ZenTimerSet(2700)).height(32).style(rail_button_style).padding(iced::Padding::new(6.0));
            let p60 = button(center(text("60m").size(10.5))).on_press(Message::ZenTimerSet(3600)).height(32).style(rail_button_style).padding(iced::Padding::new(6.0));
            let rst = button(center(render_svg_icon(SVG_RESET, 12, TEXT_MUTED))).on_press(Message::Sidebar(SidebarAction::TimerReset)).width(32).height(32).style(rail_button_style);

            right_cluster = right_cluster.push(
                row![
                    with_tooltip(p15, "Focus for 15 minutes"),
                    with_tooltip(p25, "Focus for 25 minutes"),
                    with_tooltip(p45, "Focus for 45 minutes"),
                    with_tooltip(p60, "Focus for 60 minutes"),
                    with_tooltip(rst, "Reset Focus Timer"),
                ]
                .spacing(2)
                .align_y(Alignment::Center),
            );
        }

        let header_row = row![
            left_cluster,
            drag_center,
            right_cluster,
        ]
        .width(Length::Fill)
        .align_y(Alignment::Center);

        container(header_row)
            .width(Length::Fill)
            .padding(iced::Padding {
                top: 3.5,
                bottom: 3.5,
                left: 10.0,
                right: 10.0,
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

        let btn_prev = button(center(text("↑").size(11)))
            .on_press(Message::SearchPrev)
            .style(rail_button_style)
            .width(32)
            .height(32);

        let btn_next = button(center(text("↓").size(11)))
            .on_press(Message::SearchNext)
            .style(rail_button_style)
            .width(32)
            .height(32);

        let btn_close = button(center(render_svg_icon(SVG_CLOSE, 11, TEXT_MUTED)))
            .on_press(Message::CloseSearch)
            .style(rail_button_style)
            .width(32)
            .height(32);

        let search_row = row![
            render_svg_icon(SVG_SEARCH, 13, ACCENT_CYAN),
            input,
            text(match_label).size(11).color(ACCENT_CYAN),
            with_tooltip(btn_prev, "Previous Match (Enter / Shift+F3)"),
            with_tooltip(btn_next, "Next Match (Enter)"),
            with_tooltip(btn_close, "Close Search (Ctrl+F)"),
        ]
        .spacing(8)
        .align_y(Alignment::Center);

        container(search_row)
            .padding(iced::Padding { top: 6.0, bottom: 6.0, left: 12.0, right: 12.0 })
            .style(search_container_style)
            .into()
    }

    fn view_whiteboard_toolbar<'a>(&'a self) -> Element<'a, Message> {
        let is_pen = self.whiteboard.active_tool == WhiteboardTool::Pen;
        let is_hl = self.whiteboard.active_tool == WhiteboardTool::Highlighter;
        let is_er = self.whiteboard.active_tool == WhiteboardTool::Eraser;

        let pen_color = if is_pen { ACCENT_CYAN } else { TEXT_MUTED };
        let btn_pen = button(
            row![
                render_svg_icon(SVG_PEN, 13, pen_color),
                text("Pen").size(11),
            ]
            .spacing(4)
            .align_y(Alignment::Center),
        )
        .on_press(Message::WhiteboardToolSelect(WhiteboardTool::Pen))
        .style(if is_pen { active_tool_button_style } else { rail_button_style })
        .padding(iced::Padding { top: 3.0, bottom: 3.0, left: 8.0, right: 8.0 });

        let hl_color = if is_hl { ACCENT_CYAN } else { TEXT_MUTED };
        let btn_hl = button(
            row![
                render_svg_icon(SVG_HIGHLIGHTER, 13, hl_color),
                text("Highlighter").size(11),
            ]
            .spacing(4)
            .align_y(Alignment::Center),
        )
        .on_press(Message::WhiteboardToolSelect(WhiteboardTool::Highlighter))
        .style(if is_hl { active_tool_button_style } else { rail_button_style })
        .padding(iced::Padding { top: 3.0, bottom: 3.0, left: 8.0, right: 8.0 });

        let er_color = if is_er { ACCENT_CYAN } else { TEXT_MUTED };
        let btn_eraser = button(
            row![
                render_svg_icon(SVG_ERASER, 13, er_color),
                text("Eraser").size(11),
            ]
            .spacing(4)
            .align_y(Alignment::Center),
        )
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

        let btn_undo = button(center(render_svg_icon(SVG_UNDO, 12, TEXT_MUTED)))
            .on_press(Message::WhiteboardUndo)
            .style(rail_button_style)
            .padding(iced::Padding::new(4.0));

        let btn_redo = button(center(render_svg_icon(SVG_REDO, 12, TEXT_MUTED)))
            .on_press(Message::WhiteboardRedo)
            .style(rail_button_style)
            .padding(iced::Padding::new(4.0));

        // Student Wipe / Delete Work Action
        let btn_wipe = button(
            row![
                render_svg_icon(SVG_WIPE, 12, ACCENT_RED),
                text("Wipe Board").size(11),
            ]
            .spacing(5)
            .align_y(Alignment::Center),
        )
        .on_press(Message::WhiteboardClear)
        .style(wipe_button_style)
        .padding(iced::Padding { top: 3.0, bottom: 3.0, left: 8.0, right: 8.0 });

        let tb_row = row![
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
            text(file_label).size(10).color(TEXT_PRIMARY),
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
            button(center(render_svg_icon(SVG_CLOSE, 12, TEXT_MUTED)))
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
            button(center(render_svg_icon(SVG_CLOSE, 12, TEXT_MUTED)))
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

    fn view_tour_modal<'a>(&self) -> Element<'a, Message> {
        let title_row = row![
            text("🦀 Welcome to Krebs Writing & Whiteboard Studio")
                .size(15)
                .color(ACCENT_CYAN),
            Space::with_width(Length::Fill),
            button(center(render_svg_icon(SVG_CLOSE, 12, TEXT_MUTED)))
                .on_press(Message::CloseTourModal)
                .style(rail_button_style)
                .width(28)
                .height(28),
        ]
        .align_y(Alignment::Center);

        let (step_title, step_desc, step_tag) = match self.tour_step {
            0 => (
                "1. Trilingual & Distraction-Free Writing",
                "Pure text focus with HarfBuzz/Swash text shaping. Write in English, German (with dedicated äöüß chords or Alt shortcuts), or Persian/Farsi with full RTL bidirectional support and Shift+Space ZWNJ (نیم‌فاصله). Each paragraph automatically detects its direction.",
                "Shortcuts: Alt+a/o/u/s for German  •  Shift+Space for ZWNJ",
            ),
            1 => (
                "2. Markdown View & Split Canvas",
                "Draft your thoughts and instantly view formatted output. Easily switch between Edit (Ctrl+1), Live Split (Ctrl+2), and Full Preview (Ctrl+3). Cycle between Standard, Wide, and Full-Width writing columns via Ctrl+W.",
                "Shortcuts: Ctrl+1/2/3 for Views  •  Ctrl+W for Width  •  Ctrl+B/I for formatting",
            ),
            2 => (
                "3. Infinite Whiteboard Studio (Tablet & Pen)",
                "Switch into Whiteboard Studio (Ctrl+M) anytime for freehand notes, sketching, and diagramming. Designed for tablet and pen stylus users with low latency, multiple paper templates (Math Grid, Ruled, Dot, Blank), and one-click Wipe Board.",
                "Shortcuts: Ctrl+M to toggle Whiteboard  •  1-click Wipe to delete scratch work",
            ),
            3 => (
                "4. Hermes AI Essay Coach & Writing Agent",
                "Grade your essays against CEFR rubrics, receive actionable writing tips, check grammar and flow, and polish German B2/C1 connectors. Runs 100% privately on your machine using local Hermes LLM (via Ollama or OpenAI-compatible endpoint) with a built-in offline evaluator.",
                "Hermes Agent • 100% Private Local AI • CEFR Essay Grading",
            ),
            _ => (
                "5. Zen Mode, Exam Trainer & Full Portability",
                "Press Ctrl+H to toggle the top toolbar for complete immersion. Track exam countdowns and word goals. All drafts, settings, and recovery buffers stay right alongside your binary in ./krebs_data/.",
                "Shortcuts: Ctrl+H to Hide/Show Toolbar  •  Ctrl+E for AI/Exam Drawer  •  F1 for Shortcuts",
            ),
        };

        let step_counter = text(format!("Step {} of 5", self.tour_step + 1))
            .size(11)
            .color(TEXT_MUTED);

        let content_card = column![
            text(step_title).size(14).color(TEXT_PRIMARY),
            text(step_desc).size(12).color(Color::from_rgb(0.80, 0.84, 0.90)),
            container(text(step_tag).size(11).color(ACCENT_AMBER))
                .padding(8)
                .width(Length::Fill)
                .style(card_container_style),
        ]
        .spacing(10);

        let mut btn_prev = button(text("← Previous").size(11))
            .style(rail_button_style)
            .padding(iced::Padding { top: 6.0, bottom: 6.0, left: 12.0, right: 12.0 });
        if self.tour_step > 0 {
            btn_prev = btn_prev.on_press(Message::TourPrev);
        }

        let next_label = if self.tour_step < 4 { "Next →" } else { "Get Started 🚀" };
        let next_msg = if self.tour_step < 4 { Message::TourNext } else { Message::TourFinish };
        let btn_next = button(text(next_label).size(11))
            .on_press(next_msg)
            .style(tab_active_button_style)
            .padding(iced::Padding { top: 6.0, bottom: 6.0, left: 14.0, right: 14.0 });

        let footer_row = row![
            step_counter,
            Space::with_width(Length::Fill),
            btn_prev,
            btn_next,
        ]
        .spacing(8)
        .align_y(Alignment::Center);

        let modal_col = column![
            title_row,
            create_divider_simple(),
            content_card,
            create_divider_simple(),
            footer_row,
        ]
        .spacing(14);

        container(modal_col)
            .width(520)
            .padding(20)
            .style(modal_container_style)
            .into()
    }

    fn view_theme_modal<'a>(&self) -> Element<'a, Message> {
        let title_row = row![
            text("🎨 Customize Theme & Colors").size(15).color(ACCENT_CYAN),
            Space::with_width(Length::Fill),
            button(center(render_svg_icon(SVG_CLOSE, 12, TEXT_MUTED)))
                .on_press(Message::CloseThemeModal)
                .style(rail_button_style)
                .width(28)
                .height(28),
        ]
        .align_y(Alignment::Center);

        let themes = [
            (ThemeMode::OmarchyDark, "Omarchy Dark", "Matte charcoal canvas with obsidian rails"),
            (ThemeMode::OledBlack, "OLED Pure Black", "True deep black for high contrast AMOLED"),
            (ThemeMode::DeepSlate, "Deep Slate", "Modern dark slate blue with crisp accents"),
            (ThemeMode::WarmEspresso, "Warm Espresso", "Warm earthy dark coffee tones"),
        ];

        let mut theme_list = column![
            text("CANVAS BACKGROUND MOOD").size(10).color(TEXT_MUTED),
        ].spacing(6);

        for (tm, name, desc) in themes {
            let is_sel = self.theme_mode == tm;
            let btn = button(
                row![
                    container(Space::with_width(12))
                        .width(12)
                        .height(12)
                        .style(move |_t| container::Style {
                            background: Some(iced::Background::Color(tm.canvas_bg())),
                            border: iced::Border {
                                color: if is_sel { ACCENT_CYAN } else { BORDER_COLOR },
                                width: 1.0,
                                radius: iced::border::Radius::from(6.0),
                            },
                            ..Default::default()
                        }),
                    column![
                        text(name).size(12).color(if is_sel { ACCENT_CYAN } else { TEXT_PRIMARY }),
                        text(desc).size(10).color(TEXT_MUTED),
                    ]
                    .spacing(2),
                    Space::with_width(Length::Fill),
                    if is_sel { text("✓").size(12).color(ACCENT_CYAN) } else { text("").size(12) },
                ]
                .spacing(10)
                .align_y(Alignment::Center),
            )
            .on_press(Message::SelectTheme(tm))
            .style(if is_sel { active_tool_button_style } else { rail_button_style })
            .padding(10)
            .width(Length::Fill);

            theme_list = theme_list.push(btn);
        }

        let accents = [
            (AccentChoice::NordicCyan, "Nordic Cyan"),
            (AccentChoice::AmberSunset, "Amber Sunset"),
            (AccentChoice::EmeraldMint, "Emerald Mint"),
            (AccentChoice::NordicFrost, "Nordic Frost"),
            (AccentChoice::RoseBlush, "Rose Blush"),
        ];

        let mut accent_row = row![].spacing(8);
        for (ac, name) in accents {
            let is_sel = self.accent_choice == ac;
            let c = ac.color();
            let btn = button(
                column![
                    container(Space::with_width(18))
                        .width(18)
                        .height(18)
                        .style(move |_t| container::Style {
                            background: Some(iced::Background::Color(c)),
                            border: iced::Border {
                                color: if is_sel { Color::WHITE } else { Color::TRANSPARENT },
                                width: 2.0,
                                radius: iced::border::Radius::from(9.0),
                            },
                            ..Default::default()
                        }),
                    text(name).size(9).color(if is_sel { Color::WHITE } else { TEXT_MUTED }),
                ]
                .spacing(4)
                .align_x(Alignment::Center),
            )
            .on_press(Message::SelectAccent(ac))
            .style(rail_button_style)
            .padding(6);

            accent_row = accent_row.push(btn);
        }

        let accent_section = column![
            text("ACCENT HIGHLIGHT COLOR").size(10).color(TEXT_MUTED),
            accent_row,
        ].spacing(6);

        let modal_col = column![
            title_row,
            create_divider_simple(),
            theme_list,
            create_divider_simple(),
            accent_section,
        ]
        .spacing(14);

        container(modal_col)
            .width(460)
            .padding(20)
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
        decorations: false,
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


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_canvas_width_mode_cycling() {
        let mode = CanvasWidthMode::Standard;
        let next = mode.next();
        assert_eq!(next, CanvasWidthMode::Wide);
        let next = next.next();
        assert_eq!(next, CanvasWidthMode::Full);
        let next = next.next();
        assert_eq!(next, CanvasWidthMode::Standard);
    }

    #[test]
    fn test_view_mode_cycling() {
        let mode = ViewMode::Edit;
        let next = mode.next();
        assert_eq!(next, ViewMode::Split);
        let next = next.next();
        assert_eq!(next, ViewMode::Preview);
        let next = next.next();
        assert_eq!(next, ViewMode::Edit);
    }

    #[test]
    fn test_font_choice_mapping() {
        let choices = [
            (FontChoice::JetBrainsMono, "jetbrains_mono"),
            (FontChoice::IbmPlexMono, "ibm_plex_mono"),
            (FontChoice::Vt323, "vt323"),
            (FontChoice::Vazirmatn, "vazirmatn"),
            (FontChoice::Sans, "sans"),
            (FontChoice::Serif, "serif"),
        ];

        for (choice, name) in choices {
            assert_eq!(choice.to_str_name(), name);
            assert_eq!(FontChoice::from_str_name(name), choice);
            let _ = choice.to_font();
        }
    }

    #[test]
    fn test_krebs_app_buttons_and_state() {
        let (mut app, _task) = Krebs::new(None);
        assert_eq!(app.app_mode, AppMode::Writing);

        // 1. Test Mode toggle button (Writing <-> Whiteboard)
        let _ = app.update(Message::Sidebar(SidebarAction::ToggleAppMode));
        assert_eq!(app.app_mode, AppMode::Whiteboard);
        let _ = app.update(Message::Sidebar(SidebarAction::ToggleAppMode));
        assert_eq!(app.app_mode, AppMode::Writing);

        // 2. Test Canvas width cycle button
        let initial_width = app.canvas_width_mode;
        let _ = app.update(Message::CycleWidthMode);
        assert_ne!(app.canvas_width_mode, initial_width);

        // 3. Test View mode button
        let _ = app.update(Message::SetViewMode(ViewMode::Edit));
        assert_eq!(app.view_mode, ViewMode::Edit);
        let _ = app.update(Message::SetViewMode(ViewMode::Split));
        assert_eq!(app.view_mode, ViewMode::Split);
        let _ = app.update(Message::SetViewMode(ViewMode::Preview));
        assert_eq!(app.view_mode, ViewMode::Preview);

        // 4. Test Search button and match navigation
        app.content = text_editor::Content::with_text("Rust is fast.\nWriting in Rust is awesome.\nPure Rust!");
        let _ = app.update(Message::ToggleSearch);
        assert!(app.search.is_open);

        let _ = app.update(Message::SearchInputChanged("Rust".to_string()));
        assert_eq!(app.search.matches.len(), 3);
        assert_eq!(app.search.current_idx, 0);

        let _ = app.update(Message::SearchNext);
        assert_eq!(app.search.current_idx, 1);
        let _ = app.update(Message::SearchNext);
        assert_eq!(app.search.current_idx, 2);
        let _ = app.update(Message::SearchNext);
        assert_eq!(app.search.current_idx, 0);

        let _ = app.update(Message::SearchPrev);
        assert_eq!(app.search.current_idx, 2);

        let _ = app.update(Message::CloseSearch);
        assert!(!app.search.is_open);

        // 5. Test Zen Timer buttons
        assert!(!app.timer_is_running);
        let _ = app.update(Message::ZenTimerToggle);
        assert!(app.timer_is_running);
        let _ = app.update(Message::ZenTimerToggle);
        assert!(!app.timer_is_running);

        let _ = app.update(Message::ZenTimerSet(1500));
        assert_eq!(app.timer_remaining_secs, 1500);
        assert!(app.timer_is_running);

        // 6. Test Typography button
        let _ = app.update(Message::SelectFont(FontChoice::Vt323));
        assert_eq!(app.font_choice, FontChoice::Vt323);
        let _ = app.update(Message::SelectFont(FontChoice::JetBrainsMono));
        assert_eq!(app.font_choice, FontChoice::JetBrainsMono);

        // 7. Test Whiteboard buttons
        let _ = app.update(Message::WhiteboardToolSelect(WhiteboardTool::Highlighter));
        assert_eq!(app.whiteboard.active_tool, WhiteboardTool::Highlighter);
        let _ = app.update(Message::WhiteboardWidthSelect(6.0));
        assert_eq!(app.whiteboard.pen_width, 6.0);
        let _ = app.update(Message::WhiteboardPatternSelect(PaperPattern::MathGrid));
        assert_eq!(app.whiteboard.paper_pattern, PaperPattern::MathGrid);

        // 8. Test German snippets insertion button
        let _ = app.update(Message::Sidebar(SidebarAction::InsertSnippet("In der heutigen Zeit".to_string())));
        assert!(app.content.text().contains("In der heutigen Zeit"));

        // 9. Test German umlaut chord button
        let _ = app.update(Message::Sidebar(SidebarAction::InsertGermanChar("ä")));
        assert!(app.content.text().contains("ä"));

        // 10. Test Theme and Accent customization
        let _ = app.update(Message::SelectTheme(ThemeMode::OledBlack));
        assert_eq!(app.theme_mode, ThemeMode::OledBlack);
        let _ = app.update(Message::SelectAccent(AccentChoice::AmberSunset));
        assert_eq!(app.accent_choice, AccentChoice::AmberSunset);

        // 11. Test Interactive Quick Tour navigation
        app.show_tour_modal = false;
        let _ = app.update(Message::ToggleTourModal);
        assert!(app.show_tour_modal);
        assert_eq!(app.tour_step, 0);
        let _ = app.update(Message::TourNext);
        assert_eq!(app.tour_step, 1);
        let _ = app.update(Message::TourPrev);
        assert_eq!(app.tour_step, 0);
        let _ = app.update(Message::TourFinish);
        assert!(!app.show_tour_modal);
        assert!(app.config.has_seen_tour);

        // 12. Test Zen Mode toolbar hide/unhide toggle
        assert!(app.show_top_toolbar);
        let _ = app.update(Message::ToggleTopToolbar);
        assert!(!app.show_top_toolbar);
        let _ = app.update(Message::ToggleTopToolbar);
        assert!(app.show_top_toolbar);

        // 13. Test Hermes AI Drawer toggle and actions
        app.config.show_right_drawer = false;
        let _ = app.update(Message::ToggleAiDrawer);
        assert!(app.config.show_right_drawer);
        assert_eq!(app.right_drawer_tab, RightDrawerTab::AiCoach);
        let _ = app.update(Message::SetRightDrawerTab(RightDrawerTab::Redemittel));
        assert_eq!(app.right_drawer_tab, RightDrawerTab::Redemittel);

        // 14. Test Offline Essay Evaluator
        let sample_essay = "In der heutigen Zeit ist Bildung von zentraler Bedeutung. Einerseits bietet sie Chancen. Andererseits erfordert sie Fleiss. Zusammenfassend laesst sich sagen, dass Lernen lebenslang ist.";
        let grade_report = ai::evaluate_essay_offline(sample_essay, ai::AiAction::GradeEssay);
        assert!(grade_report.contains("Academic Essay Scorecard"));
        assert!(grade_report.contains("Quantitative Metrics"));

        let tips_report = ai::evaluate_essay_offline(sample_essay, ai::AiAction::TipsAndFeedback);
        assert!(tips_report.contains("Writing Tips & Diagnostics"));

        // 15. Test Resizable sections (Live Split resizing, presets, and drawer width)
        let _ = app.update(Message::SetSplitRatio(0.50));
        assert!((app.config.split_ratio - 0.50).abs() < 0.001);
        let _ = app.update(Message::AdjustSplitRatio(0.05));
        assert!((app.config.split_ratio - 0.55).abs() < 0.001);
        let _ = app.update(Message::AdjustSplitRatio(-0.10));
        assert!((app.config.split_ratio - 0.45).abs() < 0.001);
        let _ = app.update(Message::CycleSplitPresets);
        assert!((app.config.split_ratio - 0.50).abs() < 0.001);

        let init_drawer = app.config.drawer_width;
        let _ = app.update(Message::AdjustDrawerWidth(30.0));
        assert_eq!(app.config.drawer_width, init_drawer + 30.0);
        let _ = app.update(Message::AdjustDrawerWidth(-30.0));
        assert_eq!(app.config.drawer_width, init_drawer);

        // 16. Test Multi-Agent Provider Switcher & API Key
        let _ = app.update(Message::SelectAiProvider(AiProvider::Claude));
        assert_eq!(app.ai_state.provider, AiProvider::Claude);
        assert!(app.ai_state.provider.requires_api_key());
        let _ = app.update(Message::AiApiKeyChanged("sk-ant-testkey".to_string()));
        assert_eq!(app.ai_state.api_key, "sk-ant-testkey");
        assert_eq!(app.config.ai_api_key, "sk-ant-testkey");

        let _ = app.update(Message::SelectAiProvider(AiProvider::ChatGPT));
        assert_eq!(app.ai_state.provider, AiProvider::ChatGPT);
        let _ = app.update(Message::SelectAiProvider(AiProvider::DeepSeek));
        assert_eq!(app.ai_state.provider, AiProvider::DeepSeek);
        let _ = app.update(Message::SelectAiProvider(AiProvider::Hermes));
        assert_eq!(app.ai_state.provider, AiProvider::Hermes);
        assert!(!app.ai_state.provider.requires_api_key());

        // 17. Test Contextual Whiteboard Sidebar Actions
        let _ = app.update(Message::Sidebar(SidebarAction::WhiteboardToolPen));
        assert_eq!(app.whiteboard.active_tool, WhiteboardTool::Pen);
        let _ = app.update(Message::Sidebar(SidebarAction::WhiteboardToolHighlighter));
        assert_eq!(app.whiteboard.active_tool, WhiteboardTool::Highlighter);
        let _ = app.update(Message::Sidebar(SidebarAction::WhiteboardToolEraser));
        assert_eq!(app.whiteboard.active_tool, WhiteboardTool::Eraser);
        let _ = app.update(Message::Sidebar(SidebarAction::WhiteboardWipe));
        assert!(app.whiteboard.strokes.is_empty());
    }
}
