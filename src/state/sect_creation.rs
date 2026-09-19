use crate::data::loader::GameData;
use crate::engine::actions::Action;
use crate::state::{StateTransition, UpdateResult};
use crate::ui::components::{draw_panel, draw_wrapped_text};
use crate::ui::theme::*;
use crate::ui::workspace as ux;
use macroquad::prelude::*;
use macroquad_toolkit::ui::draw_ui_text;

pub struct SectCreationState {
    pub input_buffer: String,
    using_suggested_name: bool,
}

impl SectCreationState {
    pub fn new() -> Self {
        Self {
            input_buffer: "Fallen Peak".to_string(),
            using_suggested_name: true,
        }
    }

    pub fn update(&mut self) -> UpdateResult {
        if is_key_pressed(KeyCode::Escape) {
            return UpdateResult::new().with_transition(StateTransition::ToMainMenu);
        }

        while let Some(c) = get_char_pressed() {
            if !is_valid_sect_name_char(c) || self.input_buffer.len() >= 24 {
                continue;
            }
            if self.using_suggested_name {
                self.input_buffer.clear();
                self.using_suggested_name = false;
            }
            self.input_buffer.push(c);
        }

        if is_key_pressed(KeyCode::Backspace) {
            if self.using_suggested_name {
                self.input_buffer.clear();
                self.using_suggested_name = false;
            } else {
                self.input_buffer.pop();
            }
        }

        if ux::header_to(
            "Name your sect",
            "Keep the suggested name, or type a name of your own.",
            "Back",
        ) {
            return UpdateResult::new().with_transition(StateTransition::ToMainMenu);
        }
        let area = ux::content();
        let w = area.w.min(660.0);
        let x = (screen_width() - w) / 2.0;
        draw_ui_text(
            "The name above the mountain gate",
            x,
            area.y + 36.0,
            FONT_HEADER_SIZE,
            TEXT_HIGHLIGHT,
        );
        draw_wrapped_text("Your disciples will carry this name into the world. Keep Fallen Peak or choose a name for your new beginning.",x,area.y+76.0,w,FONT_BODY_SIZE,TEXT_PRIMARY);
        let input = Rect::new(x, area.y + 142.0, w, 60.0);
        draw_panel(input, None);
        draw_ui_text(
            if self.input_buffer.is_empty() {
                "Enter a sect name"
            } else {
                &self.input_buffer
            },
            x + 16.0,
            input.y + 39.0,
            FONT_HEADER_SIZE,
            TEXT_HIGHLIGHT,
        );
        draw_ui_text(
            "Up to 24 letters, numbers, spaces, hyphens or apostrophes",
            x,
            input.y + 88.0,
            FONT_SMALL_SIZE,
            TEXT_SECONDARY,
        );
        if ux::button(
            Rect::new(x, input.y + 112.0, 190.0, 48.0),
            "Use Fallen Peak",
            false,
            true,
        ) {
            self.input_buffer = "Fallen Peak".to_owned();
            self.using_suggested_name = true;
        }
        let name = self.input_buffer.trim();
        if (ux::button(
            Rect::new(x, area.y + area.h - 56.0, w, 52.0),
            "Raise the Banner",
            true,
            !name.is_empty(),
        ) || is_key_pressed(KeyCode::Enter))
            && !name.is_empty()
        {
            return UpdateResult::new().with_action(Action::StartNewGame(name.to_owned()));
        }
        UpdateResult::new()
    }
    pub fn draw(&self, _data: &GameData) {}
}

fn is_valid_sect_name_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == ' ' || c == '-' || c == '\''
}
