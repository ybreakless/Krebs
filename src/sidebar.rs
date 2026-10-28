use iced::alignment::Horizontal;
use iced::widget::{
    button, column, container, row, scrollable, text, text_input, Space,
};
use iced::{Alignment, Color, Element, Length};

use crate::config::SnippetsConfig;
use crate::editor::{DirectionMode, DocumentStats};
use crate::theme::{
    accent_button_style, card_container_style, chip_button_style, drawer_container_style,
    rail_button_active_style, rail_button_style, rail_container_style, text_input_style,
    ACCENT_AMBER, ACCENT_CYAN, ACCENT_GREEN, ACCENT_RED, BORDER_COLOR, TEXT_MUTED, TEXT_PRIMARY,
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
) -> Element<'a, SidebarAction> {
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

    let btn_export = button(text("⎙").size(14).align_x(Horizontal::Center))
        .on_press(SidebarAction::ExportHtml)
        .style(rail_button_style)
        .width(34)
        .height(34);

    // Formatting buttons
    let btn_h1 = button(text("H1").size(11).align_x(Horizontal::Center))
        .on_press(SidebarAction::FormatHeading1)
        .style(rail_button_style)
        .width(34)
        .height(28);

    let btn_h2 = button(text("H2").size(11).align_x(Horizontal::Center))
        .on_press(SidebarAction::FormatHeading2)
        .style(rail_button_style)
        .width(34)
        .height(28);

    let btn_bold = button(text("B").size(12).align_x(Horizontal::Center))
        .on_press(SidebarAction::FormatBold)
        .style(rail_button_style)
        .width(34)
        .height(28);

    let btn_italic = button(text("I").size(12).align_x(Horizontal::Center))
        .on_press(SidebarAction::FormatItalic)
        .style(rail_button_style)
        .width(34)
        .height(28);

    let btn_strike = button(text("S̶").size(12).align_x(Horizontal::Center))
        .on_press(SidebarAction::FormatStrikethrough)
        .style(rail_button_style)
        .width(34)
        .height(28);

    let btn_quote = button(text("”").size(13).align_x(Horizontal::Center))
        .on_press(SidebarAction::FormatBlockquote)
        .style(rail_button_style)
        .width(34)
        .height(28);

    let btn_list = button(text("•").size(14).align_x(Horizontal::Center))
        .on_press(SidebarAction::FormatList)
        .style(rail_button_style)
        .width(34)
        .height(28);

    let btn_code = button(text("`").size(13).align_x(Horizontal::Center))
        .on_press(SidebarAction::FormatCode)
        .style(rail_button_style)
        .width(34)
        .height(28);

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
        .height(34);

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
        .height(34);

    // Exam & Essay Trainer toggle
    let exam_btn_style = if is_exam_drawer_open {
        rail_button_active_style
    } else {
        rail_button_style
    };
    let btn_exam = button(text("⏱").size(14).align_x(Horizontal::Center))
        .on_press(SidebarAction::ToggleRightDrawer)
        .style(exam_btn_style)
        .width(34)
        .height(34);

    // Shortcuts modal button
    let btn_help = button(text("?").size(12).align_x(Horizontal::Center))
        .on_press(SidebarAction::ToggleShortcutsModal)
        .style(rail_button_style)
        .width(34)
        .height(34);

    let mut rail_col = column![
        btn_new,
        btn_open,
        btn_save,
        btn_export,
        create_divider(),
        btn_h1,
        btn_h2,
        btn_bold,
        btn_italic,
        btn_strike,
        btn_quote,
        btn_list,
        btn_code,
        create_divider(),
        btn_direction,
        btn_german,
        btn_exam,
    ]
    .spacing(4)
    .align_x(Alignment::Center)
    .padding(iced::Padding {
        top: 8.0,
        bottom: 8.0,
        left: 3.0,
        right: 3.0,
    });

    // If German Quick-Insert bar is opened in left rail, show one-click keys
    if is_german_bar_open {
        let de_keys = [
            "ä", "ö", "ü", "ß", "Ä", "Ö", "Ü", "„", "“", "–",
        ];
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

/// Renders the Right Collapsible Drawer (Exam & Essay Trainer Panel).
pub fn view_right_drawer<'a>(
    timer_remaining_secs: u64,
    timer_is_running: bool,
    timer_total_secs: u64,
    target_min_words: usize,
    target_max_words: usize,
    stats: &DocumentStats,
    snippets: &'a SnippetsConfig,
) -> Element<'a, SidebarAction> {
    // Header
    let header = row![
        text("Exam & Essay Trainer")
            .size(13)
            .color(TEXT_PRIMARY),
        Space::with_width(Length::Fill),
        button(text("✕").size(11))
            .on_press(SidebarAction::ToggleRightDrawer)
            .style(rail_button_style)
            .width(26)
            .height(26),
    ]
    .align_y(Alignment::Center);

    // 1. Timer Card
    let minutes = timer_remaining_secs / 60;
    let seconds = timer_remaining_secs % 60;
    let timer_str = format!("{:02}:{:02}", minutes, seconds);

    let progress_fraction = if timer_total_secs > 0 {
        1.0 - (timer_remaining_secs as f32 / timer_total_secs as f32)
    } else {
        0.0
    };

    let timer_color = if timer_remaining_secs == 0 {
        ACCENT_RED
    } else if timer_remaining_secs <= 300 {
        ACCENT_AMBER
    } else {
        ACCENT_CYAN
    };

    let progress_bar_bg = container(
        container(Space::with_height(3))
            .width(Length::FillPortion((progress_fraction * 100.0).max(1.0) as u16))
            .style(move |_t| container::Style {
                background: Some(iced::Background::Color(timer_color)),
                ..Default::default()
            }),
    )
    .width(Length::Fill)
    .height(3)
    .style(|_t| container::Style {
        background: Some(iced::Background::Color(BORDER_COLOR)),
        ..Default::default()
    });

    let timer_display = text(timer_str).size(26).color(timer_color);

    let preset_30m = button(text("30m").size(11))
        .on_press(SidebarAction::TimerSetDuration(1800))
        .style(chip_button_style)
        .padding(iced::Padding::new(4.0));

    let preset_45m = button(text("45m").size(11))
        .on_press(SidebarAction::TimerSetDuration(2700))
        .style(chip_button_style)
        .padding(iced::Padding::new(4.0));

    let preset_60m = button(text("60m").size(11))
        .on_press(SidebarAction::TimerSetDuration(3600))
        .style(chip_button_style)
        .padding(iced::Padding::new(4.0));

    let presets_row = row![preset_30m, preset_45m, preset_60m]
        .spacing(6)
        .align_y(Alignment::Center);

    let btn_start_pause = if timer_is_running {
        button(text("⏸ Pause").size(11))
            .on_press(SidebarAction::TimerPause)
            .style(accent_button_style)
            .padding(iced::Padding::new(6.0))
    } else {
        button(text("▶ Start").size(11))
            .on_press(SidebarAction::TimerStart)
            .style(accent_button_style)
            .padding(iced::Padding::new(6.0))
    };

    let btn_reset = button(text("↺ Reset").size(11))
        .on_press(SidebarAction::TimerReset)
        .style(chip_button_style)
        .padding(iced::Padding::new(6.0));

    let timer_controls = row![btn_start_pause, btn_reset]
        .spacing(8)
        .align_y(Alignment::Center);

    let timer_card = container(
        column![
            row![
                text("COUNTDOWN TIMER").size(10).color(TEXT_MUTED),
                Space::with_width(Length::Fill),
                presets_row,
            ]
            .align_y(Alignment::Center),
            progress_bar_bg,
            row![timer_display, Space::with_width(Length::Fill), timer_controls]
                .align_y(Alignment::Center),
        ]
        .spacing(10),
    )
    .padding(12)
    .style(card_container_style);

    // 2. Word Target & Live Stats Card
    let in_target = stats.word_count >= target_min_words && stats.word_count <= target_max_words;
    let under_target = stats.word_count < target_min_words;

    let (badge_text, badge_color) = if in_target {
        (
            format!("✓ In Target Range ({} words)", stats.word_count),
            ACCENT_GREEN,
        )
    } else if under_target {
        (
            format!("▲ {} words to min ({})", target_min_words.saturating_sub(stats.word_count), target_min_words),
            ACCENT_AMBER,
        )
    } else {
        (
            format!("▼ {} words over max ({})", stats.word_count.saturating_sub(target_max_words), target_max_words),
            ACCENT_AMBER,
        )
    };

    let target_badge = container(
        text(badge_text)
            .size(11)
            .color(badge_color)
            .align_x(Horizontal::Center),
    )
    .padding(iced::Padding {
        top: 4.0,
        bottom: 4.0,
        left: 8.0,
        right: 8.0,
    })
    .style(move |_t| container::Style {
        background: Some(iced::Background::Color(Color::from_rgba(
            badge_color.r,
            badge_color.g,
            badge_color.b,
            0.12,
        ))),
        border: iced::Border {
            color: badge_color,
            width: 1.0,
            radius: iced::border::Radius::from(4.0),
        },
        ..Default::default()
    })
    .width(Length::Fill);

    let min_input = text_input("Min", &target_min_words.to_string())
        .on_input(move |s| {
            if let Ok(val) = s.parse::<usize>() {
                SidebarAction::TargetMinChanged(val)
            } else if s.is_empty() {
                SidebarAction::TargetMinChanged(0)
            } else {
                SidebarAction::TargetMinChanged(target_min_words)
            }
        })
        .width(55)
        .size(11)
        .padding(iced::Padding::new(4.0))
        .style(text_input_style);

    let max_input = text_input("Max", &target_max_words.to_string())
        .on_input(move |s| {
            if let Ok(val) = s.parse::<usize>() {
                SidebarAction::TargetMaxChanged(val)
            } else if s.is_empty() {
                SidebarAction::TargetMaxChanged(0)
            } else {
                SidebarAction::TargetMaxChanged(target_max_words)
            }
        })
        .width(55)
        .size(11)
        .padding(iced::Padding::new(4.0))
        .style(text_input_style);

    let target_controls = row![
        text("Target:").size(11).color(TEXT_MUTED),
        Space::with_width(6),
        min_input,
        text("–").size(11).color(TEXT_MUTED),
        max_input,
        text("words").size(11).color(TEXT_MUTED),
    ]
    .align_y(Alignment::Center);

    let stats_grid = column![
        row![
            text("Words:").size(11).color(TEXT_MUTED),
            Space::with_width(Length::Fill),
            text(format!("{}", stats.word_count)).size(11).color(TEXT_PRIMARY),
        ],
        row![
            text("Characters:").size(11).color(TEXT_MUTED),
            Space::with_width(Length::Fill),
            text(format!("{} (no space: {})", stats.char_count, stats.char_count_no_spaces))
                .size(11)
                .color(TEXT_PRIMARY),
        ],
        row![
            text("Paragraphs:").size(11).color(TEXT_MUTED),
            Space::with_width(Length::Fill),
            text(format!("{}", stats.paragraph_count)).size(11).color(TEXT_PRIMARY),
        ],
        row![
            text("Est. Reading Time:").size(11).color(TEXT_MUTED),
            Space::with_width(Length::Fill),
            text(format!("{:.1} min", stats.reading_time_mins)).size(11).color(TEXT_PRIMARY),
        ],
    ]
    .spacing(6);

    let tracker_card = container(
        column![
            row![
                text("WORD COUNT TRACKER").size(10).color(TEXT_MUTED),
                Space::with_width(Length::Fill),
            ],
            target_badge,
            target_controls,
            stats_grid,
        ]
        .spacing(10),
    )
    .padding(12)
    .style(card_container_style);

    // 3. German Redemittel Chips Drawer
    let mut redemittel_col = column![
        row![
            text("GERMAN REDEMITTEL (B2/C1)").size(10).color(TEXT_MUTED),
            Space::with_width(Length::Fill),
        ],
    ]
    .spacing(10);

    for category in &snippets.categories {
        let cat_title = text(&category.name).size(11).color(ACCENT_CYAN);
        let mut chips_col = column![cat_title].spacing(6);

        for item in &category.items {
            let snippet_text = item.text.clone();
            let chip = button(
                column![
                    text(&item.label).size(11).color(TEXT_PRIMARY),
                    text(&item.text).size(9).color(TEXT_MUTED),
                ]
                .spacing(2),
            )
            .on_press(SidebarAction::InsertSnippet(snippet_text))
            .style(chip_button_style)
            .padding(iced::Padding {
                top: 6.0,
                bottom: 6.0,
                left: 8.0,
                right: 8.0,
            })
            .width(Length::Fill);

            chips_col = chips_col.push(chip);
        }

        let cat_card = container(chips_col).padding(8).style(card_container_style);
        redemittel_col = redemittel_col.push(cat_card);
    }

    let drawer_content = scrollable(
        column![
            header,
            timer_card,
            tracker_card,
            redemittel_col,
        ]
        .spacing(12)
        .padding(iced::Padding {
            top: 10.0,
            bottom: 16.0,
            left: 12.0,
            right: 12.0,
        }),
    )
    .height(Length::Fill);

    container(drawer_content)
        .width(310)
        .height(Length::Fill)
        .style(drawer_container_style)
        .into()
}
