use iced::border::Radius;
use iced::widget::button;
use iced::widget::container;
use iced::widget::scrollable;
use iced::widget::text_editor;
use iced::widget::text_input;
use iced::{Background, Border, Color, Shadow};

// Omarchy Dark Mode Palette & High-Contrast Neutral Grays
pub const CANVAS_BG: Color = Color::from_rgb(0.067, 0.071, 0.086); // #111216
pub const RAIL_BG: Color = Color::from_rgb(0.043, 0.047, 0.055); // #0b0c0e
pub const BORDER_COLOR: Color = Color::from_rgba(1.0, 1.0, 1.0, 0.10); // subtle 1px divider
pub const TEXT_PRIMARY: Color = Color::from_rgb(0.92, 0.94, 0.96); // #ebf0f5 crisp warm off-white
pub const TEXT_MUTED: Color = Color::from_rgba(1.0, 1.0, 1.0, 0.48); // High-contrast readable muted (was too dark)
pub const TEXT_HOVER: Color = Color::from_rgba(1.0, 1.0, 1.0, 0.92); // Crisp bright active hover
pub const SURFACE_HOVER: Color = Color::from_rgba(1.0, 1.0, 1.0, 0.08); // Subtle background highlight
pub const SURFACE_ACTIVE: Color = Color::from_rgba(1.0, 1.0, 1.0, 0.14); // Active surface highlight
pub const ACCENT_CYAN: Color = Color::from_rgb(0.533, 0.753, 0.816); // #88c0d0 Nordic Cyan
pub const ACCENT_AMBER: Color = Color::from_rgb(0.922, 0.796, 0.545); // #ebcb8b Amber
pub const ACCENT_GREEN: Color = Color::from_rgb(0.639, 0.745, 0.549); // #a3be8c Green
pub const ACCENT_RED: Color = Color::from_rgb(0.85, 0.38, 0.42); // #bf616a Red
pub const BANNER_BG: Color = Color::from_rgb(0.18, 0.15, 0.08); // warm dark amber

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeMode {
    OmarchyDark,
    OledBlack,
    DeepSlate,
    WarmEspresso,
}

impl ThemeMode {
    pub fn from_str_name(s: &str) -> Self {
        match s {
            "oled" => Self::OledBlack,
            "slate" => Self::DeepSlate,
            "espresso" => Self::WarmEspresso,
            _ => Self::OmarchyDark,
        }
    }

    pub fn to_str_name(&self) -> &'static str {
        match self {
            Self::OmarchyDark => "dark",
            Self::OledBlack => "oled",
            Self::DeepSlate => "slate",
            Self::WarmEspresso => "espresso",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::OmarchyDark => "Omarchy Dark",
            Self::OledBlack => "OLED Pure Black",
            Self::DeepSlate => "Deep Slate",
            Self::WarmEspresso => "Warm Espresso",
        }
    }

    pub fn canvas_bg(&self) -> Color {
        match self {
            Self::OmarchyDark => Color::from_rgb(0.067, 0.071, 0.086), // #111216
            Self::OledBlack => Color::from_rgb(0.0, 0.0, 0.0), // #000000
            Self::DeepSlate => Color::from_rgb(0.086, 0.094, 0.122), // #16181f
            Self::WarmEspresso => Color::from_rgb(0.090, 0.078, 0.070), // #171412
        }
    }

    pub fn rail_bg(&self) -> Color {
        match self {
            Self::OmarchyDark => Color::from_rgb(0.043, 0.047, 0.055), // #0b0c0e
            Self::OledBlack => Color::from_rgb(0.035, 0.035, 0.035), // #090909
            Self::DeepSlate => Color::from_rgb(0.059, 0.063, 0.082), // #0f1015
            Self::WarmEspresso => Color::from_rgb(0.063, 0.051, 0.043), // #100d0b
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccentChoice {
    NordicCyan,
    AmberSunset,
    EmeraldMint,
    NordicFrost,
    RoseBlush,
}

impl AccentChoice {
    pub fn from_str_name(s: &str) -> Self {
        match s {
            "amber" => Self::AmberSunset,
            "green" => Self::EmeraldMint,
            "white" => Self::NordicFrost,
            "rose" => Self::RoseBlush,
            _ => Self::NordicCyan,
        }
    }

    pub fn to_str_name(&self) -> &'static str {
        match self {
            Self::NordicCyan => "cyan",
            Self::AmberSunset => "amber",
            Self::EmeraldMint => "green",
            Self::NordicFrost => "white",
            Self::RoseBlush => "rose",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::NordicCyan => "Nordic Cyan",
            Self::AmberSunset => "Amber Sunset",
            Self::EmeraldMint => "Emerald Mint",
            Self::NordicFrost => "Nordic Frost",
            Self::RoseBlush => "Rose Blush",
        }
    }

    pub fn color(&self) -> Color {
        match self {
            Self::NordicCyan => ACCENT_CYAN,
            Self::AmberSunset => ACCENT_AMBER,
            Self::EmeraldMint => ACCENT_GREEN,
            Self::NordicFrost => Color::from_rgb(0.92, 0.94, 0.96),
            Self::RoseBlush => ACCENT_RED,
        }
    }
}

pub fn canvas_container_style(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(CANVAS_BG)),
        text_color: Some(TEXT_PRIMARY),
        ..Default::default()
    }
}

