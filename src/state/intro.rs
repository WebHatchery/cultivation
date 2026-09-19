use crate::state::{StateTransition, UpdateResult};
use crate::ui::theme::*;
use crate::ui::workspace as ux;
use macroquad::prelude::*;
use macroquad_toolkit::ui::draw_ui_text;

struct IntroBeat {
    title: &'static str,
    stamp: &'static str,
    text: &'static str,
}

const BEATS: &[IntroBeat] = &[
    IntroBeat {
        title: "The Long Road Home",
        stamp: "RETURN",
        text: "Snow chokes the mountain pass. After thirty years of wandering, the patriarch \
               climbs the ten thousand steps alone. No gongs greet him. No incense burns.",
    },
    IntroBeat {
        title: "Ashes of the Ancestral Hall",
        stamp: "RUIN",
        text: "The gate hangs from one hinge. The ancestral hall lies cracked open to the sky, \
               its tablets scattered like teeth. Of the hundred halls of the sect, not one \
               still stands whole.",
    },
    IntroBeat {
        title: "What the Enemy Left Behind",
        stamp: "GRIEF",
        text: "He buries what the crows left. An elder's shattered sword. A junior's single \
               sandal. The mountain remembers every name, and so will he.",
    },
    IntroBeat {
        title: "The Vow",
        stamp: "VOW",
        text: "Kneeling before the broken tablets, the patriarch presses his brow to cold \
               stone. 'By my blood and dao-heart: this sect will rise again. Higher than \
               before. Beyond the reach of any enemy under heaven.'",
    },
    IntroBeat {
        title: "The First Decree",
        stamp: "SECT",
        text: "Dawn. One ruined hall, fifty spirit stones, and an immortal ambition. The \
               rebuilding begins with a single decree: restore the Sect Hall.",
    },
];

pub struct IntroState {
    beat: usize,
    page: usize,
}
impl IntroState {
    pub fn new() -> Self {
        Self { beat: 0, page: 0 }
    }
    pub fn update(&mut self, sect_name: &str) -> UpdateResult {
        if ux::header_to("The fallen mountain", sect_name, "Skip story") {
            return UpdateResult::new().with_transition(StateTransition::ToSectBase);
        }
        let area = ux::content();
        let beat = &BEATS[self.beat];
        draw_ui_text(
            &format!("{} / {}   {}", self.beat + 1, BEATS.len(), beat.stamp),
            area.x,
            area.y + 28.0,
            FONT_SMALL_SIZE,
            PRIMARY,
        );
        ux::text_pages(
            Rect::new(area.x, area.y + 52.0, area.w, area.h - 124.0),
            &[beat.title.to_owned(), beat.text.to_owned()],
            &mut self.page,
        );
        let y = area.y + area.h - 52.0;
        if ux::button(
            Rect::new(area.x, y, 140.0, 48.0),
            "Previous",
            false,
            self.beat > 0,
        ) {
            self.beat -= 1;
            self.page = 0;
        }
        let last = self.beat + 1 == BEATS.len();
        if ux::button(
            Rect::new(area.x + area.w - 200.0, y, 200.0, 48.0),
            if last { "Enter the sect" } else { "Continue" },
            true,
            true,
        ) || is_key_pressed(KeyCode::Space)
            || is_key_pressed(KeyCode::Enter)
        {
            if last {
                return UpdateResult::new().with_transition(StateTransition::ToSectBase);
            }
            self.beat += 1;
            self.page = 0;
        }
        UpdateResult::new()
    }
    pub fn draw(&self) {}
}
