use iced::font;
use iced::widget::{column, container, row, text, Space};
use iced::{Alignment, Color, Element, Font, Length};
use pulldown_cmark::{Event, HeadingLevel, Parser, Tag, TagEnd};

use crate::editor::detect_paragraph_direction;
use crate::theme::{
    ACCENT_AMBER, ACCENT_CYAN, ACCENT_GREEN, BORDER_COLOR, SURFACE_ACTIVE, SURFACE_HOVER,
    TEXT_MUTED, TEXT_PRIMARY,
};

#[derive(Debug, Clone)]
enum InlineStyle {
    Normal,
    Bold,
    Italic,
    BoldItalic,
    Code,
    Strikethrough,
}

#[derive(Debug, Clone)]
struct InlineSpan {
    text: String,
    style: InlineStyle,
}

pub fn render_markdown<'a, Message: 'a>(
    content: &str,
    base_font: Font,
    zoom_level: f32,
    lang: crate::i18n::Lang,
) -> Element<'a, Message> {
    if content.trim().is_empty() {
        return container(
            text(crate::i18n::t(lang, crate::i18n::Key::NothingToPreview))
                .size(13.0 * zoom_level)
                .color(TEXT_MUTED),
        )
        .padding(20)
        .into();
    }

    let parser = Parser::new(content);
    let mut elements: Vec<Element<'a, Message>> = Vec::new();

    let mut current_spans: Vec<InlineSpan> = Vec::new();
    let mut in_heading: Option<HeadingLevel> = None;
    let mut is_bold = false;
    let mut is_italic = false;
    let mut is_strikethrough = false;
    let mut code_block_buffer = String::new();
    let mut in_code_block = false;

    let h1_size = 24.0 * zoom_level;
    let h2_size = 20.0 * zoom_level;
    let h3_size = 17.0 * zoom_level;
    let body_size = 15.0 * zoom_level;

    for event in parser {
        match event {
            Event::Start(Tag::Heading { level, .. }) => {
                flush_paragraph(&mut current_spans, &mut elements, body_size, base_font);
                in_heading = Some(level);
            }
            Event::End(TagEnd::Heading(_)) => {
                let text_content = current_spans
                    .drain(..)
                    .map(|s| s.text)
                    .collect::<Vec<_>>()
                    .join("");

                let (size, color) = match in_heading {
                    Some(HeadingLevel::H1) => (h1_size, ACCENT_CYAN),
                    Some(HeadingLevel::H2) => (h2_size, ACCENT_AMBER),
                    _ => (h3_size, TEXT_PRIMARY),
                };

                let bold_font = Font {
                    weight: font::Weight::Bold,
                    ..base_font
                };

                let heading_el = container(
                    text(text_content).size(size).font(bold_font).color(color),
                )
                .padding(iced::Padding {
                    top: 14.0,
                    bottom: 6.0,
                    left: 0.0,
                    right: 0.0,
                });

                elements.push(heading_el.into());
                in_heading = None;
            }

            Event::Start(Tag::BlockQuote(_)) => {
                flush_paragraph(&mut current_spans, &mut elements, body_size, base_font);
            }
            Event::End(TagEnd::BlockQuote(_)) => {
                let quote_text = current_spans
                    .drain(..)
                    .map(|s| s.text)
                    .collect::<Vec<_>>()
                    .join("");

                let italic_font = Font {
                    style: font::Style::Italic,
                    ..base_font
                };

                let quote_el = container(
                    text(quote_text)
                        .size(body_size)
                        .font(italic_font)
                        .color(Color::from_rgb(0.75, 0.80, 0.88)),
                )
                .padding(iced::Padding {
                    top: 6.0,
                    bottom: 6.0,
                    left: 14.0,
                    right: 8.0,
                })
                .style(|_t| container::Style {
                    background: Some(iced::Background::Color(Color::from_rgba(
                        0.12, 0.14, 0.18, 0.5,
                    ))),
                    border: iced::Border {
                        color: ACCENT_CYAN,
                        width: 3.0,
                        radius: iced::border::Radius::from(2.0),
                    },
                    ..Default::default()
                });

                elements.push(quote_el.into());
            }

            Event::Start(Tag::CodeBlock(_)) => {
                flush_paragraph(&mut current_spans, &mut elements, body_size, base_font);
                in_code_block = true;
                code_block_buffer.clear();
            }
            Event::End(TagEnd::CodeBlock) => {
                let mono_font = Font {
                    family: font::Family::Monospace,
                    ..base_font
                };

                let code_el = container(
                    text(code_block_buffer.clone())
                        .size(13.0 * zoom_level)
                        .font(mono_font)
                        .color(Color::from_rgb(0.85, 0.90, 0.95)),
                )
                .padding(12)
                .width(Length::Fill)
                .style(|_t| container::Style {
                    background: Some(iced::Background::Color(SURFACE_HOVER)),
                    border: iced::Border {
                        color: BORDER_COLOR,
                        width: 1.0,
                        radius: iced::border::Radius::from(6.0),
                    },
                    ..Default::default()
                });

                elements.push(code_el.into());
                in_code_block = false;
                code_block_buffer.clear();
            }

            Event::Start(Tag::List(_)) => {
                flush_paragraph(&mut current_spans, &mut elements, body_size, base_font);
            }
            Event::End(TagEnd::List(_)) => {}

            Event::Start(Tag::Item) => {
                // start list item
            }
            Event::End(TagEnd::Item) => {
                let item_text = current_spans
                    .drain(..)
                    .map(|s| s.text)
                    .collect::<Vec<_>>()
                    .join("");

                let item_row = row![
                    text("•").size(body_size).color(ACCENT_CYAN),
                    Space::with_width(8),
                    text(item_text).size(body_size).color(TEXT_PRIMARY),
                ]
                .align_y(Alignment::Center);

                elements.push(
                    container(item_row)
                        .padding(iced::Padding {
                            top: 2.0,
                            bottom: 2.0,
                            left: 12.0,
                            right: 0.0,
                        })
                        .into(),
                );
            }

            Event::Start(Tag::Strong) => is_bold = true,
            Event::End(TagEnd::Strong) => is_bold = false,

            Event::Start(Tag::Emphasis) => is_italic = true,
            Event::End(TagEnd::Emphasis) => is_italic = false,

            Event::Start(Tag::Strikethrough) => is_strikethrough = true,
            Event::End(TagEnd::Strikethrough) => is_strikethrough = false,

            Event::Code(code_text) => {
                if in_code_block {
                    code_block_buffer.push_str(&code_text);
                } else {
                    current_spans.push(InlineSpan {
                        text: format!(" {} ", code_text),
                        style: InlineStyle::Code,
                    });
                }
            }

            Event::Text(t) => {
                if in_code_block {
                    code_block_buffer.push_str(&t);
                } else {
                    let style = match (is_bold, is_italic) {
                        (true, true) => InlineStyle::BoldItalic,
                        (true, false) => InlineStyle::Bold,
                        (false, true) => InlineStyle::Italic,
                        (false, false) => {
                            if is_strikethrough {
                                InlineStyle::Strikethrough
                            } else {
                                InlineStyle::Normal
                            }
                        }
                    };
                    current_spans.push(InlineSpan {
                        text: t.to_string(),
                        style,
                    });
                }
            }

            Event::Rule => {
                flush_paragraph(&mut current_spans, &mut elements, body_size, base_font);
                let rule = container(Space::with_height(1))
                    .width(Length::Fill)
                    .style(|_t| container::Style {
                        background: Some(iced::Background::Color(BORDER_COLOR)),
                        ..Default::default()
                    });
                elements.push(
                    container(rule)
                        .padding(iced::Padding {
                            top: 12.0,
                            bottom: 12.0,
                            left: 0.0,
                            right: 0.0,
                        })
                        .into(),
                );
            }

            Event::SoftBreak | Event::HardBreak => {
                if in_code_block {
                    code_block_buffer.push('\n');
                } else {
                    current_spans.push(InlineSpan {
                        text: " ".to_string(),
                        style: InlineStyle::Normal,
                    });
                }
            }

            _ => {}
        }
    }

    flush_paragraph(&mut current_spans, &mut elements, body_size, base_font);

    column(elements).spacing(8).width(Length::Fill).into()
}

