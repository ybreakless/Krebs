pub mod config;
pub mod editor;
pub mod export;
pub mod sidebar;
pub mod theme;

use std::borrow::Cow;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

use iced::event::{self, Event};
use iced::font;
use iced::keyboard::{self, Key, Modifiers};
use iced::mouse;
use iced::widget::{
    button, center, column, container, row, scrollable, stack, text, text_editor, Space,
};
use iced::window;
use iced::{Alignment, Element, Font, Length, Size, Subscription, Task, Theme};

use config::{AppConfig, RecoveryManager, SnippetsConfig, SnippetsDeManager};
use editor::{
    apply_smart_typography, detect_paragraph_direction, insert_text_at_cursor,
    insert_zwnj_at_cursor, normalize_persian_char, normalize_persian_string, wrap_markdown_syntax,
    DirectionMode, DocumentStats, EditorHistory,
};
use sidebar::{view_left_rail, view_right_drawer, SidebarAction};
use theme::{
    banner_container_style, canvas_container_style, card_container_style, modal_container_style,
    rail_button_style, text_editor_style, ACCENT_AMBER, ACCENT_CYAN, ACCENT_GREEN, BORDER_COLOR,
    CANVAS_BG, TEXT_MUTED, TEXT_PRIMARY,
};

// Bundled Fonts
pub static IBM_PLEX_MONO_REGULAR: &[u8] = include_bytes!("../fonts/IBMPlexMono-Regular.ttf");
pub static IBM_PLEX_MONO_BOLD: &[u8] = include_bytes!("../fonts/IBMPlexMono-Bold.ttf");
pub static VAZIRMATN_REGULAR: &[u8] = include_bytes!("../fonts/Vazirmatn-Regular.ttf");
pub static VAZIRMATN_BOLD: &[u8] = include_bytes!("../fonts/Vazirmatn-Bold.ttf");