pub fn rail_container_style(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(RAIL_BG)),
        border: Border {
            color: BORDER_COLOR,
            width: 1.0,
            radius: Radius::default(),
        },
        text_color: Some(TEXT_MUTED),
        ..Default::default()
    }
}

pub fn drawer_container_style(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(RAIL_BG)),
        border: Border {
            color: BORDER_COLOR,
            width: 1.0,
            radius: Radius::default(),
        },
        text_color: Some(TEXT_PRIMARY),
        ..Default::default()
    }
}

pub fn card_container_style(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(CANVAS_BG)),
        border: Border {
            color: BORDER_COLOR,
            width: 1.0,
            radius: Radius::from(6.0),
        },
        text_color: Some(TEXT_PRIMARY),
        ..Default::default()
    }
}

pub fn modal_container_style(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(RAIL_BG)),
        border: Border {
            color: ACCENT_CYAN,
            width: 1.0,
            radius: Radius::from(8.0),
        },
        shadow: Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.6),
            offset: iced::Vector::new(0.0, 8.0),
            blur_radius: 24.0,
        },
        text_color: Some(TEXT_PRIMARY),
    }
}

pub fn banner_container_style(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(BANNER_BG)),
        border: Border {
            color: ACCENT_AMBER,
            width: 1.0,
            radius: Radius::from(4.0),
        },
        text_color: Some(ACCENT_AMBER),
        ..Default::default()
    }
}

pub fn rail_button_style(_theme: &iced::Theme, status: button::Status) -> button::Style {
    let base = button::Style {
        background: Some(Background::Color(Color::TRANSPARENT)),
        text_color: TEXT_MUTED,
        border: Border {
            radius: Radius::from(6.0),
            ..Default::default()
        },
        ..Default::default()
    };

    match status {
        button::Status::Active => base,
        button::Status::Hovered => button::Style {
            background: Some(Background::Color(SURFACE_HOVER)),
            text_color: TEXT_HOVER,
            ..base
        },
        button::Status::Pressed => button::Style {
            background: Some(Background::Color(SURFACE_ACTIVE)),
            text_color: ACCENT_CYAN,
            ..base
        },
        button::Status::Disabled => button::Style {
            text_color: Color::from_rgba(1.0, 1.0, 1.0, 0.2),
            ..base
        },
    }
}

