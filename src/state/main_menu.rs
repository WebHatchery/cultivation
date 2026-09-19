use crate::data::loader::GameData;
use crate::save::storage;
use crate::state::{StateTransition, UpdateResult};
use crate::ui::{components::*, theme::*, workspace as ux};
use macroquad::prelude::*;
use macroquad_toolkit::ui::draw_ui_text;

pub struct MainMenuState {
    has_save: bool,
    checked_save: bool,
}
impl MainMenuState {
    pub fn new() -> Self {
        Self {
            has_save: false,
            checked_save: false,
        }
    }
    pub fn update(&mut self) -> UpdateResult {
        if !self.checked_save {
            self.has_save = storage::exists();
            self.checked_save = true;
        }
        draw_mountain_sect_backdrop();
        let wide = screen_width() >= 980.0;
        let x = if wide { 64.0 } else { 32.0 };
        let y = if wide { screen_height() * 0.32 } else { 56.0 };
        draw_ui_text("HEAVENLY MANDATE", x, y, FONT_TITLE_SIZE, PRIMARY);
        draw_ui_text(
            "Restore a fallen sect. Shape its future.",
            x,
            y + 36.0,
            FONT_BODY_SIZE,
            TEXT_PRIMARY,
        );
        let w = if wide {
            400.0
        } else {
            (screen_width() - 64.0).min(580.0)
        };
        let p = Rect::new(
            if wide { screen_width() - w - 48.0 } else { x },
            if wide { 120.0 } else { 130.0 },
            w,
            366.0,
        );
        draw_panel(p, None);
        draw_ui_text(
            if self.has_save {
                "Your mountain awaits"
            } else {
                "A new beginning"
            },
            p.x + 24.0,
            p.y + 38.0,
            FONT_HEADER_SIZE,
            TEXT_HIGHLIGHT,
        );
        draw_wrapped_text(
            "Rebuild the halls, guide your disciples, and choose who ventures beyond the gate.",
            p.x + 24.0,
            p.y + 78.0,
            p.w - 48.0,
            FONT_BODY_SIZE,
            TEXT_PRIMARY,
        );
        let primary = Rect::new(p.x + 24.0, p.y + 164.0, p.w - 48.0, 52.0);
        if ux::button(
            primary,
            if self.has_save {
                "Resume Mandate"
            } else {
                "Rebuild the Sect"
            },
            true,
            true,
        ) || is_key_pressed(KeyCode::Enter)
        {
            return if self.has_save {
                UpdateResult::new().with_action(crate::engine::actions::Action::LoadGame)
            } else {
                UpdateResult::new().with_transition(StateTransition::ToSectCreation)
            };
        }
        if self.has_save {
            if ux::button(
                Rect::new(primary.x, primary.y + 64.0, primary.w, 48.0),
                "Begin Anew",
                false,
                true,
            ) || is_key_pressed(KeyCode::Space)
            {
                return UpdateResult::new().with_transition(StateTransition::ToSectCreation);
            }
            draw_wrapped_text(
                "Beginning a new sect replaces the current save.",
                p.x + 24.0,
                p.y + 310.0,
                p.w - 48.0,
                FONT_SMALL_SIZE,
                TEXT_SECONDARY,
            );
        } else {
            draw_wrapped_text(
                "Your first task: restore the Sect Hall. A short guide will help you begin.",
                p.x + 24.0,
                p.y + 258.0,
                p.w - 48.0,
                FONT_SMALL_SIZE,
                TEXT_SECONDARY,
            );
        }
        UpdateResult::new()
    }
    pub fn draw(&self, _data: &GameData, _spirit_stones: u32) {}
}
