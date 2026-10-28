use iced::alignment::Horizontal;
use iced::widget::{button, center, column, container, row, text, Space};
use iced::{Alignment, Color, Element, Length};

use crate::config::SnippetsConfig;
use crate::editor::{DirectionMode, DocumentStats};
use crate::icons::*;
use crate::theme::{
    card_container_style, drawer_container_style, rail_button_active_style,
    rail_button_style, rail_container_style, ACCENT_AMBER, ACCENT_CYAN, ACCENT_GREEN,
    BORDER_COLOR, TEXT_HOVER, TEXT_MUTED, TEXT_PRIMARY,
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
        .width(22)
        .height(1)
        .into()
}

/// Helper to build uniform 32x32 flex-centered sidebar tool button with an SVG icon
fn sidebar_icon_button<'a>(
    svg_bytes: &'static [u8],
    icon_size: u16,
    icon_color: Color,
    action: SidebarAction,
    is_active: bool,
) -> button::Button<'a, SidebarAction> {
    let btn_style = if is_active {
        rail_button_active_style
    } else {
        rail_button_style
    };

    button(center(render_svg_icon(svg_bytes, icon_size, icon_color)))
        .on_press(action)
        .style(btn_style)
        .width(32)
        .height(32)
}

/// Helper to build uniform 32x32 flex-centered sidebar tool button with text
fn sidebar_text_button<'a>(
    label: &'static str,
    font_size: u16,
    action: SidebarAction,
    is_active: bool,
) -> button::Button<'a, SidebarAction> {
    let btn_style = if is_active {
        rail_button_active_style
    } else {
        rail_button_style
    };

    button(
        center(
            text(label)
                .size(font_size)
                .color(if is_active { ACCENT_CYAN } else { TEXT_PRIMARY }),
        ),
    )
    .on_press(action)
    .style(btn_style)
    .width(32)
    .height(32)
}

/// Renders the left ultra-minimal vertical icon rail (42px wide).
/// Every button is uniformly 32x32px with 6px border radius and 4px spacing.
pub fn view_left_rail<'a>(
    _direction_mode: DirectionMode,
    is_exam_drawer_open: bool,
    is_german_bar_open: bool,
    is_whiteboard_active: bool,
    is_search_open: bool,
) -> Element<'a, SidebarAction> {
    // 1. Mode switcher at very top: Write Studio vs Whiteboard Studio
    let (mode_svg, mode_color) = if is_whiteboard_active {
        (SVG_WHITEBOARD, ACCENT_CYAN)
    } else {
        (SVG_WRITE, TEXT_HOVER)
    };
    let btn_mode = sidebar_icon_button(
        mode_svg,
        16,
        mode_color,
        SidebarAction::ToggleAppMode,
        is_whiteboard_active,
    );

    // 2. File Actions
    let btn_new = sidebar_icon_button(SVG_NEW, 16, TEXT_MUTED, SidebarAction::NewFile, false);
    let btn_open = sidebar_icon_button(SVG_OPEN, 16, TEXT_MUTED, SidebarAction::OpenFile, false);
    let btn_save = sidebar_icon_button(SVG_SAVE, 16, TEXT_MUTED, SidebarAction::SaveFile, false);
    let btn_search = sidebar_icon_button(
        SVG_SEARCH,
        15,
        if is_search_open { ACCENT_CYAN } else { TEXT_MUTED },
        SidebarAction::ToggleSearch,
        is_search_open,
    );

    // 3. View & Typography Tools
    let btn_view = sidebar_icon_button(SVG_SPLIT, 16, TEXT_MUTED, SidebarAction::CycleViewMode, false);
    let btn_width = sidebar_icon_button(SVG_WIDTH, 16, TEXT_MUTED, SidebarAction::CycleWidthMode, false);
    let btn_font = sidebar_icon_button(SVG_FONT, 16, TEXT_MUTED, SidebarAction::ToggleFontModal, false);
    let btn_export = sidebar_icon_button(SVG_EXPORT, 16, TEXT_MUTED, SidebarAction::ExportHtml, false);

    // 4. Markdown Quick Formatting Buttons (Uniform 32x32)
    let btn_h1 = sidebar_text_button("H1", 11, SidebarAction::FormatHeading1, false);
    let btn_bold = sidebar_text_button("B", 13, SidebarAction::FormatBold, false);
    let btn_italic = sidebar_text_button("I", 13, SidebarAction::FormatItalic, false);
    let btn_strike = sidebar_text_button("S", 12, SidebarAction::FormatStrikethrough, false);
    let btn_quote = sidebar_text_button("”", 14, SidebarAction::FormatBlockquote, false);
    let btn_list = sidebar_icon_button(SVG_LIST, 15, TEXT_MUTED, SidebarAction::FormatList, false);

    // 5. Trilingual & Exam Tools
    let btn_direction = sidebar_icon_button(SVG_BIDI, 15, TEXT_MUTED, SidebarAction::CycleDirection, false);
    let btn_german = sidebar_text_button("äö", 11, SidebarAction::ToggleGermanBar, is_german_bar_open);
    let btn_exam = sidebar_icon_button(
        SVG_TIMER,
        16,
        if is_exam_drawer_open { ACCENT_CYAN } else { TEXT_MUTED },
        SidebarAction::ToggleRightDrawer,
        is_exam_drawer_open,
    );

    // 6. Help / Quick Tour
    let btn_help = sidebar_icon_button(
        SVG_TOUR,
        16,
        TEXT_MUTED,
        SidebarAction::ToggleShortcutsModal,
        false,
    );

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
        button(center(render_svg_icon(SVG_CLOSE, 12, TEXT_MUTED)))
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
                    iced::widget::button::Status::Hovered => Color::from_rgba(1.0, 1.0, 1.0, 0.08),
                    _ => Color::from_rgba(1.0, 1.0, 1.0, 0.03),
                };
                iced::widget::button::Style {
                    background: Some(iced::Background::Color(bg)),
                    border: iced::Border {
                        color: BORDER_COLOR,
                        width: 1.0,
                        radius: iced::border::Radius::from(6.0),
                    },
                    text_color: TEXT_PRIMARY,
                    ..Default::default()
                }
            })
            .padding(iced::Padding {
                top: 5.0,
                bottom: 5.0,
                left: 10.0,
                right: 10.0,
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