pub fn rail_button_active_style(_theme: &iced::Theme, _status: button::Status) -> button::Style {
    button::Style {
        background: Some(Background::Color(SURFACE_ACTIVE)),
        text_color: ACCENT_CYAN,
        border: Border {
            color: ACCENT_CYAN,
            width: 1.0,
            radius: Radius::from(6.0),
        },
        ..Default::default()
    }
}

pub fn chip_button_style(_theme: &iced::Theme, status: button::Status) -> button::Style {
    let base = button::Style {
        background: Some(Background::Color(CANVAS_BG)),
        text_color: TEXT_PRIMARY,
        border: Border {
            color: BORDER_COLOR,
            width: 1.0,
            radius: Radius::from(6.0),
        },
        ..Default::default()
    };

    match status {
        button::Status::Active => base,
        button::Status::Hovered => button::Style {
            background: Some(Background::Color(SURFACE_HOVER)),
            border: Border {
                color: ACCENT_CYAN,
                width: 1.0,
                radius: Radius::from(6.0),
            },
            text_color: ACCENT_CYAN,
            ..base
        },
        button::Status::Pressed => button::Style {
            background: Some(Background::Color(SURFACE_ACTIVE)),
            text_color: TEXT_PRIMARY,
            ..base
        },
        button::Status::Disabled => button::Style {
            text_color: TEXT_MUTED,
            ..base
        },
    }
}

pub fn accent_button_style(_theme: &iced::Theme, status: button::Status) -> button::Style {
    let base = button::Style {
        background: Some(Background::Color(Color::from_rgb(0.2, 0.4, 0.5))),
        text_color: Color::WHITE,
        border: Border {
            radius: Radius::from(6.0),
            ..Default::default()
        },
        ..Default::default()
    };

    match status {
        button::Status::Active => base,
        button::Status::Hovered => button::Style {
            background: Some(Background::Color(Color::from_rgb(0.25, 0.5, 0.62))),
            ..base
        },
        button::Status::Pressed => button::Style {
            background: Some(Background::Color(Color::from_rgb(0.15, 0.35, 0.45))),
            ..base
        },
        button::Status::Disabled => button::Style {
            background: Some(Background::Color(Color::from_rgb(0.12, 0.13, 0.16))),
            text_color: TEXT_MUTED,
            ..base
        },
    }
}

pub fn text_editor_style(_theme: &iced::Theme, _status: text_editor::Status) -> text_editor::Style {
    text_editor::Style {
        background: Background::Color(CANVAS_BG),
        border: Border {
            width: 0.0,
            color: Color::TRANSPARENT,
            radius: Radius::default(),
        },
        icon: TEXT_MUTED,
        placeholder: TEXT_MUTED,
        value: TEXT_PRIMARY,
        selection: Color::from_rgba(0.533, 0.753, 0.816, 0.25),
    }
}

pub fn text_input_style(_theme: &iced::Theme, status: text_input::Status) -> text_input::Style {
    let base = text_input::Style {
        background: Background::Color(CANVAS_BG),
        border: Border {
            color: BORDER_COLOR,
            width: 1.0,
            radius: Radius::from(6.0),
        },
        icon: TEXT_MUTED,
        placeholder: TEXT_MUTED,
        value: TEXT_PRIMARY,
        selection: Color::from_rgba(0.533, 0.753, 0.816, 0.25),
    };

    match status {
        text_input::Status::Active | text_input::Status::Hovered => base,
        text_input::Status::Focused => text_input::Style {
            border: Border {
                color: ACCENT_CYAN,
                width: 1.0,
                radius: Radius::from(6.0),
            },
            ..base
        },
        text_input::Status::Disabled => text_input::Style {
            value: TEXT_MUTED,
            ..base
        },
    }
}

