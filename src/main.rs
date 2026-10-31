#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

pub mod ai;
pub mod config;
pub mod editor;
pub mod export;
pub mod i18n;
pub mod icons;
pub mod markdown_view;
pub mod prompt;
pub mod sidebar;
pub mod study;
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
    button, center, column, container, mouse_area, row, scrollable, stack, text, text_editor,
    text_input, Space,
};
use iced::window;
use iced::{Alignment, Color, Element, Font, Length, Size, Subscription, Task, Theme};

use crate::icons::*;
use ai::{AiAction, AiProvider, AiState, TutorProfile};
use config::{AppConfig, RecoveryManager, SnippetsConfig, SnippetsDeManager};
use editor::{
    apply_smart_typography, detect_paragraph_direction, insert_text_at_cursor,
    insert_zwnj_at_cursor, normalize_persian_char, normalize_persian_string, wrap_markdown_syntax,
    DirectionMode, DocumentStats, EditorHistory,
};
use sidebar::{view_left_rail, SidebarAction};
use theme::{
    accent_button_style, active_tool_button_style, banner_container_style, canvas_container_style,
    card_container_style,
    drawer_container_style, modal_container_style, rail_button_style, search_container_style,
    tab_active_button_style, tab_inactive_button_style, text_editor_style, text_input_style,
    toolbar_floating_style, wipe_button_style, AccentChoice, ThemeMode, ACCENT_AMBER, ACCENT_CYAN,
    ACCENT_GREEN, ACCENT_RED, BORDER_COLOR, RAIL_BG, TEXT_MUTED, TEXT_PRIMARY,
};
use whiteboard::{
    PaperPattern, WhiteboardCanvas, WhiteboardMessage, WhiteboardState, WhiteboardTool,
};

// Bundled Offline Fonts & Assets
pub static KREBS_LOGO_PNG: &[u8] = include_bytes!("../assets/logo.png");
pub static IBM_PLEX_MONO_REGULAR: &[u8] = include_bytes!("../fonts/IBMPlexMono-Regular.ttf");
pub static IBM_PLEX_MONO_BOLD: &[u8] = include_bytes!("../fonts/IBMPlexMono-Bold.ttf");
pub static VAZIRMATN_REGULAR: &[u8] = include_bytes!("../fonts/Vazirmatn-Regular.ttf");
pub static VAZIRMATN_BOLD: &[u8] = include_bytes!("../fonts/Vazirmatn-Bold.ttf");
pub static JETBRAINS_MONO_REGULAR: &[u8] = include_bytes!("../fonts/JetBrainsMono-Regular.ttf");
pub static VT323_REGULAR: &[u8] = include_bytes!("../fonts/VT323-Regular.ttf");
pub static INTER_REGULAR: &[u8] = include_bytes!("../fonts/Inter-Variable.ttf");
pub static LORA_REGULAR: &[u8] = include_bytes!("../fonts/Lora-Variable.ttf");

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
        background: Some(iced::Background::Color(Color::from_rgba(
            0.03, 0.04, 0.07, 0.97,
        ))),
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

pub fn font_inter() -> Font {
    Font {
        family: font::Family::Name("Inter"),
        weight: font::Weight::Normal,
        stretch: font::Stretch::Normal,
        style: font::Style::Normal,
    }
}

