//! Common workspace composition for decision screens and touch-accessible paging.
use crate::ui::{components::*, theme::*};
use macroquad::prelude::*;
use macroquad_toolkit::ui::{button_rect_enabled_styled, draw_ui_text, ButtonStyle, VirtualUi};
use std::ops::Range;

pub fn button(rect: Rect, label: &str, primary: bool, enabled: bool) -> bool {
    let style = ButtonStyle {
        normal: if primary {
            Color::new(0.23, 0.28, 0.22, 1.0)
        } else {
            Color::new(0.10, 0.11, 0.10, 1.0)
        },
        hovered: Color::new(0.30, 0.30, 0.22, 1.0),
        pressed: PANEL_BG,
        border: if primary { PRIMARY } else { PANEL_BORDER },
        text_color: TEXT_PRIMARY,
        disabled: Color::new(0.12, 0.12, 0.11, 1.0),
    };
    button_rect_enabled_styled(rect, label, enabled, &style)
}

pub fn header(title: &str, purpose: &str) -> bool {
    header_to(title, purpose, "Back to Sect")
}

pub fn header_to(title: &str, purpose: &str, back: &str) -> bool {
    draw_rectangle(
        0.0,
        0.0,
        screen_width(),
        screen_height(),
        Color::new(0.045, 0.05, 0.045, 1.0),
    );
    draw_panel(Rect::new(0.0, 0.0, screen_width(), 100.0), None);
    draw_ui_text(title, 24.0, 38.0, FONT_HEADER_SIZE, PRIMARY);
    draw_wrapped_text(
        purpose,
        24.0,
        70.0,
        screen_width() - 180.0,
        FONT_SMALL_SIZE,
        TEXT_SECONDARY,
    );
    if back.is_empty() {
        return false;
    }
    button(
        Rect::new(screen_width() - 142.0, 24.0, 118.0, 48.0),
        back,
        false,
        true,
    ) || is_key_pressed(KeyCode::Escape)
}

pub fn content() -> Rect {
    Rect::new(24.0, 116.0, screen_width() - 48.0, screen_height() - 140.0)
}

pub fn split(rect: Rect) -> (Rect, Rect) {
    let list_w = (rect.w * 0.34).clamp(240.0, 380.0);
    (
        Rect::new(rect.x, rect.y, list_w, rect.h),
        Rect::new(
            rect.x + list_w + 16.0,
            rect.y,
            rect.w - list_w - 16.0,
            rect.h,
        ),
    )
}

pub fn tabs(rect: Rect, labels: &[&str], selected: &mut usize) -> bool {
    let width = (rect.w / labels.len() as f32).min(220.0);
    let mut changed = false;
    for (index, label) in labels.iter().enumerate() {
        if button(
            Rect::new(rect.x + index as f32 * width, rect.y, width - 8.0, 44.0),
            label,
            *selected == index,
            true,
        ) {
            changed = *selected != index;
            *selected = index;
        }
    }
    changed
}

/// Reserve a footer and return a range of complete rows; every item is reachable by tap.
pub fn page(rect: Rect, count: usize, row_height: f32, current: &mut usize) -> Range<usize> {
    let capacity = ((rect.h - 60.0) / row_height).floor().max(1.0) as usize;
    let pages = count.div_ceil(capacity).max(1);
    *current = (*current).min(pages - 1);
    let y = rect.y + rect.h - 48.0;
    if button(
        Rect::new(rect.x, y, 88.0, 44.0),
        "Previous",
        false,
        *current > 0,
    ) {
        *current -= 1;
    }
    if button(
        Rect::new(rect.x + rect.w - 88.0, y, 88.0, 44.0),
        "Next",
        false,
        *current + 1 < pages,
    ) {
        *current += 1;
    }
    draw_ui_text(
        &format!("{}/{}", *current + 1, pages),
        rect.x + rect.w / 2.0 - 14.0,
        y + 28.0,
        FONT_SMALL_SIZE,
        TEXT_SECONDARY,
    );
    let start = *current * capacity;
    start..(start + capacity).min(count)
}

pub fn clipped<R>(rect: Rect, draw: impl FnOnce() -> R) -> R {
    VirtualUi::from_responsive_screen_size(screen_width(), screen_height(), 1.0)
        .with_clip(rect, draw)
}

pub fn text_pages(rect: Rect, paragraphs: &[String], current: &mut usize) {
    let lines: Vec<String> = paragraphs
        .iter()
        .flat_map(|text| {
            let mut lines = macroquad_toolkit::ui::wrap_text(text, rect.w - 16.0, FONT_BODY_SIZE);
            lines.push(String::new());
            lines
        })
        .collect();
    let range = page(rect, lines.len(), 26.0, current);
    for (row, index) in range.enumerate() {
        draw_ui_text(
            &lines[index],
            rect.x + 4.0,
            rect.y + 22.0 + row as f32 * 26.0,
            FONT_BODY_SIZE,
            TEXT_PRIMARY,
        );
    }
}
