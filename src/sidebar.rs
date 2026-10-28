use iced::alignment::Horizontal;
use iced::widget::{button, column, container, row, text, Space};
use iced::{Alignment, Color, Element, Length};

use crate::config::SnippetsConfig;
use crate::editor::{DirectionMode, DocumentStats};
use crate::theme::{
    card_container_style, drawer_container_style,
    rail_button_active_style, rail_button_style, rail_container_style,
    ACCENT_AMBER, ACCENT_GREEN, BORDER_COLOR, TEXT_MUTED, TEXT_PRIMARY,
};

#[derive(Debug, Clone)]
pub enum SidebarAction {
    NewFile,
    OpenFile,
    SaveFile,
    ExportHtml,
    ToggleLeftRail,
    ToggleRightDrawer,
    ToggleShortcutsModal,
    ToggleFontModal,
    ToggleSearch,
    CycleWidthMode,
    CycleViewMode,
    ToggleAppMode,
    FormatHeading1,
    FormatHeading2,
    FormatBold,
    FormatItalic,
    FormatStrikethrough,
    FormatBlockquote,
    FormatList,
    FormatCode,
    CycleDirection,
    ToggleGermanBar,
    InsertGermanChar(&'static str),
    InsertSnippet(String),
    TimerStart,
    TimerPause,
    TimerReset,
    TimerSetDuration(u64),
    TargetMinChanged(usize),
    TargetMaxChanged(usize),
}

fn create_divider<'a>() -> Element<'a, SidebarAction> {
    container(Space::with_height(1))
        .style(|_t| container::Style {
            background: Some(iced::Background::Color(BORDER_COLOR)),
            ..Default::default()
        })
        .width(24)
        .height(1)
        .into()
}

/// Renders the left ultra-minimal vertical icon rail (42px wide).
pub fn view_left_rail<'a>(
    direction_mode: DirectionMode,
    is_exam_drawer_open: bool,
    is_german_bar_open: bool,
    is_whiteboard_active: bool,
    is_search_open: bool,
) -> Element<'a, SidebarAction> {
    // Mode switcher at very top: Write vs Whiteboard
    let mode_btn_style = if is_whiteboard_active {
        rail_button_active_style
    } else {
        rail_button_style
    };
    let btn_mode = button(
        text(if is_whiteboard_active { "🎨" } else { "📝" })
            .size(14)
            .align_x(Horizontal::Center),
    )
    .on_press(SidebarAction::ToggleAppMode)
    .style(mode_btn_style)
    .width(34)
    .height(34);

    let btn_new = button(text("+").size(16).align_x(Horizontal::Center))
        .on_press(SidebarAction::NewFile)
        .style(rail_button_style)
        .width(34)
        .height(34);

    let btn_open = button(text("📂").size(13).align_x(Horizontal::Center))
        .on_press(SidebarAction::OpenFile)
        .style(rail_button_style)
        .width(34)
        .height(34);

    let btn_save = button(text("💾").size(13).align_x(Horizontal::Center))
        .on_press(SidebarAction::SaveFile)
        .style(rail_button_style)
        .width(34)
        .height(34);

    let search_style = if is_search_open {
        rail_button_active_style
    } else {
        rail_button_style
    };
    let btn_search = button(text("🔍").size(12).align_x(Horizontal::Center))
        .on_press(SidebarAction::ToggleSearch)
        .style(search_style)
        .width(34)
        .height(34);

    let btn_width = button(text("↔").size(14).align_x(Horizontal::Center))
        .on_press(SidebarAction::CycleWidthMode)
        .style(rail_button_style)
        .width(34)
        .height(34);

    let btn_view = button(text("◫").size(13).align_x(Horizontal::Center))
        .on_press(SidebarAction::CycleViewMode)
        .style(rail_button_style)
        .width(34)
        .height(34);

    let btn_font = button(text("🔤").size(12).align_x(Horizontal::Center))
        .on_press(SidebarAction::ToggleFontModal)
        .style(rail_button_style)
        .width(34)
        .height(34);

    let btn_export = button(text("⎙").size(14).align_x(Horizontal::Center))
        .on_press(SidebarAction::ExportHtml)
        .style(rail_button_style)
        .width(34)
        .height(34);

    // Markdown Quick formatting buttons
    let btn_h1 = button(text("H1").size(11).align_x(Horizontal::Center))
        .on_press(SidebarAction::FormatHeading1)
        .style(rail_button_style)
        .width(34)
        .height(26);

    let btn_bold = button(text("B").size(12).align_x(Horizontal::Center))
        .on_press(SidebarAction::FormatBold)
        .style(rail_button_style)
        .width(34)
        .height(26);

    let btn_italic = button(text("I").size(12).align_x(Horizontal::Center))
        .on_press(SidebarAction::FormatItalic)
        .style(rail_button_style)
        .width(34)
        .height(26);

    let btn_strike = button(text("S̶").size(12).align_x(Horizontal::Center))
        .on_press(SidebarAction::FormatStrikethrough)
        .style(rail_button_style)
        .width(34)
        .height(26);

    let btn_quote = button(text("”").size(13).align_x(Horizontal::Center))
        .on_press(SidebarAction::FormatBlockquote)
        .style(rail_button_style)
        .width(34)
        .height(26);

    let btn_list = button(text("•").size(14).align_x(Horizontal::Center))
        .on_press(SidebarAction::FormatList)
        .style(rail_button_style)
        .width(34)
        .height(26);

    // Direction & BiDi toggle
    let dir_label = match direction_mode {
        DirectionMode::Auto => "⇄",
        DirectionMode::Ltr => "→",
        DirectionMode::Rtl => "←",
    };
    let btn_direction = button(text(dir_label).size(14).align_x(Horizontal::Center))
        .on_press(SidebarAction::CycleDirection)
        .style(rail_button_style)
        .width(34)
        .height(32);

    // German Quick Insert Bar toggle
    let german_btn_style = if is_german_bar_open {
        rail_button_active_style
    } else {
        rail_button_style
    };
    let btn_german = button(text("äö").size(10).align_x(Horizontal::Center))
        .on_press(SidebarAction::ToggleGermanBar)
        .style(german_btn_style)
        .width(34)
        .height(32);

    // Exam / Redemittel drawer toggle
    let exam_btn_style = if is_exam_drawer_open {
        rail_button_active_style
    } else {
        rail_button_style
    };
    let btn_exam = button(text("⏱").size(13).align_x(Horizontal::Center))
        .on_press(SidebarAction::ToggleRightDrawer)
        .style(exam_btn_style)
        .width(34)
        .height(32);

    // Shortcuts modal button
    let btn_help = button(text("?").size(12).align_x(Horizontal::Center))
        .on_press(SidebarAction::ToggleShortcutsModal)
        .style(rail_button_style)
        .width(34)
        .height(32);

    let mut rail_col = column![
        btn_mode,
        create_divider(),
        btn_new,
        btn_open,
        btn_save,
        btn_search,
        btn_view,
        btn_width,
        btn_font,
        btn_export,
        create_divider(),
        btn_h1,
        btn_bold,
        btn_italic,
        btn_strike,
        btn_quote,
        btn_list,
        create_divider(),
        btn_direction,
        btn_german,
        btn_exam,
    ]
    .spacing(4)
    .align_x(Alignment::Center)
    .padding(iced::Padding {
        top: 6.0,
        bottom: 6.0,
        left: 3.0,
        right: 3.0,
    });

    // If German Quick-Insert bar is opened in left rail, show one-click keys
    if is_german_bar_open {
        let de_keys = ["ä", "ö", "ü", "ß", "Ä", "Ö", "Ü", "„", "“", "–"];
        let mut de_col = column![create_divider()].spacing(3).align_x(Alignment::Center);
        for k in de_keys {
            let b = button(text(k).size(11).align_x(Horizontal::Center))
                .on_press(SidebarAction::InsertGermanChar(k))
                .style(rail_button_style)
                .width(28)
                .height(24);
            de_col = de_col.push(b);
        }
        rail_col = rail_col.push(de_col);
    }

    rail_col = rail_col.push(Space::with_height(Length::Fill));
    rail_col = rail_col.push(btn_help);

    container(rail_col)
        .width(42)
        .height(Length::Fill)
        .style(rail_container_style)
        .into()
}