fn flush_paragraph<'a, Message: 'a>(
    spans: &mut Vec<InlineSpan>,
    elements: &mut Vec<Element<'a, Message>>,
    body_size: f32,
    base_font: Font,
) {
    if spans.is_empty() {
        return;
    }

    let all_normal = spans.iter().all(|s| matches!(s.style, InlineStyle::Normal));
    if all_normal {
        let full_text: String = spans.drain(..).map(|s| s.text).collect();
        let p_text = text(full_text)
            .size(body_size)
            .font(base_font)
            .color(TEXT_PRIMARY)
            .wrapping(iced::widget::text::Wrapping::Word);

        let p_container = container(p_text)
            .width(Length::Fill)
            .padding(iced::Padding {
                top: 4.0,
                bottom: 4.0,
                left: 0.0,
                right: 0.0,
            });

        elements.push(p_container.into());
        return;
    }

    // Check if paragraph is RTL (Persian)
    let full_text = spans
        .iter()
        .map(|s| s.text.as_str())
        .collect::<Vec<_>>()
        .join("");
    let _is_rtl = detect_paragraph_direction(&full_text) == unicode_bidi::Direction::Rtl;

    let mut span_widgets: Vec<Element<'a, Message>> = Vec::new();

    for span in spans.drain(..) {
        let (font_style, weight, color) = match span.style {
            InlineStyle::Bold => (font::Style::Normal, font::Weight::Bold, ACCENT_CYAN),
            InlineStyle::Italic => (font::Style::Italic, font::Weight::Normal, ACCENT_AMBER),
            InlineStyle::BoldItalic => (font::Style::Italic, font::Weight::Bold, ACCENT_GREEN),
            InlineStyle::Code => (
                font::Style::Normal,
                font::Weight::Normal,
                Color::from_rgb(0.9, 0.9, 0.7),
            ),
            InlineStyle::Strikethrough => (font::Style::Normal, font::Weight::Normal, TEXT_MUTED),
            InlineStyle::Normal => (font::Style::Normal, font::Weight::Normal, TEXT_PRIMARY),
        };

        let span_font = Font {
            style: font_style,
            weight,
            ..base_font
        };

        let text_el = text(span.text)
            .size(body_size)
            .font(span_font)
            .color(color)
            .wrapping(iced::widget::text::Wrapping::Word);

        if matches!(span.style, InlineStyle::Code) {
            span_widgets.push(
                container(text_el)
                    .padding(iced::Padding {
                        top: 1.0,
                        bottom: 1.0,
                        left: 5.0,
                        right: 5.0,
                    })
                    .style(|_t| container::Style {
                        background: Some(iced::Background::Color(SURFACE_ACTIVE)),
                        border: iced::Border {
                            color: BORDER_COLOR,
                            width: 1.0,
                            radius: iced::border::Radius::from(3.0),
                        },
                        ..Default::default()
                    })
                    .into(),
            );
        } else {
            span_widgets.push(text_el.into());
        }
    }

    let p_row = row(span_widgets)
        .spacing(2)
        .align_y(Alignment::Center)
        .wrap();

    let p_container = container(p_row).width(Length::Fill).padding(iced::Padding {
        top: 4.0,
        bottom: 4.0,
        left: 0.0,
        right: 0.0,
    });

    elements.push(p_container.into());
}