pub fn font_ibm_plex_mono() -> Font {
    Font {
        family: font::Family::Name("IBM Plex Mono"),
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

struct OmaWrite {
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
    is_german_bar_open: bool,
    status_notification: Option<(String, Instant)>,
    stats: DocumentStats,
    zoom_level: f32,
    direction_mode: DirectionMode,
}

#[derive(Debug, Clone)]
enum Message {
    EditorAction(text_editor::Action),
    Sidebar(SidebarAction),
    FileOpened(Option<PathBuf>),
    FileSaved(Option<PathBuf>),
    FileExported(Option<PathBuf>),
    Tick,
    KeyboardEvent(Key, Modifiers),
    MouseScrolled(f32, Modifiers),
    ReloadFromDisk,
    DismissExternalBanner,
    CloseShortcutsModal,
    FontLoaded(Result<(), iced::font::Error>),
}

impl OmaWrite {
    fn new(initial_file: Option<PathBuf>) -> (Self, Task<Message>) {
        let config = AppConfig::load_or_default();
        let snippets = SnippetsDeManager::load_or_create_default();
        let direction_mode = DirectionMode::from_str_name(&config.direction_mode);
        let zoom_level = config.zoom_level.clamp(0.5, 3.0);
        let timer_remaining_secs = config.timer_duration_secs;
        let timer_total_secs = config.timer_duration_secs;

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

        // If no file loaded, check for recent crash recovery
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
            is_german_bar_open: false,
            status_notification: None,
            stats,
            zoom_level,
            direction_mode,
        };

        // Load all bundled fonts into fontdb
        let load_fonts_task = Task::batch(vec![
            font::load(Cow::Borrowed(IBM_PLEX_MONO_REGULAR)).map(Message::FontLoaded),
            font::load(Cow::Borrowed(IBM_PLEX_MONO_BOLD)).map(Message::FontLoaded),
            font::load(Cow::Borrowed(VAZIRMATN_REGULAR)).map(Message::FontLoaded),
            font::load(Cow::Borrowed(VAZIRMATN_BOLD)).map(Message::FontLoaded),
        ]);

        (app, load_fonts_task)
    }

    fn notify(&mut self, text: impl Into<String>) {
        self.status_notification = Some((text.into(), Instant::now()));
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::FontLoaded(_) => Task::none(),

            Message::EditorAction(action) => {
                let prev_text = self.content.text();

                // Intercept character insertion for Persian & German smart typing
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

                    // Smart typography check (e.g. -- to –)
                    if self.config.smart_dashes || self.config.smart_quotes_de {
                        let transformed = apply_smart_typography(
                            &new_text,
                            self.config.smart_quotes_de,
                            self.config.smart_dashes,
                        );
                        if transformed != new_text {
                            let (line, col) = self.content.cursor_position();
                            self.content = text_editor::Content::with_text(&transformed);
                            let _ = line;
                            let _ = col;
                        }
                    }
                }

                Task::none()
            }

            Message::Sidebar(sidebar_action) => self.handle_sidebar_action(sidebar_action),

            Message::FileOpened(path_opt) => {
                if let Some(path) = path_opt {
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        self.content = text_editor::Content::with_text(&content);
                        self.stats = DocumentStats::compute(&content);
                        self.current_file_path = Some(path.clone());
                        self.is_dirty = false;
                        self.last_saved_time = Instant::now();
                        self.external_modification_detected = false;
                        self.config.add_recent_file(&path.to_string_lossy());
                        self.notify(format!("Opened {}", path.file_name().unwrap_or_default().to_string_lossy()));
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
                        self.config.add_recent_file(&path.to_string_lossy());

                        // If an untitled checkpoint existed, clean it up
                        if let Some(rec) = self.active_recovery_file.take() {
                            RecoveryManager::cleanup_checkpoint(&rec);
                        }

                        self.notify(format!("Saved {}", path.file_name().unwrap_or_default().to_string_lossy()));
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
                        .unwrap_or_else(|| "OmaWrite Export".to_string());

                    if export::export_to_file(&text, &title, &path).is_ok() {
                        let _ = export::open_exported_html(&path);
                        self.notify("Exported and opened in browser");
                    }
                }
                Task::none()
            }

            Message::Tick => {
                // Countdown timer decrement
                if self.timer_is_running && self.timer_remaining_secs > 0 {
                    self.timer_remaining_secs -= 1;
                    if self.timer_remaining_secs == 0 {
                        self.timer_is_running = false;
                        self.notify("⏱ Exam time is up!");
                    }
                }

                // Debounced auto-save for explicitly saved files (1.5 seconds)
                if let Some(path) = self.current_file_path.clone() {
                    if self.is_dirty && self.last_edit_time.elapsed() >= Duration::from_millis(1500) {
                        let text = self.content.text();
                        if std::fs::write(&path, &text).is_ok() {
                            self.is_dirty = false;
                            self.last_saved_time = Instant::now();
                            self.notify("Auto-saved");
                        }
                    }

                    // Check for external modification on disk
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
                    // Untitled buffer silent checkpoint every 5 seconds
                    if self.is_dirty && self.last_checkpoint_time.elapsed() >= Duration::from_secs(5) {
                        let text = self.content.text();
                        if let Some(rec_path) = RecoveryManager::save_untitled_checkpoint(&text, &mut self.active_recovery_file) {
                            self.active_recovery_file = Some(rec_path);
                            self.last_checkpoint_time = Instant::now();
                        }
                    }
                }

                // Clear temporary status notification after 3 seconds
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
                        .add_filter("Markdown & Text", &["md", "markdown", "txt"])
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
                                .add_filter("Markdown", &["md"])
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
                self.stats = DocumentStats::compute(&self.content.text());
                Task::none()
            }

            SidebarAction::FormatHeading2 => {
                insert_text_at_cursor(&mut self.content, "## ");
                self.stats = DocumentStats::compute(&self.content.text());
                Task::none()
            }

            SidebarAction::FormatBold => {
                wrap_markdown_syntax(&mut self.content, "**", "**", "bold text");
                self.stats = DocumentStats::compute(&self.content.text());
                Task::none()
            }

            SidebarAction::FormatItalic => {
                wrap_markdown_syntax(&mut self.content, "*", "*", "italic text");
                self.stats = DocumentStats::compute(&self.content.text());
                Task::none()
            }

            SidebarAction::FormatStrikethrough => {
                wrap_markdown_syntax(&mut self.content, "~~", "~~", "strikethrough");
                self.stats = DocumentStats::compute(&self.content.text());
                Task::none()
            }

            SidebarAction::FormatBlockquote => {
                insert_text_at_cursor(&mut self.content, "> ");
                self.stats = DocumentStats::compute(&self.content.text());
                Task::none()
            }

            SidebarAction::FormatList => {
                insert_text_at_cursor(&mut self.content, "- ");
                self.stats = DocumentStats::compute(&self.content.text());
                Task::none()
            }

            SidebarAction::FormatCode => {
                wrap_markdown_syntax(&mut self.content, "`", "`", "code");
                self.stats = DocumentStats::compute(&self.content.text());
                Task::none()
            }

            SidebarAction::CycleDirection => {
                self.direction_mode = self.direction_mode.next();
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
                    return self.handle_sidebar_action(SidebarAction::FormatBold);
                }
                Key::Character(ref s) if s.eq_ignore_ascii_case("i") => {
                    return self.handle_sidebar_action(SidebarAction::FormatItalic);
                }
                Key::Character(ref s) if s.eq_ignore_ascii_case("x") && modifiers.shift() => {
                    return self.handle_sidebar_action(SidebarAction::FormatStrikethrough);
                }
                Key::Character(ref s) if s == "2" && modifiers.shift() => {
                    insert_zwnj_at_cursor(&mut self.content);
                    self.notify("Inserted Persian ZWNJ (نیم‌فاصله)");
                    return Task::none();
                }
                Key::Character(ref s) if s == "/" || s == "?" => {
                    self.show_shortcuts_modal = !self.show_shortcuts_modal;
                    return Task::none();
                }
                Key::Character(ref s) if s.eq_ignore_ascii_case("z") => {
                    if modifiers.shift() {
                        let cur = self.content.text();
                        if let Some(redo_text) = self.history.redo(cur) {
                            self.content = text_editor::Content::with_text(&redo_text);
                            self.stats = DocumentStats::compute(&redo_text);
                        }
                    } else {
                        let cur = self.content.text();
                        if let Some(undo_text) = self.history.undo(cur) {
                            self.content = text_editor::Content::with_text(&undo_text);
                            self.stats = DocumentStats::compute(&undo_text);
                        }
                    }
                    return Task::none();
                }
                Key::Character(ref s) if s.eq_ignore_ascii_case("y") => {
                    let cur = self.content.text();
                    if let Some(redo_text) = self.history.redo(cur) {
                        self.content = text_editor::Content::with_text(&redo_text);
                        self.stats = DocumentStats::compute(&redo_text);
                    }
                    return Task::none();
                }
                _ => {}
            }
        }

        // Persian ZWNJ: Shift + Space
        if modifiers.shift() && matches!(key, Key::Named(keyboard::key::Named::Space)) {
            insert_zwnj_at_cursor(&mut self.content);
            self.notify("Inserted Persian ZWNJ (نیم‌فاصله)");
            return Task::none();
        }

        // F1: Toggle Shortcuts Modal
        if matches!(key, Key::Named(keyboard::key::Named::F1)) {
            self.show_shortcuts_modal = !self.show_shortcuts_modal;
            return Task::none();
        }

        // Escape: Close modal / external banner
        if matches!(key, Key::Named(keyboard::key::Named::Escape)) {
            if self.show_shortcuts_modal {
                self.show_shortcuts_modal = false;
                return Task::none();
            }
            if self.external_modification_detected {
                self.external_modification_detected = false;
                return Task::none();
            }
        }

        // German chord shortcuts on Alt:
        // Alt + a/o/u/s -> ä, ö, ü, ß (or with Shift -> Ä, Ö, Ü)
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
            _ => None,
        });

        Subscription::batch(vec![tick_sub, event_sub])
    }

    fn view(&self) -> Element<'_, Message> {
        let base_font_size = 15.0 * self.zoom_level;

        // Determine font family: If document is Persian or RTL mode is active, prioritize Vazirmatn
        let current_text = self.content.text();
        let is_rtl = match self.direction_mode {
            DirectionMode::Rtl => true,
            DirectionMode::Ltr => false,
            DirectionMode::Auto => {
                let first_line = current_text.lines().next().unwrap_or("");
                detect_paragraph_direction(first_line) == unicode_bidi::Direction::Rtl
            }
        };

        let active_font = if is_rtl {
            font_vazirmatn()
        } else {
            font_ibm_plex_mono()
        };

        // Text Editor Widget
        let editor_widget = text_editor(&self.content)
            .font(active_font)
            .size(base_font_size)
            .style(text_editor_style)
            .placeholder("No vaults, no plugins, just you and the words...\nBegin writing your essay or notes here.")
            .on_action(Message::EditorAction);

        // Center writing column (72ch max-width)
        let canvas_max_width = (self.config.canvas_max_width * 10.0 * self.zoom_level).clamp(450.0, 920.0);

        let center_column = container(editor_widget)
            .max_width(canvas_max_width)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(iced::Padding {
                top: 40.0,
                bottom: 40.0,
                left: 20.0,
                right: 20.0,
            });

        // External modification banner (if triggered)
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

        // Status notification banner / toast (e.g. "Auto-saved", "Inserted ZWNJ")
        let maybe_toast = self.status_notification.as_ref().map(|(msg, _)| {
            container(
                text(msg)
                    .size(11)
                    .color(ACCENT_CYAN),
            )
            .padding(iced::Padding {
                top: 3.0,
                bottom: 3.0,
                left: 10.0,
                right: 10.0,
            })
            .style(|_t| container::Style {
                background: Some(iced::Background::Color(iced::Color::from_rgba(0.04, 0.05, 0.06, 0.85))),
                border: iced::Border {
                    color: ACCENT_CYAN,
                    width: 1.0,
                    radius: iced::border::Radius::from(12.0),
                },
                ..Default::default()
            })
        });

        // Minimal Bottom Status Bar
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
        let mode_label = format!("{} | {}%", self.direction_mode.label(), zoom_pct);

        let bottom_status_bar = container(
            row![
                text(file_label).size(10).color(TEXT_MUTED),
                Space::with_width(Length::Fill),
                text(stats_label).size(10).color(TEXT_MUTED),
                Space::with_width(Length::Fill),
                text(mode_label).size(10).color(TEXT_MUTED),
            ]
            .align_y(Alignment::Center),
        )
        .padding(iced::Padding {
            top: 4.0,
            bottom: 4.0,
            left: 14.0,
            right: 14.0,
        })
        .style(|_t| container::Style {
            background: Some(iced::Background::Color(CANVAS_BG)),
            border: iced::Border {
                color: BORDER_COLOR,
                width: 1.0,
                ..Default::default()
            },
            ..Default::default()
        })
        .height(24)
        .width(Length::Fill);

        // Center writing pane
        let mut center_stack = column![];
        if let Some(banner) = maybe_banner {
            center_stack = center_stack.push(banner);
        }

        let editor_scroll = scrollable(center(center_column))
            .width(Length::Fill)
            .height(Length::Fill);

        center_stack = center_stack.push(editor_scroll);
        center_stack = center_stack.push(bottom_status_bar);

        let center_pane = container(center_stack)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(canvas_container_style);

        // Main layout row: [Left Rail] + [Center Canvas] + [Right Drawer]
        let mut main_row = row![];

        if self.config.show_left_rail {
            let rail = view_left_rail(
                self.direction_mode,
                self.config.show_right_drawer,
                self.is_german_bar_open,
            )
            .map(Message::Sidebar);
            main_row = main_row.push(rail);
        }

        main_row = main_row.push(center_pane);

        if self.config.show_right_drawer {
            let drawer = view_right_drawer(
                self.timer_remaining_secs,
                self.timer_is_running,
                self.timer_total_secs,
                self.config.target_min_words,
                self.config.target_max_words,
                &self.stats,
                &self.snippets,
            )
            .map(Message::Sidebar);
            main_row = main_row.push(drawer);
        }

        let main_view: Element<'_, Message> = container(main_row)
            .width(Length::Fill)
            .height(Length::Fill)
            .into();

        // If shortcuts modal is active or toast is visible, render on top via stack!
        let mut root_stack = stack![main_view];

        if let Some(toast) = maybe_toast {
            let toast_layer = container(toast)
                .width(Length::Fill)
                .height(Length::Fill)
                .align_x(Alignment::Center)
                .align_y(Alignment::Start)
                .padding(iced::Padding {
                    top: 16.0,
                    bottom: 0.0,
                    left: 0.0,
                    right: 0.0,
                });
            root_stack = root_stack.push(toast_layer);
        }

        if self.show_shortcuts_modal {
            let modal = self.view_shortcuts_modal();
            let modal_backdrop = container(center(modal))
                .width(Length::Fill)
                .height(Length::Fill)
                .style(|_t| container::Style {
                    background: Some(iced::Background::Color(iced::Color::from_rgba(0.0, 0.0, 0.0, 0.65))),
                    ..Default::default()
                });
            root_stack = root_stack.push(modal_backdrop);
        }

        root_stack.into()
    }

    fn view_shortcuts_modal<'a>(&self) -> Element<'a, Message> {
        let title_row = row![
            text("⌨️ OmaWrite-RS Shortcuts & Trilingual Guide")
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
            text("FILE & NAVIGATION").size(10).color(TEXT_MUTED),
            row![text("Ctrl + N").size(11).color(ACCENT_CYAN), Space::with_width(Length::Fill), text("New Untitled Document").size(11).color(TEXT_PRIMARY)],
            row![text("Ctrl + O").size(11).color(ACCENT_CYAN), Space::with_width(Length::Fill), text("Open Document").size(11).color(TEXT_PRIMARY)],
            row![text("Ctrl + S").size(11).color(ACCENT_CYAN), Space::with_width(Length::Fill), text("Save Document").size(11).color(TEXT_PRIMARY)],
            row![text("Ctrl + P").size(11).color(ACCENT_CYAN), Space::with_width(Length::Fill), text("Export HTML / Print").size(11).color(TEXT_PRIMARY)],
            row![text("Ctrl + \\").size(11).color(ACCENT_CYAN), Space::with_width(Length::Fill), text("Toggle Left Icon Rail").size(11).color(TEXT_PRIMARY)],
            row![text("Ctrl + E").size(11).color(ACCENT_CYAN), Space::with_width(Length::Fill), text("Toggle Exam Trainer Panel").size(11).color(TEXT_PRIMARY)],
            row![text("Ctrl + = / - / 0").size(11).color(ACCENT_CYAN), Space::with_width(Length::Fill), text("Zoom Canvas / Reset").size(11).color(TEXT_PRIMARY)],
        ]
        .spacing(6);

        let markdown_section = column![
            text("MARKDOWN FORMATTING").size(10).color(TEXT_MUTED),
            row![text("Ctrl + B").size(11).color(ACCENT_CYAN), Space::with_width(Length::Fill), text("Bold (**text**)").size(11).color(TEXT_PRIMARY)],
            row![text("Ctrl + I").size(11).color(ACCENT_CYAN), Space::with_width(Length::Fill), text("Italic (*text*)").size(11).color(TEXT_PRIMARY)],
            row![text("Ctrl + Shift + X").size(11).color(ACCENT_CYAN), Space::with_width(Length::Fill), text("Strikethrough (~~text~~)").size(11).color(TEXT_PRIMARY)],
            row![text("Ctrl + Z / Y").size(11).color(ACCENT_CYAN), Space::with_width(Length::Fill), text("Undo / Redo Buffer").size(11).color(TEXT_PRIMARY)],
        ]
        .spacing(6);

        let persian_section = column![
            text("PERSIAN / FARSI (BiDi)").size(10).color(ACCENT_GREEN),
            row![text("Shift + Space").size(11).color(ACCENT_CYAN), Space::with_width(Length::Fill), text("Zero-Width Non-Joiner (نیم‌فاصله)").size(11).color(TEXT_PRIMARY)],
            row![text("Ctrl + Shift + 2").size(11).color(ACCENT_CYAN), Space::with_width(Length::Fill), text("Alternative ZWNJ shortcut").size(11).color(TEXT_PRIMARY)],
            row![text("Automatic").size(11).color(TEXT_MUTED), Space::with_width(Length::Fill), text("Normalizes Arabic ي/ك to Persian ی/ک").size(11).color(TEXT_PRIMARY)],
            row![text("Auto-BiDi").size(11).color(TEXT_MUTED), Space::with_width(Length::Fill), text("Persian paragraphs auto-align RTL").size(11).color(TEXT_PRIMARY)],
        ]
        .spacing(6);

        let german_section = column![
            text("GERMAN EXAM CHORDS & REDEMITTEL").size(10).color(ACCENT_AMBER),
            row![text("Alt + a / o / u / s").size(11).color(ACCENT_CYAN), Space::with_width(Length::Fill), text("Insert ä, ö, ü, ß").size(11).color(TEXT_PRIMARY)],
            row![text("Alt + Shift + A/O/U").size(11).color(ACCENT_CYAN), Space::with_width(Length::Fill), text("Insert Ä, Ö, Ü").size(11).color(TEXT_PRIMARY)],
            row![text("Typographic quotes").size(11).color(TEXT_MUTED), Space::with_width(Length::Fill), text("\"text\" auto-converts to „text“").size(11).color(TEXT_PRIMARY)],
            row![text("Gedankenstrich").size(11).color(TEXT_MUTED), Space::with_width(Length::Fill), text("-- auto-converts to en-dash –").size(11).color(TEXT_PRIMARY)],
        ]
        .spacing(6);

        let content_col = column![
            title_row,
            container(file_section).padding(10).style(card_container_style),
            container(markdown_section).padding(10).style(card_container_style),
            container(persian_section).padding(10).style(card_container_style),
            container(german_section).padding(10).style(card_container_style),
            row![
                Space::with_width(Length::Fill),
                button(text("Close (Esc)").size(11))
                    .on_press(Message::CloseShortcutsModal)
                    .style(rail_button_style)
                    .padding(iced::Padding::new(6.0)),
            ],
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

fn main() -> iced::Result {
    let args: Vec<String> = std::env::args().collect();
    let initial_file = if args.len() > 1 {
        Some(PathBuf::from(&args[1]))
    } else {
        None
    };

    let config = AppConfig::load_or_default();

    iced::application(
        "OmaWrite-RS — Portable Writing Studio",
        OmaWrite::update,
        OmaWrite::view,
    )
    .window(window::Settings {
        size: Size::new(config.window_width, config.window_height),
        min_size: Some(Size::new(640.0, 420.0)),
        ..Default::default()
    })
    .subscription(OmaWrite::subscription)
    .theme(OmaWrite::theme)
    .run_with(move || OmaWrite::new(initial_file))
}