pub fn font_lora() -> Font {
    Font {
        family: font::Family::Name("Lora"),
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
    Inter,
    Lora,
    Vt323,
    Vazirmatn,
    Sans,
    Serif,
}

impl FontChoice {
    pub fn from_str_name(s: &str) -> Self {
        match s {
            "ibm_plex_mono" => Self::IbmPlexMono,
            "inter" => Self::Inter,
            "lora" => Self::Lora,
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
            Self::Inter => "inter",
            Self::Lora => "lora",
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
            Self::Inter => "Inter",
            Self::Lora => "Lora",
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
            Self::Inter => font_inter(),
            Self::Lora => font_lora(),
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
    show_lang_modal: bool,
    show_export_modal: bool,
    /// Active Kartei review session; `Some` while the study overlay is open.
    study: Option<study::Review>,
    show_tour_modal: bool,
    tour_step: usize,
    is_maximized: bool,
    show_top_toolbar: bool,
    ai_state: AiState,
    prompt: PromptState,
    show_agent_modal: bool,
    agent: AgentModalState,
    keyboard_modifiers: Modifiers,
    right_drawer_tab: RightDrawerTab,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RightDrawerTab {
    Redemittel,
    PromptStudio,
}

/// Session state of the centered Agent Studio modal.
#[derive(Debug, Clone)]
struct AgentModalState {
    is_testing: bool,
    test_status: Option<(bool, String)>,
    is_running: bool,
    result: String,
}

impl Default for AgentModalState {
    fn default() -> Self {
        Self {
            is_testing: false,
            test_status: None,
            is_running: false,
            result: String::new(),
        }
    }
}

/// The visual prompt builder state (session-scoped, not persisted).
#[derive(Debug, Clone)]
struct PromptState {
    spec: prompt::PromptSpec,
    tools: Vec<bool>,
    is_improving: bool,
    is_running: bool,
    improved_text: String,
    run_text: String,
}

impl Default for PromptState {
    fn default() -> Self {
        Self {
            spec: prompt::PromptSpec::default(),
            tools: vec![false; prompt::MCP_TOOLS.len()],
            is_improving: false,
            is_running: false,
            improved_text: String::new(),
            run_text: String::new(),
        }
    }
}

/// The export formats offered by the unified export modal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportFormat {
    /// Paginated PDF rendered with genpdf.
    Pdf,
    /// Markdown file with YAML front matter.
    Markdown,
    /// Print-ready HTML document.
    Html,
}

#[derive(Debug, Clone)]
enum Message {
    EditorAction(text_editor::Action),
    Sidebar(SidebarAction),
    FileOpened(Option<PathBuf>),
    FileSaved(Option<PathBuf>),
    FileExported(Option<PathBuf>, ExportFormat),
    FileDropped(PathBuf),
    Tick,
    KeyboardEvent(Key, Modifiers),
    ModifiersChanged(Modifiers),
    MouseScrolled(f32, Modifiers),
    ReloadFromDisk,
    DismissExternalBanner,
    CloseShortcutsModal,
    CloseFontModal,
    SelectFont(FontChoice),
    ToggleFontModal,
    ToggleTopToolbar,
    ToggleAiDrawer,
    ToggleAgentModal,
    CloseAgentModal,
    AgentTestConnection,
    AgentTestFinished(bool, String),
    TutorSetLanguage(String),
    TutorSetLevel(String),
    TutorSetGoal(String),
    TutorAsk,
    TutorAnswered(String, bool),
    CopyTeacherInstruction,
    CopyTutorResult,
    InsertTutorFeedback,
    AdjustFontSize(f32),
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
    ToggleThemeModal,
    CloseThemeModal,
    SelectTheme(ThemeMode),
    SelectAccent(AccentChoice),
    ToggleLangModal,
    CloseLangModal,
    SelectLanguage(i18n::Lang),
    ToggleExportModal,
    CloseExportModal,
    ExportPdf,
    ExportMarkdown,
    ExportHtml,
    CopyMarkdown,
    ToggleStudyModal,
    CloseStudyModal,
    StudyReveal,
    StudyGrade(study::Grade),
    PromptSetField(prompt::PromptField, String),
    PromptToggleTool(usize),
    PromptImprove,
    PromptImproved(String, bool),
    PromptRunTest,
    PromptRanTest(String, bool),
    PromptCopyPack,
    PromptCopyFor(prompt::CopyTarget),
    PromptCopyImproved,
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
        let mut recovered_path = None;

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
                        recovered_path = Some(first_recovery.clone());
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
            active_recovery_file: recovered_path,
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
            show_lang_modal: false,
            show_export_modal: false,
            study: None,
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
            prompt: PromptState::default(),
            show_agent_modal: false,
            agent: AgentModalState::default(),
            keyboard_modifiers: Modifiers::default(),
            right_drawer_tab: RightDrawerTab::Redemittel,
        };

        let load_fonts_task = Task::batch(vec![
            font::load(Cow::Borrowed(IBM_PLEX_MONO_REGULAR)).map(Message::FontLoaded),
            font::load(Cow::Borrowed(IBM_PLEX_MONO_BOLD)).map(Message::FontLoaded),
            font::load(Cow::Borrowed(VAZIRMATN_REGULAR)).map(Message::FontLoaded),
            font::load(Cow::Borrowed(VAZIRMATN_BOLD)).map(Message::FontLoaded),
            font::load(Cow::Borrowed(JETBRAINS_MONO_REGULAR)).map(Message::FontLoaded),
            font::load(Cow::Borrowed(VT323_REGULAR)).map(Message::FontLoaded),
            font::load(Cow::Borrowed(INTER_REGULAR)).map(Message::FontLoaded),
            font::load(Cow::Borrowed(LORA_REGULAR)).map(Message::FontLoaded),
        ]);

        (app, load_fonts_task)
    }

    fn notify(&mut self, text: impl Into<String>) {
        self.status_notification = Some((text.into(), Instant::now()));
    }

    /// The active UI language, resolved from the persisted config code.
    #[must_use]
    fn lang(&self) -> i18n::Lang {
        i18n::Lang::from_code(&self.config.language)
    }

    fn is_rtl(&self) -> bool {
        i18n::is_rtl(&self.config.language)
    }

    /// Marks the buffer dirty after a programmatic edit (inserts, formatting).
    fn touch_content(&mut self) {
        self.is_dirty = true;
        self.last_edit_time = Instant::now();
        self.stats = DocumentStats::compute(&self.content.text());
    }

    /// The indices of the MCP tools the user switched on.
    #[must_use]
    fn prompt_tools_enabled(&self) -> Vec<usize> {
        self.prompt
            .tools
            .iter()
            .enumerate()
            .filter(|(_, enabled)| **enabled)
            .map(|(index, _)| index)
            .collect()
    }

    /// The full agent pack: rendered brief plus the enabled tool section.
    #[must_use]
    fn prompt_pack(&self) -> String {
        prompt::render_agent_pack(&self.prompt.spec, &self.prompt_tools_enabled())
    }

    /// The learner profile the built-in teacher agent adapts to.
    #[must_use]
    fn tutor_profile(&self) -> TutorProfile {
        TutorProfile {
            language: self.config.tutor_language.clone(),
            level: self.config.tutor_level.clone(),
            goal: self.config.tutor_goal.clone(),
        }
    }

    /// Opens the centered Agent Studio and resets its transient status.
    fn open_agent_modal(&mut self) {
        self.show_agent_modal = true;
        self.agent.test_status = None;
        self.ai_state.temp_endpoint = self.ai_state.endpoint.clone();
        self.ai_state.temp_model = self.ai_state.model.clone();
    }

    /// Persists the endpoint/model currently held in the modal’s temp fields.
    fn commit_ai_settings(&mut self) {
        self.ai_state.endpoint = self.ai_state.temp_endpoint.clone();
        self.ai_state.model = self.ai_state.temp_model.clone();
        self.config.ai_endpoint = self.ai_state.endpoint.clone();
        self.config.ai_model = self.ai_state.model.clone();
        self.config.ai_api_key = self.ai_state.api_key.clone();
        self.config.ai_provider = self.ai_state.provider.to_str_id().to_string();
        let _ = self.config.save();
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
                    i18n::t(self.lang(), i18n::Key::ToolbarVisible)
                } else {
                    i18n::t(self.lang(), i18n::Key::ZenToolbarHidden)
                });
                Task::none()
            }

            Message::ToggleAiDrawer => {
                self.open_agent_modal();
                Task::none()
            }

            Message::ToggleAgentModal => {
                if self.show_agent_modal {
                    self.show_agent_modal = false;
                    self.commit_ai_settings();
                } else {
                    self.open_agent_modal();
                }
                Task::none()
            }

            Message::CloseAgentModal => {
                self.show_agent_modal = false;
                self.commit_ai_settings();
                Task::none()
            }

            Message::AgentTestConnection => {
                self.agent.is_testing = true;
                self.agent.test_status = None;
                let provider = self.ai_state.provider;
                let endpoint = self.ai_state.temp_endpoint.clone();
                let model = self.ai_state.temp_model.clone();
                let api_key = self.ai_state.api_key.clone();
                Task::perform(
                    async move { ai::probe_connection(provider, endpoint, model, api_key) },
                    |(ok, msg)| Message::AgentTestFinished(ok, msg),
                )
            }

            Message::AgentTestFinished(ok, detail) => {
                self.agent.is_testing = false;
                self.agent.test_status = Some((ok, detail.clone()));
                if ok {
                    self.commit_ai_settings();
                    self.notify(i18n::t(self.lang(), i18n::Key::Connected));
                } else {
                    self.notify(format!("{}: {}", i18n::t(self.lang(), i18n::Key::NotConnected), detail));
                }
                Task::none()
            }

            Message::TutorSetLanguage(value) => {
                self.config.tutor_language = value;
                let _ = self.config.save();
                Task::none()
            }

            Message::TutorSetLevel(value) => {
                self.config.tutor_level = value;
                let _ = self.config.save();
                Task::none()
            }

            Message::TutorSetGoal(value) => {
                self.config.tutor_goal = value;
                let _ = self.config.save();
                Task::none()
            }

            Message::TutorAsk => {
                self.agent.is_running = true;
                let provider = self.ai_state.provider;
                let endpoint = self.ai_state.temp_endpoint.clone();
                let model = self.ai_state.temp_model.clone();
                let api_key = self.ai_state.api_key.clone();
                let profile = self.tutor_profile();
                let note = self.content.text();
                let temperature = self.ai_state.temperature;
                let max_tokens = self.ai_state.max_tokens;
                Task::perform(
                    async move { ai::run_tutor_task(provider, endpoint, model, api_key, profile, note, temperature, max_tokens) },
                    |(res, offline)| Message::TutorAnswered(res, offline),
                )
            }

            Message::TutorAnswered(res, offline) => {
                self.agent.is_running = false;
                self.agent.result = res;
                self.notify(if offline {
                    "Offline study plan ready — connect a model for corrections"
                } else {
                    "Teacher feedback ready"
                });
                Task::none()
            }

            Message::CopyTeacherInstruction => {
                if let Ok(mut cb) = arboard::Clipboard::new() {
                    let _ = cb.set_text(self.tutor_profile().instruction());
                    self.notify(i18n::t(self.lang(), i18n::Key::PromptCopied));
                }
                Task::none()
            }

            Message::CopyTutorResult => {
                if !self.agent.result.is_empty() {
                    if let Ok(mut cb) = arboard::Clipboard::new() {
                        let _ = cb.set_text(&self.agent.result);
                        self.notify(i18n::t(self.lang(), i18n::Key::PromptCopied));
                    }
                }
                Task::none()
            }

            Message::InsertTutorFeedback => {
                if !self.agent.result.is_empty() {
                    let formatted = format!("\n\n---\n{}\n---\n", self.agent.result);
                    insert_text_at_cursor(&mut self.content, &formatted);
                    self.stats = DocumentStats::compute(&self.content.text());
                    self.is_dirty = true;
                    self.last_edit_time = Instant::now();
                    self.notify("Teacher feedback inserted");
                }
                Task::none()
            }

            Message::AdjustFontSize(delta) => {
                self.config.font_size = (self.config.font_size + delta).clamp(11.0, 28.0);
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
                let temperature = self.ai_state.temperature;
                let max_tokens = self.ai_state.max_tokens;
                Task::perform(
                    async move { ai::run_ai_task(provider, endpoint, model, api_key, text, action, temperature, max_tokens) },
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
                    self.touch_content();
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

            Message::ToggleLangModal => {
                self.show_lang_modal = !self.show_lang_modal;
                Task::none()
            }

            Message::CloseLangModal => {
                self.show_lang_modal = false;
                Task::none()
            }

            Message::SelectLanguage(l) => {
                self.show_lang_modal = false;
                self.config.language = l.code().to_string();
                let _ = self.config.save();
                let rtl = if i18n::is_rtl(l.code()) { " (RTL)" } else { "" };
                self.notify(format!(
                    "{}: {}{}",
                    i18n::t(l, i18n::Key::LanguageTitle).trim_start_matches("🌐 "),
                    l.display(),
                    rtl
                ));
                Task::none()
            }

            Message::ToggleExportModal => {
                self.show_export_modal = !self.show_export_modal;
                Task::none()
            }

            Message::CloseExportModal => {
                self.show_export_modal = false;
                Task::none()
            }

            Message::ExportPdf => {
                self.show_export_modal = false;
                Task::perform(
                    async {
                        rfd::FileDialog::new()
                            .add_filter("PDF Document", &["pdf"])
                            .set_file_name("document.pdf")
                            .save_file()
                    },
                    |p| Message::FileExported(p, ExportFormat::Pdf),
                )
            }

            Message::ExportMarkdown => {
                self.show_export_modal = false;
                Task::perform(
                    async {
                        rfd::FileDialog::new()
                            .add_filter("Markdown (*.md)", &["md"])
                            .set_file_name("document.md")
                            .save_file()
                    },
                    |p| Message::FileExported(p, ExportFormat::Markdown),
                )
            }

            Message::ExportHtml => {
                self.show_export_modal = false;
                Task::perform(
                    async {
                        rfd::FileDialog::new()
                            .add_filter("HTML Document", &["html", "htm"])
                            .set_file_name("document.html")
                            .save_file()
                    },
                    |p| Message::FileExported(p, ExportFormat::Html),
                )
            }

            Message::CopyMarkdown => {
                let text = self.content.text();
                match export::copy_markdown_to_clipboard(&text) {
                    Ok(()) => {
                        let lang = self.lang();
                        self.notify(i18n::t(lang, i18n::Key::MarkdownCopied));
                    }
                    Err(_) => {
                        let lang = self.lang();
                        self.notify(i18n::t(lang, i18n::Key::ExportFailed));
                    }
                }
                Task::none()
            }

            Message::ToggleStudyModal => {
                if self.study.is_some() {
                    self.study = None;
                } else {
                    let text = self.content.text();
                    self.study = Some(study::Review::open(
                        &text,
                        study::StudyTable::load(),
                        study::now_ms(),
                    ));
                }
                Task::none()
            }

            Message::CloseStudyModal => {
                self.study = None;
                Task::none()
            }

            Message::StudyReveal => {
                if let Some(review) = &mut self.study {
                    review.revealed = true;
                }
                Task::none()
            }

            Message::StudyGrade(grade) => {
                if let Some(review) = &mut self.study {
                    review.advance(grade, study::now_ms());
                    let _ = review.table.save();
                    if review.is_done() {
                        let lang = self.lang();
                        self.notify(i18n::t(lang, i18n::Key::StudyDone));
                    }
                }
                Task::none()
            }

            Message::PromptSetField(field, value) => {
                match field {
                    prompt::PromptField::Persona => self.prompt.spec.persona = value,
                    prompt::PromptField::Task => self.prompt.spec.task = value,
                    prompt::PromptField::Context => self.prompt.spec.context = value,
                    prompt::PromptField::Constraints => self.prompt.spec.constraints = value,
                    prompt::PromptField::OutputFormat => self.prompt.spec.output_format = value,
                }
                Task::none()
            }

            Message::PromptToggleTool(index) => {
                if let Some(flag) = self.prompt.tools.get_mut(index) {
                    *flag = !*flag;
                }
                Task::none()
            }

            Message::PromptImprove | Message::PromptRunTest => {
                let is_improve = matches!(message, Message::PromptImprove);
                let action = if is_improve {
                    AiAction::PromptImprove
                } else {
                    AiAction::Freeform
                };
                let text = self.prompt_pack();
                if is_improve {
                    self.prompt.is_improving = true;
                } else {
                    self.prompt.is_running = true;
                }
                let provider = self.ai_state.provider;
                let endpoint = self.ai_state.endpoint.clone();
                let model = self.ai_state.model.clone();
                let api_key = self.ai_state.api_key.clone();
                let temperature = self.ai_state.temperature;
                let max_tokens = self.ai_state.max_tokens;
                Task::perform(
                    async move { ai::run_ai_task(provider, endpoint, model, api_key, text, action, temperature, max_tokens) },
                    move |(res, is_offline)| {
                        if is_improve {
                            Message::PromptImproved(res, is_offline)
                        } else {
                            Message::PromptRanTest(res, is_offline)
                        }
                    },
                )
            }

            Message::PromptImproved(res, is_offline) => {
                self.prompt.is_improving = false;
                self.prompt.improved_text = res;
                self.notify(if is_offline {
                    "Prompt improved via offline templates"
                } else {
                    "Prompt improved"
                });
                Task::none()
            }

            Message::PromptRanTest(res, is_offline) => {
                self.prompt.is_running = false;
                self.prompt.run_text = res;
                self.notify(if is_offline {
                    "Offline: connect an AI endpoint to run briefs"
                } else {
                    "Brief executed"
                });
                Task::none()
            }

            Message::PromptCopyPack => {
                if let Ok(mut cb) = arboard::Clipboard::new() {
                    let _ = cb.set_text(self.prompt_pack());
                    self.notify(i18n::t(self.lang(), i18n::Key::PromptCopied));
                }
                Task::none()
            }

            Message::PromptCopyFor(target) => {
                let text = prompt::render_for_target(target, &self.prompt_pack());
                if let Ok(mut cb) = arboard::Clipboard::new() {
                    let _ = cb.set_text(text);
                    self.notify(i18n::t(self.lang(), i18n::Key::PromptCopied));
                }
                Task::none()
            }

            Message::PromptCopyImproved => {
                if !self.prompt.improved_text.is_empty() {
                    if let Ok(mut cb) = arboard::Clipboard::new() {
                        let _ = cb.set_text(&self.prompt.improved_text);
                        self.notify(i18n::t(self.lang(), i18n::Key::PromptCopied));
                    }
                }
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

            Message::WindowClose => window::get_latest().and_then(window::close),

            Message::WindowDrag => window::get_latest().and_then(window::drag),

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
                    self.search.current_idx =
                        (self.search.current_idx + 1) % self.search.matches.len();
                    let (line, col) = self.search.matches[self.search.current_idx];
                    self.notify(format!(
                        "Match {} of {} (Line {}, Col {})",
                        self.search.current_idx + 1,
                        self.search.matches.len(),
                        line,
                        col
                    ));
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
                    self.notify(format!(
                        "Match {} of {} (Line {}, Col {})",
                        self.search.current_idx + 1,
                        self.search.matches.len(),
                        line,
                        col
                    ));
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
                            text_editor::Action::Edit(text_editor::Edit::Paste(Arc::new(
                                "    ".to_string(),
                            )))
                        } else {
                            let normalized_c = normalize_persian_char(*c);
                            text_editor::Action::Edit(text_editor::Edit::Insert(normalized_c))
                        }
                    }
                    text_editor::Action::Edit(text_editor::Edit::Paste(arc_str)) => {
                        let replaced = arc_str.replace('\t', "    ");
                        let normalized_str = normalize_persian_string(&replaced);
                        text_editor::Action::Edit(text_editor::Edit::Paste(Arc::new(
                            normalized_str,
                        )))
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

            Message::FileExported(path_opt, format) => {
                if let Some(path) = path_opt {
                    let text = self.content.text();
                    let title = path
                        .file_stem()
                        .map(|s| s.to_string_lossy().to_string())
                        .unwrap_or_else(|| "Krebs Export".to_string());
                    let lang = self.lang();

                    let result = match format {
                        ExportFormat::Pdf => export::export_pdf(
                            &text,
                            &title,
                            &path,
                            i18n::t(lang, i18n::Key::PageLabel),
                        ),
                        ExportFormat::Markdown => export::export_markdown_to_file(
                            &text,
                            &title,
                            &self.config.language,
                            &path,
                        )
                        .map_err(|e| e.to_string()),
                        ExportFormat::Html => {
                            export::export_to_file(&text, &title, &path).map_err(|e| e.to_string())
                        }
                    };

                    match result {
                        Ok(()) => {
                            if format != ExportFormat::Markdown {
                                if let Err(e) = export::open_exported_html(&path) {
                                    self.notify(e);
                                }
                            }
                            self.notify(i18n::t(lang, i18n::Key::Exported));
                        }
                        Err(_) => {
                            self.notify(i18n::t(lang, i18n::Key::ExportFailed));
                        }
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
                    if self.is_dirty && self.last_edit_time.elapsed() >= Duration::from_millis(1500)
                    {
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
                                    if elapsed_since_save.as_secs() < 3
                                        && self.last_saved_time.elapsed() > Duration::from_secs(4)
                                    {
                                        self.external_modification_detected = true;
                                    }
                                }
                            }
                        }
                    }
                } else {
                    if self.is_dirty
                        && self.last_checkpoint_time.elapsed() >= Duration::from_secs(5)
                    {
                        let text = self.content.text();
                        if let Some(rec_path) = RecoveryManager::save_untitled_checkpoint(
                            &text,
                            &mut self.active_recovery_file,
                        ) {
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

            Message::KeyboardEvent(key, modifiers) => self.handle_keyboard_shortcut(key, modifiers),

            Message::ModifiersChanged(modifiers) => {
                self.keyboard_modifiers = modifiers;
                Task::none()
            }

            Message::MouseScrolled(delta, _modifiers) => {
                if self.keyboard_modifiers.control() {
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
                    let _ = RecoveryManager::save_untitled_checkpoint(
                        &current_text,
                        &mut self.active_recovery_file,
                    );
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
                        .add_filter(
                            "Markdown & Text Files",
                            &[
                                "md", "markdown", "mdown", "mkd", "txt", "text", "note", "org",
                                "rst",
                            ],
                        )
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

            SidebarAction::ExportHtml => {
                self.show_export_modal = !self.show_export_modal;
                Task::none()
            }

            SidebarAction::ToggleStudyModal => self.update(Message::ToggleStudyModal),

            SidebarAction::ToggleLeftRail => {
                self.config.show_left_rail = !self.config.show_left_rail;
                let _ = self.config.save();
                Task::none()
            }

            SidebarAction::ToggleAiCoach => {
                self.open_agent_modal();
                Task::none()
            }

            SidebarAction::ToggleRightDrawer => {
                self.config.show_right_drawer = !self.config.show_right_drawer;
                if self.config.show_right_drawer
                    && self.right_drawer_tab != RightDrawerTab::PromptStudio
                {
                    self.right_drawer_tab = RightDrawerTab::Redemittel;
                }
                let _ = self.config.save();
                Task::none()
            }

            SidebarAction::ToggleShortcutsModal => self.update(Message::ToggleTourModal),

            SidebarAction::FormatHeading1 => {
                insert_text_at_cursor(&mut self.content, "# ");
                self.touch_content();
                Task::none()
            }

            SidebarAction::FormatHeading2 => {
                insert_text_at_cursor(&mut self.content, "## ");
                self.touch_content();
                Task::none()
            }

            SidebarAction::FormatBold => {
                wrap_markdown_syntax(&mut self.content, "**", "**", "bold text");
                self.touch_content();
                Task::none()
            }

            SidebarAction::FormatItalic => {
                wrap_markdown_syntax(&mut self.content, "*", "*", "italic text");
                self.touch_content();
                Task::none()
            }

            SidebarAction::FormatStrikethrough => {
                wrap_markdown_syntax(&mut self.content, "~~", "~~", "strikethrough");
                self.touch_content();
                Task::none()
            }

            SidebarAction::FormatBlockquote => {
                insert_text_at_cursor(&mut self.content, "> ");
                self.touch_content();
                Task::none()
            }

            SidebarAction::FormatList => {
                insert_text_at_cursor(&mut self.content, "- ");
                self.touch_content();
                Task::none()
            }

            SidebarAction::FormatCode => {
                wrap_markdown_syntax(&mut self.content, "`", "`", "code");
                self.touch_content();
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
                self.touch_content();
                Task::none()
            }

            SidebarAction::InsertSnippet(snippet_text) => {
                insert_text_at_cursor(&mut self.content, &snippet_text);
                self.touch_content();
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
                    return self.update(Message::ToggleExportModal);
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
                Key::Character(ref s) if s.eq_ignore_ascii_case("k") => {
                    return self.update(Message::ToggleStudyModal);
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
                Key::Character(ref s) if s == "2" && modifiers.shift() => {
                    insert_zwnj_at_cursor(&mut self.content);
                    self.touch_content();
                    self.notify(i18n::t(self.lang(), i18n::Key::ZwnjInserted));
                    return Task::none();
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
                    self.touch_content();
                    return Task::none();
                }
                Key::Character(ref s) if s.eq_ignore_ascii_case("i") => {
                    wrap_markdown_syntax(&mut self.content, "*", "*", "italic text");
                    self.touch_content();
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
                _ => {}
            }
        }

        if modifiers.shift() && matches!(key, Key::Named(keyboard::key::Named::Space)) {
            insert_zwnj_at_cursor(&mut self.content);
            self.notify(i18n::t(self.lang(), i18n::Key::ZwnjInserted));
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
            if self.show_agent_modal {
                self.show_agent_modal = false;
                self.commit_ai_settings();
                return Task::none();
            }
            if self.show_tour_modal {
                self.show_tour_modal = false;
                return Task::none();
            }
            if self.show_theme_modal {
                self.show_theme_modal = false;
                return Task::none();
            }
            if self.show_lang_modal {
                self.show_lang_modal = false;
                return Task::none();
            }
            if self.show_export_modal {
                self.show_export_modal = false;
                return Task::none();
            }
            if self.study.is_some() {
                self.study = None;
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
                    self.touch_content();
                    return Task::none();
                }
                Key::Character(ref s) if s.eq_ignore_ascii_case("o") => {
                    let ch = if modifiers.shift() { "Ö" } else { "ö" };
                    insert_text_at_cursor(&mut self.content, ch);
                    self.touch_content();
                    return Task::none();
                }
                Key::Character(ref s) if s.eq_ignore_ascii_case("u") => {
                    let ch = if modifiers.shift() { "Ü" } else { "ü" };
                    insert_text_at_cursor(&mut self.content, ch);
                    self.touch_content();
                    return Task::none();
                }
                Key::Character(ref s) if s.eq_ignore_ascii_case("s") => {
                    insert_text_at_cursor(&mut self.content, "ß");
                    self.touch_content();
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
            Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. }) => {
                Some(Message::KeyboardEvent(key, modifiers))
            }
            Event::Keyboard(keyboard::Event::ModifiersChanged(modifiers)) => {
                Some(Message::ModifiersChanged(modifiers))
            }
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
        let base_font_size = self.config.font_size * self.zoom_level;

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
            let elapsed = self
                .timer_total_secs
                .saturating_sub(self.timer_remaining_secs) as f32;
            let pct = (elapsed / self.timer_total_secs as f32).clamp(0.0, 1.0);
            Some(
                container(row![
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
                ])
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
                    self.show_agent_modal,
                    self.lang(),
                )
                .map(Message::Sidebar),
            )
        } else {
            None
        };

        // Right Exam & Hermes AI Assistant Drawer
        let maybe_right_drawer =
            if self.config.show_right_drawer && self.app_mode == AppMode::Writing {
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

        if self.show_agent_modal {
            root_stack = root_stack.push(
                container(self.view_agent_modal())
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .center(Length::Fill)
                    .style(|_t| container::Style {
                        background: Some(iced::Background::Color(Color::from_rgba(
                            0.0, 0.0, 0.0, 0.72,
                        ))),
                        ..Default::default()
                    }),
            );
        }

        if self.show_shortcuts_modal {
            root_stack = root_stack.push(
                container(self.view_shortcuts_modal())
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .center(Length::Fill)
                    .style(|_t| container::Style {
                        background: Some(iced::Background::Color(Color::from_rgba(
                            0.0, 0.0, 0.0, 0.65,
                        ))),
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
                        background: Some(iced::Background::Color(Color::from_rgba(
                            0.0, 0.0, 0.0, 0.65,
                        ))),
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
                        background: Some(iced::Background::Color(Color::from_rgba(
                            0.0, 0.0, 0.0, 0.65,
                        ))),
                        ..Default::default()
                    }),
            );
        }

        if self.show_lang_modal {
            root_stack = root_stack.push(
                container(self.view_lang_modal())
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .center(Length::Fill)
                    .style(|_t| container::Style {
                        background: Some(iced::Background::Color(Color::from_rgba(
                            0.0, 0.0, 0.0, 0.65,
                        ))),
                        ..Default::default()
                    }),
            );
        }

        if self.show_export_modal {
            root_stack = root_stack.push(
                container(self.view_export_modal())
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .center(Length::Fill)
                    .style(|_t| container::Style {
                        background: Some(iced::Background::Color(Color::from_rgba(
                            0.0, 0.0, 0.0, 0.65,
                        ))),
                        ..Default::default()
                    }),
            );
        }

        if self.study.is_some() {
            root_stack = root_stack.push(
                container(self.view_study_modal())
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .center(Length::Fill)
                    .style(|_t| container::Style {
                        background: Some(iced::Background::Color(Color::from_rgba(
                            0.0, 0.0, 0.0, 0.65,
                        ))),
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
                        background: Some(iced::Background::Color(Color::from_rgba(
                            0.0, 0.0, 0.0, 0.70,
                        ))),
                        ..Default::default()
                    }),
            );
        }

        root_stack.into()
    }

    fn view_writing_studio<'a>(
        &'a self,
        base_font_size: f32,
        active_font: Font,
    ) -> Element<'a, Message> {
        let lang = self.lang();
        // Text Editor Widget
        let editor_widget = text_editor(&self.content)
            .font(active_font)
            .size(base_font_size)
            .style(text_editor_style)
            .wrapping(iced::widget::text::Wrapping::Word)
            .height(Length::Fill)
            .placeholder(i18n::t(lang, i18n::Key::EditorPlaceholder))
            .on_action(Message::EditorAction);

        // Rendered Markdown View (supports Bold, Italic, Quotes, Lists, Headings)
        let rendered_markdown = scrollable(markdown_view::render_markdown(
            &self.content.text(),
            active_font,
            self.zoom_level,
            self.lang(),
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
                            i18n::t(lang, i18n::Key::ShrinkPane),
                        ),
                        with_tooltip(
                            button(center(
                                text(format!("{}:{}", editor_portion, preview_portion))
                                    .size(9)
                                    .color(ACCENT_CYAN)
                            ))
                            .on_press(Message::CycleSplitPresets)
                            .style(rail_button_style)
                            .width(42)
                            .height(24),
                            i18n::t(lang, i18n::Key::SplitPresets),
                        ),
                        with_tooltip(
                            button(center(text("▶").size(10).color(TEXT_MUTED)))
                                .on_press(Message::AdjustSplitRatio(0.05))
                                .style(rail_button_style)
                                .width(20)
                                .height(32),
                            i18n::t(lang, i18n::Key::GrowPane),
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
            let lang = self.lang();
            Some(
                container(
                    row![
                        text(format!("⚠️ {}", i18n::t(lang, i18n::Key::FileModified)))
                            .size(12)
                            .color(ACCENT_AMBER),
                        Space::with_width(Length::Fill),
                        button(text(i18n::t(lang, i18n::Key::ReloadFromDisk)).size(11))
                            .on_press(Message::ReloadFromDisk)
                            .style(rail_button_style)
                            .padding(iced::Padding::new(4.0)),
                        Space::with_width(8),
                        button(text(i18n::t(lang, i18n::Key::KeepLocal)).size(11))
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
        let lang = self.lang();

        let is_de = self.right_drawer_tab == RightDrawerTab::Redemittel;
        let de_tab_style = if is_de {
            tab_active_button_style
        } else {
            tab_inactive_button_style
        };
        let btn_de_tab = button(
            row![
                render_svg_icon(SVG_TIMER, 14, if is_de { accent } else { TEXT_MUTED }),
                text(i18n::t(lang, i18n::Key::ExamGoals)).size(12),
            ]
            .spacing(5)
            .align_y(Alignment::Center),
        )
        .on_press(Message::SetRightDrawerTab(RightDrawerTab::Redemittel))
        .style(de_tab_style)
        .padding(iced::Padding {
            top: 4.0,
            bottom: 4.0,
            left: 10.0,
            right: 10.0,
        });

        let is_prompt = self.right_drawer_tab == RightDrawerTab::PromptStudio;
        let prompt_tab_style = if is_prompt {
            tab_active_button_style
        } else {
            tab_inactive_button_style
        };
        let btn_prompt_tab = button(
            row![
                render_svg_icon(SVG_AI, 14, if is_prompt { accent } else { TEXT_MUTED }),
                text(i18n::t(lang, i18n::Key::PromptStudio)).size(12),
            ]
            .spacing(5)
            .align_y(Alignment::Center),
        )
        .on_press(Message::SetRightDrawerTab(RightDrawerTab::PromptStudio))
        .style(prompt_tab_style)
        .padding(iced::Padding {
            top: 4.0,
            bottom: 4.0,
            left: 10.0,
            right: 10.0,
        });

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
            btn_de_tab,
            btn_prompt_tab,
            Space::with_width(Length::Fill),
            btn_shrink,
            btn_expand,
            btn_close,
        ]
        .spacing(4)
        .align_y(Alignment::Center);

        let drawer_body: Element<'a, Message> = match self.right_drawer_tab {
            RightDrawerTab::Redemittel => self.view_redemittel_drawer_content(),
            RightDrawerTab::PromptStudio => self.view_prompt_drawer_content(),
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

    fn view_redemittel_drawer_content<'a>(&'a self) -> Element<'a, Message> {
        let is_in_range = self.stats.word_count >= self.config.target_min_words
            && self.stats.word_count <= self.config.target_max_words;
        let badge_color = if self.stats.word_count == 0 {
            TEXT_MUTED
        } else if is_in_range {
            ACCENT_GREEN
        } else {
            ACCENT_AMBER
        };

        let target_row = row![
            text(format!(
                "Target: {} - {} words",
                self.config.target_min_words, self.config.target_max_words
            ))
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

        let mut snippets_col = column![text(i18n::t(self.lang(), i18n::Key::ConnectorTitle))
            .size(9.5)
            .color(TEXT_MUTED),]
        .spacing(6);

        for cat in &self.snippets.categories {
            let mut cat_col = column![text(&cat.name).size(10.5).color(ACCENT_AMBER),].spacing(3);

            for item in &cat.items {
                let snippet_text = item.text.clone();
                let chip = button(text(&item.label).size(11).color(TEXT_PRIMARY))
                    .on_press(Message::Sidebar(SidebarAction::InsertSnippet(snippet_text)))
                    .style(rail_button_style)
                    .padding(iced::Padding {
                        top: 4.0,
                        bottom: 4.0,
                        left: 8.0,
                        right: 8.0,
                    })
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

    /// Prompt Studio & MCP hub inside the right drawer.
    fn view_prompt_drawer_content<'a>(&'a self) -> Element<'a, Message> {
        let lang = self.lang();
        let prompt = &self.prompt;

        let mut col = column![text(i18n::t(lang, i18n::Key::PromptStudio))
            .size(11)
            .color(TEXT_MUTED)]
        .spacing(8);

        // ---- Builder fields --------------------------------------------------
        let fields: [(prompt::PromptField, &str, &str, &str); 5] = [
            (
                prompt::PromptField::Persona,
                i18n::t(lang, i18n::Key::PromptPersona),
                "You are a meticulous German B2 grader…",
                &prompt.spec.persona,
            ),
            (
                prompt::PromptField::Task,
                i18n::t(lang, i18n::Key::PromptTask),
                "Grade the attached essay…",
                &prompt.spec.task,
            ),
            (
                prompt::PromptField::Context,
                i18n::t(lang, i18n::Key::PromptContext),
                "Relevant files, facts, audience…",
                &prompt.spec.context,
            ),
            (
                prompt::PromptField::Constraints,
                i18n::t(lang, i18n::Key::PromptConstraints),
                "One rule per line",
                &prompt.spec.constraints,
            ),
            (
                prompt::PromptField::OutputFormat,
                i18n::t(lang, i18n::Key::PromptOutput),
                "Markdown, 3 sections, ≤ 400 words…",
                &prompt.spec.output_format,
            ),
        ];
        for (field, label, placeholder, value) in fields {
            col = col.push(text(label).size(10).color(ACCENT_CYAN));
            col = col.push(
                text_input(placeholder, value)
                    .on_input(move |s| Message::PromptSetField(field, s))
                    .size(12)
                    .style(text_input_style)
                    .padding(6),
            );
        }

        // ---- Live counters --------------------------------------------------
        let rendered = self.prompt_pack();
        col = col.push(
            text(format!(
                "{} · {} chars · ~{} tok",
                i18n::t(lang, i18n::Key::PromptPack),
                rendered.chars().count(),
                prompt::estimate_tokens(&rendered)
            ))
            .size(10)
            .color(TEXT_MUTED),
        );

        // ---- MCP tool toggles ------------------------------------------------
        col = col.push(text(i18n::t(lang, i18n::Key::PromptTools)).size(10).color(TEXT_MUTED));
        let mut tools_row = row![].spacing(4);
        for (index, tool) in prompt::MCP_TOOLS.iter().enumerate() {
            let on = prompt.tools.get(index).copied().unwrap_or(false);
            let btn = button(
                text(tool.name).size(9).color(if on {
                    ACCENT_CYAN
                } else {
                    TEXT_MUTED
                }),
            )
            .on_press(Message::PromptToggleTool(index))
            .style(if on {
                active_tool_button_style
            } else {
                rail_button_style
            })
            .padding(iced::Padding {
                top: 3.0,
                bottom: 3.0,
                left: 6.0,
                right: 6.0,
            });
            tools_row = tools_row.push(btn);
        }
        col = col.push(tools_row.wrap());

        // ---- Action buttons --------------------------------------------------
        let improve_btn = if prompt.is_improving {
            button(text("⏳").size(12)).style(rail_button_style)
        } else {
            button(text(i18n::t(lang, i18n::Key::PromptImprove)).size(11).color(ACCENT_CYAN))
                .on_press(Message::PromptImprove)
                .style(rail_button_style)
        }
        .padding(6);
        let run_btn = if prompt.is_running {
            button(text("⏳").size(12)).style(rail_button_style)
        } else {
            button(text(i18n::t(lang, i18n::Key::PromptRun)).size(11).color(ACCENT_GREEN))
                .on_press(Message::PromptRunTest)
                .style(rail_button_style)
        }
        .padding(6);
        let copy_pack_btn =
            button(text(i18n::t(lang, i18n::Key::PromptCopyPack)).size(11).color(TEXT_PRIMARY))
                .on_press(Message::PromptCopyPack)
                .style(rail_button_style)
                .padding(6);
        col = col.push(row![improve_btn, run_btn, copy_pack_btn].spacing(6));

        // ---- Copy for coding agents ----------------------------------------
        col = col.push(text(i18n::t(lang, i18n::Key::PromptCopyFor)).size(10).color(TEXT_MUTED));
        let mut targets_row = row![].spacing(4);
        for target in prompt::CopyTarget::ALL {
            targets_row = targets_row.push(
                button(text(target.label()).size(10).color(TEXT_PRIMARY))
                    .on_press(Message::PromptCopyFor(target))
                    .style(rail_button_style)
                    .padding(iced::Padding {
                        top: 3.0,
                        bottom: 3.0,
                        left: 6.0,
                        right: 6.0,
                    }),
            );
        }
        col = col.push(targets_row);

        // ---- Results --------------------------------------------------------
        if !prompt.improved_text.is_empty() {
            col = col.push(
                button(
                    text(i18n::t(lang, i18n::Key::PromptCopyImproved))
                        .size(10)
                        .color(ACCENT_CYAN),
                )
                .on_press(Message::PromptCopyImproved)
                .style(rail_button_style)
                .padding(4),
            );
            col = col.push(
                container(
                    scrollable(
                        text(&prompt.improved_text)
                            .size(10.5)
                            .color(TEXT_PRIMARY),
                    )
                    .height(140),
                )
                .padding(8)
                .width(Length::Fill)
                .style(card_container_style),
            );
        }
        if !prompt.run_text.is_empty() {
            col = col.push(
                container(
                    scrollable(text(&prompt.run_text).size(10.5).color(TEXT_PRIMARY)).height(140),
                )
                .padding(8)
                .width(Length::Fill)
                .style(card_container_style),
            );
        }

        scrollable(col).height(Length::Fill).into()
    }

    fn view_top_header<'a>(&'a self) -> Element<'a, Message> {
        let accent = self.accent_choice.color();
        let lang = self.lang();

        // ---- App Logo/Wordmark ----------------------------------------------
        let logo_mark = container(
            row![
                // Logo icon
                container(render_svg_icon(SVG_CRAB, 20, accent))
                    .padding(iced::Padding {
                        top: 2.0,
                        bottom: 2.0,
                        left: 2.0,
                        right: 6.0,
                    }),
                // App name
                column![
                    text("Krebs").size(15).color(TEXT_PRIMARY).font(Font {
                        family: font::Family::Name("Inter"),
                        weight: font::Weight::Bold,
                        ..Default::default()
                    }),
                    text(match self.app_mode {
                        AppMode::Writing => "Studio",
                        AppMode::Whiteboard => "Whiteboard",
                    })
                    .size(9)
                    .color(TEXT_MUTED)
                    .font(Font {
                        family: font::Family::Name("Inter"),
                        weight: font::Weight::Normal,
                        ..Default::default()
                    }),
                ]
                .spacing(0)
                .align_x(Alignment::Start),
            ]
            .spacing(0)
            .align_y(Alignment::Center),
        )
        .padding(iced::Padding {
            left: 8.0,
            right: 12.0,
            top: 4.0,
            bottom: 4.0,
        });

        // ---- 1. Studio mode chips (Write / Whiteboard) ----------------------
        let is_writing = self.app_mode == AppMode::Writing;
        let btn_write = button(center(
            row![
                render_svg_icon(SVG_WRITE, 16, if is_writing { accent } else { TEXT_MUTED }),
                text(i18n::t(lang, i18n::Key::Write)).size(11.5),
            ]
            .spacing(5)
            .align_y(Alignment::Center),
        ))
        .on_press(Message::SetAppMode(AppMode::Writing))
        .height(30)
        .style(if is_writing {
            tab_active_button_style
        } else {
            tab_inactive_button_style
        })
        .padding(iced::Padding {
            left: 10.0,
            right: 10.0,
            ..Default::default()
        });

        let is_whiteboard = self.app_mode == AppMode::Whiteboard;
        let btn_board = button(center(
            row![
                render_svg_icon(
                    SVG_WHITEBOARD,
                    16,
                    if is_whiteboard { accent } else { TEXT_MUTED }
                ),
                text(i18n::t(lang, i18n::Key::Whiteboard)).size(11.5),
            ]
            .spacing(5)
            .align_y(Alignment::Center),
        ))
        .on_press(Message::SetAppMode(AppMode::Whiteboard))
        .height(30)
        .style(if is_whiteboard {
            tab_active_button_style
        } else {
            tab_inactive_button_style
        })
        .padding(iced::Padding {
            left: 10.0,
            right: 10.0,
            ..Default::default()
        });

        // ---- 2. View mode chips (Edit / Split / Preview) --------------------
        let view_chip = |mode: ViewMode, icon: &'static [u8], label: &'static str| {
            let active = self.view_mode == mode;
            button(center(
                row![
                    render_svg_icon(icon, 15, if active { accent } else { TEXT_MUTED }),
                    text(label).size(11),
                ]
                .spacing(4)
                .align_y(Alignment::Center),
            ))
            .on_press(Message::SetViewMode(mode))
            .height(30)
            .style(if active {
                tab_active_button_style
            } else {
                tab_inactive_button_style
            })
            .padding(iced::Padding {
                left: 8.0,
                right: 8.0,
                ..Default::default()
            })
        };

        let make_separator = || container(Space::with_height(18))
            .width(1)
            .style(|_t| container::Style {
                background: Some(iced::Background::Color(BORDER_COLOR)),
                ..Default::default()
            });

        let mut left_cluster = row![logo_mark, make_separator(), btn_write, btn_board].spacing(4);
        if self.app_mode == AppMode::Writing {
            left_cluster = left_cluster
                .push(make_separator())
                .push(with_tooltip(
                    view_chip(ViewMode::Edit, SVG_WRITE, i18n::t(lang, i18n::Key::Edit)),
                    i18n::t(lang, i18n::Key::DescViews),
                ))
                .push(with_tooltip(
                    view_chip(ViewMode::Split, SVG_SPLIT, i18n::t(lang, i18n::Key::LiveSplit)),
                    i18n::t(lang, i18n::Key::SplitPresets),
                ))
                .push(with_tooltip(
                    view_chip(ViewMode::Preview, SVG_PREVIEW, i18n::t(lang, i18n::Key::Preview)),
                    i18n::t(lang, i18n::Key::DescViews),
                ));
        }
        let left_cluster = left_cluster.align_y(Alignment::Center);

        // ---- 3. Draggable center (window drag) -------------------------------
        let drag_center = mouse_area(
            container(Space::new(Length::Fill, Length::Fill))
                .center_x(Length::Fill)
                .width(Length::Fill),
        )
        .on_press(Message::WindowDrag);

        // ---- 4. Utilities ----------------------------------------------------
        let agent_connected = self.agent.test_status.as_ref().map(|(ok, _)| *ok) == Some(true);
        let btn_agent = button(center(
            row![
                render_svg_icon(SVG_ROBOT, 16, Color::WHITE),
                text(format!(
                    "{} · {}",
                    i18n::t(lang, i18n::Key::Agent),
                    self.ai_state.provider.label()
                ))
                .size(11.5)
                .color(Color::WHITE),
                if agent_connected {
                    text("●").size(9).color(ACCENT_GREEN)
                } else {
                    text("").size(9)
                },
            ]
            .spacing(5)
            .align_y(Alignment::Center),
        ))
        .on_press(Message::ToggleAgentModal)
        .height(30)
        .style(accent_button_style)
        .padding(iced::Padding {
            left: 10.0,
            right: 10.0,
            ..Default::default()
        });

        let icon_btn = |svg: &'static [u8], size: u16, msg: Message| {
            button(center(render_svg_icon(svg, size, TEXT_MUTED)))
                .on_press(msg)
                .width(28)
                .height(28)
                .style(rail_button_style)
        };

        let btn_font = with_tooltip(
            icon_btn(SVG_FONT, 17, Message::ToggleFontModal),
            i18n::t(lang, i18n::Key::Typeface),
        );
        let btn_theme = with_tooltip(
            icon_btn(SVG_THEME, 17, Message::ToggleThemeModal),
            i18n::t(lang, i18n::Key::ThemeSelect),
        );
        let btn_lang = with_tooltip(
            icon_btn(SVG_GLOBE, 17, Message::ToggleLangModal),
            i18n::t(lang, i18n::Key::LanguageTitle),
        );
        let btn_help = with_tooltip(
            icon_btn(SVG_HELP, 16, Message::ToggleTourModal),
            i18n::t(lang, i18n::Key::TourHelp),
        );
        let btn_zen_hide = with_tooltip(
            icon_btn(SVG_EYE, 17, Message::ToggleTopToolbar),
            i18n::t(lang, i18n::Key::ZenHide),
        );

        let mins = self.timer_remaining_secs / 60;
        let secs = self.timer_remaining_secs % 60;
        let clock_str = format!("{:02}:{:02}", mins, secs);
        let clock_color = if self.timer_is_running {
            accent
        } else {
            TEXT_MUTED
        };
        let btn_clock = button(center(
            row![
                render_svg_icon(SVG_TIMER, 14, clock_color),
                text(clock_str).size(11).color(clock_color),
            ]
            .spacing(4)
            .align_y(Alignment::Center),
        ))
        .on_press(Message::ZenTimerToggle)
        .height(30)
        .style(rail_button_style)
        .padding(iced::Padding {
            left: 8.0,
            right: 8.0,
            ..Default::default()
        });

        let btn_clock_presets = with_tooltip(
            button(center(render_svg_icon(SVG_CHEVRON_DOWN, 11, TEXT_MUTED)))
                .on_press(Message::ZenTimerTogglePresets)
                .width(24)
                .height(28)
                .style(rail_button_style),
            i18n::t(lang, i18n::Key::TimerPresets),
        );

        // Window controls with resize support
        let btn_min = button(center(render_svg_icon(SVG_MINIMIZE, 13, TEXT_MUTED)))
            .on_press(Message::WindowMinimize)
            .style(rail_button_style)
            .width(28)
            .height(28);
        let btn_max = button(center(render_svg_icon(SVG_FULLSCREEN, 13, TEXT_MUTED)))
            .on_press(Message::WindowToggleMaximize)
            .style(rail_button_style)
            .width(28)
            .height(28);
        let btn_close = button(center(render_svg_icon(SVG_CLOSE, 14, TEXT_MUTED)))
            .on_press(Message::WindowClose)
            .style(rail_button_style)
            .width(28)
            .height(28);

        let mut right_cluster = row![btn_agent, Space::with_width(4)]
            .spacing(3)
            .align_y(Alignment::Center);
        right_cluster = right_cluster
            .push(btn_font)
            .push(btn_theme)
            .push(btn_lang)
            .push(btn_clock)
            .push(btn_clock_presets)
            .push(btn_help)
            .push(btn_zen_hide)
            .push(Space::with_width(4))
            .push(with_tooltip(btn_min, i18n::t(lang, i18n::Key::MinimizeWin)))
            .push(with_tooltip(btn_max, i18n::t(lang, i18n::Key::MaximizeWin)))
            .push(with_tooltip(btn_close, i18n::t(lang, i18n::Key::CloseWin)));

        if self.show_timer_presets {
            let preset = |label: &'static str, secs: u64| {
                button(center(text(label).size(10.5)))
                    .on_press(Message::ZenTimerSet(secs))
                    .height(28)
                    .style(rail_button_style)
                    .padding(iced::Padding::new(6.0))
            };
            let rst = button(center(render_svg_icon(SVG_RESET, 12, TEXT_MUTED)))
                .on_press(Message::Sidebar(SidebarAction::TimerReset))
                .width(26)
                .height(28)
                .style(rail_button_style);

            right_cluster = right_cluster.push(
                row![
                    with_tooltip(preset("15m", 900), i18n::t(lang, i18n::Key::Focus15)),
                    with_tooltip(preset("25m", 1500), i18n::t(lang, i18n::Key::Focus25)),
                    with_tooltip(preset("45m", 2700), i18n::t(lang, i18n::Key::Focus45)),
                    with_tooltip(preset("60m", 3600), i18n::t(lang, i18n::Key::Focus60)),
                    with_tooltip(rst, i18n::t(lang, i18n::Key::ResetTimer)),
                ]
                .spacing(2)
                .align_y(Alignment::Center),
            );
        }

        let header_row = row![left_cluster, drag_center, right_cluster]
            .width(Length::Fill)
            .align_y(Alignment::Center);

        container(header_row)
            .width(Length::Fill)
            .padding(iced::Padding {
                top: 4.0,
                bottom: 4.0,
                left: 8.0,
                right: 8.0,
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

    /// Centered Agent Studio: connect a model and direct it as a language teacher.
    fn view_agent_modal<'a>(&'a self) -> Element<'a, Message> {
        let lang = self.lang();
        let accent = self.accent_choice.color();

        let (status_text, status_color) = match self.agent.test_status.as_ref() {
            Some((true, _)) => (i18n::t(lang, i18n::Key::Connected), ACCENT_GREEN),
            Some((false, _)) => (i18n::t(lang, i18n::Key::NotConnected), ACCENT_RED),
            None => (i18n::t(lang, i18n::Key::NotConnected), TEXT_MUTED),
        };

        let close_btn = button(center(render_svg_icon(SVG_CLOSE, 14, TEXT_MUTED)))
            .on_press(Message::CloseAgentModal)
            .style(rail_button_style)
            .width(30)
            .height(30);

        let title_row = row![
            render_svg_icon(SVG_CRAB, 22, accent),
            column![
                text(i18n::t(lang, i18n::Key::AgentStudio))
                    .size(15)
                    .color(TEXT_PRIMARY),
                text(format!(
                    "{} · {}",
                    self.ai_state.provider.full_name(),
                    status_text
                ))
                .size(10)
                .color(status_color),
            ]
            .spacing(2),
            Space::with_width(Length::Fill),
            close_btn,
        ]
        .spacing(10)
        .align_y(Alignment::Center);

        // ---- Connection column ----------------------------------------------
        let mut provider_row = row![].spacing(4);
        for provider in AiProvider::all() {
            let selected = self.ai_state.provider == *provider;
            provider_row = provider_row.push(
                button(
                    text(provider.label())
                        .size(10.5)
                        .color(if selected { accent } else { TEXT_PRIMARY }),
                )
                .on_press(Message::SelectAiProvider(*provider))
                .style(if selected {
                    tab_active_button_style
                } else {
                    rail_button_style
                })
                .padding(iced::Padding {
                    top: 4.0,
                    bottom: 4.0,
                    left: 7.0,
                    right: 7.0,
                }),
            );
        }

        let mut connect_col = column![
            text(i18n::t(lang, i18n::Key::Connect))
                .size(10)
                .color(TEXT_MUTED),
            provider_row.wrap(),
            text(format!(
                "{}  ·  {}  ·  {}",
                self.ai_state.provider.default_endpoint(),
                self.ai_state.provider.default_model(),
                if self.ai_state.provider.requires_api_key() {
                    "API key required"
                } else {
                    "local · no key"
                }
            ))
            .size(9.5)
            .color(TEXT_MUTED),
            text(i18n::t(lang, i18n::Key::AiEndpoint))
                .size(10)
                .color(TEXT_MUTED),
            text_input("http://localhost:11434/v1", &self.ai_state.temp_endpoint)
                .on_input(Message::AiSetEndpoint)
                .size(11.5)
                .style(text_input_style)
                .padding(7),
            text(i18n::t(lang, i18n::Key::ModelName))
                .size(10)
                .color(TEXT_MUTED),
            text_input("hermes3", &self.ai_state.temp_model)
                .on_input(Message::AiSetModel)
                .size(11.5)
                .style(text_input_style)
                .padding(7),
        ]
        .spacing(6);

        if self.ai_state.provider.requires_api_key() {
            connect_col = connect_col
                .push(
                    text(i18n::t(lang, i18n::Key::ApiKey))
                        .size(10)
                        .color(TEXT_MUTED),
                )
                .push(
                    text_input("sk-…", &self.ai_state.api_key)
                        .on_input(Message::AiApiKeyChanged)
                        .secure(true)
                        .size(11.5)
                        .style(text_input_style)
                        .padding(7),
                );
        }

        let test_btn = if self.agent.is_testing {
            button(center(text("⏳ …").size(11)))
                .style(rail_button_style)
                .padding(iced::Padding {
                    top: 6.0,
                    bottom: 6.0,
                    left: 12.0,
                    right: 12.0,
                })
        } else {
            button(center(
                row![
                    render_svg_icon(SVG_KEY, 14, accent),
                    text(i18n::t(lang, i18n::Key::Connect))
                        .size(11.5)
                        .color(accent),
                ]
                .spacing(6)
                .align_y(Alignment::Center),
            ))
            .on_press(Message::AgentTestConnection)
            .style(tab_active_button_style)
            .padding(iced::Padding {
                top: 6.0,
                bottom: 6.0,
                left: 12.0,
                right: 12.0,
            })
        };

        let status_line: Element<'a, Message> = match self.agent.test_status.as_ref() {
            Some((ok, detail)) => text(format!("{} {detail}", if *ok { "✓" } else { "✗" }))
                .size(9.5)
                .color(if *ok { ACCENT_GREEN } else { ACCENT_RED })
                .into(),
            None => text("Ollama runs fully offline: ollama run hermes3")
                .size(9.5)
                .color(TEXT_MUTED)
                .into(),
        };

        let connect_col = connect_col
            .push(row![test_btn].spacing(6))
            .push(status_line);

        let connect_col = column![
            connect_col,
            Space::with_height(Length::Fill),
            text(
                "Replies arrive in one piece, and nothing leaves your machine \
                 unless you pick a cloud model."
            )
            .size(9)
            .color(TEXT_MUTED),
        ]
        .spacing(8);

        // ---- Language teacher column ----------------------------------------
        let mut level_row = row![].spacing(4);
        for level in ["A1", "A2", "B1", "B2", "C1", "C2"] {
            let selected = self.config.tutor_level == level;
            level_row = level_row.push(
                button(
                    text(level)
                        .size(10.5)
                        .color(if selected { accent } else { TEXT_PRIMARY }),
                )
                .on_press(Message::TutorSetLevel(level.to_string()))
                .style(if selected {
                    tab_active_button_style
                } else {
                    rail_button_style
                })
                .padding(iced::Padding {
                    top: 3.0,
                    bottom: 3.0,
                    left: 8.0,
                    right: 8.0,
                }),
            );
        }

        let instruction = self.tutor_profile().instruction();
        let instruction_box =
            container(scrollable(text(instruction).size(10).color(TEXT_PRIMARY)).height(120))
                .padding(8)
                .width(Length::Fill)
                .style(card_container_style);

        let copy_instruction_btn = button(center(
            row![
                render_svg_icon(SVG_CHECK, 12, TEXT_MUTED),
                text(i18n::t(lang, i18n::Key::CopyInstruction)).size(10.5),
            ]
            .spacing(5)
            .align_y(Alignment::Center),
        ))
        .on_press(Message::CopyTeacherInstruction)
        .style(rail_button_style)
        .padding(iced::Padding {
            top: 4.0,
            bottom: 4.0,
            left: 8.0,
            right: 8.0,
        });

        let ask_btn = if self.agent.is_running {
            button(center(text("⏳").size(12)))
                .style(rail_button_style)
                .padding(8)
        } else {
            button(center(
                row![
                    render_svg_icon(SVG_AI, 14, ACCENT_GREEN),
                    text(i18n::t(lang, i18n::Key::AskTeacher))
                        .size(11)
                        .color(ACCENT_GREEN),
                ]
                .spacing(6)
                .align_y(Alignment::Center),
            ))
            .on_press(Message::TutorAsk)
            .style(rail_button_style)
            .padding(8)
        };

        let teacher_col = column![
            text(i18n::t(lang, i18n::Key::LanguageTeacher))
                .size(10)
                .color(TEXT_MUTED),
            text(i18n::t(lang, i18n::Key::TargetLanguage))
                .size(10)
                .color(TEXT_MUTED),
            text_input(
                "German, English, Persian, Spanish…",
                &self.config.tutor_language
            )
            .on_input(Message::TutorSetLanguage)
            .size(11.5)
            .style(text_input_style)
            .padding(7),
            text(i18n::t(lang, i18n::Key::Level))
                .size(10)
                .color(TEXT_MUTED),
            level_row,
            text(i18n::t(lang, i18n::Key::Goal))
                .size(10)
                .color(TEXT_MUTED),
            text_input(
                "Pass the B2 exam, write clearer emails, speak with confidence…",
                &self.config.tutor_goal,
            )
            .on_input(Message::TutorSetGoal)
            .size(11.5)
            .style(text_input_style)
            .padding(7),
            row![ask_btn, copy_instruction_btn]
                .spacing(6)
                .align_y(Alignment::Center),
            text(i18n::t(lang, i18n::Key::TeacherInstruction))
                .size(10)
                .color(TEXT_MUTED),
            instruction_box,
        ]
        .spacing(6);

        let mut body = column![
            row![
                container(connect_col)
                    .width(Length::FillPortion(44))
                    .padding(12)
                    .style(card_container_style),
                container(teacher_col)
                    .width(Length::FillPortion(56))
                    .padding(12)
                    .style(card_container_style),
            ]
            .spacing(12)
            .align_y(Alignment::Start),
        ]
        .spacing(10);

        if !self.agent.result.is_empty() {
            let result_actions = row![
                button(center(
                    row![
                        render_svg_icon(SVG_CHECK, 12, TEXT_MUTED),
                        text(i18n::t(lang, i18n::Key::CopyFeedback)).size(10.5),
                    ]
                    .spacing(5)
                    .align_y(Alignment::Center),
                ))
                .on_press(Message::CopyTutorResult)
                .style(rail_button_style)
                .padding(6),
                button(center(
                    row![
                        render_svg_icon(SVG_WRITE, 12, TEXT_MUTED),
                        text(i18n::t(lang, i18n::Key::InsertAtCursor)).size(10.5),
                    ]
                    .spacing(5)
                    .align_y(Alignment::Center),
                ))
                .on_press(Message::InsertTutorFeedback)
                .style(rail_button_style)
                .padding(6),
            ]
            .spacing(6);

            body = body.push(
                column![
                    row![
                        text(i18n::t(lang, i18n::Key::LanguageTeacher))
                            .size(10)
                            .color(TEXT_MUTED),
                        Space::with_width(Length::Fill),
                        result_actions,
                    ]
                    .align_y(Alignment::Center),
                    container(
                        scrollable(text(&self.agent.result).size(11).color(TEXT_PRIMARY)).height(200),
                    )
                    .padding(10)
                    .width(Length::Fill)
                    .style(card_container_style),
                ]
                .spacing(6),
            );
        }

        // ---- Quick checks on the current note --------------------------------
        let quick_chip = |action: AiAction| {
            let active = self.ai_state.last_action == Some(action) && !self.ai_state.is_loading;
            button(
                text(action.title())
                    .size(10.5)
                    .color(if active { accent } else { TEXT_PRIMARY }),
            )
            .on_press(Message::RunAiAction(action))
            .style(if active {
                tab_active_button_style
            } else {
                rail_button_style
            })
            .padding(iced::Padding {
                top: 4.0,
                bottom: 4.0,
                left: 8.0,
                right: 8.0,
            })
        };

        body = body.push(
            column![
                text("Quick checks on this note")
                    .size(10)
                    .color(TEXT_MUTED),
                row![
                    quick_chip(AiAction::GradeEssay),
                    quick_chip(AiAction::TipsAndFeedback),
                    quick_chip(AiAction::FixGrammar),
                    quick_chip(AiAction::GermanB2C1),
                ]
                .spacing(6)
                .wrap(),
            ]
            .spacing(6),
        );

        if self.ai_state.is_loading {
            body = body.push(
                text("✨ Analyzing your note…")
                    .size(11)
                    .color(accent),
            );
        } else if !self.ai_state.response_text.is_empty() {
            body = body.push(
                column![
                    row![
                        text(if self.ai_state.is_offline_fallback {
                            "Offline evaluator"
                        } else {
                            "Agent response"
                        })
                        .size(10)
                        .color(if self.ai_state.is_offline_fallback {
                            ACCENT_AMBER
                        } else {
                            ACCENT_GREEN
                        }),
                        Space::with_width(Length::Fill),
                        button(text(i18n::t(lang, i18n::Key::CopyFeedback)).size(10.5))
                            .on_press(Message::AiCopyFeedback)
                            .style(rail_button_style)
                            .padding(4),
                        button(text(i18n::t(lang, i18n::Key::InsertAtCursor)).size(10.5))
                            .on_press(Message::AiInsertFeedback)
                            .style(rail_button_style)
                            .padding(4),
                    ]
                    .align_y(Alignment::Center)
                    .spacing(6),
                    container(
                        scrollable(
                            text(&self.ai_state.response_text)
                                .size(11)
                                .color(TEXT_PRIMARY),
                        )
                        .height(180),
                    )
                    .padding(10)
                    .width(Length::Fill)
                    .style(card_container_style),
                ]
                .spacing(6),
            );
        }

        container(column![title_row, create_divider_simple(), body].spacing(12))
            .width(760)
            .max_height(640)
            .padding(18)
            .style(modal_container_style)
            .into()
    }

    fn view_search_bar<'a>(&'a self) -> Element<'a, Message> {
        let lang = self.lang();
        let input = text_input(
            i18n::t(lang, i18n::Key::FindPlaceholder),
            &self.search.query,
        )
        .on_input(Message::SearchInputChanged)
        .on_submit(Message::SearchNext)
        .style(text_input_style)
        .size(12)
        .width(220);

        let match_label = if self.search.query.trim().is_empty() {
            "".to_string()
        } else if self.search.matches.is_empty() {
            i18n::t(lang, i18n::Key::NoMatches).to_string()
        } else {
            format!(
                "{}/{}",
                self.search.current_idx + 1,
                self.search.matches.len()
            )
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
            with_tooltip(btn_prev, i18n::t(lang, i18n::Key::Previous)),
            with_tooltip(btn_next, i18n::t(lang, i18n::Key::Next)),
            with_tooltip(btn_close, i18n::t(lang, i18n::Key::Search)),
        ]
        .spacing(8)
        .align_y(Alignment::Center);

        container(search_row)
            .padding(iced::Padding {
                top: 6.0,
                bottom: 6.0,
                left: 12.0,
                right: 12.0,
            })
            .style(search_container_style)
            .into()
    }

    fn view_whiteboard_toolbar<'a>(&'a self) -> Element<'a, Message> {
        let lang = self.lang();
        let is_pen = self.whiteboard.active_tool == WhiteboardTool::Pen;
        let is_hl = self.whiteboard.active_tool == WhiteboardTool::Highlighter;
        let is_er = self.whiteboard.active_tool == WhiteboardTool::Eraser;

        let pen_color = if is_pen { ACCENT_CYAN } else { TEXT_MUTED };
        let btn_pen = button(
            row![
                render_svg_icon(SVG_PEN, 13, pen_color),
                text(i18n::t(lang, i18n::Key::Pen)).size(11),
            ]
            .spacing(4)
            .align_y(Alignment::Center),
        )
        .on_press(Message::WhiteboardToolSelect(WhiteboardTool::Pen))
        .style(if is_pen {
            active_tool_button_style
        } else {
            rail_button_style
        })
        .padding(iced::Padding {
            top: 3.0,
            bottom: 3.0,
            left: 8.0,
            right: 8.0,
        });

        let hl_color = if is_hl { ACCENT_CYAN } else { TEXT_MUTED };
        let btn_hl = button(
            row![
                render_svg_icon(SVG_HIGHLIGHTER, 13, hl_color),
                text(i18n::t(lang, i18n::Key::Highlighter)).size(11),
            ]
            .spacing(4)
            .align_y(Alignment::Center),
        )
        .on_press(Message::WhiteboardToolSelect(WhiteboardTool::Highlighter))
        .style(if is_hl {
            active_tool_button_style
        } else {
            rail_button_style
        })
        .padding(iced::Padding {
            top: 3.0,
            bottom: 3.0,
            left: 8.0,
            right: 8.0,
        });

        let er_color = if is_er { ACCENT_CYAN } else { TEXT_MUTED };
        let btn_eraser = button(
            row![
                render_svg_icon(SVG_ERASER, 13, er_color),
                text(i18n::t(lang, i18n::Key::Eraser)).size(11),
            ]
            .spacing(4)
            .align_y(Alignment::Center),
        )
        .on_press(Message::WhiteboardToolSelect(WhiteboardTool::Eraser))
        .style(if is_er {
            active_tool_button_style
        } else {
            rail_button_style
        })
        .padding(iced::Padding {
            top: 3.0,
            bottom: 3.0,
            left: 8.0,
            right: 8.0,
        });

        // Stroke Width chips
        let w_fine = button(text("1.5px").size(10))
            .on_press(Message::WhiteboardWidthSelect(1.5))
            .style(if (self.whiteboard.pen_width - 1.5).abs() < 0.1 {
                active_tool_button_style
            } else {
                rail_button_style
            })
            .padding(iced::Padding::new(3.0));

        let w_med = button(text("3.0px").size(10))
            .on_press(Message::WhiteboardWidthSelect(3.0))
            .style(if (self.whiteboard.pen_width - 3.0).abs() < 0.1 {
                active_tool_button_style
            } else {
                rail_button_style
            })
            .padding(iced::Padding::new(3.0));

        let w_bold = button(text("6.0px").size(10))
            .on_press(Message::WhiteboardWidthSelect(6.0))
            .style(if (self.whiteboard.pen_width - 6.0).abs() < 0.1 {
                active_tool_button_style
            } else {
                rail_button_style
            })
            .padding(iced::Padding::new(3.0));

        // Color chips
        let c_white = button(text("●").size(13).color(Color::from_rgb(0.88, 0.90, 0.94)))
            .on_press(Message::WhiteboardColorSelect(Color::from_rgb(
                0.88, 0.90, 0.94,
            )))
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
        let p_dots = button(text(i18n::t(lang, i18n::Key::PatternDots)).size(10))
            .on_press(Message::WhiteboardPatternSelect(PaperPattern::DotGrid))
            .style(if self.whiteboard.paper_pattern == PaperPattern::DotGrid {
                active_tool_button_style
            } else {
                rail_button_style
            })
            .padding(iced::Padding::new(3.0));

        let p_ruled = button(text(i18n::t(lang, i18n::Key::PatternRuled)).size(10))
            .on_press(Message::WhiteboardPatternSelect(PaperPattern::Ruled))
            .style(if self.whiteboard.paper_pattern == PaperPattern::Ruled {
                active_tool_button_style
            } else {
                rail_button_style
            })
            .padding(iced::Padding::new(3.0));

        let p_math = button(text(i18n::t(lang, i18n::Key::PatternMathGrid)).size(10))
            .on_press(Message::WhiteboardPatternSelect(PaperPattern::MathGrid))
            .style(if self.whiteboard.paper_pattern == PaperPattern::MathGrid {
                active_tool_button_style
            } else {
                rail_button_style
            })
            .padding(iced::Padding::new(3.0));

        let p_blank = button(text(i18n::t(lang, i18n::Key::PatternBlank)).size(10))
            .on_press(Message::WhiteboardPatternSelect(PaperPattern::Blank))
            .style(if self.whiteboard.paper_pattern == PaperPattern::Blank {
                active_tool_button_style
            } else {
                rail_button_style
            })
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
                text(i18n::t(lang, i18n::Key::WipeBoard)).size(11),
            ]
            .spacing(5)
            .align_y(Alignment::Center),
        )
        .on_press(Message::WhiteboardClear)
        .style(wipe_button_style)
        .padding(iced::Padding {
            top: 3.0,
            bottom: 3.0,
            left: 8.0,
            right: 8.0,
        });

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
            text(i18n::t(lang, i18n::Key::Paper))
                .size(10)
                .color(TEXT_MUTED),
            p_dots,
            p_ruled,
            p_math,
            p_blank,
            Space::with_width(Length::Fill),
            with_tooltip(btn_undo, i18n::t(lang, i18n::Key::UndoStroke)),
            with_tooltip(btn_redo, i18n::t(lang, i18n::Key::RedoStroke)),
            with_tooltip(btn_wipe, i18n::t(lang, i18n::Key::WipeClean)),
        ]
        .spacing(4)
        .align_y(Alignment::Center);

        container(tb_row)
            .width(Length::Fill)
            .padding(iced::Padding {
                top: 6.0,
                bottom: 6.0,
                left: 12.0,
                right: 12.0,
            })
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
        let mode_label = format!(
            "{} | {}% | {}",
            self.direction_mode.label(),
            zoom_pct,
            self.canvas_width_mode.label()
        );

        let maybe_toast = self
            .status_notification
            .as_ref()
            .map(|(msg, _)| text(msg).size(10).color(ACCENT_CYAN));

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
        let lang = self.lang();
        let title_row = row![
            text(i18n::t(lang, i18n::Key::SelectTypography))
                .size(15)
                .color(ACCENT_CYAN),
            Space::with_width(Length::Fill),
            button(center(render_svg_icon(SVG_CLOSE, 12, TEXT_MUTED)))
                .on_press(Message::CloseFontModal)
                .style(rail_button_style)
                .width(28)
                .height(28),
        ]
        .align_y(Alignment::Center);

        // Body text size, applied before zoom.
        let size_value = self.config.font_size.round() as u32;
        let size_row = row![
            text(i18n::t(lang, i18n::Key::TextSize))
                .size(10)
                .color(TEXT_MUTED),
            Space::with_width(Length::Fill),
            button(center(text("A−").size(12)))
                .on_press(Message::AdjustFontSize(-1.0))
                .style(rail_button_style)
                .width(34)
                .height(28),
            text(format!("{size_value} px"))
                .size(12)
                .color(TEXT_PRIMARY),
            button(center(text("A+").size(12)))
                .on_press(Message::AdjustFontSize(1.0))
                .style(rail_button_style)
                .width(34)
                .height(28),
        ]
        .spacing(8)
        .align_y(Alignment::Center);

        let font_items = [
            (
                FontChoice::JetBrainsMono,
                "JetBrains Mono",
                i18n::t(lang, i18n::Key::FontDescJetBrains),
            ),
            (
                FontChoice::Inter,
                "Inter",
                i18n::t(lang, i18n::Key::FontDescSans),
            ),
            (
                FontChoice::Lora,
                "Lora",
                i18n::t(lang, i18n::Key::FontDescSerif),
            ),
            (
                FontChoice::IbmPlexMono,
                "IBM Plex Mono",
                i18n::t(lang, i18n::Key::FontDescIbmPlex),
            ),
            (
                FontChoice::Vt323,
                "VT323 Retro CRT",
                i18n::t(lang, i18n::Key::FontDescVt323),
            ),
            (
                FontChoice::Vazirmatn,
                "Vazirmatn",
                i18n::t(lang, i18n::Key::FontDescVazirmatn),
            ),
            (
                FontChoice::Sans,
                "System Sans-Serif",
                i18n::t(lang, i18n::Key::FontDescSans),
            ),
            (
                FontChoice::Serif,
                "System Serif",
                i18n::t(lang, i18n::Key::FontDescSerif),
            ),
        ];

        let mut font_list = column![].spacing(4);
        for (choice, name, desc) in font_items {
            let is_selected = self.font_choice == choice;
            let item_btn = button(
                row![
                    container(
                        text("Aa")
                            .font(choice.to_font())
                            .size(19)
                            .color(if is_selected { ACCENT_CYAN } else { TEXT_PRIMARY })
                    )
                    .width(40),
                    column![
                        text(name).size(12.5).color(if is_selected {
                            ACCENT_CYAN
                        } else {
                            TEXT_PRIMARY
                        }),
                        text(desc).size(9.5).color(TEXT_MUTED),
                    ]
                    .spacing(1),
                    Space::with_width(Length::Fill),
                    if is_selected {
                        render_svg_icon(SVG_CHECK, 13, ACCENT_CYAN)
                    } else {
                        render_svg_icon(SVG_CHEVRON_DOWN, 13, Color::TRANSPARENT)
                    },
                ]
                .spacing(10)
                .align_y(Alignment::Center),
            )
            .on_press(Message::SelectFont(choice))
            .style(if is_selected {
                active_tool_button_style
            } else {
                rail_button_style
            })
            .padding(8)
            .width(Length::Fill);

            font_list = font_list.push(item_btn);
        }

        container(
            column![
                title_row,
                size_row,
                create_divider_simple(),
                scrollable(font_list).height(Length::Fixed(380.0)),
            ]
            .spacing(10),
        )
        .width(520)
        .padding(18)
        .style(modal_container_style)
        .into()
    }

    fn view_shortcuts_modal<'a>(&self) -> Element<'a, Message> {
        let lang = self.lang();
        let title_row = row![
            text(i18n::t(lang, i18n::Key::ShortcutsTitle))
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
            text(i18n::t(lang, i18n::Key::SecWorkspace))
                .size(10)
                .color(TEXT_MUTED),
            row![
                text("Ctrl + M").size(11).color(ACCENT_CYAN),
                Space::with_width(Length::Fill),
                text(i18n::t(lang, i18n::Key::DescToggleStudio))
                    .size(11)
                    .color(TEXT_PRIMARY)
            ],
            row![
                text("Ctrl + 1 / 2 / 3").size(11).color(ACCENT_CYAN),
                Space::with_width(Length::Fill),
                text(i18n::t(lang, i18n::Key::DescViews))
                    .size(11)
                    .color(TEXT_PRIMARY)
            ],
            row![
                text("Ctrl + W").size(11).color(ACCENT_CYAN),
                Space::with_width(Length::Fill),
                text(i18n::t(lang, i18n::Key::DescWidth))
                    .size(11)
                    .color(TEXT_PRIMARY)
            ],
            row![
                text("Ctrl + F").size(11).color(ACCENT_CYAN),
                Space::with_width(Length::Fill),
                text(i18n::t(lang, i18n::Key::DescSearch))
                    .size(11)
                    .color(TEXT_PRIMARY)
            ],
            row![
                text("Ctrl + N / O / S").size(11).color(ACCENT_CYAN),
                Space::with_width(Length::Fill),
                text(i18n::t(lang, i18n::Key::DescFileOps))
                    .size(11)
                    .color(TEXT_PRIMARY)
            ],
            row![
                text("Ctrl + P").size(11).color(ACCENT_CYAN),
                Space::with_width(Length::Fill),
                text(i18n::t(lang, i18n::Key::DescExport))
                    .size(11)
                    .color(TEXT_PRIMARY)
            ],
        ]
        .spacing(6);

        let format_section = column![
            text(i18n::t(lang, i18n::Key::SecMarkdown))
                .size(10)
                .color(TEXT_MUTED),
            row![
                text("Ctrl + B / I").size(11).color(ACCENT_CYAN),
                Space::with_width(Length::Fill),
                text(i18n::t(lang, i18n::Key::DescBoldItalic))
                    .size(11)
                    .color(TEXT_PRIMARY)
            ],
            row![
                text("Ctrl + + / - / 0").size(11).color(ACCENT_CYAN),
                Space::with_width(Length::Fill),
                text(i18n::t(lang, i18n::Key::DescZoom))
                    .size(11)
                    .color(TEXT_PRIMARY)
            ],
            row![
                text("Shift + Space").size(11).color(ACCENT_CYAN),
                Space::with_width(Length::Fill),
                text(i18n::t(lang, i18n::Key::DescZwnj))
                    .size(11)
                    .color(TEXT_PRIMARY)
            ],
            row![
                text("Alt + a/o/u/s").size(11).color(ACCENT_CYAN),
                Space::with_width(Length::Fill),
                text(i18n::t(lang, i18n::Key::DescGerman))
                    .size(11)
                    .color(TEXT_PRIMARY)
            ],
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
        let lang = self.lang();
        let rtl = self.is_rtl();
        let title_text = i18n::t(lang, i18n::Key::TourTitle);
        let close_btn = button(center(render_svg_icon(SVG_CLOSE, 12, TEXT_MUTED)))
            .on_press(Message::CloseTourModal)
            .style(rail_button_style)
            .width(28)
            .height(28);
        let title_row: Element<'a, Message> = if rtl {
            row![
                close_btn,
                Space::with_width(Length::Fill),
                text(title_text).size(15).color(ACCENT_CYAN)
            ]
            .align_y(Alignment::Center)
            .into()
        } else {
            row![
                text(title_text).size(15).color(ACCENT_CYAN),
                Space::with_width(Length::Fill),
                close_btn
            ]
            .align_y(Alignment::Center)
            .into()
        };

        let (step_title, step_desc, step_tag) = match self.tour_step {
            0 => (
                i18n::t(lang, i18n::Key::Tour1Title),
                i18n::t(lang, i18n::Key::Tour1Desc),
                i18n::t(lang, i18n::Key::Tour1Tag),
            ),
            1 => (
                i18n::t(lang, i18n::Key::Tour2Title),
                i18n::t(lang, i18n::Key::Tour2Desc),
                i18n::t(lang, i18n::Key::Tour2Tag),
            ),
            2 => (
                i18n::t(lang, i18n::Key::Tour3Title),
                i18n::t(lang, i18n::Key::Tour3Desc),
                i18n::t(lang, i18n::Key::Tour3Tag),
            ),
            3 => (
                i18n::t(lang, i18n::Key::Tour4Title),
                i18n::t(lang, i18n::Key::Tour4Desc),
                i18n::t(lang, i18n::Key::Tour4Tag),
            ),
            _ => (
                i18n::t(lang, i18n::Key::Tour5Title),
                i18n::t(lang, i18n::Key::Tour5Desc),
                i18n::t(lang, i18n::Key::Tour5Tag),
            ),
        };

        let tour_counter = text(i18n::step_counter(lang, self.tour_step + 1, 5))
            .size(11)
            .color(TEXT_MUTED);

        let content_card = column![
            text(step_title).size(14).color(TEXT_PRIMARY),
            text(step_desc)
                .size(12)
                .color(Color::from_rgb(0.80, 0.84, 0.90)),
            container(text(step_tag).size(11).color(ACCENT_AMBER))
                .padding(8)
                .width(Length::Fill)
                .style(card_container_style),
        ]
        .spacing(10);

        let mut btn_prev = button(text(i18n::t(lang, i18n::Key::TourPrevious)).size(11))
            .style(rail_button_style)
            .padding(iced::Padding {
                top: 6.0,
                bottom: 6.0,
                left: 12.0,
                right: 12.0,
            });
        if self.tour_step > 0 {
            btn_prev = btn_prev.on_press(Message::TourPrev);
        }

        let next_label = if self.tour_step < 4 {
            i18n::t(lang, i18n::Key::TourNext)
        } else {
            i18n::t(lang, i18n::Key::TourGetStarted)
        };
        let next_msg = if self.tour_step < 4 {
            Message::TourNext
        } else {
            Message::TourFinish
        };
        let btn_next = button(text(next_label).size(11))
            .on_press(next_msg)
            .style(tab_active_button_style)
            .padding(iced::Padding {
                top: 6.0,
                bottom: 6.0,
                left: 14.0,
                right: 14.0,
            });

        let footer_row = row![
            tour_counter,
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
        let lang = self.lang();
        let title_row = row![
            text(i18n::t(lang, i18n::Key::CustomizeTheme))
                .size(15)
                .color(ACCENT_CYAN),
            Space::with_width(Length::Fill),
            button(center(render_svg_icon(SVG_CLOSE, 12, TEXT_MUTED)))
                .on_press(Message::CloseThemeModal)
                .style(rail_button_style)
                .width(28)
                .height(28),
        ]
        .align_y(Alignment::Center);

        let themes = [
            (
                ThemeMode::OmarchyDark,
                "Omarchy Dark",
                i18n::t(lang, i18n::Key::ThemeOmarchy),
            ),
            (
                ThemeMode::OledBlack,
                "OLED Pure Black",
                i18n::t(lang, i18n::Key::ThemeOled),
            ),
            (
                ThemeMode::DeepSlate,
                "Deep Slate",
                i18n::t(lang, i18n::Key::ThemeDeepSlate),
            ),
            (
                ThemeMode::WarmEspresso,
                "Warm Espresso",
                i18n::t(lang, i18n::Key::ThemeEspresso),
            ),
        ];

        let mut theme_list = column![text(i18n::t(lang, i18n::Key::CanvasMood))
            .size(10)
            .color(TEXT_MUTED),]
        .spacing(6);

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
                        text(name)
                            .size(12)
                            .color(if is_sel { ACCENT_CYAN } else { TEXT_PRIMARY }),
                        text(desc).size(10).color(TEXT_MUTED),
                    ]
                    .spacing(2),
                    Space::with_width(Length::Fill),
                    if is_sel {
                        text("✓").size(12).color(ACCENT_CYAN)
                    } else {
                        text("").size(12)
                    },
                ]
                .spacing(10)
                .align_y(Alignment::Center),
            )
            .on_press(Message::SelectTheme(tm))
            .style(if is_sel {
                active_tool_button_style
            } else {
                rail_button_style
            })
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
                                color: if is_sel {
                                    Color::WHITE
                                } else {
                                    Color::TRANSPARENT
                                },
                                width: 2.0,
                                radius: iced::border::Radius::from(9.0),
                            },
                            ..Default::default()
                        }),
                    text(name)
                        .size(9)
                        .color(if is_sel { Color::WHITE } else { TEXT_MUTED }),
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
            text(i18n::t(lang, i18n::Key::AccentColor))
                .size(10)
                .color(TEXT_MUTED),
            accent_row,
        ]
        .spacing(6);

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

    /// Language selector modal: ten autonym buttons, RTL-aware layout,
    /// persisted instantly via `Message::SelectLanguage`.
    fn view_lang_modal<'a>(&self) -> Element<'a, Message> {
        let lang = self.lang();
        let rtl = self.is_rtl();

        let title_text = i18n::t(lang, i18n::Key::LanguageTitle);
        let close_btn = button(center(render_svg_icon(SVG_CLOSE, 12, TEXT_MUTED)))
            .on_press(Message::CloseLangModal)
            .style(rail_button_style)
            .width(28)
            .height(28);

        // Mirrored header for right-to-left languages: close button on the left.
        let title_row: Element<'a, Message> = if rtl {
            row![
                close_btn,
                Space::with_width(Length::Fill),
                text(title_text).size(15).color(ACCENT_CYAN)
            ]
            .align_y(Alignment::Center)
            .into()
        } else {
            row![
                text(title_text).size(15).color(ACCENT_CYAN),
                Space::with_width(Length::Fill),
                close_btn
            ]
            .align_y(Alignment::Center)
            .into()
        };

        let mut lang_list = column![].spacing(6);
        for option in i18n::Lang::ALL {
            let is_sel = lang == option;
            let rtl_badge = if i18n::is_rtl(option.code()) {
                " ⇄"
            } else {
                ""
            };
            let label = format!("{}{} — {}", option.code(), rtl_badge, option.display());
            let btn =
                button(
                    text(label)
                        .size(13)
                        .color(if is_sel { ACCENT_CYAN } else { TEXT_PRIMARY }),
                )
                .on_press(Message::SelectLanguage(option))
                .style(if is_sel {
                    active_tool_button_style
                } else {
                    rail_button_style
                })
                .padding(9)
                .width(Length::Fill);

            lang_list = lang_list.push(btn);
        }

        let modal_col = column![title_row, create_divider_simple(), lang_list].spacing(12);

        container(modal_col)
            .width(380)
            .padding(18)
            .style(modal_container_style)
            .into()
    }

    /// Unified export modal: PDF, Markdown and HTML in one place.
    fn view_export_modal<'a>(&self) -> Element<'a, Message> {
        let lang = self.lang();
        let rtl = self.is_rtl();

        let title_text = i18n::t(lang, i18n::Key::ExportTitle);
        let close_btn = button(center(render_svg_icon(SVG_CLOSE, 12, TEXT_MUTED)))
            .on_press(Message::CloseExportModal)
            .style(rail_button_style)
            .width(28)
            .height(28);

        // Mirrored header for right-to-left languages: close button on the left.
        let title_row: Element<'a, Message> = if rtl {
            row![
                close_btn,
                Space::with_width(Length::Fill),
                text(title_text).size(15).color(ACCENT_CYAN)
            ]
            .align_y(Alignment::Center)
            .into()
        } else {
            row![
                text(title_text).size(15).color(ACCENT_CYAN),
                Space::with_width(Length::Fill),
                close_btn
            ]
            .align_y(Alignment::Center)
            .into()
        };

        let entries: [(&str, &'static str, Message); 4] = [
            (
                "📄",
                i18n::t(lang, i18n::Key::ExportPdf),
                Message::ExportPdf,
            ),
            (
                "📝",
                i18n::t(lang, i18n::Key::ExportMarkdown),
                Message::ExportMarkdown,
            ),
            (
                "🌐",
                i18n::t(lang, i18n::Key::ExportHtml),
                Message::ExportHtml,
            ),
            (
                "📋",
                i18n::t(lang, i18n::Key::CopyMarkdown),
                Message::CopyMarkdown,
            ),
        ];
        let mut actions = column![].spacing(6);
        for (icon, label, msg) in entries {
            let btn = button(
                row![
                    text(icon).size(14),
                    text(label).size(13).color(TEXT_PRIMARY)
                ]
                .spacing(10)
                .align_y(Alignment::Center),
            )
            .on_press(msg)
            .style(rail_button_style)
            .padding(9)
            .width(Length::Fill);
            actions = actions.push(btn);
        }

        let hint = text(i18n::t(lang, i18n::Key::ExportHint))
            .size(11)
            .color(TEXT_MUTED);

        let modal_col = column![title_row, create_divider_simple(), actions, hint].spacing(12);

        container(modal_col)
            .width(420)
            .padding(18)
            .style(modal_container_style)
            .into()
    }

    /// Kartei study overlay: review the due flashcards of the current note.
    fn view_study_modal<'a>(&self) -> Element<'a, Message> {
        let lang = self.lang();
        let rtl = self.is_rtl();

        let Some(review) = &self.study else {
            return column![].into();
        };

        let title_text = i18n::t(lang, i18n::Key::StudyTitle);
        let close_btn = button(center(render_svg_icon(SVG_CLOSE, 12, TEXT_MUTED)))
            .on_press(Message::CloseStudyModal)
            .style(rail_button_style)
            .width(28)
            .height(28);

        // Mirrored header for right-to-left languages: close button on the left.
        let title_row: Element<'a, Message> = if rtl {
            row![
                close_btn,
                Space::with_width(Length::Fill),
                text(title_text).size(15).color(ACCENT_CYAN)
            ]
            .align_y(Alignment::Center)
            .into()
        } else {
            row![
                text(title_text).size(15).color(ACCENT_CYAN),
                Space::with_width(Length::Fill),
                close_btn
            ]
            .align_y(Alignment::Center)
            .into()
        };

        let body: Element<'a, Message> = if review.total_in_note == 0 {
            // The note contains no flashcards yet — explain the syntax.
            column![
                text("🃏").size(26),
                text(i18n::t(lang, i18n::Key::StudyNoCards))
                    .size(13)
                    .color(TEXT_PRIMARY),
                text(i18n::t(lang, i18n::Key::StudyHint))
                    .size(11)
                    .color(TEXT_MUTED)
            ]
            .spacing(10)
            .width(Length::Fill)
            .into()
        } else if review.is_done() {
            column![
                text("🎉").size(26),
                text(i18n::t(lang, i18n::Key::StudyDone))
                    .size(13)
                    .color(TEXT_PRIMARY),
            ]
            .spacing(10)
            .width(Length::Fill)
            .into()
        } else {
            let Some(card) = review.current() else {
                return column![].into();
            };
            let planned = review.planned_count();
            let step = review.index + 1;
            let q = card.question.replace('\n', " ");
            let a = card.answer.replace('\n', " ");
            let question_block = text(q).size(15).color(TEXT_PRIMARY).width(Length::Fill);
            let answer_area: Element<'a, Message> = if review.revealed {
                text(a)
                    .size(13)
                    .color(ACCENT_GREEN)
                    .width(Length::Fill)
                    .into()
            } else {
                button(
                    text(i18n::t(lang, i18n::Key::StudyShow))
                        .size(12)
                        .color(ACCENT_CYAN),
                )
                .on_press(Message::StudyReveal)
                .style(rail_button_style)
                .padding(9)
                .width(Length::Shrink)
                .into()
            };
            let grade_row: Element<'a, Message> = if review.revealed {
                row![
                    button(
                        text(i18n::t(lang, i18n::Key::StudyAgain))
                            .size(12)
                            .color(ACCENT_RED)
                    )
                    .on_press(Message::StudyGrade(study::Grade::Again))
                    .style(rail_button_style)
                    .padding(9),
                    button(
                        text(i18n::t(lang, i18n::Key::StudyGood))
                            .size(12)
                            .color(ACCENT_GREEN)
                    )
                    .on_press(Message::StudyGrade(study::Grade::Good))
                    .style(rail_button_style)
                    .padding(9),
                    button(
                        text(i18n::t(lang, i18n::Key::StudyEasy))
                            .size(12)
                            .color(ACCENT_CYAN)
                    )
                    .on_press(Message::StudyGrade(study::Grade::Easy))
                    .style(rail_button_style)
                    .padding(9),
                ]
                .spacing(8)
                .into()
            } else {
                Space::with_height(0).into()
            };
            column![
                text(format!("{step} / {planned}"))
                    .size(11)
                    .color(TEXT_MUTED),
                question_block,
                create_divider_simple(),
                answer_area,
                grade_row
            ]
            .spacing(10)
            .width(Length::Fill)
            .into()
        };

        let modal_col = column![title_row, create_divider_simple(), body].spacing(12);

        container(modal_col)
            .width(480)
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
            (FontChoice::Inter, "inter"),
            (FontChoice::Lora, "lora"),
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
        app.content = text_editor::Content::with_text(
            "Rust is fast.\nWriting in Rust is awesome.\nPure Rust!",
        );
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
        let _ = app.update(Message::Sidebar(SidebarAction::InsertSnippet(
            "In der heutigen Zeit".to_string(),
        )));
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

        // 13. Test the centered Agent Studio and the drawer tabs
        app.config.show_right_drawer = false;
        let _ = app.update(Message::ToggleAiDrawer);
        assert!(app.show_agent_modal);
        let _ = app.update(Message::CloseAgentModal);
        assert!(!app.show_agent_modal);
        let _ = app.update(Message::SetRightDrawerTab(RightDrawerTab::Redemittel));
        assert_eq!(app.right_drawer_tab, RightDrawerTab::Redemittel);
        let _ = app.update(Message::SetRightDrawerTab(RightDrawerTab::PromptStudio));
        assert_eq!(app.right_drawer_tab, RightDrawerTab::PromptStudio);

        // Teacher profile drives a non-empty agent instruction.
        let _ = app.update(Message::TutorSetLanguage("German".to_string()));
        let _ = app.update(Message::TutorSetLevel("C1".to_string()));
        let _ = app.update(Message::TutorSetGoal("debate fluently".to_string()));
        let instruction = app.tutor_profile().instruction();
        assert!(instruction.contains("Language Teacher Agent"));
        assert!(instruction.contains("C1"));
        assert!(instruction.contains("debate fluently"), "{instruction}");

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
