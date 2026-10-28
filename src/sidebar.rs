use iced::alignment::Horizontal;
use iced::widget::{button, center, column, container, row, text, Space};
use iced::{Alignment, Color, Element, Length};

use crate::config::SnippetsConfig;
use crate::editor::{DirectionMode, DocumentStats};
use crate::icons::*;
use crate::theme::{
    card_container_style, drawer_container_style, rail_button_active_style,
    rail_button_style, rail_container_style, ACCENT_AMBER, ACCENT_CYAN, ACCENT_GREEN, ACCENT_RED,
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
    ToggleAiCoach,
    WhiteboardToolPen,
    WhiteboardToolHighlighter,
    WhiteboardToolEraser,
    WhiteboardUndo,
    WhiteboardRedo,
    WhiteboardWipe,
}

fn create_divider<'a>() -> Element<'a, SidebarAction> {
    container(Space::with_height(1))
        .style(|_t| container::Style {
            background: Some(iced::Background::Color(BORDER_COLOR)),
            ..Default::default()
        })
        .width(28)
        .height(1)
        .into()
}

/// Helper to build uniform 36x36 flex-centered sidebar tool button with an SVG icon
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
        .width(36)
        .height(36)
}

/// Helper to build uniform 36x36 flex-centered sidebar tool button with text
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
    .width(36)
    .height(36)
}