pub fn scrollable_style(_theme: &iced::Theme, status: scrollable::Status) -> scrollable::Style {
    let scrollbar_color = match status {
        scrollable::Status::Active => Color::from_rgba(1.0, 1.0, 1.0, 0.15),
        scrollable::Status::Hovered { .. } => Color::from_rgba(0.53, 0.75, 0.82, 0.6),
        scrollable::Status::Dragged { .. } => ACCENT_CYAN,
    };

    scrollable::Style {
        container: container::Style::default(),
        vertical_rail: scrollable::Rail {
            background: None,
            border: Border::default(),
            scroller: scrollable::Scroller {
                color: scrollbar_color,
                border: Border {
                    radius: Radius::from(3.0),
                    ..Default::default()
                },
            },
        },
        horizontal_rail: scrollable::Rail {
            background: None,
            border: Border::default(),
            scroller: scrollable::Scroller {
                color: scrollbar_color,
                border: Border {
                    radius: Radius::from(3.0),
                    ..Default::default()
                },
            },
        },
        gap: None,
    }
}

pub fn search_container_style(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgb(0.06, 0.07, 0.09))),
        border: Border {
            color: ACCENT_CYAN,
            width: 1.0,
            radius: Radius::from(8.0),
        },
        shadow: Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.5),
            offset: iced::Vector::new(0.0, 4.0),
            blur_radius: 12.0,
        },
        text_color: Some(TEXT_PRIMARY),
    }
}

pub fn toolbar_floating_style(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgba(0.05, 0.06, 0.07, 0.95))),
        border: Border {
            color: BORDER_COLOR,
            width: 1.0,
            radius: Radius::from(8.0),
        },
        shadow: Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.5),
            offset: iced::Vector::new(0.0, 3.0),
            blur_radius: 10.0,
        },
        text_color: Some(TEXT_PRIMARY),
    }
}

pub fn active_tool_button_style(_theme: &iced::Theme, _status: button::Status) -> button::Style {
    button::Style {
        background: Some(Background::Color(SURFACE_ACTIVE)),
        border: Border {
            color: ACCENT_CYAN,
            width: 1.0,
            radius: Radius::from(6.0),
        },
        text_color: ACCENT_CYAN,
        ..Default::default()
    }
}

pub fn tab_active_button_style(_theme: &iced::Theme, _status: button::Status) -> button::Style {
    button::Style {
        background: Some(Background::Color(SURFACE_ACTIVE)),
        border: Border {
            color: ACCENT_CYAN,
            width: 1.0,
            radius: Radius::from(6.0),
        },
        text_color: ACCENT_CYAN,
        ..Default::default()
    }
}

pub fn tab_inactive_button_style(_theme: &iced::Theme, status: button::Status) -> button::Style {
    let base = button::Style {
        background: Some(Background::Color(Color::TRANSPARENT)),
        border: Border {
            color: Color::TRANSPARENT,
            width: 1.0,
            radius: Radius::from(6.0),
        },
        text_color: TEXT_MUTED,
        ..Default::default()
    };

    match status {
        button::Status::Active => base,
        button::Status::Hovered => button::Style {
            background: Some(Background::Color(SURFACE_HOVER)),
            text_color: TEXT_HOVER,
            ..base
        },
        button::Status::Pressed => button::Style {
            background: Some(Background::Color(SURFACE_ACTIVE)),
            text_color: TEXT_HOVER,
            ..base
        },
        button::Status::Disabled => base,
    }
}

pub fn wipe_button_style(_theme: &iced::Theme, status: button::Status) -> button::Style {
    let base = button::Style {
        background: Some(Background::Color(Color::from_rgba(0.85, 0.38, 0.42, 0.15))),
        border: Border {
            color: ACCENT_RED,
            width: 1.0,
            radius: Radius::from(6.0),
        },
        text_color: ACCENT_RED,
        ..Default::default()
    };

    match status {
        button::Status::Active => base,
        button::Status::Hovered | button::Status::Pressed => button::Style {
            background: Some(Background::Color(Color::from_rgba(0.85, 0.38, 0.42, 0.30))),
            ..base
        },
        button::Status::Disabled => base,
    }
}