/// Renders the Right Collapsible Drawer (Exam Redemittel & B2/C1 Tools).
pub fn view_right_drawer<'a>(
    _timer_remaining_secs: u64,
    _timer_is_running: bool,
    _timer_total_secs: u64,
    target_min_words: usize,
    target_max_words: usize,
    stats: &DocumentStats,
    snippets: &'a SnippetsConfig,
) -> Element<'a, SidebarAction> {
    let header = row![
        text("German Exam Redemittel & Goals")
            .size(12)
            .color(TEXT_PRIMARY),
        Space::with_width(Length::Fill),
        button(text("✕").size(11))
            .on_press(SidebarAction::ToggleRightDrawer)
            .style(rail_button_style)
            .width(24)
            .height(24),
    ]
    .align_y(Alignment::Center);

    // Minimal word count target badge
    let is_in_range = stats.word_count >= target_min_words && stats.word_count <= target_max_words;
    let badge_color = if stats.word_count == 0 {
        TEXT_MUTED
    } else if is_in_range {
        ACCENT_GREEN
    } else {
        ACCENT_AMBER
    };

    let target_row = row![
        text(format!("Target: {} - {} words", target_min_words, target_max_words))
            .size(11)
            .color(TEXT_MUTED),
        Space::with_width(Length::Fill),
        text(format!("{} words", stats.word_count))
            .size(12)
            .color(badge_color),
    ]
    .align_y(Alignment::Center);

    let target_card = container(target_row)
        .padding(10)
        .width(Length::Fill)
        .style(card_container_style);

    // Redemittel Categories
    let mut snippets_col = column![
        text("B2/C1 CONNECTORS (CLICK TO INSERT)")
            .size(10)
            .color(TEXT_MUTED),
    ]
    .spacing(8);

    for cat in &snippets.categories {
        let mut cat_col = column![
            text(&cat.name).size(11).color(ACCENT_AMBER),
        ]
        .spacing(4);

        for item in &cat.items {
            let snippet_text = item.text.clone();
            let chip = button(
                text(&item.label)
                    .size(11)
                    .color(TEXT_PRIMARY),
            )
            .on_press(SidebarAction::InsertSnippet(snippet_text))
            .style(|_t, status| {
                let bg = match status {
                    iced::widget::button::Status::Hovered => Color::from_rgb(0.18, 0.20, 0.26),
                    _ => Color::from_rgb(0.11, 0.12, 0.16),
                };
                iced::widget::button::Style {
                    background: Some(iced::Background::Color(bg)),
                    border: iced::Border {
                        color: BORDER_COLOR,
                        width: 1.0,
                        radius: iced::border::Radius::from(4.0),
                    },
                    text_color: TEXT_PRIMARY,
                    ..Default::default()
                }
            })
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

    let scrollable_snippets = iced::widget::scrollable(snippets_col)
        .height(Length::Fill);

    let drawer_content = column![
        header,
        target_card,
        create_divider(),
        scrollable_snippets,
    ]
    .spacing(12)
    .padding(12);

    container(drawer_content)
        .width(280)
        .height(Length::Fill)
        .style(drawer_container_style)
        .into()
}