/// Renders the left ultra-minimal vertical icon rail (48px wide).
pub fn view_left_rail<'a>(
    _direction_mode: DirectionMode,
    is_exam_drawer_open: bool,
    is_german_bar_open: bool,
    is_whiteboard_active: bool,
    is_search_open: bool,
    is_ai_drawer_open: bool,
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

    let btn_help = sidebar_icon_button(
        SVG_TOUR,
        16,
        TEXT_MUTED,
        SidebarAction::ToggleShortcutsModal,
        false,
    );

    // If Whiteboard Studio is active, display dedicated tablet drawing tools in the rail
    if is_whiteboard_active {
        let btn_pen = sidebar_icon_button(SVG_PEN, 16, ACCENT_CYAN, SidebarAction::WhiteboardToolPen, false);
        let btn_hl = sidebar_icon_button(SVG_HIGHLIGHTER, 16, TEXT_MUTED, SidebarAction::WhiteboardToolHighlighter, false);
        let btn_eraser = sidebar_icon_button(SVG_ERASER, 16, TEXT_MUTED, SidebarAction::WhiteboardToolEraser, false);
        let btn_undo = sidebar_icon_button(SVG_UNDO, 16, TEXT_MUTED, SidebarAction::WhiteboardUndo, false);
        let btn_redo = sidebar_icon_button(SVG_REDO, 16, TEXT_MUTED, SidebarAction::WhiteboardRedo, false);
        let btn_wipe = sidebar_icon_button(SVG_WIPE, 16, ACCENT_RED, SidebarAction::WhiteboardWipe, false);

        let rail_col = column![
            btn_mode,
            create_divider(),
            btn_pen,
            btn_hl,
            btn_eraser,
            create_divider(),
            btn_undo,
            btn_redo,
            create_divider(),
            btn_wipe,
            Space::with_height(Length::Fill),
            btn_help,
        ]
        .spacing(4)
        .align_x(Alignment::Center)
        .padding(iced::Padding {
            top: 6.0,
            bottom: 6.0,
            left: 3.0,
            right: 3.0,
        });

        return container(rail_col)
            .width(48)
            .height(Length::Fill)
            .style(rail_container_style)
            .into();
    }

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

    // 4. Markdown Quick Formatting Buttons
    let btn_h1 = sidebar_text_button("H1", 11, SidebarAction::FormatHeading1, false);
    let btn_bold = sidebar_text_button("B", 13, SidebarAction::FormatBold, false);
    let btn_italic = sidebar_text_button("I", 13, SidebarAction::FormatItalic, false);
    let btn_strike = sidebar_text_button("S", 12, SidebarAction::FormatStrikethrough, false);
    let btn_quote = sidebar_text_button("”", 14, SidebarAction::FormatBlockquote, false);
    let btn_list = sidebar_icon_button(SVG_LIST, 15, TEXT_MUTED, SidebarAction::FormatList, false);

    // 5. Trilingual & Exam Tools
    let btn_direction = sidebar_icon_button(SVG_BIDI, 15, TEXT_MUTED, SidebarAction::CycleDirection, false);
    let btn_german = sidebar_text_button("äö", 11, SidebarAction::ToggleGermanBar, is_german_bar_open);
    let btn_ai = sidebar_icon_button(
        SVG_AI,
        16,
        if is_ai_drawer_open { ACCENT_CYAN } else { TEXT_MUTED },
        SidebarAction::ToggleAiCoach,
        is_ai_drawer_open,
    );
    let btn_exam = sidebar_icon_button(
        SVG_TIMER,
        16,
        if is_exam_drawer_open { ACCENT_CYAN } else { TEXT_MUTED },
        SidebarAction::ToggleRightDrawer,
        is_exam_drawer_open,
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
        btn_ai,
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
        .width(48)
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
    let word_count = stats.word_count;
    let is_in_range = word_count >= target_min_words && word_count <= target_max_words;
    let badge_color = if word_count == 0 {
        TEXT_MUTED
    } else if is_in_range {
        ACCENT_GREEN
    } else {
        ACCENT_AMBER
    };

    let word_target_header = row![
        text("EXAM WORD TARGET").size(11).color(TEXT_MUTED),
        Space::with_width(Length::Fill),
        text(format!("{word_count} / {target_min_words}–{target_max_words} w"))
            .size(11)
            .color(badge_color),
    ]
    .align_y(Alignment::Center);

    let progress_val = if target_max_words > 0 {
        (word_count as f32 / target_max_words as f32).clamp(0.0, 1.0)
    } else {
        0.0
    };

    let target_progress_bar = container(
        row![
            container(Space::with_height(4))
                .width(Length::FillPortion((progress_val * 100.0) as u16 + 1))
                .style(move |_t| container::Style {
                    background: Some(iced::Background::Color(badge_color)),
                    ..Default::default()
                }),
            container(Space::with_height(4))
                .width(Length::FillPortion(((1.0 - progress_val) * 100.0) as u16 + 1))
                .style(|_t| container::Style {
                    background: Some(iced::Background::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.05))),
                    ..Default::default()
                }),
        ],
    )
    .width(Length::Fill)
    .height(4);

    let words_card = container(
        column![
            word_target_header,
            Space::with_height(4),
            target_progress_bar,
        ]
        .spacing(6),
    )
    .padding(10)
    .style(card_container_style);

    let mut snippets_col = column![].spacing(10);
    for cat in &snippets.categories {
        let cat_title = text(&cat.name).size(11).color(ACCENT_CYAN);
        let mut items_col = column![cat_title].spacing(4);
        for item in &cat.items {
            let btn = button(text(&item.label).size(11).color(TEXT_PRIMARY))
                .on_press(SidebarAction::InsertSnippet(item.text.clone()))
                .style(rail_button_style)
                .padding(iced::Padding {
                    top: 4.0,
                    bottom: 4.0,
                    left: 8.0,
                    right: 8.0,
                })
                .width(Length::Fill);
            items_col = items_col.push(btn);
        }
        snippets_col = snippets_col.push(container(items_col).padding(8).style(card_container_style));
    }

    let drawer_content = column![
        words_card,
        Space::with_height(4),
        text("GERMAN B2/C1 REDEMITTEL").size(11).color(TEXT_MUTED),
        snippets_col,
    ]
    .spacing(8)
    .padding(12);

    container(drawer_content)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(drawer_container_style)
        .into()
}
